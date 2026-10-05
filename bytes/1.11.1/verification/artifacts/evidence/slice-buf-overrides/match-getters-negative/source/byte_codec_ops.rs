//! Scalar big-endian byte conversion helpers.
//!
//! These helpers spell out the byte weights so Creusot can prove the relation
//! between scalar values and byte contents without relying on contracts for
//! the standard library's `from_be_bytes` or `to_be_bytes` primitives.

#[cfg(creusot)]
use creusot_std::prelude::*;

/// Reconstructs a `u16` from two big-endian bytes.
#[inline]
#[cfg_attr(creusot, ensures(result@ == bytes@[0]@ * 256 + bytes@[1]@))]
pub(crate) fn decode_be_u16(bytes: [u8; 2]) -> u16 {
    u16::from(bytes[0]) * 256 + u16::from(bytes[1])
}

/// Encodes a `u16` as two big-endian bytes.
#[inline]
#[cfg_attr(creusot, ensures(result@[0]@ == value@ / 256))]
#[cfg_attr(creusot, ensures(result@[1]@ == value@ % 256))]
pub(crate) fn encode_be_u16(value: u16) -> [u8; 2] {
    [(value / 256) as u8, (value % 256) as u8]
}

/// Reconstructs a `u32` from four big-endian bytes.
#[inline]
#[cfg_attr(creusot, ensures(result@ == bytes@[0]@ * 16_777_216 + bytes@[1]@ * 65_536 + bytes@[2]@ * 256 + bytes@[3]@))]
pub(crate) fn decode_be_u32(bytes: [u8; 4]) -> u32 {
    u32::from(bytes[0]) * 16_777_216
        + u32::from(bytes[1]) * 65_536
        + u32::from(bytes[2]) * 256
        + u32::from(bytes[3])
}

/// Encodes a `u32` as four big-endian bytes.
#[inline]
#[cfg_attr(creusot, ensures(result@[0]@ == value@ / 16_777_216))]
#[cfg_attr(creusot, ensures(result@[1]@ == value@ / 65_536 % 256))]
#[cfg_attr(creusot, ensures(result@[2]@ == value@ / 256 % 256))]
#[cfg_attr(creusot, ensures(result@[3]@ == value@ % 256))]
pub(crate) fn encode_be_u32(value: u32) -> [u8; 4] {
    [
        (value / 16_777_216) as u8,
        (value / 65_536 % 256) as u8,
        (value / 256 % 256) as u8,
        (value % 256) as u8,
    ]
}
