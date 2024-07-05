use alloc::alloc::{
  alloc,
  dealloc,
  handle_alloc_error,
  realloc,
  Layout,
};
use core::ptr;

const PAGE_SIZE: usize = 65_536;
const ALIGN: usize = 4;

#[derive(Debug)]
pub(crate) struct Memory32 {
  /// Pointer to the start of the block of the memory.
  pub(crate) ptr: *mut u8,
  /// Allocated page size of the memory.
  pub(crate) size: u32,
  /// Initial allocating size.
  pub(crate) initial: u32,
  /// Maximum allocating size.
  pub(crate) max: Option<u32>,
}

impl Memory32 {
  pub(crate) fn alloc(&mut self) {
    if !self.ptr.is_null() {
      panic!("memory is initially allocated");
    }

    let layout = Layout::from_size_align((self.initial as usize) * PAGE_SIZE, ALIGN).unwrap();
    let ptr = unsafe { alloc(layout) };
    if self.ptr.is_null() {
      handle_alloc_error(layout);
    }

    self.ptr = ptr;
    self.size = self.initial;
  }

  /// Returns the current size of a memory.
  /// The function operates in units of page size.
  pub(crate) fn size(&self) -> (i32,) {
    (self.size as i32,)
  }

  /// Grows memory by a given delta and returns the previous size,
  /// or -1 if enough memory cannot be allocated.
  /// The function operates in units of page size.
  pub(crate) fn grow(&mut self, (delta,): (i32,)) -> (i32,) {
    // FIXME: should handle a nagative number?

    let old_size = self.size;
    let new_size = self.size + (delta as u32);

    if new_size > self.max.unwrap_or_else(|| PAGE_SIZE as u32) || new_size < self.initial {
      return (-1,);
    }

    let layout = Layout::from_size_align((old_size as usize) * PAGE_SIZE, ALIGN).unwrap();
    let new_ptr = unsafe { realloc(self.ptr, layout, new_size as usize) };
    if new_ptr.is_null() {
      handle_alloc_error(layout);
    }

    self.ptr = new_ptr;
    self.size = new_size;

    (old_size as i32,)
  }

  /// Sets all values in a region to a given byte.
  ///
  /// # Arguments
  ///
  /// * `dst` - Destination address.
  /// * `val` - Byte value to set.
  /// * `n` - Size of memory region in bytes.
  ///
  /// # Panics
  ///
  /// Panics when the destination offset plus size is greater than the length of the target memory.
  pub(crate) fn fill(&mut self, (dst, val, n): (i32, i32, i32)) {
    unsafe {
      if (dst + n) as usize > ((self.size as usize) * PAGE_SIZE) {
        panic!("memory access out of bounds");
      }

      ptr::write_bytes(self.ptr.add(dst as usize), val as u8, n as usize);
    }
  }

  /// Copies data from a source memory region to a possibly overlapping destination region.
  ///
  /// # Arguments
  ///
  /// * `dst` - Destination address.
  /// * `src` - Source address.
  /// * `n` - Size of memory region in bytes.
  ///
  /// # Panics
  ///
  /// * When the source offset plus size is greater than the length of the source memory.
  /// * When the destination offset plus size is greater than the length of the target memory.
  pub(crate) fn copy(&mut self, (dst, src, n): (i32, i32, i32)) {
    unsafe {
      if (dst + n) as usize > ((self.size as usize) * PAGE_SIZE) {
        panic!("memory access out of bounds");
      }
      if (src + n) as usize > ((self.size as usize) * PAGE_SIZE) {
        panic!("memory access out of bounds");
      }

      ptr::copy(self.ptr.add(src as usize), self.ptr.add(dst as usize), n as usize);
    }
  }

  /// Copies data from a passive data segment into a memory.
  ///
  /// * `dst` - Destination address.
  /// * `src` - Offset into the source segment.
  /// * `n` - Size of memory region in bytes.
  ///
  /// # Panics
  ///
  /// Panics when the destination offset plus size is greater than the length of the target memory.
  pub(crate) fn init(&mut self, data: &str, (dst, src, n): (i32, i32, i32)) {
    unsafe {
      if (dst + n) as usize > ((self.size as usize) * PAGE_SIZE) {
        panic!("memory access out of bounds");
      }
      if (src + n) as usize > data.len() {
        panic!("memory access out of bounds");
      }

      let data_ptr = data[(src as usize)..].as_ptr();
      ptr::copy(data_ptr, self.ptr.add(dst as usize), n as usize);
    }
  }
}

impl Drop for Memory32 {
  fn drop(&mut self) {
    if self.ptr.is_null() {
      return;
    }

    let layout = Layout::from_size_align((self.size as usize) * PAGE_SIZE, ALIGN).unwrap();

    unsafe { dealloc(self.ptr, layout) }
  }
}
