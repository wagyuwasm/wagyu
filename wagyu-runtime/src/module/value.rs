use core::str;

#[derive(Debug, Clone)]
pub(crate) enum ValType {
  I32,
  I64,
  F32,
  F64,
  Vec,
  FucnRef,
  ExternRef
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
  Vec(u128),
  FucnRef(u32),
  ExternRef(u32)
}

impl TryFrom<u8> for ValType {
  type Error = String;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x7F => Ok(Self::I32),
      0x7E => Ok(Self::I64),
      0x7D => Ok(Self::F32),
      0x7C => Ok(Self::F64),
      _ => Err(format!("invalid numtype value"))
    }
  }
}

pub(crate) enum RefType {
  FuncRef,
  ExternRef
}

pub(crate) struct Limit(u32, Option<u32>);

pub(crate) enum GlobalType {
  Const,
  Var
}