use crate::{function::Function, parse::{parse, Error}, types::Type};

pub struct ComponentInstance<'a> {
  pub(crate) src_bin: &'a [u8],
  pub(crate) types: Vec<Type<'a>>,
  pub(crate) functions: Vec<Function<'a>>
}

impl<'a> ComponentInstance<'a> {
  pub fn new(src_bin: &'a [u8]) -> Result<Self, Error> {
    parse(src_bin)
  }

  pub fn run(&mut self) {

  }
}
