use core::fmt;

use crate::stack::Stack;

pub enum Error {
  OutOfBoundMemoryAccess,
  MemoryExhaustion,
  StackOverflow,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::OutOfBoundMemoryAccess => write!(f, "Runtime error: memory access out of bounds"),
      Self::MemoryExhaustion => write!(f, "Runtime error: memory exhausion"),
      Self::StackOverflow => write!(f, "Runtime error: stack overflow")
    }
  }
}

fn execute(func_body: &[u8], stack: &mut Stack) -> Result<(), Error> {
  let mut pc: u32 = 0;

  Ok(())
}
