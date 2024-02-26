use super::value::ValType;

#[derive(Debug, Clone)]
pub(crate) struct Type<'a> {
  pub(crate) params: Vec<ValType>,
  pub(crate) results: Vec<ValType>,
  pub(crate) raw: &'a [u8],
}