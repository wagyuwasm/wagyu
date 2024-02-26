use crate::module::value::Value;

pub struct Stack {
  operand: OperandStack,
  control: ControlStack,
  call: CallStack
}

pub struct OperandStack {
  items: Vec<Value>
}

pub struct ControlStack {

}

pub struct CallStack {

}