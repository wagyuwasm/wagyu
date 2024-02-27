pub(crate) struct Table {}

impl Table {
  /// Loads an element in a table.
  pub(crate) fn get(&self, table_idx: u32) {}

  /// Stores an element in a table.
  pub(crate) fn set(&self, table_idx: u32) {}

  /// Returns the current size of a table.
  pub(crate) fn size(&self) {}

  /// Grows table by a given delta and returns the previous size,
  /// or -1 if enough space cannot be allocated.
  /// It also takes an initialization value for the newly allocated entries.
  pub(crate) fn grow(&self) {}

  /// Sets all entries in a range to a given value.
  pub(crate) fn fill(&self) {}

  /// Copies elements from a source table region to a possibly overlapping destination region; the first index denotes the destination.
  pub(crate) fn copy(&self) {}

  /// Copies elements from a passive element segment into a table.
  pub(crate) fn init(&self) {}

  /// Prevents further use of a passive element segment. This instruction is intended to be used as an optimization hint.
  /// After an element segment is dropped its elements can no longer be retrieved, so the memory used by this segment may be freed.
  pub(crate) fn drop(&self) {}
}
