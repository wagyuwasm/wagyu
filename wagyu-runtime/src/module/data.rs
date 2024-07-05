use super::value::DataMode;

#[derive(Debug)]
pub struct Data {
  pub(crate) mode: DataMode,
  pub(crate) data: String,
}

impl Data {
  /// Prevents further use of a passive data segment. This instruction is intended to be used as an optimization hint.
  /// After a data segment is dropped its data can no longer be retrieved, so the memory used by this segment may be freed.
  pub(crate) fn drop(&mut self) {
    self.data.drain(..);
  }
}
