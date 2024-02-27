use super::value::TypeIdx;

pub(crate) struct Function {
  pub(crate) signature_idx: TypeIdx,
  pub(crate) parsed_body: Option<ParsedBody>,
}

pub(crate) struct ParsedBody {}
