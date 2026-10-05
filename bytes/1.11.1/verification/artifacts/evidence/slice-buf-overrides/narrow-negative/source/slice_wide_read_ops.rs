//! Checked wide integer reads from the `&[u8]` cursor.
//!
//! Failed reads preserve the complete input; successful reads consume exactly
//! the fixed-width prefix and decode its bytes in the requested endian order.
use creusot_std::prelude::*;

/// Reads a big-endian `u64`; a short slice is returned unchanged.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == (input@[0]@ * 72_057_594_037_927_936 + input@[1]@ * 281_474_976_710_656 + input@[2]@ * 1_099_511_627_776 + input@[3]@ * 4_294_967_296 + input@[4]@ * 16_777_216 + input@[5]@ * 65_536 + input@[6]@ * 256 + input@[7]@)
        && (^input)@ == input@[8..],
    None => input@.len() < 8 && (^input)@ == input@
})]
pub(crate) fn read_be_u64(input: &mut &[u8]) -> Option<u64> {
    if input.len() < 8 {
        None
    } else {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&input[..8]);
        crate::slice_ops::advance_slice(input, 8);
        Some(crate::byte_codec_wide_ops::decode_be_u64(bytes))
    }
}

/// Reads a little-endian `u64`; a short slice is returned unchanged.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == (input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65_536 + input@[3]@ * 16_777_216 + input@[4]@ * 4_294_967_296 + input@[5]@ * 1_099_511_627_776 + input@[6]@ * 281_474_976_710_656 + input@[7]@ * 72_057_594_037_927_936)
        && (^input)@ == input@[8..],
    None => input@.len() < 8 && (^input)@ == input@
})]
pub(crate) fn read_le_u64(input: &mut &[u8]) -> Option<u64> {
    if input.len() < 8 {
        None
    } else {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&input[..8]);
        crate::slice_ops::advance_slice(input, 8);
        Some(crate::byte_codec_wide_ops::decode_le_u64(bytes))
    }
}

/// Reads a big-endian `u128`; a short slice is returned unchanged.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == (input@[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576 + input@[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096 + input@[2]@ * 20_282_409_603_651_670_423_947_251_286_016 + input@[3]@ * 79_228_162_514_264_337_593_543_950_336 + input@[4]@ * 309_485_009_821_345_068_724_781_056 + input@[5]@ * 1_208_925_819_614_629_174_706_176 + input@[6]@ * 4_722_366_482_869_645_213_696 + input@[7]@ * 18_446_744_073_709_551_616 + input@[8]@ * 72_057_594_037_927_936 + input@[9]@ * 281_474_976_710_656 + input@[10]@ * 1_099_511_627_776 + input@[11]@ * 4_294_967_296 + input@[12]@ * 16_777_216 + input@[13]@ * 65_536 + input@[14]@ * 256 + input@[15]@)
        && (^input)@ == input@[16..],
    None => input@.len() < 16 && (^input)@ == input@
})]
pub(crate) fn read_be_u128(input: &mut &[u8]) -> Option<u128> {
    if input.len() < 16 {
        None
    } else {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&input[..16]);
        crate::slice_ops::advance_slice(input, 16);
        Some(crate::byte_codec_wide_ops::decode_be_u128(bytes))
    }
}

/// Reads a little-endian `u128`; a short slice is returned unchanged.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == (input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65_536 + input@[3]@ * 16_777_216 + input@[4]@ * 4_294_967_296 + input@[5]@ * 1_099_511_627_776 + input@[6]@ * 281_474_976_710_656 + input@[7]@ * 72_057_594_037_927_936 + input@[8]@ * 18_446_744_073_709_551_616 + input@[9]@ * 4_722_366_482_869_645_213_696 + input@[10]@ * 1_208_925_819_614_629_174_706_176 + input@[11]@ * 309_485_009_821_345_068_724_781_056 + input@[12]@ * 79_228_162_514_264_337_593_543_950_336 + input@[13]@ * 20_282_409_603_651_670_423_947_251_286_016 + input@[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096 + input@[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576)
        && (^input)@ == input@[16..],
    None => input@.len() < 16 && (^input)@ == input@
})]
pub(crate) fn read_le_u128(input: &mut &[u8]) -> Option<u128> {
    if input.len() < 16 {
        None
    } else {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&input[..16]);
        crate::slice_ops::advance_slice(input, 16);
        Some(crate::byte_codec_wide_ops::decode_le_u128(bytes))
    }
}
