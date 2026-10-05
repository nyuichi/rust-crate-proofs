//! Exact checked-read bodies from Buf for &[u8], with concrete receiver adapters.
#![allow(unexpected_cfgs)]
#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;
#[path = "../../../../src/byte_codec_ops.rs"]
mod byte_codec_ops;
#[path = "../../../../src/slice_read_ops.rs"]
mod slice_read_ops;
include!(concat!(env!("OUT_DIR"), "/actual_slice_overrides.rs"));

#[cfg(feature = "wrong_available")]
#[requires(input@.len() == 1)]
#[ensures(match result { Err(error) => error.available == 0usize, Ok(_) => false })]
pub fn reject_wrong_available(input: &mut &[u8]) -> Result<u16, TryGetError> {
    try_get_u16(input)
}
