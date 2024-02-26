const PAGE_SIZEL: u32 = 65_536;

pub(crate) struct Memory32 {
  pub(crate) mem: Vec<u8>,
  pub(crate) current_size: u32,
  pub(crate) min: u32,
  pub(crate) max: Option<u32>
}

pub(crate) struct Memory64 {
  pub(crate) min: u64,
  pub(crate) max: Option<u64>
}

impl Memory32 {
  pub(crate) fn allocate(&mut self, page_count: usize) {

  }

  pub(crate) fn size(&self) {

  }

  pub(crate) fn grow(&self) {

  }

  pub(crate) fn fill(&self) {

  }

  pub(crate) fn copy(&self) {

  }

  pub(crate) fn init(&self, data_idx: u32) {

  }
}

impl Memory32 {
  pub(crate) fn data_drop(&mut self, data_idx: u32) {

  }
}