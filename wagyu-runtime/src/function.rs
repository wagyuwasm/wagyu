pub(crate) struct Function<'a> {
  pub(crate) type_idx: u64,
  pub(crate) raw_body: &'a [u8],
  pub(crate) parsed_body: Option<ParsedBody>
}

pub(crate) struct ParsedBody {

}