//! Proof probe for wide scalar big- and little-endian codecs.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/byte_codec_wide_ops.rs"]
mod byte_codec_wide_ops;
use byte_codec_wide_ops::{
    decode_be_u64, decode_be_u128, decode_le_u64, decode_le_u128,
    encode_be_u64, encode_be_u128, encode_le_u64, encode_le_u128,
};

/// The decoder equation holds for arbitrary byte contents.
#[ensures(result@ == bytes@[0]@ * 72_057_594_037_927_936 + bytes@[1]@ * 281_474_976_710_656 + bytes@[2]@ * 1_099_511_627_776 + bytes@[3]@ * 4_294_967_296 + bytes@[4]@ * 16_777_216 + bytes@[5]@ * 65_536 + bytes@[6]@ * 256 + bytes@[7]@)]
pub fn reconstruct_be_u64(bytes: [u8; 8]) -> u64 {
    decode_be_u64(bytes)
}

/// The decoder equation holds for arbitrary byte contents.
#[ensures(result@ == bytes@[0]@ + bytes@[1]@ * 256 + bytes@[2]@ * 65_536 + bytes@[3]@ * 16_777_216 + bytes@[4]@ * 4_294_967_296 + bytes@[5]@ * 1_099_511_627_776 + bytes@[6]@ * 281_474_976_710_656 + bytes@[7]@ * 72_057_594_037_927_936)]
pub fn reconstruct_le_u64(bytes: [u8; 8]) -> u64 {
    decode_le_u64(bytes)
}

/// BE and LE conversions round trip after peeling 64-bit quotient digits.
#[ensures(result.0@ == value@ && result.1@ == value@)]
pub fn round_trip_u64(value: u64) -> (u64, u64) {
    proof_assert!(value@ == value@ / 256 * 256 + value@ % 256);
    proof_assert!(value@ / 256 == value@ / 65_536 * 256 + (value@ / 256) % 256);
    proof_assert!(value@ / 65_536 == value@ / 16_777_216 * 256 + (value@ / 65_536) % 256);
    proof_assert!(value@ / 16_777_216 == value@ / 4_294_967_296 * 256 + (value@ / 16_777_216) % 256);
    proof_assert!(value@ / 4_294_967_296 == value@ / 1_099_511_627_776 * 256 + (value@ / 4_294_967_296) % 256);
    proof_assert!(value@ / 1_099_511_627_776 == value@ / 281_474_976_710_656 * 256 + (value@ / 1_099_511_627_776) % 256);
    proof_assert!(value@ / 281_474_976_710_656 == value@ / 72_057_594_037_927_936 * 256 + (value@ / 281_474_976_710_656) % 256);
    proof_assert!(value@ / 72_057_594_037_927_936 == value@ / 18_446_744_073_709_551_616 * 256 + (value@ / 72_057_594_037_927_936) % 256);
    proof_assert!(value@ == value@ / 72_057_594_037_927_936 * 72_057_594_037_927_936
        + value@ / 281_474_976_710_656 % 256 * 281_474_976_710_656
        + value@ / 1_099_511_627_776 % 256 * 1_099_511_627_776
        + value@ / 4_294_967_296 % 256 * 4_294_967_296
        + value@ / 16_777_216 % 256 * 16_777_216
        + value@ / 65_536 % 256 * 65_536
        + value@ / 256 % 256 * 256
        + value@ % 256);
    (
        decode_be_u64(encode_be_u64(value)),
        decode_le_u64(encode_le_u64(value)),
    )
}

/// The decoder equation holds for arbitrary byte contents.
#[ensures(result@ == bytes@[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576 + bytes@[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096 + bytes@[2]@ * 20_282_409_603_651_670_423_947_251_286_016 + bytes@[3]@ * 79_228_162_514_264_337_593_543_950_336 + bytes@[4]@ * 309_485_009_821_345_068_724_781_056 + bytes@[5]@ * 1_208_925_819_614_629_174_706_176 + bytes@[6]@ * 4_722_366_482_869_645_213_696 + bytes@[7]@ * 18_446_744_073_709_551_616 + bytes@[8]@ * 72_057_594_037_927_936 + bytes@[9]@ * 281_474_976_710_656 + bytes@[10]@ * 1_099_511_627_776 + bytes@[11]@ * 4_294_967_296 + bytes@[12]@ * 16_777_216 + bytes@[13]@ * 65_536 + bytes@[14]@ * 256 + bytes@[15]@)]
pub fn reconstruct_be_u128(bytes: [u8; 16]) -> u128 {
    decode_be_u128(bytes)
}

/// The decoder equation holds for arbitrary byte contents.
#[ensures(result@ == bytes@[0]@ + bytes@[1]@ * 256 + bytes@[2]@ * 65_536 + bytes@[3]@ * 16_777_216 + bytes@[4]@ * 4_294_967_296 + bytes@[5]@ * 1_099_511_627_776 + bytes@[6]@ * 281_474_976_710_656 + bytes@[7]@ * 72_057_594_037_927_936 + bytes@[8]@ * 18_446_744_073_709_551_616 + bytes@[9]@ * 4_722_366_482_869_645_213_696 + bytes@[10]@ * 1_208_925_819_614_629_174_706_176 + bytes@[11]@ * 309_485_009_821_345_068_724_781_056 + bytes@[12]@ * 79_228_162_514_264_337_593_543_950_336 + bytes@[13]@ * 20_282_409_603_651_670_423_947_251_286_016 + bytes@[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096 + bytes@[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576)]
pub fn reconstruct_le_u128(bytes: [u8; 16]) -> u128 {
    decode_le_u128(bytes)
}

/// BE and LE conversions round trip after peeling 128-bit quotient digits.
#[ensures(result.0@ == value@ && result.1@ == value@)]
pub fn round_trip_u128(value: u128) -> (u128, u128) {
    proof_assert!(value@ == value@ / 256 * 256 + value@ % 256);
    proof_assert!(value@ / 256 == value@ / 65_536 * 256 + (value@ / 256) % 256);
    proof_assert!(value@ / 65_536 == value@ / 16_777_216 * 256 + (value@ / 65_536) % 256);
    proof_assert!(value@ / 16_777_216 == value@ / 4_294_967_296 * 256 + (value@ / 16_777_216) % 256);
    proof_assert!(value@ / 4_294_967_296 == value@ / 1_099_511_627_776 * 256 + (value@ / 4_294_967_296) % 256);
    proof_assert!(value@ / 1_099_511_627_776 == value@ / 281_474_976_710_656 * 256 + (value@ / 1_099_511_627_776) % 256);
    proof_assert!(value@ / 281_474_976_710_656 == value@ / 72_057_594_037_927_936 * 256 + (value@ / 281_474_976_710_656) % 256);
    proof_assert!(value@ / 72_057_594_037_927_936 == value@ / 18_446_744_073_709_551_616 * 256 + (value@ / 72_057_594_037_927_936) % 256);
    proof_assert!(value@ / 18_446_744_073_709_551_616 == value@ / 4_722_366_482_869_645_213_696 * 256 + (value@ / 18_446_744_073_709_551_616) % 256);
    proof_assert!(value@ / 4_722_366_482_869_645_213_696 == value@ / 1_208_925_819_614_629_174_706_176 * 256 + (value@ / 4_722_366_482_869_645_213_696) % 256);
    proof_assert!(value@ / 1_208_925_819_614_629_174_706_176 == value@ / 309_485_009_821_345_068_724_781_056 * 256 + (value@ / 1_208_925_819_614_629_174_706_176) % 256);
    proof_assert!(value@ / 309_485_009_821_345_068_724_781_056 == value@ / 79_228_162_514_264_337_593_543_950_336 * 256 + (value@ / 309_485_009_821_345_068_724_781_056) % 256);
    proof_assert!(value@ / 79_228_162_514_264_337_593_543_950_336 == value@ / 20_282_409_603_651_670_423_947_251_286_016 * 256 + (value@ / 79_228_162_514_264_337_593_543_950_336) % 256);
    proof_assert!(value@ / 20_282_409_603_651_670_423_947_251_286_016 == value@ / 5_192_296_858_534_827_628_530_496_329_220_096 * 256 + (value@ / 20_282_409_603_651_670_423_947_251_286_016) % 256);
    proof_assert!(value@ / 5_192_296_858_534_827_628_530_496_329_220_096 == value@ / 1_329_227_995_784_915_872_903_807_060_280_344_576 * 256 + (value@ / 5_192_296_858_534_827_628_530_496_329_220_096) % 256);
    proof_assert!(value@ == value@ / 1_329_227_995_784_915_872_903_807_060_280_344_576 * 1_329_227_995_784_915_872_903_807_060_280_344_576
        + value@ / 5_192_296_858_534_827_628_530_496_329_220_096 % 256 * 5_192_296_858_534_827_628_530_496_329_220_096
        + value@ / 20_282_409_603_651_670_423_947_251_286_016 % 256 * 20_282_409_603_651_670_423_947_251_286_016
        + value@ / 79_228_162_514_264_337_593_543_950_336 % 256 * 79_228_162_514_264_337_593_543_950_336
        + value@ / 309_485_009_821_345_068_724_781_056 % 256 * 309_485_009_821_345_068_724_781_056
        + value@ / 1_208_925_819_614_629_174_706_176 % 256 * 1_208_925_819_614_629_174_706_176
        + value@ / 4_722_366_482_869_645_213_696 % 256 * 4_722_366_482_869_645_213_696
        + value@ / 18_446_744_073_709_551_616 % 256 * 18_446_744_073_709_551_616
        + value@ / 72_057_594_037_927_936 % 256 * 72_057_594_037_927_936
        + value@ / 281_474_976_710_656 % 256 * 281_474_976_710_656
        + value@ / 1_099_511_627_776 % 256 * 1_099_511_627_776
        + value@ / 4_294_967_296 % 256 * 4_294_967_296
        + value@ / 16_777_216 % 256 * 16_777_216
        + value@ / 65_536 % 256 * 65_536
        + value@ / 256 % 256 * 256
        + value@ % 256);
    (
        decode_be_u128(encode_be_u128(value)),
        decode_le_u128(encode_le_u128(value)),
    )
}

/// Negative control: claims a big-endian encoder writes the least-significant
/// byte first. The runtime encoder is intentionally left correct.
#[cfg(feature = "wrong_wide_encode")]
#[ensures(result@[0]@ == value@ % 256)]
pub fn wrong_wide_encode_contract(value: u64) -> [u8; 8] {
    encode_be_u64(value)
}

/// Negative control: claims a big-endian decoder uses little-endian weights.
/// The runtime decoder is intentionally left correct.
#[cfg(feature = "wrong_wide_decode")]
#[ensures(result@ == bytes@[0]@ + bytes@[1]@ * 256 + bytes@[2]@ * 65_536
    + bytes@[3]@ * 16_777_216 + bytes@[4]@ * 4_294_967_296
    + bytes@[5]@ * 1_099_511_627_776 + bytes@[6]@ * 281_474_976_710_656
    + bytes@[7]@ * 72_057_594_037_927_936)]
pub fn wrong_wide_decode_contract(bytes: [u8; 8]) -> u64 {
    decode_be_u64(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_endian_codecs_match_std_at_boundaries_and_patterns() {
        let values = [
            0, 1, 255, 256, 0x0102_0304_0506_0708, 1u64 << 63, u64::MAX,
        ];
        for value in values {
            let be = encode_be_u64(value);
            let le = encode_le_u64(value);
            assert_eq!(be, value.to_be_bytes());
            assert_eq!(le, value.to_le_bytes());
            assert_eq!(decode_be_u64(be), u64::from_be_bytes(be));
            assert_eq!(decode_le_u64(le), u64::from_le_bytes(le));
        }
        for bytes in [
            [0u8; 8], [255u8; 8], [0, 1, 2, 3, 4, 5, 6, 7],
            [128, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 128],
        ] {
            assert_eq!(decode_be_u64(bytes), u64::from_be_bytes(bytes));
            assert_eq!(decode_le_u64(bytes), u64::from_le_bytes(bytes));
        }
    }

    #[test]
    fn u128_endian_codecs_match_std_at_boundaries_and_patterns() {
        let values = [
            0, 1, 255, 256, 0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10,
            1u128 << 127, u128::MAX,
        ];
        for value in values {
            let be = encode_be_u128(value);
            let le = encode_le_u128(value);
            assert_eq!(be, value.to_be_bytes());
            assert_eq!(le, value.to_le_bytes());
            assert_eq!(decode_be_u128(be), u128::from_be_bytes(be));
            assert_eq!(decode_le_u128(le), u128::from_le_bytes(le));
        }
        for bytes in [
            [0u8; 16], [255u8; 16],
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            [128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128],
        ] {
            assert_eq!(decode_be_u128(bytes), u128::from_be_bytes(bytes));
            assert_eq!(decode_le_u128(bytes), u128::from_le_bytes(bytes));
        }
    }
}
