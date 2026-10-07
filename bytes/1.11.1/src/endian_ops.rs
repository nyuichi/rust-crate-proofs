//! Little-endian encoders and checked signed/unsigned scalar reads.
//!
//! The byte sequences are modeled by their scalar byte weights. Checked reads
//! leave a short input unchanged and advance exactly once on success.

use creusot_std::prelude::*;

#[logic(open)]
pub fn signed_u16(word: Int) -> Int {
    if word <= 32_767 {
        word
    } else {
        word - 65_536
    }
}

#[logic(open)]
pub fn signed_u32(word: Int) -> Int {
    if word <= 2_147_483_647 {
        word
    } else {
        word - 4_294_967_296
    }
}

#[inline]
#[ensures(result@ == signed_u16(word@))]
fn signed_from_u16(word: u16) -> i16 {
    if word <= i16::MAX as u16 {
        word as i16
    } else {
        (word - 32_768) as i16 + i16::MIN
    }
}

#[inline]
#[ensures(result@ == signed_u32(word@))]
fn signed_from_u32(word: u32) -> i32 {
    if word <= i32::MAX as u32 {
        word as i32
    } else {
        (word - 2_147_483_648) as i32 + i32::MIN
    }
}

/// Encodes a `u16` in little-endian order.
#[inline]
#[ensures(result@[0]@ == value@ % 256)]
#[ensures(result@[1]@ == value@ / 256)]
pub(crate) fn encode_le_u16(value: u16) -> [u8; 2] {
    let big_endian = crate::byte_codec_ops::encode_be_u16(value);
    [big_endian[1], big_endian[0]]
}

/// Encodes a `u32` in little-endian order.
#[inline]
#[ensures(result@[0]@ == value@ % 256)]
#[ensures(result@[1]@ == value@ / 256 % 256)]
#[ensures(result@[2]@ == value@ / 65_536 % 256)]
#[ensures(result@[3]@ == value@ / 16_777_216)]
pub(crate) fn encode_le_u32(value: u32) -> [u8; 4] {
    let big_endian = crate::byte_codec_ops::encode_be_u32(value);
    [big_endian[3], big_endian[2], big_endian[1], big_endian[0]]
}

/// Reads two little-endian bytes, preserving a short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == input@[0]@ + input@[1]@ * 256
        && (^input)@ == input@[2..],
    None => input@.len() < 2 && (^input)@ == input@
})]
pub(crate) fn read_le_u16(input: &mut &[u8]) -> Option<u16> {
    if input.len() < 2 {
        None
    } else {
        let reversed_bytes = [input[1], input[0]];
        let mut reversed_input: &[u8] = &reversed_bytes;
        let value = crate::slice_read_ops::read_be_u16(&mut reversed_input)?;
        crate::slice_ops::advance_slice(input, 2);
        Some(value)
    }
}

/// Reads four little-endian bytes, preserving a short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == input@[0]@ + input@[1]@ * 256
            + input@[2]@ * 65_536
            + input@[3]@ * 16_777_216
        && (^input)@ == input@[4..],
    None => input@.len() < 4 && (^input)@ == input@
})]
pub(crate) fn read_le_u32(input: &mut &[u8]) -> Option<u32> {
    if input.len() < 4 {
        None
    } else {
        let reversed_bytes = [input[3], input[2], input[1], input[0]];
        let mut reversed_input: &[u8] = &reversed_bytes;
        let value = crate::slice_read_ops::read_be_u32(&mut reversed_input)?;
        crate::slice_ops::advance_slice(input, 4);
        Some(value)
    }
}

/// Reads a big-endian signed 16-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == signed_u16(input@[0]@ * 256 + input@[1]@)
        && (^input)@ == input@[2..],
    None => input@.len() < 2 && (^input)@ == input@
})]
pub(crate) fn read_be_i16(input: &mut &[u8]) -> Option<i16> {
    match crate::slice_read_ops::read_be_u16(input) {
        Some(word) => Some(signed_from_u16(word)),
        None => None,
    }
}

/// Reads a little-endian signed 16-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == signed_u16(input@[0]@ + input@[1]@ * 256)
        && (^input)@ == input@[2..],
    None => input@.len() < 2 && (^input)@ == input@
})]
pub(crate) fn read_le_i16(input: &mut &[u8]) -> Option<i16> {
    match read_le_u16(input) {
        Some(word) => Some(signed_from_u16(word)),
        None => None,
    }
}

/// Reads a big-endian signed 32-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == signed_u32(
            input@[0]@ * 16_777_216
                + input@[1]@ * 65_536
                + input@[2]@ * 256
                + input@[3]@
        )
        && (^input)@ == input@[4..],
    None => input@.len() < 4 && (^input)@ == input@
})]
pub(crate) fn read_be_i32(input: &mut &[u8]) -> Option<i32> {
    match crate::slice_read_ops::read_be_u32(input) {
        Some(word) => Some(signed_from_u32(word)),
        None => None,
    }
}

/// Reads a little-endian signed 32-bit value, preserving short input on failure.
#[inline]
#[ensures(match result {
    Some(value) => input@.len() >= 4
        && value@ == signed_u32(
            input@[0]@
                + input@[1]@ * 256
                + input@[2]@ * 65_536
                + input@[3]@ * 16_777_216
        )
        && (^input)@ == input@[4..],
    None => input@.len() < 4 && (^input)@ == input@
})]
pub(crate) fn read_le_i32(input: &mut &[u8]) -> Option<i32> {
    match read_le_u32(input) {
        Some(word) => Some(signed_from_u32(word)),
        None => None,
    }
}
