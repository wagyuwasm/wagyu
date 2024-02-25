use std::ops::Range;

use crate::{function::{Function, ParsedBody}, helper::leb128::decode_unsigned_leb128, instance::ComponentInstance, types::Type, value::ValType};

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

  if !binary_magic.eq("0asm".as_bytes()) {
    return Err(Error::InvalidBinaryMagic);
  }
  if !binary_version.eq(&[0x01, 0x00, 0x00, 0x00]) {
    return Err(Error::InvalidBinaryVersion);
  }

  let mut section_idx = 8;
  let mut tmp_types = vec![];
  let mut tmp_function_types = vec![];
  let mut tmp_functions = vec![];

  loop {
    if section_idx >= src_bin.len() {
      break;
    }

    match src_bin[section_idx] {
      // custom section
      0 => (),
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
  
        if section_size == 0 {
          let fixup_idx = type_idx;
          let (_, fixup_size_b) = decode_unsigned_leb128(&src_bin[fixup_idx..]);

          tmp_types = result_items;
  
          section_idx = fixup_idx + fixup_size_b;
        }
      },
      // import section
      2 => (),
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

        if section_size == 0 {
          let fixup_idx = section_idx + section_size_b + funcs_num_b + (funcs_num as usize) + 1;
          let (_, fixup_size_b) = decode_unsigned_leb128(&src_bin[fixup_idx..]);

          tmp_function_types = result_items;
  
          section_idx = fixup_idx + fixup_size_b;
        }
      },
      // table section
      4 => (),
      // memory section
      5 => (),
      // global section
      6 => (),
      // export section
      7 => (),
      // start section
      8 => (),
      // element section
      9 => (),
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
            type_idx: func_type_idx,
            raw_body: &src_bin[func_idx..(func_idx + body_size_b + 1)],
            parsed_body
          });

          func_idx += body_size_b + 1;
        }

        if section_size == 0 {
          let fixup_idx = func_idx;
          let (_, fixup_size_b) = decode_unsigned_leb128(&src_bin[fixup_idx..]);

          tmp_functions = result_items;
  
          section_idx = fixup_idx + fixup_size_b;
        }
      },
      // data section
      11 => (),
      // data count section
      12 => (),
      _ => return Err(Error::InvalidSectionFormat(format!("invalid section id {}", src_bin[section_idx])))
    }
  }

  Ok(ComponentInstance {
    src_bin,
    types: tmp_types,
    functions: tmp_functions,
  })
}

fn parse_func_body(src_bin: &[u8], code_idx: usize) -> Result<(ParsedBody, usize), Error> {
  todo!();
}
