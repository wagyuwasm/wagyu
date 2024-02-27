use std::{ops::Range, ptr};

use crate::{helper::leb128::decode_unsigned_leb128, instance::ComponentInstance, module::{function::{Function, ParsedBody}, import::{Import, ImportKind}, memory::Memory32, types::Type, value::ValType}};

pub enum Error {
  InvalidBinaryMagic,
  InvalidBinaryVersion,
  InvalidSectionFormat(String),
  InvalidValue(String),
  MissingSection(String),
}

pub(crate) fn parse(src_bin: &[u8]) -> Result<ComponentInstance, Error> {
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
  let mut tmp_memories = Vec::new();

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
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        
        todo!()
      },
      // type section
      1 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_type, n_type_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let parse_type = |range: Range<usize>| {
          range
            .into_iter()
            .map(|i| ValType::try_from(src_bin[i]))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::InvalidValue)
        };

        let mut type_ofs = section_ofs + section_size_b + n_type_b + 1;
        let mut result_items = Vec::with_capacity(n_type as usize);
        for _ in 0..n_type {
          let (n_param, n_param_b) = decode_unsigned_leb128(&src_bin[(type_ofs + 1)..]);
          let (n_result, n_result_b) = decode_unsigned_leb128(&src_bin[(type_ofs + n_param_b + (n_param as usize) + 1)..]);
  
          let param_ofs = type_ofs + n_param_b + 1;
          let param_types = parse_type(param_ofs..(param_ofs + (n_param as usize)))?;

          let result_ofs = type_ofs + n_param_b + (n_param as usize) + n_result_b + 1;
          let next_func_ofs = result_ofs + (n_result as usize);
          let result_types = parse_type(result_ofs..next_func_ofs)?;
  
          let raw_signature = &src_bin[type_ofs..next_func_ofs];
  
          result_items.push(Type {
            params: param_types,
            results: result_types
          });
  
          type_ofs = next_func_ofs;
        }

        tmp_types = result_items;

        finalize_section(section_ofs, section_size, type_ofs)
      },
      // import section
      2 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_import, n_import_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut import_ofs = section_ofs + section_size_b + n_import_b + 1;
        let mut result_items = Vec::with_capacity(n_import as usize);
        for _ in 0..n_import {
          let (module_name_len, module_name_len_b) = decode_unsigned_leb128(&src_bin[import_ofs..]);
          let module_name = parse_utf8(import_ofs + module_name_len_b, module_name_len as usize)?;

          let (field_name_len, field_name_len_b) = decode_unsigned_leb128(&src_bin[(import_ofs + module_name_len_b + (module_name_len as usize))..]);
          let field_name_ofs = import_ofs + module_name_len_b + (module_name_len as usize) + field_name_len_b;
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

          result_items.push(Import {
            module_name,
            field_name,
            kind
          });

          import_ofs = kind_ofs + kind_b + 1;
        }

        tmp_imports = result_items;

        finalize_section(section_ofs, section_size, import_ofs)
      },
      // function section
      3 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_func, n_func_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut func_ofs = section_ofs + section_size_b + n_func_b + 1;
        let mut result_items = Vec::with_capacity(n_func as usize);
        for _ in 0..n_func {
          let (type_pos, type_pos_b) = decode_unsigned_leb128(&src_bin[func_ofs..]);

          result_items.push(type_pos);

          func_ofs += type_pos_b;
        }

        tmp_function_types = result_items;

        finalize_section(section_ofs, section_size, func_ofs)
      },
      // table section
      4 => {
        todo!()
      },
      // memory section
      5 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_mem, n_mem_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut mem_flag_ofs = section_ofs + section_size_b + n_mem_b + 1;
        let mut result_items = Vec::with_capacity(n_mem as usize);
        for _ in 0..n_mem {
          let limit_flag = src_bin[mem_flag_ofs];
          let (limit_initial, limit_initial_b) = decode_unsigned_leb128(&src_bin[(mem_flag_ofs + 1)..]);

          let (max_b, max) = match limit_flag {
            0 => (0, None),
            1 => {
              let max_ofs = mem_flag_ofs + limit_initial_b + 1;
              let (limit_max, limit_max_b) = decode_unsigned_leb128(&src_bin[max_ofs..]);
  
              (limit_max_b, Some(limit_max))
            },
            _ => return Err(Error::InvalidValue(format!("limit flag byte is invalid")))
          };

          result_items.push(Memory32 {
            ptr: ptr::null_mut(),
            size: 0,
            min: limit_initial as u32,
            max: max.map(|x| x as u32)
          });

          mem_flag_ofs += limit_initial_b + max_b + 1;
        }

        tmp_memories = result_items;

        finalize_section(section_ofs, section_size, mem_flag_ofs)
      },
      // global section
      6 => {
        todo!()
      },
      // export section
      7 => {
        todo!()
      },
      // start section
      8 => {
        todo!()
      },
      // element section
      9 => {
        todo!()
      },
      // code section
      10 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_ofs + 1)..]);
        let (n_func, n_func_b) = decode_unsigned_leb128(&src_bin[(section_ofs + section_size_b + 1)..]);

        let mut func_ofs = section_ofs + section_size_b + n_func_b + 1;
        let mut result_items = Vec::with_capacity(n_func as usize);
        for i in 0..n_func {
          let func_type_idx = tmp_function_types[i as usize];
          let (body_size, body_size_b) = decode_unsigned_leb128(&src_bin[func_ofs..]);

          let (parsed_body, body_size_b) = if body_size > 0 {
            (None, body_size_b)
          } else {
            let (parsed_body, parsed_body_b) = parse_func_body(src_bin, func_ofs)?;

            (Some(parsed_body), parsed_body_b)
          };

          result_items.push(Function {
            signature_idx: func_type_idx as u32,
            parsed_body
          });

          func_ofs += body_size_b + 1;
        }

        tmp_functions = result_items;

        finalize_section(section_ofs, section_size, func_ofs)
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

  Ok(ComponentInstance {
    types: tmp_types,
    imports: tmp_imports,
    functions: tmp_functions,
    tables: vec![],
    memories: tmp_memories,
    globals: vec![],
    exports: vec![],
    start_func: None,
  })
}

fn parse_func_body(src_bin: &[u8], code_ofs: usize) -> Result<(ParsedBody, usize), Error> {
  todo!();
}
