use alloc::string::String;

pub(crate) type TypeIdx = u32;
pub(crate) type FuncIdx = u32;
pub(crate) type TableIdx = u32;
pub(crate) type MemIdx = u32;
pub(crate) type GlobalIdx = u32;
pub(crate) type ElemIdx = u32;
pub(crate) type DataIdx = u32;
pub(crate) type LocalIdx = u32;
pub(crate) type LabelIdx = u32;

pub(crate) enum ValType {
  I32,
  I64,
  F32,
  F64,
  V128,
  FuncRef,
  ExternRef
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
  V128(u128),
  FuncRef(u32),
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
      0x7B => Ok(Self::V128),
      0x70 => Ok(Self::FuncRef),
      0x6F => Ok(Self::ExternRef),
      _ => Err(format!("invalid valtype"))
    }
  }
}

pub(crate) enum RefType {
  FuncRef,
  ExternRef
}

pub(crate) struct Limit(u32, Option<u32>);

pub(crate) enum V128ConstValue {
  I8X16([i8; 16]),
  I16X8([i16; 8]),
  I32X4([i32; 4]),
  I64X2([i64; 2]),
  F32X4([f32; 4]),
  F64X2([f64; 2]),
}

pub(crate) enum HeapType {
  Func,
  Extern
}

pub(crate) enum ExportDesc {
  FuncIdx,
  TableIdx,
  MemIdx,
  GlobalIdx
}

impl TryFrom<u8> for ExportDesc {
  type Error = String;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x00 => Ok(Self::FuncIdx),
      0x01 => Ok(Self::TableIdx),
      0x02 => Ok(Self::MemIdx),
      0x03 => Ok(Self::GlobalIdx),
      _ => Err(format!("invalid export kind"))
    }
  }
}

pub(crate) enum GlobalMut {
  Const,
  Var,
}

impl TryFrom<u8> for GlobalMut {
  type Error = String;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x00 => Ok(Self::Const),
      0x01 => Ok(Self::Var),
      _ => Err(format!("invalid global mut"))
    }
  }
}