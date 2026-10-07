//! Checked, variable-width reads from the `&[u8]` cursor.
//!
//! The requested width is at most eight bytes. Short inputs are left intact;
//! successful reads consume exactly the requested prefix.

use creusot_std::prelude::*;

#[logic(open(crate))]
pub(crate) fn be_weight(bytes: Seq<u8>, n: Int) -> Int {
    pearlite! {
        (if n > 0 { bytes[n - 1]@ } else { 0 })
            + (if n > 1 { bytes[n - 2]@ * 256 } else { 0 })
            + (if n > 2 { bytes[n - 3]@ * 65_536 } else { 0 })
            + (if n > 3 { bytes[n - 4]@ * 16_777_216 } else { 0 })
            + (if n > 4 { bytes[n - 5]@ * 4_294_967_296 } else { 0 })
            + (if n > 5 { bytes[n - 6]@ * 1_099_511_627_776 } else { 0 })
            + (if n > 6 { bytes[n - 7]@ * 281_474_976_710_656 } else { 0 })
            + (if n > 7 { bytes[n - 8]@ * 72_057_594_037_927_936 } else { 0 })
    }
}

#[logic(open(crate))]
pub(crate) fn le_weight(bytes: Seq<u8>, n: Int) -> Int {
    pearlite! {
        (if n > 0 { bytes[0]@ } else { 0 })
            + (if n > 1 { bytes[1]@ * 256 } else { 0 })
            + (if n > 2 { bytes[2]@ * 65_536 } else { 0 })
            + (if n > 3 { bytes[3]@ * 16_777_216 } else { 0 })
            + (if n > 4 { bytes[4]@ * 4_294_967_296 } else { 0 })
            + (if n > 5 { bytes[5]@ * 1_099_511_627_776 } else { 0 })
            + (if n > 6 { bytes[6]@ * 281_474_976_710_656 } else { 0 })
            + (if n > 7 { bytes[7]@ * 72_057_594_037_927_936 } else { 0 })
    }
}

/// Reads an unsigned variable-width big-endian integer.
#[inline]
#[requires(nbytes@ <= 8)]
#[ensures(match result {
    Some(value) => input@.len() >= nbytes@
        && value@ == be_weight(input@, nbytes@)
        && (^input)@ == input@[nbytes@..],
    None => input@.len() < nbytes@ && (^input)@ == input@
})]
pub(crate) fn read_be_u64(input: &mut &[u8], nbytes: usize) -> Option<u64> {
    if input.len() < nbytes {
        None
    } else {
        let bytes = match nbytes {
            0 => [0u8; 8],
            1 => [0, 0, 0, 0, 0, 0, 0, input[0]],
            2 => [0, 0, 0, 0, 0, 0, input[0], input[1]],
            3 => [0, 0, 0, 0, 0, input[0], input[1], input[2]],
            4 => [0, 0, 0, 0, input[0], input[1], input[2], input[3]],
            5 => [0, 0, 0, input[0], input[1], input[2], input[3], input[4]],
            6 => [
                0, 0, input[0], input[1], input[2], input[3], input[4], input[5],
            ],
            7 => [
                0, input[0], input[1], input[2], input[3], input[4], input[5], input[6],
            ],
            _ => [
                input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
            ],
        };
        crate::slice_ops::advance_slice(input, nbytes);
        Some(crate::byte_codec_wide_ops::decode_be_u64(bytes))
    }
}

/// Reads an unsigned variable-width little-endian integer.
#[inline]
#[requires(nbytes@ <= 8)]
#[ensures(match result {
    Some(value) => input@.len() >= nbytes@
        && value@ == le_weight(input@, nbytes@)
        && (^input)@ == input@[nbytes@..],
    None => input@.len() < nbytes@ && (^input)@ == input@
})]
pub(crate) fn read_le_u64(input: &mut &[u8], nbytes: usize) -> Option<u64> {
    if input.len() < nbytes {
        None
    } else {
        let bytes = match nbytes {
            0 => [0u8; 8],
            1 => [input[0], 0, 0, 0, 0, 0, 0, 0],
            2 => [input[0], input[1], 0, 0, 0, 0, 0, 0],
            3 => [input[0], input[1], input[2], 0, 0, 0, 0, 0],
            4 => [input[0], input[1], input[2], input[3], 0, 0, 0, 0],
            5 => [input[0], input[1], input[2], input[3], input[4], 0, 0, 0],
            6 => [
                input[0], input[1], input[2], input[3], input[4], input[5], 0, 0,
            ],
            7 => [
                input[0], input[1], input[2], input[3], input[4], input[5], input[6], 0,
            ],
            _ => [
                input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
            ],
        };
        crate::slice_ops::advance_slice(input, nbytes);
        Some(crate::byte_codec_wide_ops::decode_le_u64(bytes))
    }
}
