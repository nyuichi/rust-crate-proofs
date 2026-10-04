//! Proof probe for scalar big-endian encoding and decoding.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/byte_codec_ops.rs"]
mod byte_codec_ops;
use byte_codec_ops::{decode_be_u16, decode_be_u32, encode_be_u16, encode_be_u32};

/// The helper contracts compose to recover every possible `u16` value.
#[ensures(result@ == value@)]
pub fn round_trip_u16(value: u16) -> u16 {
    decode_be_u16(encode_be_u16(value))
}

/// The helper contracts compose to recover every possible `u32` value.
#[ensures(result@ == value@)]
pub fn round_trip_u32(value: u32) -> u32 {
    decode_be_u32(encode_be_u32(value))
}

/// The decoder's postcondition applies to arbitrary byte contents.
#[ensures(result@ == bytes@[0]@ * 16_777_216 + bytes@[1]@ * 65_536 + bytes@[2]@ * 256 + bytes@[3]@)]
pub fn reconstruct_u32(bytes: [u8; 4]) -> u32 {
    decode_be_u32(bytes)
}

#[cfg(feature = "wrong_postcondition")]
#[ensures(result@ == 1)]
pub fn wrong_postcondition() -> u16 {
    decode_be_u16([0, 0])
}

#[cfg(feature = "reachable_false")]
pub fn reachable_false() {
    let _ = round_trip_u16(0);
    assert!(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_matches_native_endian_conversions_for_every_value() {
        for value in 0..=u16::MAX {
            let bytes = value.to_be_bytes();
            let encoded_be = encode_be_u16(value);
            let bytes_le = value.to_le_bytes();
            assert_eq!(encoded_be, bytes);
            assert_eq!(decode_be_u16(bytes), value);
            assert_eq!([encoded_be[1], encoded_be[0]], bytes_le);
            assert_eq!(decode_be_u16([bytes_le[1], bytes_le[0]]), value);
        }
    }

    #[test]
    fn u32_matches_native_endian_conversions_at_boundaries_and_mixed_bytes() {
        for value in [0, 1, 255, 256, 65_535, 65_536, 0x0102_0304, u32::MAX] {
            let bytes = value.to_be_bytes();
            assert_eq!(encode_be_u32(value), bytes);
            assert_eq!(decode_be_u32(bytes), value);
        }
    }

    #[test]
    fn u32_decode_matches_native_conversion_for_arbitrary_byte_patterns() {
        for bytes in [
            [0, 0, 0, 0],
            [0, 0, 0, 1],
            [1, 2, 3, 4],
            [255, 0, 128, 17],
            [255, 255, 255, 255],
        ] {
            assert_eq!(decode_be_u32(bytes), u32::from_be_bytes(bytes));
        }
    }
}
