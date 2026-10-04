//! Proof probe for little-endian and signed scalar byte operations.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/byte_codec_ops.rs"]
mod byte_codec_ops;
#[path = "../../../../src/endian_ops.rs"]
mod endian_ops;
#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;
#[path = "../../../../src/slice_read_ops.rs"]
mod slice_read_ops;

#[ensures(result@[0]@ == value@ % 256)]
#[ensures(result@[1]@ == value@ / 256)]
pub fn encode_le_u16(value: u16) -> [u8; 2] {
    endian_ops::encode_le_u16(value)
}

#[ensures(result@[0]@ == value@ % 256)]
#[ensures(result@[1]@ == value@ / 256 % 256)]
#[ensures(result@[2]@ == value@ / 65_536 % 256)]
#[ensures(result@[3]@ == value@ / 16_777_216)]
pub fn encode_le_u32(value: u32) -> [u8; 4] {
    endian_ops::encode_le_u32(value)
}

#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == input@[0]@ + input@[1]@ * 256
        && (^input)@ == input@[2..],
    None => input@.len() < 2 && (^input)@ == input@
})]
pub fn read_le_u16(input: &mut &[u8]) -> Option<u16> {
    endian_ops::read_le_u16(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == input@[0]@ + input@[1]@ * 256
            + input@[2]@ * 65_536
            + input@[3]@ * 16_777_216
        && (^input)@ == input@[4..],
    None => input@.len() < 4 && (^input)@ == input@
})]
pub fn read_le_u32(input: &mut &[u8]) -> Option<u32> {
    endian_ops::read_le_u32(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == endian_ops::signed_u16(input@[0]@ * 256 + input@[1]@)
        && (^input)@ == input@[2..],
    None => input@.len() < 2 && (^input)@ == input@
})]
pub fn read_be_i16(input: &mut &[u8]) -> Option<i16> {
    endian_ops::read_be_i16(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == endian_ops::signed_u16(input@[0]@ + input@[1]@ * 256)
        && (^input)@ == input@[2..],
    None => input@.len() < 2 && (^input)@ == input@
})]
pub fn read_le_i16(input: &mut &[u8]) -> Option<i16> {
    endian_ops::read_le_i16(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == endian_ops::signed_u32(
            input@[0]@ * 16_777_216 + input@[1]@ * 65_536
                + input@[2]@ * 256 + input@[3]@
        )
        && (^input)@ == input@[4..],
    None => input@.len() < 4 && (^input)@ == input@
})]
pub fn read_be_i32(input: &mut &[u8]) -> Option<i32> {
    endian_ops::read_be_i32(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == endian_ops::signed_u32(
            input@[0]@ + input@[1]@ * 256
                + input@[2]@ * 65_536 + input@[3]@ * 16_777_216
        )
        && (^input)@ == input@[4..],
    None => input@.len() < 4 && (^input)@ == input@
})]
pub fn read_le_i32(input: &mut &[u8]) -> Option<i32> {
    endian_ops::read_le_i32(input)
}

#[cfg(feature = "wrong_endian")]
#[ensures(result@[0]@ == value@ / 256)]
pub fn wrong_endian(value: u16) -> [u8; 2] {
    endian_ops::encode_le_u16(value)
}

#[cfg(feature = "wrong_signed")]
#[requires(input@.len() >= 2)]
#[ensures(match result {
    Some(value) => value@ == input@[0]@ * 256 + input@[1]@,
    None => false
})]
pub fn wrong_signed(input: &mut &[u8]) -> Option<i16> {
    endian_ops::read_be_i16(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_le_encoding_and_signed_reads_match_native_conversions_exhaustively() {
        for word in 0..=u16::MAX {
            let little_endian = word.to_le_bytes();
            let big_endian = word.to_be_bytes();

            assert_eq!(encode_le_u16(word), little_endian);

            let mut input = &little_endian[..];
            assert_eq!(read_le_u16(&mut input), Some(word));
            assert!(input.is_empty());

            let mut input = &big_endian[..];
            assert_eq!(read_be_i16(&mut input), Some(word as i16));
            assert!(input.is_empty());

            let mut input = &little_endian[..];
            assert_eq!(read_le_i16(&mut input), Some(word as i16));
            assert!(input.is_empty());
        }
    }

    #[test]
    fn u32_le_encoding_and_all_signed_read_orders_match_native_boundaries() {
        for word in [0, 1, 255, 256, 0x0102_0304, 0x8000_0000, u32::MAX] {
            let little_endian = word.to_le_bytes();
            assert_eq!(encode_le_u32(word), little_endian);
            let mut input = &little_endian[..];
            assert_eq!(read_le_u32(&mut input), Some(word));
            assert!(input.is_empty());
        }

        for value in [
            i32::MIN,
            i32::MIN + 1,
            -1,
            0,
            1,
            0x1234_5678,
            i32::MAX - 1,
            i32::MAX,
        ] {
            let big_endian = value.to_be_bytes();
            let little_endian = value.to_le_bytes();

            let mut input = &big_endian[..];
            assert_eq!(read_be_i32(&mut input), Some(value));
            assert!(input.is_empty());

            let mut input = &little_endian[..];
            assert_eq!(read_le_i32(&mut input), Some(value));
            assert!(input.is_empty());
        }
    }

    #[test]
    fn short_reads_leave_the_input_unchanged() {
        for len in 0..2 {
            let mut input = &b"\x01"[..len];
            let original = input;
            assert_eq!(read_le_u16(&mut input), None);
            assert_eq!(read_be_i16(&mut input), None);
            assert_eq!(read_le_i16(&mut input), None);
            assert_eq!(input, original);
        }

        for len in 0..4 {
            let mut input = &b"\x01\x02\x03"[..len];
            let original = input;
            assert_eq!(read_le_u32(&mut input), None);
            assert_eq!(read_be_i32(&mut input), None);
            assert_eq!(read_le_i32(&mut input), None);
            assert_eq!(input, original);
        }
    }
}
