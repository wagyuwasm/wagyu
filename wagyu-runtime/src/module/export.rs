extern crate alloc;

use alloc::string::String;

use super::value::ExportDesc;

#[derive(Debug)]
pub(crate) struct Export {
  pub(crate) name: String,
  pub(crate) desc: ExportDesc,
  pub(crate) idx: u32,
}
