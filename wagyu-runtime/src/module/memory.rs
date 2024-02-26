const PAGE_SIZEL: u32 = 65_536;

pub(crate) struct Memory32 {
  pub(crate) min: u32,
  pub(crate) max: Option<u32>
}

pub(crate) struct Memory64 {
  pub(crate) min: u64,
  pub(crate) max: Option<u64>
}

impl Memory32 {
  pub(crate) fn allocate(page_count: usize) {

  }
}