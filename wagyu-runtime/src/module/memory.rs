use std::alloc::{alloc, realloc, handle_alloc_error, Layout};
use std::ptr;

const PAGE_SIZE: usize = 65_536;
const ALIGN: usize = 4;

pub(crate) struct Memory32 {
  /// Pointer to the start of the block of the memory.
  pub(crate) ptr: *mut u8,
  /// Allocated page size of the memory.
  pub(crate) size: u32,
  /// Minimum allocating size.
  pub(crate) min: u32,
  /// Maximum allocating size.
  pub(crate) max: Option<u32>
}

impl Memory32 {
  pub(crate) fn alloc(&mut self) {
    if !self.ptr.is_null() {
      panic!("memory is initially allocated");
    }

    let layout = Layout::from_size_align((self.min as usize) * PAGE_SIZE, ALIGN).unwrap();
    let ptr = unsafe { alloc(layout) };
    if self.ptr.is_null() {
      handle_alloc_error(layout);
    }

    self.ptr = ptr;
    self.size = self.min;
  }

  /// Returns the current size of a memory.
  /// The function operates in units of page size.
  pub(crate) fn size(&self) -> i32 {
    self.size as i32
  }

  /// Grows memory by a given delta and returns the previous size,
  /// or -1 if enough memory cannot be allocated.
  /// The function operates in units of page size.
  pub(crate) fn grow(&mut self, delta: i32) -> i32 {
    // FIXME: should handle a nagative number?

    let old_size = self.size;
    let new_size = self.size + (delta as u32);

    if new_size > self.max.unwrap_or_else(|| PAGE_SIZE as u32) || new_size < self.min {
      return -1;
    }

    let layout = Layout::from_size_align((old_size as usize) * PAGE_SIZE, ALIGN).unwrap();
    let new_ptr = unsafe { realloc(self.ptr, layout, new_size as usize) };
    if new_ptr.is_null() {
      handle_alloc_error(layout);
    }

    self.ptr = new_ptr;
    self.size = new_size;

    old_size as i32
  }

  /// Sets all values in a region to a given byte.
  pub(crate) fn fill(&mut self, start: i32, val: i32, n: i32) {
    todo!()
  }

  /// Copies data from a source memory region to a possibly overlapping destination region.
  pub(crate) fn copy(&mut self) {
    todo!()
  }

  /// Copies data from a passive data segment into a memory.
  pub(crate) fn init(&mut self) {
    todo!()
  }

  /// Prevents further use of a passive data segment. This instruction is intended to be used as an optimization hint.
  /// After a data segment is dropped its data can no longer be retrieved, so the memory used by this segment may be freed.
  pub(crate) fn drop(&mut self) {
    todo!()
    // unsafe {
    //   ptr::drop_in_place(self.ptr.add(data_idx.try_into().unwrap()));
    // }
  }
}
