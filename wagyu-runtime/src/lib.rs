// #![no_std]

use instance::{
  ImportObject,
  ModuleInstance,
};
use module::Module;

#[macro_use]
extern crate alloc;

pub mod executor;
pub mod instance;
pub mod instr;
pub mod module;
pub mod parse;
pub mod stack;
pub mod wasi;

pub(crate) mod helper;

pub fn instantiate<'a>(buf_src: &'a [u8], import_obj: ImportObject<'a>) -> Result<ModuleInstance<'a>, parse::Error> {
  let module = parse::parse(buf_src)?;

  Ok(ModuleInstance::new(module, import_obj))
}

pub fn compile(buf_src: &[u8]) -> Result<Module, parse::Error> {
  parse::parse(buf_src)
}

pub fn validate(buf_src: &[u8]) -> bool {
  parse::parse(buf_src).is_ok()
}
