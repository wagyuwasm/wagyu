use alloc::vec::Vec;

use crate::module::{
  export::Export,
  function::Function,
  global::Global,
  import::Import,
  memory::Memory32,
  table::Table,
  types::Type,
  value::FuncIdx,
};

pub mod custom;
pub mod data;
pub mod elem;
pub mod export;
pub mod function;
pub mod global;
pub mod import;
pub mod memory;
pub mod table;
pub mod types;
pub mod value;

#[derive(Debug)]
pub struct Module {
  pub(crate) types: Vec<Type>,
  pub(crate) imports: Vec<Import>,
  pub(crate) functions: Vec<Function>,
  pub(crate) tables: Vec<Table>,
  pub(crate) memories: Vec<Memory32>,
  pub(crate) globals: Vec<Global>,
  pub(crate) exports: Vec<Export>,
  pub(crate) start_func: Option<FuncIdx>,
}

impl Module {
  pub fn custom_sections(module: Self) {
    todo!()
  }

  pub fn exports(module: Self) -> Vec<Export> {
    todo!()
  }

  pub fn imports(module: Self) -> Vec<Import> {
    todo!()
  }
}