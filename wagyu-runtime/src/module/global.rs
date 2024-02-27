use super::value::{GlobalType, Value};

pub(crate) struct Global {
  kind: GlobalType,
  value: Value
}

impl Global {
  pub(crate) fn get(&self) -> Value {
    self.value.clone()
  }

  pub(crate) fn set(&mut self, val: Value) {
    self.value = val;
  }
}