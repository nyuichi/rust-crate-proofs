//! Checked signed 64- and 128-bit reads from a byte-slice cursor.
//!
//! The unsigned reader supplies the endian decoding and cursor update. These
//! helpers give the decoded bit pattern its exact two's-complement value.

use creusot_std::prelude::*;

#[logic]
pub(crate) fn signed_u64(word: Int) -> Int {
    if word <= 9_223_372_036_854_775_807 {
        word
    } else {
        word - 18_446_744_073_709_551_616
    }
}

#[logic]
pub(crate) fn signed_u128(word: Int) -> Int {
    if word <= 170_141_183_460_469_231_731_687_303_715_884_105_727 {
        word
    } else {
        word - (2 * 170_141_183_460_469_231_731_687_303_715_884_105_727 + 2)
    }
}

#[inline]
#[ensures(result@ == signed_u64(word@))]
pub(crate) fn signed_from_u64(word: u64) -> i64 {
    if word <= i64::MAX as u64 {
        word as i64
    } else {
        (word - 9_223_372_036_854_775_808) as i64 + i64::MIN
    }
}

#[inline]
#[ensures(result@ == signed_u128(word@))]
pub(crate) fn signed_from_u128(word: u128) -> i128 {
    if word <= i128::MAX as u128 {
        word as i128
    } else {
        (word - 170_141_183_460_469_231_731_687_303_715_884_105_728) as i128
            + i128::MIN
    }
}

/// Reads a big-endian signed 64-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == signed_u64(
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
pub(crate) fn read_be_i64(input: &mut &[u8]) -> Option<i64> {
    match crate::slice_wide_read_ops::read_be_u64(input) {
        Some(word) => Some(signed_from_u64(word)),
        None => None,
    }
}

/// Reads a little-endian signed 64-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 8
        && value@ == signed_u64(
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
pub(crate) fn read_le_i64(input: &mut &[u8]) -> Option<i64> {
    match crate::slice_wide_read_ops::read_le_u64(input) {
        Some(word) => Some(signed_from_u64(word)),
        None => None,
    }
}

/// Reads a big-endian signed 128-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == signed_u128(
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
pub(crate) fn read_be_i128(input: &mut &[u8]) -> Option<i128> {
    match crate::slice_wide_read_ops::read_be_u128(input) {
        Some(word) => Some(signed_from_u128(word)),
        None => None,
    }
}

/// Reads a little-endian signed 128-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 16
        && value@ == signed_u128(
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
pub(crate) fn read_le_i128(input: &mut &[u8]) -> Option<i128> {
    match crate::slice_wide_read_ops::read_le_u128(input) {
        Some(word) => Some(signed_from_u128(word)),
        None => None,
    }
}
