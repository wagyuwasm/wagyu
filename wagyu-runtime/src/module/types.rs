use super::value::ValType;

#[derive(Debug, Clone)]
pub(crate) struct Type {
  pub(crate) params: Vec<ValType>,
  pub(crate) results: Vec<ValType>,
}