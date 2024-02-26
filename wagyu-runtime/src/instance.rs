use crate::{module::{function::Function, import::Import, memory::Memory32, types::Type}, parse::{parse, Error}};

pub struct ComponentInstance {
  pub(crate) types: Vec<Type>,
  pub(crate) imports: Vec<Import>,
  pub(crate) memories: Vec<Memory32>,
  pub(crate) functions: Vec<Function>
}

impl ComponentInstance {
  pub fn new(src_bin: &[u8]) -> Result<Self, Error> {
    parse(src_bin)
  }

  pub fn run_start(&mut self) {

  }
}
