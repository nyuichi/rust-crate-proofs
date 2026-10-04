//! Proof probe for checked reads from a real slice cursor.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;

#[path = "../../../../src/byte_codec_ops.rs"]
mod byte_codec_ops;

#[path = "../../../../src/slice_read_ops.rs"]
mod slice_read_ops;

#[ensures(match result {
    Some(value) => input@.len() >= 1
        && value@ == input@[0]@
        && (^input)@ == input@[1..],
    None => input@.len() == 0
        && (^input)@ == input@
})]
pub fn read_u8(input: &mut &[u8]) -> Option<u8> {
    slice_read_ops::read_u8(input)
}

#[ensures(match result {
    Some(value) => input@.len() >= 2
        && value@ == input@[0]@ * 256 + input@[1]@
        && (^input)@ == input@[2..],
    None => input@.len() < 2
        && (^input)@ == input@
})]
pub fn read_be_u16(input: &mut &[u8]) -> Option<u16> {
    slice_read_ops::read_be_u16(input)
}

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
pub fn read_be_u32(input: &mut &[u8]) -> Option<u32> {
    slice_read_ops::read_be_u32(input)
}

/// Negative control: preserve the state but claim an incorrect byte value.
#[cfg(feature = "wrong_value")]
#[requires(input@.len() >= 1)]
#[ensures(match result {
    Some(value) => value@ == input@[0]@ + 1,
    None => false
})]
pub fn wrong_value(input: &mut &[u8]) -> Option<u8> {
    slice_read_ops::read_u8(input)
}

/// Negative control: this broken short-read path consumes its one-byte input.
#[cfg(feature = "short_consumed")]
#[requires(input@.len() == 1)]
#[ensures((^input)@ == input@)]
pub fn consumes_short_u16(input: &mut &[u8]) -> Option<u16> {
    if input.len() < 2 {
        crate::slice_ops::advance_slice(input, input.len());
        None
    } else {
        let bytes = [input[0], input[1]];
        crate::slice_ops::advance_slice(input, 2);
        Some(crate::byte_codec_ops::decode_be_u16(bytes))
    }
}
