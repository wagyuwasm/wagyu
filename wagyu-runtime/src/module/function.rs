use alloc::vec::Vec;

use crate::instr::Instr;

use super::value::{TypeIdx, ValType};

pub(crate) struct Function {
  pub(crate) signature_idx: TypeIdx,
  pub(crate) locals: Vec<ValType>,
  pub(crate) parsed_body: ParsedBody,
}

pub(crate) struct ParsedBody {
  instrs: Vec<Instr>,
}

impl ParsedBody {
  pub const fn new(instrs: Vec<Instr>) -> Self {
    Self { instrs }
  }
}