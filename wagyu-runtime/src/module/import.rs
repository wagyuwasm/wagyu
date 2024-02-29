use alloc::string::String;

use super::value::{
  GlobalMut,
  Limit,
  RefType,
  ValType,
};

#[derive(Debug)]
pub(crate) struct Import {
  pub(crate) module_name: String,
  pub(crate) field_name: String,
  pub(crate) kind: ImportKind,
}

#[derive(Debug)]
pub(crate) enum ImportKind {
  TypeIdx(u32),
  TableType(RefType, Limit),
  MemType(Limit),
  GlobalType(ValType, GlobalMut),
}
