//! Checked reads from the `&[u8]` cursor used by the runtime `Buf` impl.
//!
//! Each helper consumes input only on success. Failed reads return `None` and
//! preserve the exact original slice.
use creusot_std::prelude::*;

/// Reads one byte, leaving the input unchanged when it is empty.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 1
        && value@ == input@[0]@
        && (^input)@ == input@[1..],
    None => input@.len() == 0
        && (^input)@ == input@
})]
pub(crate) fn read_u8(input: &mut &[u8]) -> Option<u8> {
    if input.len() == 0 {
        None
    } else {
        let value = input[0];
        crate::slice_ops::advance_slice(input, 1);
        Some(value)
    }
}

/// Reads two big-endian bytes, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == input@[0]@ * 256 + input@[1]@
        && (^input)@ == input@[2..],
    None => input@.len() < 2
        && (^input)@ == input@
})]
pub(crate) fn read_be_u16(input: &mut &[u8]) -> Option<u16> {
    if input.len() < 2 {
        None
    } else {
        let bytes = [input[0], input[1]];
        crate::slice_ops::advance_slice(input, 2);
        Some(crate::byte_codec_ops::decode_be_u16(bytes))
    }
}

/// Reads four big-endian bytes, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == input@[0]@ * 16_777_216
            + input@[1]@ * 65_536
            + input@[2]@ * 256
            + input@[3]@
        && (^input)@ == input@[4..],
    None => input@.len() < 4
        && (^input)@ == input@
})]
pub(crate) fn read_be_u32(input: &mut &[u8]) -> Option<u32> {
    if input.len() < 4 {
        None
    } else {
        let bytes = [input[0], input[1], input[2], input[3]];
        crate::slice_ops::advance_slice(input, 4);
        Some(crate::byte_codec_ops::decode_be_u32(bytes))
    }
}
