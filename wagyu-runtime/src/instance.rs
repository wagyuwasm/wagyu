use crate::module::{value::Value, Module};

pub(crate) type ImportObject<'a> = &'a [(&'a str, &'a [(&'a str, Value)])];

pub(crate) struct ModuleInstance<'a> {
  import_obj: ImportObject<'a>,
  module: Module,
}

impl<'a> ModuleInstance<'a> {
  pub(crate) const fn new(module: Module, import_obj: ImportObject<'a>) -> Self {
    Self {
      import_obj,
      module
    }
  }

  pub(crate) fn run_start(&mut self) {}
}
