use alloc::string::String;
use alloc::vec::Vec;
use core::ptr;
use core::ops::Range;

use crate::{helper::leb128::decode_unsigned_leb128, instance::ModuleInstance, module::{export::Export, function::{Function, ParsedBody}, global::Global, import::{Import, ImportKind}, memory::Memory32, types::Type, value::{ExportDesc, GlobalMut, ValType}}};

pub enum Error {
  InvalidBinaryMagic,
  InvalidBinaryVersion,
  InvalidSectionFormat(String),
  InvalidValue(String),
  MissingSection(String),
}

pub(crate) fn parse(src_bin: &[u8]) -> Result<ModuleInstance, Error> {
  let binary_magic = &src_bin[0..4];
  let binary_version = &src_bin[4..8];

  if binary_magic != &[0x00, 0x61, 0x73, 0x6d] {
    return Err(Error::InvalidBinaryMagic);
  }
  if binary_version != &[0x01, 0x00, 0x00, 0x00] {
    return Err(Error::InvalidBinaryVersion);
  }

  let mut section_ofs = 8;

  let mut tmp_types = Vec::new();
  let mut tmp_imports = Vec::new();
  let mut tmp_function_types = Vec::new();
  let mut tmp_functions = Vec::new();
  let mut tmp_tables = Vec::new();
  let mut tmp_memories = Vec::new();
  let mut tmp_globals = Vec::new();
  let mut tmp_exports = Vec::new();
  let mut tmp_start_func = None;

  // Calculates the fixup size of a section if body size is not provided.
  let finalize_section = |section_ofs: usize, section_size: u64, fixup_ofs: usize| {
    match section_size {
      0 => {
        let (_, fixup_size_b) = decode_unsigned_leb128(&src_bin[fixup_ofs..]);

        fixup_ofs + fixup_size_b
      },
      _ => section_ofs + (section_size as usize) + 1
    }
  };

  // Parses string in a given offset and len to the reading source binary.
  let parse_utf8 = |ofs: usize, len: usize| {
    String::from_utf8(Vec::from(&src_bin[ofs..(ofs + len)]))
      .map_err(|err| Error::InvalidValue(format!("{}", err)))
  };

  loop {
    if section_ofs >= src_bin.len() {
      break;
    }

    section_ofs = match src_bin[section_ofs] {
      // custom section
      0 => {
        // assumption: the custom section is the last section to be parsed, so we can skip this section.
        break;
      },
      // type section
      1 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let parse_type = |range: Range<usize>| {
          range
            .map(|i| ValType::try_from(src_bin[i]))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::InvalidValue)
        };

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_types = (0..n_item)
          .map(|_| {
            if src_bin[item_ofs] != 0x60 {
              return Err(Error::InvalidValue(format!("not func type")));
            }

            let (n_param, n_param_b) = decode_unsigned_leb128(&src_bin[(item_ofs + 1)..]);
            let (n_result, n_result_b) = decode_unsigned_leb128(&src_bin[(item_ofs + n_param_b + (n_param as usize) + 1)..]);
    
            let param_ofs = item_ofs + n_param_b + 1;
            let param_types = parse_type(param_ofs..(param_ofs + (n_param as usize)))?;
  
            let result_ofs = item_ofs + n_param_b + (n_param as usize) + n_result_b + 1;
            let next_func_ofs = result_ofs + (n_result as usize);
            let result_types = parse_type(result_ofs..next_func_ofs)?;

            item_ofs = next_func_ofs;

            Ok(Type {
              params: param_types,
              results: result_types
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // import section
      2 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_imports = (0..n_item)
          .map(|_| {
            let (module_name_len, module_name_len_b) = decode_unsigned_leb128(&src_bin[item_ofs..]);
            let module_name = parse_utf8(item_ofs + module_name_len_b, module_name_len as usize)?;
  
            let (field_name_len, field_name_len_b) = decode_unsigned_leb128(&src_bin[(item_ofs + module_name_len_b + (module_name_len as usize))..]);
            let field_name_ofs = item_ofs + module_name_len_b + (module_name_len as usize) + field_name_len_b;
            let field_name = parse_utf8(field_name_ofs, field_name_len as usize)?;
  
            let kind_ofs = field_name_ofs + (field_name_len as usize);
            let (kind, kind_b) = match src_bin[kind_ofs] {
                0 => {
                  let (type_idx, type_idx_b) = decode_unsigned_leb128(&src_bin[(kind_ofs + 1)..]);
  
                  (ImportKind::TypeIdx(type_idx as u32), type_idx_b)
                },
                1 => todo!(),
                2 => todo!(),
                3 => todo!(),
                _ => return Err(Error::InvalidValue(format!("invalid import kind")))
            };

            item_ofs = kind_ofs + kind_b + 1;

            Ok(Import {
              module_name,
              field_name,
              kind
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // function section
      3 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_function_types = (0..n_item)
          .map(|_| {
            let (type_pos, type_pos_b) = decode_unsigned_leb128(&src_bin[item_ofs..]);

            item_ofs += type_pos_b;

            Ok(type_pos)
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // table section
      4 => {
        todo!()
      },
      // memory section
      5 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_memories = (0..n_item)
          .map(|_| {
            let limit_flag = src_bin[item_ofs];
            let (limit_initial, limit_initial_b) = decode_unsigned_leb128(&src_bin[(item_ofs + 1)..]);
  
            let (max_b, max) = match limit_flag {
              0 => (0, None),
              1 => {
                let max_ofs = item_ofs + limit_initial_b + 1;
                let (limit_max, limit_max_b) = decode_unsigned_leb128(&src_bin[max_ofs..]);
    
                (limit_max_b, Some(limit_max))
              },
              _ => return Err(Error::InvalidValue(format!("limit flag byte is invalid")))
            };

            item_ofs += limit_initial_b + max_b + 1;

            Ok(Memory32 {
              ptr: ptr::null_mut(),
              size: 0,
              min: limit_initial as u32,
              max: max.map(|x| x as u32)
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // global section
      6 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_globals = (0..n_item)
          .map(|_| {
            let global_valtype = ValType::try_from(src_bin[item_ofs]).map_err(Error::InvalidValue)?;
            let global_mut = GlobalMut::try_from(src_bin[item_ofs + 1]).map_err(Error::InvalidValue)?;

            Ok(Global {
              kind: global_mut,
              valtype: global_valtype,
              value: todo!()
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // export section
      7 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_exports = (0..n_item)
          .map(|_| {
            let (export_name_len, export_name_len_b) = decode_unsigned_leb128(&src_bin[item_ofs..]);
            let export_name = parse_utf8(item_ofs + export_name_len_b, export_name_len as usize)?;
            
            let export_idx_ofs = item_ofs + export_name_len_b + (export_name_len as usize) + 1;
            let export_desc = ExportDesc::try_from(src_bin[export_idx_ofs - 1])
              .map_err(Error::InvalidValue)?;

            let (export_idx, export_idx_b) = decode_unsigned_leb128(&src_bin[export_idx_ofs..]);

            item_ofs = export_idx_ofs + export_idx_b;

            Ok(Export {
              name: export_name,
              desc: export_desc,
              idx: export_idx as u32
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // start section
      8 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (start_func_idx, start_func_idx_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        tmp_start_func = Some(start_func_idx as u32);

        finalize_section(section_ofs, section_size, section_ofs + section_size_b + start_func_idx_b + 1)
      },
      // element section
      9 => {
        todo!()
      },
      // code section
      10 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_functions = (0..n_item)
          .map(|i| {
            let func_type_idx = tmp_function_types[i as usize];
            let (body_size, body_size_b) = decode_unsigned_leb128(&src_bin[item_ofs..]);
  
            let (parsed_body, body_size_b) = if body_size > 0 {
              (None, body_size_b)
            } else {
              let (parsed_body, parsed_body_b) = parse_func_body(src_bin, item_ofs)?;
  
              (Some(parsed_body), parsed_body_b)
            };

            item_ofs += body_size_b + 1;

            Ok(Function {
              signature_idx: func_type_idx as u32,
              parsed_body
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, item_ofs)
      },
      // data section
      11 => {
        todo!()
      },
      // data count section
      12 => {
        todo!()
      },
      _ => return Err(Error::InvalidSectionFormat(format!("invalid section id {}", src_bin[section_ofs])))
    }
  }

  Ok(ModuleInstance {
    types: tmp_types,
    imports: tmp_imports,
    functions: tmp_functions,
    tables: tmp_tables,
    memories: tmp_memories,
    globals: tmp_globals,
    exports: tmp_exports,
    start_func: tmp_start_func,
  })
}

fn parse_func_body(src_bin: &[u8], code_ofs: usize) -> Result<(ParsedBody, usize), Error> {
  todo!();
}

fn parse_expr(src_bin: &[u8], code_ofs: usize) -> Result<(), Error> {
  todo!()
}