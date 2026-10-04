//! Proof probe for checked signed 64- and 128-bit reads.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;
#[path = "../../../../src/byte_codec_wide_ops.rs"]
mod byte_codec_wide_ops;
#[path = "../../../../src/slice_wide_read_ops.rs"]
mod slice_wide_read_ops;
#[path = "../../../../src/signed_wide_ops.rs"]
mod signed_wide_ops;

#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == signed_wide_ops::signed_u64(
            input@[0]@ * 72_057_594_037_927_936
                + input@[1]@ * 281_474_976_710_656
                + input@[2]@ * 1_099_511_627_776
                + input@[3]@ * 4_294_967_296
                + input@[4]@ * 16_777_216
                + input@[5]@ * 65_536
                + input@[6]@ * 256
                + input@[7]@
        )
        && (^input)@ == input@[8..],
    None => input@.len() < 8 && (^input)@ == input@
})]
pub fn read_be_i64(input: &mut &[u8]) -> Option<i64> {
    signed_wide_ops::read_be_i64(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == signed_wide_ops::signed_u64(
            input@[0]@
                + input@[1]@ * 256
                + input@[2]@ * 65_536
                + input@[3]@ * 16_777_216
                + input@[4]@ * 4_294_967_296
                + input@[5]@ * 1_099_511_627_776
                + input@[6]@ * 281_474_976_710_656
                + input@[7]@ * 72_057_594_037_927_936
        )
        && (^input)@ == input@[8..],
    None => input@.len() < 8 && (^input)@ == input@
})]
pub fn read_le_i64(input: &mut &[u8]) -> Option<i64> {
    signed_wide_ops::read_le_i64(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == signed_wide_ops::signed_u128(
            input@[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
                + input@[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096
                + input@[2]@ * 20_282_409_603_651_670_423_947_251_286_016
                + input@[3]@ * 79_228_162_514_264_337_593_543_950_336
                + input@[4]@ * 309_485_009_821_345_068_724_781_056
                + input@[5]@ * 1_208_925_819_614_629_174_706_176
                + input@[6]@ * 4_722_366_482_869_645_213_696
                + input@[7]@ * 18_446_744_073_709_551_616
                + input@[8]@ * 72_057_594_037_927_936
                + input@[9]@ * 281_474_976_710_656
                + input@[10]@ * 1_099_511_627_776
                + input@[11]@ * 4_294_967_296
                + input@[12]@ * 16_777_216
                + input@[13]@ * 65_536
                + input@[14]@ * 256
                + input@[15]@
        )
        && (^input)@ == input@[16..],
    None => input@.len() < 16 && (^input)@ == input@
})]
pub fn read_be_i128(input: &mut &[u8]) -> Option<i128> {
    signed_wide_ops::read_be_i128(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == signed_wide_ops::signed_u128(
            input@[0]@
                + input@[1]@ * 256
                + input@[2]@ * 65_536
                + input@[3]@ * 16_777_216
                + input@[4]@ * 4_294_967_296
                + input@[5]@ * 1_099_511_627_776
                + input@[6]@ * 281_474_976_710_656
                + input@[7]@ * 72_057_594_037_927_936
                + input@[8]@ * 18_446_744_073_709_551_616
                + input@[9]@ * 4_722_366_482_869_645_213_696
                + input@[10]@ * 1_208_925_819_614_629_174_706_176
                + input@[11]@ * 309_485_009_821_345_068_724_781_056
                + input@[12]@ * 79_228_162_514_264_337_593_543_950_336
                + input@[13]@ * 20_282_409_603_651_670_423_947_251_286_016
                + input@[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096
                + input@[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
        )
        && (^input)@ == input@[16..],
    None => input@.len() < 16 && (^input)@ == input@
})]
pub fn read_le_i128(input: &mut &[u8]) -> Option<i128> {
    signed_wide_ops::read_le_i128(input)
}

/// A deliberately incorrect signed value contract for a high-bit-set BE word.
#[cfg(feature = "wrong_signed")]
#[requires(input@.len() >= 8)]
#[requires(input@[0]@ >= 128)]
#[ensures(match result {
    Some(value) => value@ == input@[0]@ * 72_057_594_037_927_936
        + input@[1]@ * 281_474_976_710_656
        + input@[2]@ * 1_099_511_627_776
        + input@[3]@ * 4_294_967_296
        + input@[4]@ * 16_777_216
        + input@[5]@ * 65_536
        + input@[6]@ * 256
        + input@[7]@,
    None => false
})]
pub fn wrong_signed_be_i64(input: &mut &[u8]) -> Option<i64> {
    signed_wide_ops::read_be_i64(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_signed_reads_match_native_endian_boundaries_and_consume_only_the_word() {
        for value in [
            i64::MIN,
            i64::MIN + 1,
            -1,
            0,
            1,
            i64::MAX - 1,
            i64::MAX,
        ] {
            let mut be_with_suffix = [0xa5u8; 9];
            be_with_suffix[..8].copy_from_slice(&value.to_be_bytes());
            let mut input: &[u8] = &be_with_suffix;
            assert_eq!(read_be_i64(&mut input), Some(value));
            assert_eq!(input, &be_with_suffix[8..]);

            let mut le_with_suffix = [0xa5u8; 9];
            le_with_suffix[..8].copy_from_slice(&value.to_le_bytes());
            let mut input: &[u8] = &le_with_suffix;
            assert_eq!(read_le_i64(&mut input), Some(value));
            assert_eq!(input, &le_with_suffix[8..]);
        }

        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        for len in 0..8 {
            let short = &bytes[..len];
            let mut input = short;
            assert_eq!(read_be_i64(&mut input), None);
            assert_eq!(input, short);
            assert_eq!(read_le_i64(&mut input), None);
            assert_eq!(input, short);
        }
    }

    #[test]
    fn i128_signed_reads_match_native_endian_boundaries_and_consume_only_the_word() {
        for value in [
            i128::MIN,
            i128::MIN + 1,
            -1,
            0,
            1,
            i128::MAX - 1,
            i128::MAX,
        ] {
            let mut be_with_suffix = [0x5au8; 17];
            be_with_suffix[..16].copy_from_slice(&value.to_be_bytes());
            let mut input: &[u8] = &be_with_suffix;
            assert_eq!(read_be_i128(&mut input), Some(value));
            assert_eq!(input, &be_with_suffix[16..]);

            let mut le_with_suffix = [0x5au8; 17];
            le_with_suffix[..16].copy_from_slice(&value.to_le_bytes());
            let mut input: &[u8] = &le_with_suffix;
            assert_eq!(read_le_i128(&mut input), Some(value));
            assert_eq!(input, &le_with_suffix[16..]);
        }

        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        for len in 0..16 {
            let short = &bytes[..len];
            let mut input = short;
            assert_eq!(read_be_i128(&mut input), None);
            assert_eq!(input, short);
            assert_eq!(read_le_i128(&mut input), None);
            assert_eq!(input, short);
        }
    }
}
