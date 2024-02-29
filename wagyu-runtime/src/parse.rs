use alloc::{
  string::String,
  vec::Vec,
};
use core::{
  fmt, iter, ops::Range, ptr
};

use crate::{
  helper::leb128::{decode_sleb128, decode_uleb128}, instr::Instr, module::{
    export::Export, function::{
      Function,
      ParsedBody,
    }, global::Global, import::{
      Import,
      ImportKind,
    }, memory::Memory32, types::Type, value::{
      ExportDesc,
      GlobalMut,
      ValType,
    }, Module
  }
};

pub enum ErrorKind {
  InvalidBinaryMagic,
  InvalidBinaryVersion,
  InvalidSectionFormat,
  InvalidInstruction,
  InvalidValue,
  MissingSection,
}

pub struct Error {
  pub message: String,
  pub kind: ErrorKind,
  pub offset: usize,
}

impl From<(usize, ErrorKind)> for Error {
  fn from(value: (usize, ErrorKind)) -> Self {
    Self {
      message: String::new(),
      kind: value.1,
      offset: value.0
    }
  }
}

impl From<(usize, ErrorKind, String)> for Error {
  fn from(value: (usize, ErrorKind, String)) -> Self {
    Self {
      message: value.2.to_owned(),
      kind: value.1,
      offset: value.0
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.kind {
      ErrorKind::InvalidBinaryMagic => write!(f, "Invalid binary magic at 0x{:07X}", self.offset),
      ErrorKind::InvalidBinaryVersion => write!(f, "Invalid binary version at 0x{:07X}", self.offset),
      ErrorKind::InvalidSectionFormat => write!(f, "Invalid section format: {} at 0x{:07X}", self.message, self.offset),
      ErrorKind::InvalidInstruction => write!(f, "Invalid instruction: {} at 0x{:07X}", self.message, self.offset),
      ErrorKind::InvalidValue => write!(f, "Invalid value: {} at 0x{:07X}", self.message, self.offset),
      ErrorKind::MissingSection => write!(f, "Missing section: {} at 0x{:07X}", self.message, self.offset),
    }
  }
}

pub(crate) fn parse(buf_src: &[u8]) -> Result<Module, Error> {
  let binary_magic = &buf_src[0..4];
  let binary_version = &buf_src[4..8];

  if binary_magic != &[0x00, 0x61, 0x73, 0x6d] {
    return Err(Error::from((0, ErrorKind::InvalidBinaryMagic)));
  }
  if binary_version != &[0x01, 0x00, 0x00, 0x00] {
    return Err(Error::from((4, ErrorKind::InvalidBinaryVersion)));
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
  let finalize_section = |section_ofs: usize, section_size: u64, section_size_b: usize| {
    section_ofs + (section_size as usize) + section_size_b + 1
  };

  // Parses string in a given offset and len to the reading source binary.
  let parse_utf8 = |ofs: usize, len: usize| {
    String::from_utf8(Vec::from(&buf_src[ofs..(ofs + len)]))
      .map_err(|err| Error::from((ofs, ErrorKind::InvalidValue, err.to_string())))
  };

  loop {
    if section_ofs >= buf_src.len() {
      break;
    }

    section_ofs = match buf_src[section_ofs] {
      // custom section
      0 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // type section
      1 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let parse_type = |range: Range<usize>| {
          range
            .map(|ofs| {
              ValType::try_from(buf_src[ofs])
                .map_err(|err| Error::from((ofs, ErrorKind::InvalidValue, err.to_string())))
            })
            .collect::<Result<Vec<_>, _>>()
        };

        let mut item_ofs = section_ofs + 1 + section_size_b + n_item_b;
        tmp_types = (0..n_item)
          .map(|_| {
            if buf_src[item_ofs] != 0x60 {
              return Err(Error::from((item_ofs, ErrorKind::InvalidValue, format!("not func type"))));
            }

            let (n_param, n_param_b) = decode_uleb128(&buf_src[(item_ofs + 1)..]);
            let (n_result, n_result_b) = decode_uleb128(&buf_src[(item_ofs + n_param_b + (n_param as usize) + 1)..]);

            let param_ofs = item_ofs + 1 + n_param_b;
            let param_types = parse_type(param_ofs..(param_ofs + (n_param as usize)))?;

            let result_ofs = item_ofs + 1 + n_param_b + (n_param as usize) + n_result_b;
            let next_func_ofs = result_ofs + (n_result as usize);
            let result_types = parse_type(result_ofs..next_func_ofs)?;

            item_ofs = next_func_ofs;

            Ok(Type {
              params: param_types,
              results: result_types,
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // import section
      2 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_imports = (0..n_item)
          .map(|_| {
            let (module_name_len, module_name_len_b) = decode_uleb128(&buf_src[item_ofs..]);
            let module_name = parse_utf8(item_ofs + module_name_len_b, module_name_len as usize)?;

            let (field_name_len, field_name_len_b) =
              decode_uleb128(&buf_src[(item_ofs + module_name_len_b + (module_name_len as usize))..]);
            let field_name_ofs = item_ofs + module_name_len_b + (module_name_len as usize) + field_name_len_b;
            let field_name = parse_utf8(field_name_ofs, field_name_len as usize)?;

            let kind_ofs = field_name_ofs + (field_name_len as usize);
            let (kind, kind_b) = match buf_src[kind_ofs] {
              0 => {
                let (type_idx, type_idx_b) = decode_uleb128(&buf_src[(kind_ofs + 1)..]);

                (ImportKind::TypeIdx(type_idx as u32), type_idx_b)
              }
              1 => todo!(),
              2 => todo!(),
              3 => todo!(),
              _ => return Err(Error::from((
                kind_ofs,
                ErrorKind::InvalidValue,
                format!("invalid import kind")
              ))),
            };

            item_ofs = kind_ofs + kind_b + 1;

            Ok(Import {
              module_name,
              field_name,
              kind,
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // function section
      3 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_function_types = (0..n_item)
          .map(|_| {
            let (type_pos, type_pos_b) = decode_uleb128(&buf_src[item_ofs..]);

            item_ofs += type_pos_b;

            Ok(type_pos)
          })
          .collect::<Result<_, Error>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // table section
      4 => {
        todo!()
      }
      // memory section
      5 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_memories = (0..n_item)
          .map(|_| {
            let limit_flag = buf_src[item_ofs];
            let (limit_initial, limit_initial_b) = decode_uleb128(&buf_src[(item_ofs + 1)..]);

            let (max_b, max) = match limit_flag {
              0 => (0, None),
              1 => {
                let max_ofs = item_ofs + limit_initial_b + 1;
                let (limit_max, limit_max_b) = decode_uleb128(&buf_src[max_ofs..]);

                (limit_max_b, Some(limit_max))
              }
              _ => return Err(Error::from((
                item_ofs,
                ErrorKind::InvalidValue,
                format!("limit flag byte is invalid")
              ))),
            };

            item_ofs += limit_initial_b + max_b + 1;

            Ok(Memory32 {
              ptr: ptr::null_mut(),
              size: 0,
              initial: limit_initial as u32,
              max: max.map(|x| x as u32),
            })
          })
          .collect::<Result<_, _>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // global section
      6 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_globals = (0..n_item)
          .map(|_| {
            let global_valtype = ValType::try_from(buf_src[item_ofs])
              .map_err(|err| Error::from((item_ofs, ErrorKind::InvalidValue, err.to_string())))?;
            let global_mut = GlobalMut::try_from(buf_src[item_ofs + 1])
              .map_err(|err| Error::from((item_ofs + 1, ErrorKind::InvalidValue, err.to_string())))?;

            Ok(Global {
              mutable: global_mut,
              valtype: global_valtype,
              value: todo!(),
            })
          })
          .collect::<Result<_, Error>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // export section
      7 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_exports = (0..n_item)
          .map(|_| {
            let (export_name_len, export_name_len_b) = decode_uleb128(&buf_src[item_ofs..]);
            let export_name = parse_utf8(item_ofs + export_name_len_b, export_name_len as usize)?;

            let export_idx_ofs = item_ofs + export_name_len_b + (export_name_len as usize) + 1;
            let export_desc = ExportDesc::try_from(buf_src[export_idx_ofs - 1])
              .map_err(|err| Error::from((export_idx_ofs - 1, ErrorKind::InvalidValue, err.to_string())))?;

            let (export_idx, export_idx_b) = decode_uleb128(&buf_src[export_idx_ofs..]);

            item_ofs = export_idx_ofs + export_idx_b;

            Ok(Export {
              name: export_name,
              desc: export_desc,
              idx: export_idx as u32,
            })
          })
          .collect::<Result<_, Error>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // start section
      8 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (start_func_idx, _) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        tmp_start_func = Some(start_func_idx as u32);

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // element section
      9 => {
        todo!()
      }
      // code section
      10 => {
        let (section_size, section_size_b) = decode_uleb128(&buf_src[(section_ofs + 1)..]);
        let (n_item, n_item_b) = decode_uleb128(&buf_src[(section_ofs + section_size_b + 1)..]);

        let mut item_ofs = section_ofs + section_size_b + n_item_b + 1;
        tmp_functions = (0..n_item)
          .map(|i| {
            let func_type_idx = tmp_function_types[i as usize];
            let (body_size, body_size_b) = decode_uleb128(&buf_src[item_ofs..]);
            let (n_local, n_local_b) = decode_uleb128(&buf_src[(item_ofs + body_size_b)..]);

            let mut local_ofs = item_ofs + body_size_b + n_local_b;
            let locals = (0..n_local)
              .map(|_| -> Result<Vec<_>, _> {
                let (n_type_count, n_type_count_b) = decode_uleb128(&buf_src[local_ofs..]);

                let valtype = ValType::try_from(buf_src[local_ofs + (n_type_count as usize)])
                  .map_err(|err| Error::from((local_ofs + (n_type_count as usize), ErrorKind::InvalidValue, err)))?;

                local_ofs += n_type_count_b + 1;

                Ok(iter::repeat(valtype).take(n_type_count as usize).collect())
              })
              .collect::<Result<Vec<_>, Error>>()?
              .into_iter()
              .flatten()
              .collect();

            let instr_ofs = if local_ofs == 0 { local_ofs + 1 } else { local_ofs };
            let parsed_body = parse_func_body(buf_src, instr_ofs)?;

            item_ofs = finalize_section(item_ofs, body_size, body_size_b);

            Ok(Function {
              signature_idx: func_type_idx as u32,
              locals,
              parsed_body,
            })
          })
          .collect::<Result<_, Error>>()?;

        finalize_section(section_ofs, section_size, section_size_b)
      }
      // data section
      11 => {
        todo!()
      }
      // data count section
      12 => {
        todo!()
      }
      _ => {
        return Err(Error::from((
          section_ofs,
          ErrorKind::InvalidSectionFormat,
          format!("invalid section id {}", buf_src[section_ofs])
        )));
      }
    }
  }

  Ok(Module {
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

fn parse_func_body(src_bin: &[u8], code_ofs: usize) -> Result<ParsedBody, Error> {
  let mut instr_ofs = code_ofs;
  let mut instrs = vec![];

  loop {
    let (instr, instr_b) = match src_bin[instr_ofs] {
      0x00 => (Instr::Unreachable, 1),
      0x01 => (Instr::Nop, 1),
      0x0F => (Instr::Return, 1),

      0x1A => (Instr::Drop, 1),
      0x1B => (Instr::Select(vec![]), 1),
      0x1C => (Instr::Select(vec![todo!()]), 1 /* TODO: */),

      0x41 => {
        let (val, val_b) = decode_sleb128(&src_bin[(instr_ofs + 1)..]);
        (Instr::I32Const(val as i32), 1 + val_b)
      },
      0x42 => {
        let (val, val_b) = decode_sleb128(&src_bin[(instr_ofs + 1)..]);
        (Instr::I64Const(val), 1 + val_b)
      },
      0x43 => {
        let arr: [u8; 4] = src_bin[(instr_ofs + 1)..(instr_ofs + 1 + 4)].try_into().unwrap();
        let val = f32::from_le_bytes(arr);
        (Instr::F32Const(val), 1 + 4)
      }
      0x44 => {
        let arr: [u8; 8] = src_bin[(instr_ofs + 1)..(instr_ofs + 1 + 8)].try_into().unwrap();
        let val = f64::from_le_bytes(arr);
        (Instr::F64Const(val), 1 + 4)
      }

      0x45 => (Instr::I32Eqz, 1),
      0x46 => (Instr::I32Eq, 1),
      0x47 => (Instr::I32Ne, 1),
      0x48 => (Instr::I32LtS, 1),
      0x49 => (Instr::I32LtU, 1),
      0x4A => (Instr::I32GtS, 1),
      0x4B => (Instr::I32GtU, 1),
      0x4C => (Instr::I32LeS, 1),
      0x4D => (Instr::I32LeU, 1),
      0x4E => (Instr::I32GeS, 1),
      0x4F => (Instr::I32GeU, 1),

      0x50 => (Instr::I64Eqz, 1),
      0x51 => (Instr::I64Eq, 1),
      0x52 => (Instr::I64Ne, 1),
      0x53 => (Instr::I64LtS, 1),
      0x54 => (Instr::I64LtU, 1),
      0x55 => (Instr::I64GtS, 1),
      0x56 => (Instr::I64GtU, 1),
      0x57 => (Instr::I64LeS, 1),
      0x58 => (Instr::I64LeU, 1),
      0x59 => (Instr::I64GeS, 1),
      0x5A => (Instr::I64GeU, 1),

      0x5B => (Instr::F32Eq, 1),
      0x5C => (Instr::F32Ne, 1),
      0x5D => (Instr::F32Lt, 1),
      0x5E => (Instr::F32Gt, 1),
      0x5F => (Instr::F32Le, 1),
      0x60 => (Instr::F32Ge, 1),

      0x61 => (Instr::F64Eq, 1),
      0x62 => (Instr::F64Ne, 1),
      0x63 => (Instr::F64Lt, 1),
      0x64 => (Instr::F64Gt, 1),
      0x65 => (Instr::F64Le, 1),
      0x66 => (Instr::F64Ge, 1),

      0x67 => (Instr::I32Clz, 1),
      0x68 => (Instr::I32Ctz, 1),
      0x69 => (Instr::I32Popcnt, 1),
      0x6A => (Instr::I32Add, 1),
      0x6B => (Instr::I32Sub, 1),
      0x6C => (Instr::I32Mul, 1),
      0x6D => (Instr::I32DivS, 1),
      0x6E => (Instr::I32DivU, 1),
      0x6F => (Instr::I32RemS, 1),
      0x70 => (Instr::I32RemU, 1),
      0x71 => (Instr::I32And, 1),
      0x72 => (Instr::I32Or, 1),
      0x73 => (Instr::I32Xor, 1),
      0x74 => (Instr::I32Shl, 1),
      0x75 => (Instr::I32ShrS, 1),
      0x76 => (Instr::I32ShrU, 1),
      0x77 => (Instr::I32Rotl, 1),
      0x78 => (Instr::I32Rotr, 1),

      0x79 => (Instr::I64Clz, 1),
      0x7A => (Instr::I64Ctz, 1),
      0x7B => (Instr::I64Popcnt, 1),
      0x7C => (Instr::I64Add, 1),
      0x7D => (Instr::I64Sub, 1),
      0x7E => (Instr::I64Mul, 1),
      0x7F => (Instr::I64DivS, 1),
      0x80 => (Instr::I64DivU, 1),
      0x81 => (Instr::I64RemS, 1),
      0x82 => (Instr::I64RemU, 1),
      0x83 => (Instr::I64And, 1),
      0x84 => (Instr::I64Or, 1),
      0x85 => (Instr::I64Xor, 1),
      0x86 => (Instr::I64Shl, 1),
      0x87 => (Instr::I64ShrS, 1),
      0x88 => (Instr::I64ShrU, 1),
      0x89 => (Instr::I64Rotl, 1),
      0x8A => (Instr::I64Rotr, 1),

      0x8B => (Instr::F32Abs, 1),
      0x8C => (Instr::F32Neg, 1),
      0x8D => (Instr::F32Ceil, 1),
      0x8E => (Instr::F32Floor, 1),
      0x8F => (Instr::F32Trunc, 1),
      0x90 => (Instr::F32Nearest, 1),
      0x91 => (Instr::F32Sqrt, 1),
      0x92 => (Instr::F32Add, 1),
      0x93 => (Instr::F32Sub, 1),
      0x94 => (Instr::F32Mul, 1),
      0x95 => (Instr::F32Div, 1),
      0x96 => (Instr::F32Min, 1),
      0x97 => (Instr::F32Max, 1),
      0x98 => (Instr::F32Copysign, 1),

      0x99 => (Instr::F64Abs, 1),
      0x9A => (Instr::F64Neg, 1),
      0x9B => (Instr::F64Ceil, 1),
      0x9C => (Instr::F64Floor, 1),
      0x9D => (Instr::F64Trunc, 1),
      0x9E => (Instr::F64Nearest, 1),
      0x9F => (Instr::F64Sqrt, 1),
      0xA0 => (Instr::F64Add, 1),
      0xA1 => (Instr::F64Sub, 1),
      0xA2 => (Instr::F64Mul, 1),
      0xA3 => (Instr::F64Div, 1),
      0xA4 => (Instr::F64Min, 1),
      0xA5 => (Instr::F64Max, 1),
      0xA6 => (Instr::F64Copysign, 1),

      0x0B => break,
      _ => return Err(Error::from((
        instr_ofs,
        ErrorKind::InvalidInstruction,
        format!("invalid instruction code {}", src_bin[instr_ofs])
      )))
    };

    instr_ofs += instr_b;

    instrs.push(instr);
  }

  Ok(ParsedBody::new(instrs))
}

fn parse_expr(src_bin: &[u8], code_ofs: usize) -> Result<(), Error> {
  todo!()
}
