use crate::{module::{function::Function, import::Import, memory::Memory32, types::Type}, parse::{parse, Error}};

pub struct ComponentInstance<'a> {
  pub(crate) src_bin: &'a [u8],
  pub(crate) types: Vec<Type<'a>>,
  pub(crate) imports: Vec<Import>,
  pub(crate) memories: Vec<Memory32>,
  pub(crate) functions: Vec<Function<'a>>
}

impl<'a> ComponentInstance<'a> {
  pub fn new(src_bin: &'a [u8]) -> Result<Self, Error> {
    parse(src_bin)
  }

  pub fn run(&mut self) {

  }
}
