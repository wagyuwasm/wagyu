use alloc::vec::Vec;

use crate::{module::{export::Export, function::Function, global::Global, import::Import, memory::Memory32, table::Table, types::Type, value::FuncIdx}, parse::{parse, Error}};

pub struct ModuleInstance {
  pub(crate) types: Vec<Type>,
  pub(crate) imports: Vec<Import>,
  pub(crate) functions: Vec<Function>,
  pub(crate) tables: Vec<Table>,
  pub(crate) memories: Vec<Memory32>,
  pub(crate) globals: Vec<Global>,
  pub(crate) exports: Vec<Export>,
  pub(crate) start_func: Option<FuncIdx>,
}

impl ModuleInstance {
  pub fn new(src_bin: &[u8]) -> Result<Self, Error> {
    parse(src_bin)
  }

  pub fn run_start(&mut self) {

  }
}
