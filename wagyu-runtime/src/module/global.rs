use super::value::{GlobalMut, ValType, Value};

pub(crate) struct Global {
  pub(crate) kind: GlobalMut,
  pub(crate) valtype: ValType,
  pub(crate) value: Option<Value>
}

impl Global {
  pub(crate) fn get(&self) -> Value {
    match self.value.as_ref() {
      Some(v) => v.clone(),
      None => panic!("cannot get uninitialized global")
    }
  }

  pub(crate) fn set(&mut self, val: Value) {
    if matches!(self.kind, GlobalMut::Const) {
      panic!("cannot set const global");
    }

    self.value = match (&self.valtype, val) {
      (ValType::I32, Value::I32(v)) => Some(Value::I32(v)),
      (ValType::I64, Value::I64(v)) => Some(Value::I64(v)),
      (ValType::F32, Value::F32(v)) => Some(Value::F32(v)),
      (ValType::F64, Value::F64(v)) => Some(Value::F64(v)),
      (ValType::V128, Value::V128(v)) => Some(Value::V128(v)),
      (ValType::FuncRef, Value::FuncRef(v)) => Some(Value::FuncRef(v)),
      (ValType::ExternRef, Value::ExternRef(v)) => Some(Value::ExternRef(v)),
      _ => panic!("incompatible type")
    };
  }
}