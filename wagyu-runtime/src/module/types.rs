use super::value::ValType;

pub(crate) struct Type {
  pub(crate) params: Vec<ValType>,
  pub(crate) results: Vec<ValType>,
}