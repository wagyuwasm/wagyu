use super::value::{GlobalType, Limit, RefType, ValType};

pub(crate) struct Import {
  pub(crate) module_name: String,
  pub(crate) field_name: String,
  pub(crate) kind: ImportKind
}

pub(crate) enum ImportKind {
  TypeIdx(u32),
  TableType(RefType, Limit),
  MemType(Limit),
  GlobalType(ValType, GlobalType),
}