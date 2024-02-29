use alloc::vec::Vec;

use super::value::ValType;

#[derive(Debug)]
pub(crate) struct Type {
  pub(crate) params: Vec<ValType>,
  pub(crate) results: Vec<ValType>,
}
