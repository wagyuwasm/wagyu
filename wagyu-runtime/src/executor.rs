use crate::stack::Stack;

pub enum Error {
  InvalidLED128Format,
  OutOfBoundMemoryAccess,
  InvalidInstruction,
  MemoryExaustion,
  StackOverflow
}

fn execute(func_body: &[u8], stack: &mut Stack) -> Result<(), Error> {
  let mut pc: u32 = 0;

  Ok(())
}