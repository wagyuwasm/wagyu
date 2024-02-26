use std::ops::Range;

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

  let mut section_idx = 8;
  let mut tmp_types = vec![];
  let mut tmp_imports = vec![];
  let mut tmp_function_types = vec![];
  let mut tmp_memories = vec![];
  let mut tmp_functions = vec![];

  // Calculates the fixup size of a section if body size is not provided.
  let finalize_section = |section_idx: usize, section_size: u64, fixup_idx: usize| {
    match section_size {
      0 => {
        let (_, fixup_size_b) = decode_unsigned_leb128(&src_bin[fixup_idx..]);

        fixup_idx + fixup_size_b
      },
      _ => section_idx + (section_size as usize) + 1
    }
  };

  loop {
    if section_idx >= src_bin.len() {
      break;
    }

    section_idx = match src_bin[section_idx] {
      // custom section
      0 => {
        todo!()
      },
      // type section
      1 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_idx + 1)..]);
        let (types_num, types_num_b) = decode_unsigned_leb128(&src_bin[(section_idx + section_size_b + 1)..]);

        let parse_type = |range: Range<usize>| {
          range
            .into_iter()
            .map(|i| ValType::try_from(src_bin[i]))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::InvalidValue)
        };

        let mut type_idx = section_idx + section_size_b + types_num_b + 1;
        let mut result_items = Vec::with_capacity(types_num as usize);
        for _ in 0..types_num {
          let (params_num, params_num_b) = decode_unsigned_leb128(&src_bin[(type_idx + 1)..]);
          let (results_num, results_num_b) = decode_unsigned_leb128(&src_bin[(type_idx + params_num_b + (params_num as usize) + 1)..]);
  
          let param_idx = type_idx + params_num_b + 1;
          let param_types = parse_type(param_idx..(param_idx + (params_num as usize)))?;

          let result_idx = type_idx + params_num_b + (params_num as usize) + results_num_b + 1;
          let next_func_idx = result_idx + (results_num as usize);
          let result_types = parse_type(result_idx..next_func_idx)?;
  
          let raw_signature = &src_bin[type_idx..next_func_idx];
  
          result_items.push(Type {
            params: param_types,
            results: result_types,
            raw: raw_signature
          });
  
          type_idx = next_func_idx;
        }

        tmp_types = result_items;

        finalize_section(section_idx, section_size, type_idx)
      },
      // import section
      2 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_idx + 1)..]);
        let (imports_num, imports_num_b) = decode_unsigned_leb128(&src_bin[(section_idx + section_size_b + 1)..]);

        let mut import_idx = section_idx + section_size_b + imports_num_b + 1;
        let mut result_items = Vec::with_capacity(imports_num as usize);
        for _ in 0..imports_num {
          let (module_name_len, module_name_len_b) = decode_unsigned_leb128(&src_bin[import_idx..]);
          let module_name_idx = import_idx + module_name_len_b;
          let module_name = String::from_utf8(Vec::from(&src_bin[module_name_idx..(module_name_idx + (module_name_len as usize))]))
            .map_err(|err| Error::InvalidValue(format!("{}", err)))?;

          let (field_name_len, field_name_len_b) = decode_unsigned_leb128(&src_bin[(import_idx + module_name_len_b + (module_name_len as usize))..]);
          let field_name_idx = import_idx + module_name_len_b + (module_name_len as usize) + field_name_len_b;
          let field_name = String::from_utf8(Vec::from(&src_bin[field_name_idx..(field_name_idx + (module_name_len as usize))]))
            .map_err(|err| Error::InvalidValue(format!("{}", err)))?;

          let kind_idx = field_name_idx + field_name_len_b;
          let (kind, kind_b) = match src_bin[kind_idx] {
              0 => {
                let (type_idx, type_idx_b) = decode_unsigned_leb128(&src_bin[(kind_idx + 1)..]);

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

          import_idx = kind_idx + kind_b + 1;
        }

        tmp_imports = result_items;

        finalize_section(section_idx, section_size, import_idx)
      },
      // function section
      3 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_idx + 1)..]);
        let (funcs_num, funcs_num_b) = decode_unsigned_leb128(&src_bin[(section_idx + section_size_b + 1)..]);

        let mut func_idx = section_idx + section_size_b + funcs_num_b + 1;
        let mut result_items = Vec::with_capacity(funcs_num as usize);
        for _ in 0..funcs_num {
          let (type_pos, type_pos_b) = decode_unsigned_leb128(&src_bin[func_idx..]);

          result_items.push(type_pos);

          func_idx += type_pos_b;
        }

        tmp_function_types = result_items;

        finalize_section(section_idx, section_size, func_idx)
      },
      // table section
      4 => {
        todo!()
      },
      // memory section
      5 => {
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_idx + 1)..]);
        let (mems_num, mems_num_b) = decode_unsigned_leb128(&src_bin[(section_idx + section_size_b + 1)..]);

        let mut mem_flag_idx = section_idx + section_size_b + mems_num_b + 1;
        let mut result_items = Vec::with_capacity(mems_num as usize);
        for _ in 0..mems_num {
          let limit_flag = src_bin[mem_flag_idx];
          let (limit_initial, limit_initial_b) = decode_unsigned_leb128(&src_bin[(mem_flag_idx + 1)..]);

          let (max_b, max) = match limit_flag {
            0 => (0, None),
            1 => {
              let max_idx = mem_flag_idx + limit_initial_b + 1;
              let (limit_max, limit_max_b) = decode_unsigned_leb128(&src_bin[max_idx..]);
  
              (limit_max_b, Some(limit_max))
            },
            _ => return Err(Error::InvalidValue(format!("limit flag byte is invalid")))
          };

          result_items.push(Memory32 {
            min: limit_initial as u32,
            max: max.map(|x| x as u32)
          });

          mem_flag_idx += limit_initial_b + max_b + 1;
        }

        tmp_memories = result_items;

        finalize_section(section_idx, section_size, mem_flag_idx)
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
        let (section_size, section_size_b) = decode_unsigned_leb128(&src_bin[(section_idx + 1)..]);
        let (funcs_num, funcs_num_b) = decode_unsigned_leb128(&src_bin[(section_idx + section_size_b + 1)..]);

        let mut func_idx = section_idx + section_size_b + funcs_num_b + 1;
        let mut result_items = Vec::with_capacity(funcs_num as usize);
        for i in 0..funcs_num {
          let func_type_idx = tmp_function_types[i as usize];
          let (body_size, body_size_b) = decode_unsigned_leb128(&src_bin[func_idx..]);

          let (parsed_body, body_size_b) = if body_size > 0 {
            (None, body_size_b)
          } else {
            let (parsed_body, parsed_body_b) = parse_func_body(src_bin, func_idx)?;

            (Some(parsed_body), parsed_body_b)
          };

          result_items.push(Function {
            signature_idx: func_type_idx as u32,
            raw_body: &src_bin[func_idx..(func_idx + body_size_b + 1)],
            parsed_body
          });

          func_idx += body_size_b + 1;
        }

        tmp_functions = result_items;

        finalize_section(section_idx, section_size, func_idx)
      },
      // data section
      11 => {
        todo!()
      },
      // data count section
      12 => {
        todo!()
      },
      _ => return Err(Error::InvalidSectionFormat(format!("invalid section id {}", src_bin[section_idx])))
    }
  }

  Ok(ComponentInstance {
    src_bin,
    types: tmp_types,
    imports: tmp_imports,
    memories: tmp_memories,
    functions: tmp_functions,
  })
}

fn parse_func_body(src_bin: &[u8], code_idx: usize) -> Result<(ParsedBody, usize), Error> {
  todo!();
}
