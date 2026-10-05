use creusot_std::prelude::*;
#[derive(Debug)]
pub struct TryGetError {
    /// The number of bytes necessary to get the value
    pub requested: usize,

    /// The number of bytes available in the buffer
    pub available: usize,
}
#[ensures(match result { Ok(value) => input@.len() >= 1 && value@ == input@[0]@ && (^input)@ == input@[1..], Err(error) => input@.len() < 1 && error.requested == 1usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u8(input: &mut &[u8]) -> Result<u8, TryGetError> {
        crate::slice_read_ops::read_u8(input).ok_or(TryGetError {
            requested: 1,
            available: input.len(),
        })
    }
#[ensures(match result { Ok(value) => input@.len() >= 2 && value@ == input@[0]@ * 256 + input@[1]@ && (^input)@ == input@[2..], Err(error) => input@.len() < 2 && error.requested == 2usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u16(input: &mut &[u8]) -> Result<u16, TryGetError> {
        crate::slice_read_ops::read_be_u16(input).ok_or(TryGetError {
            requested: 2,
            available: input.len(),
        })
    }
#[ensures(match result { Ok(value) => input@.len() >= 4 && value@ == input@[0]@ * 16777216 + input@[1]@ * 65536 + input@[2]@ * 256 + input@[3]@ && (^input)@ == input@[4..], Err(error) => input@.len() < 4 && error.requested == 4usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u32(input: &mut &[u8]) -> Result<u32, TryGetError> {
        crate::slice_read_ops::read_be_u32(input).ok_or(TryGetError {
            requested: 4,
            available: input.len(),
        })
    }
