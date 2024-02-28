use alloc::vec::Vec;

/// Encodes an unsigned integer into a variable-length little-endian base 128 (LEB128) representation.
///
/// # Arguments
///
/// * `value` - The value to be encoded, which must implement `Into<u64>`.
///
/// # Returns
///
/// A vector of bytes representing the encoded LEB128 value.
///
/// # Example
///
/// ```
/// use wagyu_runtime::helper::leb128::encode_uleb128;
///
/// let value: u64 = 128;
/// let encoded = encode_uleb128(value);
/// assert_eq!(encoded, vec![0x80, 0x01]);
/// ```
pub(crate) fn encode_uleb128<T: Into<u64>>(value: T) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();
  let mut value: u64 = value.into();

  loop {
    let mut byte = (value & 0x7F) as u8;
    value >>= 7;
    if value != 0 {
      byte |= 0x80; // Set the continuation bit
    }
    bytes.push(byte);
    if value == 0 {
      break;
    }
  }

  bytes
}

/// Decodes a variable-length little-endian base 128 (LEB128) representation into an unsigned integer.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes representing the LEB128 encoded value.
///
/// # Returns
///
/// A tuple containing the decoded value as `u64` and the count of read bytes as `usize`.
///
/// # Example
///
/// ```
/// use wagyu_runtime::helper::leb128::decode_uleb128;
///
/// let encoded: Vec<u8> = vec![0x80, 0x01];
/// let (decoded, count) = decode_uleb128(&encoded);
/// assert_eq!(decoded, 128);
/// assert_eq!(count, 2);
/// ```
pub(crate) fn decode_uleb128(bytes: &[u8]) -> (u64, usize) {
  let mut result: u64 = 0;
  let mut shift = 0;
  let mut count = 0;

  for byte in bytes.iter() {
    result |= ((*byte & 0x7F) as u64) << shift;
    count += 1;
    if *byte & 0x80 == 0 {
      break;
    }
    shift += 7;
  }

  (result, count)
}

/// Encodes a signed 64-bit integer into a byte vector using Signed Little-Endian Base 128 (LEB128) encoding.
///
/// # Arguments
///
/// * `value` - The signed 64-bit integer to be encoded.
///
/// # Returns
///
/// A vector of bytes containing the LEB128 encoded representation of the input value.
///
/// # Examples
///
/// ```
/// use wagyu_runtime::helper::leb128::encode_sleb128;
///
/// let value: i64 = -624485;
/// let encoded = encode_sleb128(value);
/// assert_eq!(encoded, vec![0x9B, 0xF1, 0x59]);
/// ```
pub(crate) fn encode_sleb128<T: Into<i64>>(value: T) -> Vec<u8> {
  let mut bytes = Vec::new();
  let mut value: i64 = value.into();

  loop {
    let mut byte = (value as u8) & 0x7F;
    value >>= 7;

    let more = !((((value == 0) && ((byte & 0x40) == 0)) || ((value == -1) && ((byte & 0x40) != 0))) as bool) as u8;

    byte |= more << 7;
    bytes.push(byte);

    if more == 0 {
      break;
    }
  }

  bytes
}

/// Decodes a byte slice containing LEB128 encoded signed integers into its corresponding `i64` value
/// along with the count of read bytes.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes containing LEB128 encoded signed integers.
///
/// # Returns
///
/// A tuple containing the decoded `i64` value and the count of read bytes.
///
/// # Examples
///
/// ```
/// use wagyu_runtime::helper::leb128::decode_sleb128;
///
/// let encoded: Vec<u8> = vec![0x9B, 0xF1, 0x59]; // Example encoded bytes
/// let (decoded, count) = decode_sleb128(&encoded);
/// assert_eq!(decoded, -624485);
/// assert_eq!(count, 3);
/// ```
pub(crate) fn decode_sleb128(bytes: &[u8]) -> (i64, usize) {
  let mut result: i64 = 0;
  let mut shift: usize = 0;
  let mut count: usize = 0;

  for byte in bytes.iter() {
    let value = (*byte & 0x7F) as i64;
    result |= value << shift;
    shift += 7;
    count += 1;
    if (*byte & 0x80) == 0 {
      break;
    }
  }

  // Sign extend the result if the last byte has its sign bit set
  if (bytes[count - 1] & 0x40) != 0 {
    result |= -(1 << shift);
  }

  (result, count)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_encode_unsigned_leb128() {
    let test_cases = vec![(0u64, vec![0]), (127u64, vec![0x7f]), (128u64, vec![0x80, 0x01])];

    for (input, expected_output) in test_cases {
      let encoded = encode_uleb128(input);
      assert_eq!(encoded, expected_output);
    }
  }

  #[test]
  fn test_decode_unsigned_leb128() {
    let test_cases = vec![
      (vec![0], (0u64, 1usize)),
      (vec![0x7f], (127u64, 1usize)),
      (vec![0x80, 0x01], (128u64, 2usize)),
      (vec![0x80, 0x01, 0x03], (128u64, 2usize)),
      (vec![0x80, 0x01, 0x80, 0x01], (128u64, 2usize)),
    ];

    for (input, (expected_value, expected_count)) in test_cases {
      let (decoded_value, decoded_count) = decode_uleb128(&input);
      assert_eq!(decoded_value, expected_value);
      assert_eq!(decoded_count, expected_count);
    }
  }

  #[test]
  fn test_encode_signed_leb128() {
    let test_cases = vec![
      (0i64, vec![0]),
      (1i64, vec![0x01]),
      (63i64, vec![0x3f]),
      (64i64, vec![0xc0, 0x00]),
      (-1i64, vec![0x7f]),
      (-65i64, vec![0xbf, 0x7f]),
      (-624485i64, vec![0x9B, 0xF1, 0x59]),
    ];

    for (input, expected_output) in test_cases {
      let encoded = encode_sleb128(input);
      assert_eq!(encoded, expected_output);
    }
  }

  #[test]
  fn test_decode_signed_leb128() {
    let test_cases = vec![
      (vec![0], (0i64, 1usize)),
      (vec![0x01], (1i64, 1usize)),
      (vec![0x3f], (63i64, 1usize)),
      (vec![0xc0, 0x00], (64i64, 2usize)),
      (vec![0xc0, 0x00, 0x7c], (64i64, 2usize)),
      (vec![0x7f], (-1i64, 1usize)),
      (vec![0xbf, 0x7f], (-65i64, 2usize)),
      (vec![0x9B, 0xF1, 0x59], (-624485i64, 3usize)),
    ];

    for (input, (expected_value, expected_count)) in test_cases {
      let (decoded_value, decoded_count) = decode_sleb128(&input);
      assert_eq!(decoded_value, expected_value);
      assert_eq!(decoded_count, expected_count);
    }
  }
}
