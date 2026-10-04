//! Proof probe for checked wide reads from a real slice cursor.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;

#[path = "../../../../src/byte_codec_wide_ops.rs"]
mod byte_codec_wide_ops;

#[path = "../../../../src/slice_wide_read_ops.rs"]
mod slice_wide_read_ops;

use slice_wide_read_ops::{
    read_be_u128 as checked_be_u128, read_be_u64 as checked_be_u64,
    read_le_u128 as checked_le_u128, read_le_u64 as checked_le_u64,
};

#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == (input@[0]@ * 72_057_594_037_927_936 + input@[1]@ * 281_474_976_710_656 + input@[2]@ * 1_099_511_627_776 + input@[3]@ * 4_294_967_296 + input@[4]@ * 16_777_216 + input@[5]@ * 65_536 + input@[6]@ * 256 + input@[7]@)
        && (^input)@ == input@[8..],
    None => input@.len() < 8 && (^input)@ == input@
})]
pub fn read_be_u64(input: &mut &[u8]) -> Option<u64> {
    checked_be_u64(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == (input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65_536 + input@[3]@ * 16_777_216 + input@[4]@ * 4_294_967_296 + input@[5]@ * 1_099_511_627_776 + input@[6]@ * 281_474_976_710_656 + input@[7]@ * 72_057_594_037_927_936)
        && (^input)@ == input@[8..],
    None => input@.len() < 8 && (^input)@ == input@
})]
pub fn read_le_u64(input: &mut &[u8]) -> Option<u64> {
    checked_le_u64(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == (input@[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576 + input@[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096 + input@[2]@ * 20_282_409_603_651_670_423_947_251_286_016 + input@[3]@ * 79_228_162_514_264_337_593_543_950_336 + input@[4]@ * 309_485_009_821_345_068_724_781_056 + input@[5]@ * 1_208_925_819_614_629_174_706_176 + input@[6]@ * 4_722_366_482_869_645_213_696 + input@[7]@ * 18_446_744_073_709_551_616 + input@[8]@ * 72_057_594_037_927_936 + input@[9]@ * 281_474_976_710_656 + input@[10]@ * 1_099_511_627_776 + input@[11]@ * 4_294_967_296 + input@[12]@ * 16_777_216 + input@[13]@ * 65_536 + input@[14]@ * 256 + input@[15]@)
        && (^input)@ == input@[16..],
    None => input@.len() < 16 && (^input)@ == input@
})]
pub fn read_be_u128(input: &mut &[u8]) -> Option<u128> {
    checked_be_u128(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == (input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65_536 + input@[3]@ * 16_777_216 + input@[4]@ * 4_294_967_296 + input@[5]@ * 1_099_511_627_776 + input@[6]@ * 281_474_976_710_656 + input@[7]@ * 72_057_594_037_927_936 + input@[8]@ * 18_446_744_073_709_551_616 + input@[9]@ * 4_722_366_482_869_645_213_696 + input@[10]@ * 1_208_925_819_614_629_174_706_176 + input@[11]@ * 309_485_009_821_345_068_724_781_056 + input@[12]@ * 79_228_162_514_264_337_593_543_950_336 + input@[13]@ * 20_282_409_603_651_670_423_947_251_286_016 + input@[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096 + input@[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576)
        && (^input)@ == input@[16..],
    None => input@.len() < 16 && (^input)@ == input@
})]
pub fn read_le_u128(input: &mut &[u8]) -> Option<u128> {
    checked_le_u128(input)
}

/// Negative control: preserve state but claim a different BE value.
#[cfg(feature = "wrong_value")]
#[requires(input@.len() >= 8)]
#[ensures(match result {
    Some(value) => value@ == (input@[0]@ * 72_057_594_037_927_936 + input@[1]@ * 281_474_976_710_656 + input@[2]@ * 1_099_511_627_776 + input@[3]@ * 4_294_967_296 + input@[4]@ * 16_777_216 + input@[5]@ * 65_536 + input@[6]@ * 256 + input@[7]@) + 1,
    None => false
})]
pub fn wrong_value(input: &mut &[u8]) -> Option<u64> {
    checked_be_u64(input)
}

/// Negative control: this short path consumes its seven-byte input.
#[cfg(feature = "short_consumed")]
#[requires(input@.len() == 7)]
#[ensures((^input)@ == input@)]
pub fn consumes_short_input(input: &mut &[u8]) -> Option<u64> {
    if input.len() < 8 {
        crate::slice_ops::advance_slice(input, input.len());
        None
    } else {
        checked_be_u64(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_reads_match_native_endian_conversions_and_preserve_short_input() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 0xaa];
        let expected_be = u64::from_be_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
        let expected_le = u64::from_le_bytes([1, 2, 3, 4, 5, 6, 7, 8]);

        let mut input: &[u8] = &bytes;
        assert_eq!(read_be_u64(&mut input), Some(expected_be));
        assert_eq!(input, &bytes[8..]);

        let mut input: &[u8] = &bytes;
        assert_eq!(read_le_u64(&mut input), Some(expected_le));
        assert_eq!(input, &bytes[8..]);

        for len in 0..8 {
            let short = &bytes[..len];
            let mut input = short;
            assert_eq!(read_be_u64(&mut input), None);
            assert_eq!(input, short);
            assert_eq!(read_le_u64(&mut input), None);
            assert_eq!(input, short);
        }

        for pattern in [
            [0u8; 8],
            [0xff; 8],
            [0x80, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0x80],
        ] {
            let mut input: &[u8] = &pattern;
            assert_eq!(
                read_be_u64(&mut input),
                Some(u64::from_be_bytes(pattern))
            );
            assert!(input.is_empty());

            let mut input: &[u8] = &pattern;
            assert_eq!(
                read_le_u64(&mut input),
                Some(u64::from_le_bytes(pattern))
            );
            assert!(input.is_empty());
        }
    }

    #[test]
    fn u128_reads_match_native_endian_conversions_and_preserve_short_input() {
        let bytes = [
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0xaa,
        ];
        let word = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected_be = u128::from_be_bytes(word);
        let expected_le = u128::from_le_bytes(word);

        let mut input: &[u8] = &bytes;
        assert_eq!(read_be_u128(&mut input), Some(expected_be));
        assert_eq!(input, &bytes[16..]);

        let mut input: &[u8] = &bytes;
        assert_eq!(read_le_u128(&mut input), Some(expected_le));
        assert_eq!(input, &bytes[16..]);

        for len in 0..16 {
            let short = &bytes[..len];
            let mut input = short;
            assert_eq!(read_be_u128(&mut input), None);
            assert_eq!(input, short);
            assert_eq!(read_le_u128(&mut input), None);
            assert_eq!(input, short);
        }

        for pattern in [
            [0u8; 16],
            [0xff; 16],
            [0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80],
        ] {
            let mut input: &[u8] = &pattern;
            assert_eq!(
                read_be_u128(&mut input),
                Some(u128::from_be_bytes(pattern))
            );
            assert!(input.is_empty());

            let mut input: &[u8] = &pattern;
            assert_eq!(
                read_le_u128(&mut input),
                Some(u128::from_le_bytes(pattern))
            );
            assert!(input.is_empty());
        }
    }
}
