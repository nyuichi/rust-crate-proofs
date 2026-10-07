#![allow(unexpected_cfgs)]

use bytes::verified::{cursor::Cursor, encoding_spec, exclusive::ExclusiveBytes};
use creusot_std::prelude::*;
use std::vec::Vec;

/// A downstream caller consumes the public byte model, writes through the
/// public method contract, reads both bytes, and closes the allocation.
#[ensures(result.0 != None && result.0.unwrap_logic()@ == value@ / 256)]
#[ensures(result.1 != None && result.1.unwrap_logic()@ == value@ % 256)]
pub fn external_be_u16_roundtrip(value: u16) -> (Option<u8>, Option<u8>) {
    proof_assert!(encoding_spec::model_be_bytes(value@, 2)[0] == value@ / 256);
    proof_assert!(encoding_spec::model_be_bytes(value@, 2)[1] == value@ % 256);

    let mut bytes = ExclusiveBytes::from_vec(Vec::new());
    bytes.write_u16_be(value);
    let first = bytes.get(0);
    let second = bytes.get(1);
    bytes.close();
    (first, second)
}

/// The same downstream composition for little-endian output.
#[ensures(result.0 != None && result.0.unwrap_logic()@ == value@ % 256)]
#[ensures(result.1 != None && result.1.unwrap_logic()@ == value@ / 256)]
pub fn external_le_u16_roundtrip(value: u16) -> (Option<u8>, Option<u8>) {
    proof_assert!(encoding_spec::model_le_bytes(value@, 2)[0] == value@ % 256);
    proof_assert!(encoding_spec::model_le_bytes(value@, 2)[1] == value@ / 256);

    let mut bytes = ExclusiveBytes::from_vec(Vec::new());
    bytes.write_u16_le(value);
    let first = bytes.get(0);
    let second = bytes.get(1);
    bytes.close();
    (first, second)
}

/// Checks whether the public checked variable-width Cursor contract can be
/// consumed by a downstream crate without access to private weight helpers.
#[ensures(match result.0 {
    Some(value) => input@.len() >= 2
        && value@ == input@[0]@ * 256 + input@[1]@
        && result.1@ == input@.len() - 2,
    None => input@.len() < 2 && result.1@ == input@.len()
})]
pub fn external_variable_u16_be(input: &[u8]) -> (Option<u64>, usize) {
    let mut cursor = Cursor::new(input);
    let value = cursor.try_get_uint_be(2);
    (value, cursor.remaining())
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn external_writer_models_match_bytes() {
        for value in [0, 1, 255, 256, 32768, u16::MAX] {
            let be = value.to_be_bytes();
            let le = value.to_le_bytes();
            assert_eq!(external_be_u16_roundtrip(value), (Some(be[0]), Some(be[1])));
            assert_eq!(external_le_u16_roundtrip(value), (Some(le[0]), Some(le[1])));
        }
    }
    #[test]
    fn external_reader_consumes_exact_width() {
        for input in [b"".as_slice(), b"a", b"ab", b"abc"] {
            let expected = if input.len() < 2 { None } else { Some(input[0] as u64 * 256 + input[1] as u64) };
            assert_eq!(external_variable_u16_be(input), (expected, input.len() - if expected.is_some() {2} else {0}));
        }
    }
}

/// Unfolds the public signed variable-width model from another crate.
#[ensures(match result {
    Some(value) => input@.len() >= 1 && value@ ==
        if input@[0]@ < 128 { input@[0]@ } else { input@[0]@ - 256 },
    None => input@.len() == 0
})]
pub fn external_signed_variable_byte(input: &[u8]) -> Option<i64> {
    Cursor::new(input).try_get_int_be(1)
}

/// Composes the fixed-width signed conversion definition with a public reader.
#[ensures(match result {
    Some(value) => input@.len() >= 2 && value@ ==
        if input@[0]@ < 128 { input@[0]@ * 256 + input@[1]@ }
        else { input@[0]@ * 256 + input@[1]@ - 65536 },
    None => input@.len() < 2
})]
pub fn external_signed_fixed_word(input: &[u8]) -> Option<i16> {
    Cursor::new(input).try_get_i16_be()
}

/// Every bit set denotes minus one at either native endianness.
#[requires(input@.len() == 16)]
#[requires(forall<i: Int> 0 <= i && i < 16 ==> input@[i]@ == 255)]
#[ensures(result == Some(-1i128))]
pub fn external_signed_wide_native(input: &[u8]) -> Option<i128> {
    Cursor::new(input).try_get_i128_ne()
}

#[cfg(all(test, not(creusot)))]
mod signed_tests {
    use super::*;
    #[test]
    fn downstream_signed_models_match_values() {
        assert_eq!(external_signed_variable_byte(&[]), None);
        assert_eq!(external_signed_variable_byte(&[127]), Some(127));
        assert_eq!(external_signed_variable_byte(&[128]), Some(-128));
        assert_eq!(external_signed_fixed_word(&[0x80, 0]), Some(i16::MIN));
        assert_eq!(external_signed_fixed_word(&[0x7f, 255]), Some(i16::MAX));
        assert_eq!(external_signed_wide_native(&[255; 16]), Some(-1));
    }
}
