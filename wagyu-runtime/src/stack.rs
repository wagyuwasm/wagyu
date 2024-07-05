use crate::module::value::Value;

pub struct Stack {
  operand: OperandStack,
  control: ControlStack,
  call: CallStack,
}

pub struct OperandStack {
  len: usize,
  stack: Vec<Value>,
}

impl OperandStack {
  pub fn new() -> Self {
    Self {
      len: 0,
      stack: vec![]
    }
  }
}

pub struct ControlStack {
  
}

pub struct CallStack {}
