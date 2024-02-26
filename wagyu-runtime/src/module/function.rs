pub(crate) struct Function<'a> {
  pub(crate) signature_idx: u32,
  /// Function body in a binary format, not including body size and fixup size.
  pub(crate) raw_body: &'a [u8],
  pub(crate) parsed_body: Option<ParsedBody>
}

pub(crate) struct ParsedBody {

}