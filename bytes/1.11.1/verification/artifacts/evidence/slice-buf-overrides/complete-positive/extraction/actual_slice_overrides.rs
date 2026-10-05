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
#[requires(input@.len() >= 1)]
#[ensures(result@ == input@[0]@ && (^input)@ == input@[1..])]
pub fn get_u8(input: &mut &[u8]) -> u8 {
        match try_get_u8(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 2 && value@ == input@[0]@ * 256 + input@[1]@ && (^input)@ == input@[2..], Err(error) => input@.len() < 2 && error.requested == 2usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u16(input: &mut &[u8]) -> Result<u16, TryGetError> {
        crate::slice_read_ops::read_be_u16(input).ok_or(TryGetError {
            requested: 2,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 2)]
#[ensures(result@ == input@[0]@ * 256 + input@[1]@ && (^input)@ == input@[2..])]
pub fn get_u16(input: &mut &[u8]) -> u16 {
        match try_get_u16(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 2 && value@ == input@[0]@ + input@[1]@ * 256 && (^input)@ == input@[2..], Err(error) => input@.len() < 2 && error.requested == 2usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u16_le(input: &mut &[u8]) -> Result<u16, TryGetError> {
        crate::endian_ops::read_le_u16(input).ok_or(TryGetError {
            requested: 2,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 2)]
#[ensures(result@ == input@[0]@ + input@[1]@ * 256 && (^input)@ == input@[2..])]
pub fn get_u16_le(input: &mut &[u8]) -> u16 {
        match try_get_u16_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 2 && value@ == crate::endian_ops::signed_u16(input@[0]@ * 256 + input@[1]@) && (^input)@ == input@[2..], Err(error) => input@.len() < 2 && error.requested == 2usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i16(input: &mut &[u8]) -> Result<i16, TryGetError> {
        crate::endian_ops::read_be_i16(input).ok_or(TryGetError {
            requested: 2,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 2)]
#[ensures(result@ == crate::endian_ops::signed_u16(input@[0]@ * 256 + input@[1]@) && (^input)@ == input@[2..])]
pub fn get_i16(input: &mut &[u8]) -> i16 {
        match try_get_i16(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 2 && value@ == crate::endian_ops::signed_u16(input@[0]@ + input@[1]@ * 256) && (^input)@ == input@[2..], Err(error) => input@.len() < 2 && error.requested == 2usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i16_le(input: &mut &[u8]) -> Result<i16, TryGetError> {
        crate::endian_ops::read_le_i16(input).ok_or(TryGetError {
            requested: 2,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 2)]
#[ensures(result@ == crate::endian_ops::signed_u16(input@[0]@ + input@[1]@ * 256) && (^input)@ == input@[2..])]
pub fn get_i16_le(input: &mut &[u8]) -> i16 {
        match try_get_i16_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 4 && value@ == input@[0]@ * 16777216 + input@[1]@ * 65536 + input@[2]@ * 256 + input@[3]@ && (^input)@ == input@[4..], Err(error) => input@.len() < 4 && error.requested == 4usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u32(input: &mut &[u8]) -> Result<u32, TryGetError> {
        crate::slice_read_ops::read_be_u32(input).ok_or(TryGetError {
            requested: 4,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 4)]
#[ensures(result@ == input@[0]@ * 16777216 + input@[1]@ * 65536 + input@[2]@ * 256 + input@[3]@ && (^input)@ == input@[4..])]
pub fn get_u32(input: &mut &[u8]) -> u32 {
        match try_get_u32(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 4 && value@ == input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 && (^input)@ == input@[4..], Err(error) => input@.len() < 4 && error.requested == 4usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u32_le(input: &mut &[u8]) -> Result<u32, TryGetError> {
        crate::endian_ops::read_le_u32(input).ok_or(TryGetError {
            requested: 4,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 4)]
#[ensures(result@ == input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 && (^input)@ == input@[4..])]
pub fn get_u32_le(input: &mut &[u8]) -> u32 {
        match try_get_u32_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 4 && value@ == crate::endian_ops::signed_u32(input@[0]@ * 16777216 + input@[1]@ * 65536 + input@[2]@ * 256 + input@[3]@) && (^input)@ == input@[4..], Err(error) => input@.len() < 4 && error.requested == 4usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i32(input: &mut &[u8]) -> Result<i32, TryGetError> {
        crate::endian_ops::read_be_i32(input).ok_or(TryGetError {
            requested: 4,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 4)]
#[ensures(result@ == crate::endian_ops::signed_u32(input@[0]@ * 16777216 + input@[1]@ * 65536 + input@[2]@ * 256 + input@[3]@) && (^input)@ == input@[4..])]
pub fn get_i32(input: &mut &[u8]) -> i32 {
        match try_get_i32(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 4 && value@ == crate::endian_ops::signed_u32(input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216) && (^input)@ == input@[4..], Err(error) => input@.len() < 4 && error.requested == 4usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i32_le(input: &mut &[u8]) -> Result<i32, TryGetError> {
        crate::endian_ops::read_le_i32(input).ok_or(TryGetError {
            requested: 4,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 4)]
#[ensures(result@ == crate::endian_ops::signed_u32(input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216) && (^input)@ == input@[4..])]
pub fn get_i32_le(input: &mut &[u8]) -> i32 {
        match try_get_i32_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 8 && value@ == input@[0]@ * 72057594037927936 + input@[1]@ * 281474976710656 + input@[2]@ * 1099511627776 + input@[3]@ * 4294967296 + input@[4]@ * 16777216 + input@[5]@ * 65536 + input@[6]@ * 256 + input@[7]@ && (^input)@ == input@[8..], Err(error) => input@.len() < 8 && error.requested == 8usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u64(input: &mut &[u8]) -> Result<u64, TryGetError> {
        crate::slice_wide_read_ops::read_be_u64(input).ok_or(TryGetError {
            requested: 8,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 8)]
#[ensures(result@ == input@[0]@ * 72057594037927936 + input@[1]@ * 281474976710656 + input@[2]@ * 1099511627776 + input@[3]@ * 4294967296 + input@[4]@ * 16777216 + input@[5]@ * 65536 + input@[6]@ * 256 + input@[7]@ && (^input)@ == input@[8..])]
pub fn get_u64(input: &mut &[u8]) -> u64 {
        match try_get_u64(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 8 && value@ == input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936 && (^input)@ == input@[8..], Err(error) => input@.len() < 8 && error.requested == 8usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u64_le(input: &mut &[u8]) -> Result<u64, TryGetError> {
        crate::slice_wide_read_ops::read_le_u64(input).ok_or(TryGetError {
            requested: 8,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 8)]
#[ensures(result@ == input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936 && (^input)@ == input@[8..])]
pub fn get_u64_le(input: &mut &[u8]) -> u64 {
        match try_get_u64_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 8 && value@ == crate::signed_wide_ops::signed_u64(input@[0]@ * 72057594037927936 + input@[1]@ * 281474976710656 + input@[2]@ * 1099511627776 + input@[3]@ * 4294967296 + input@[4]@ * 16777216 + input@[5]@ * 65536 + input@[6]@ * 256 + input@[7]@) && (^input)@ == input@[8..], Err(error) => input@.len() < 8 && error.requested == 8usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i64(input: &mut &[u8]) -> Result<i64, TryGetError> {
        crate::signed_wide_ops::read_be_i64(input).ok_or(TryGetError {
            requested: 8,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 8)]
#[ensures(result@ == crate::signed_wide_ops::signed_u64(input@[0]@ * 72057594037927936 + input@[1]@ * 281474976710656 + input@[2]@ * 1099511627776 + input@[3]@ * 4294967296 + input@[4]@ * 16777216 + input@[5]@ * 65536 + input@[6]@ * 256 + input@[7]@) && (^input)@ == input@[8..])]
pub fn get_i64(input: &mut &[u8]) -> i64 {
        match try_get_i64(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 8 && value@ == crate::signed_wide_ops::signed_u64(input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936) && (^input)@ == input@[8..], Err(error) => input@.len() < 8 && error.requested == 8usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i64_le(input: &mut &[u8]) -> Result<i64, TryGetError> {
        crate::signed_wide_ops::read_le_i64(input).ok_or(TryGetError {
            requested: 8,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 8)]
#[ensures(result@ == crate::signed_wide_ops::signed_u64(input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936) && (^input)@ == input@[8..])]
pub fn get_i64_le(input: &mut &[u8]) -> i64 {
        match try_get_i64_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 16 && value@ == input@[0]@ * 1329227995784915872903807060280344576 + input@[1]@ * 5192296858534827628530496329220096 + input@[2]@ * 20282409603651670423947251286016 + input@[3]@ * 79228162514264337593543950336 + input@[4]@ * 309485009821345068724781056 + input@[5]@ * 1208925819614629174706176 + input@[6]@ * 4722366482869645213696 + input@[7]@ * 18446744073709551616 + input@[8]@ * 72057594037927936 + input@[9]@ * 281474976710656 + input@[10]@ * 1099511627776 + input@[11]@ * 4294967296 + input@[12]@ * 16777216 + input@[13]@ * 65536 + input@[14]@ * 256 + input@[15]@ && (^input)@ == input@[16..], Err(error) => input@.len() < 16 && error.requested == 16usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u128(input: &mut &[u8]) -> Result<u128, TryGetError> {
        crate::slice_wide_read_ops::read_be_u128(input).ok_or(TryGetError {
            requested: 16,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 16)]
#[ensures(result@ == input@[0]@ * 1329227995784915872903807060280344576 + input@[1]@ * 5192296858534827628530496329220096 + input@[2]@ * 20282409603651670423947251286016 + input@[3]@ * 79228162514264337593543950336 + input@[4]@ * 309485009821345068724781056 + input@[5]@ * 1208925819614629174706176 + input@[6]@ * 4722366482869645213696 + input@[7]@ * 18446744073709551616 + input@[8]@ * 72057594037927936 + input@[9]@ * 281474976710656 + input@[10]@ * 1099511627776 + input@[11]@ * 4294967296 + input@[12]@ * 16777216 + input@[13]@ * 65536 + input@[14]@ * 256 + input@[15]@ && (^input)@ == input@[16..])]
pub fn get_u128(input: &mut &[u8]) -> u128 {
        match try_get_u128(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 16 && value@ == input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936 + input@[8]@ * 18446744073709551616 + input@[9]@ * 4722366482869645213696 + input@[10]@ * 1208925819614629174706176 + input@[11]@ * 309485009821345068724781056 + input@[12]@ * 79228162514264337593543950336 + input@[13]@ * 20282409603651670423947251286016 + input@[14]@ * 5192296858534827628530496329220096 + input@[15]@ * 1329227995784915872903807060280344576 && (^input)@ == input@[16..], Err(error) => input@.len() < 16 && error.requested == 16usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_u128_le(input: &mut &[u8]) -> Result<u128, TryGetError> {
        crate::slice_wide_read_ops::read_le_u128(input).ok_or(TryGetError {
            requested: 16,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 16)]
#[ensures(result@ == input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936 + input@[8]@ * 18446744073709551616 + input@[9]@ * 4722366482869645213696 + input@[10]@ * 1208925819614629174706176 + input@[11]@ * 309485009821345068724781056 + input@[12]@ * 79228162514264337593543950336 + input@[13]@ * 20282409603651670423947251286016 + input@[14]@ * 5192296858534827628530496329220096 + input@[15]@ * 1329227995784915872903807060280344576 && (^input)@ == input@[16..])]
pub fn get_u128_le(input: &mut &[u8]) -> u128 {
        match try_get_u128_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 16 && value@ == crate::signed_wide_ops::signed_u128(input@[0]@ * 1329227995784915872903807060280344576 + input@[1]@ * 5192296858534827628530496329220096 + input@[2]@ * 20282409603651670423947251286016 + input@[3]@ * 79228162514264337593543950336 + input@[4]@ * 309485009821345068724781056 + input@[5]@ * 1208925819614629174706176 + input@[6]@ * 4722366482869645213696 + input@[7]@ * 18446744073709551616 + input@[8]@ * 72057594037927936 + input@[9]@ * 281474976710656 + input@[10]@ * 1099511627776 + input@[11]@ * 4294967296 + input@[12]@ * 16777216 + input@[13]@ * 65536 + input@[14]@ * 256 + input@[15]@) && (^input)@ == input@[16..], Err(error) => input@.len() < 16 && error.requested == 16usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i128(input: &mut &[u8]) -> Result<i128, TryGetError> {
        crate::signed_wide_ops::read_be_i128(input).ok_or(TryGetError {
            requested: 16,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 16)]
#[ensures(result@ == crate::signed_wide_ops::signed_u128(input@[0]@ * 1329227995784915872903807060280344576 + input@[1]@ * 5192296858534827628530496329220096 + input@[2]@ * 20282409603651670423947251286016 + input@[3]@ * 79228162514264337593543950336 + input@[4]@ * 309485009821345068724781056 + input@[5]@ * 1208925819614629174706176 + input@[6]@ * 4722366482869645213696 + input@[7]@ * 18446744073709551616 + input@[8]@ * 72057594037927936 + input@[9]@ * 281474976710656 + input@[10]@ * 1099511627776 + input@[11]@ * 4294967296 + input@[12]@ * 16777216 + input@[13]@ * 65536 + input@[14]@ * 256 + input@[15]@) && (^input)@ == input@[16..])]
pub fn get_i128(input: &mut &[u8]) -> i128 {
        match try_get_i128(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[ensures(match result { Ok(value) => input@.len() >= 16 && value@ == crate::signed_wide_ops::signed_u128(input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936 + input@[8]@ * 18446744073709551616 + input@[9]@ * 4722366482869645213696 + input@[10]@ * 1208925819614629174706176 + input@[11]@ * 309485009821345068724781056 + input@[12]@ * 79228162514264337593543950336 + input@[13]@ * 20282409603651670423947251286016 + input@[14]@ * 5192296858534827628530496329220096 + input@[15]@ * 1329227995784915872903807060280344576) && (^input)@ == input@[16..], Err(error) => input@.len() < 16 && error.requested == 16usize && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_i128_le(input: &mut &[u8]) -> Result<i128, TryGetError> {
        crate::signed_wide_ops::read_le_i128(input).ok_or(TryGetError {
            requested: 16,
            available: input.len(),
        })
    }
#[requires(input@.len() >= 16)]
#[ensures(result@ == crate::signed_wide_ops::signed_u128(input@[0]@ + input@[1]@ * 256 + input@[2]@ * 65536 + input@[3]@ * 16777216 + input@[4]@ * 4294967296 + input@[5]@ * 1099511627776 + input@[6]@ * 281474976710656 + input@[7]@ * 72057594037927936 + input@[8]@ * 18446744073709551616 + input@[9]@ * 4722366482869645213696 + input@[10]@ * 1208925819614629174706176 + input@[11]@ * 309485009821345068724781056 + input@[12]@ * 79228162514264337593543950336 + input@[13]@ * 20282409603651670423947251286016 + input@[14]@ * 5192296858534827628530496329220096 + input@[15]@ * 1329227995784915872903807060280344576) && (^input)@ == input@[16..])]
pub fn get_i128_le(input: &mut &[u8]) -> i128 {
        match try_get_i128_le(input) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[requires(false)]
fn panic_advance(error_info: &TryGetError) -> ! {
    panic!(
        "advance out of bounds: the len is {} but advancing by {}",
        error_info.available, error_info.requested
    );
}
#[requires(false)]
fn panic_does_not_fit(size: usize, nbytes: usize) -> ! {
    panic!(
        "size too large: the integer type can fit {} bytes, but nbytes is {}",
        size, nbytes
    );
}
#[ensures(result@ == input@.len())]
pub fn remaining(input: &&[u8]) -> usize {
        input.len()
    }
#[ensures(result@ == input@)]
pub fn chunk<'a>(input: &'a &[u8]) -> &'a [u8] {
        input
    }
#[requires(nbytes <= 8usize)]
#[ensures(match result { Ok(value) => input@.len() >= nbytes@ && value@ == crate::variable_read_ops::be_weight(input@, nbytes@) && (^input)@ == input@[nbytes@..], Err(error) => input@.len() < nbytes@ && error.requested == nbytes && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_uint(input: &mut &[u8], nbytes: usize) -> Result<u64, TryGetError> {
        let _slice_at = match 8usize.checked_sub(nbytes) {
            Some(slice_at) => slice_at,
            None => panic_does_not_fit(8, nbytes),
        };

        crate::variable_read_ops::read_be_u64(input, nbytes).ok_or(TryGetError {
            requested: nbytes,
            available: input.len(),
        })
    }
#[requires(nbytes <= 8usize && input@.len() >= nbytes@)]
#[ensures(result@ == crate::variable_read_ops::be_weight(input@, nbytes@) && (^input)@ == input@[nbytes@..])]
pub fn get_uint(input: &mut &[u8], nbytes: usize) -> u64 {
        match try_get_uint(input, nbytes) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
#[requires(nbytes <= 8usize)]
#[ensures(match result { Ok(value) => input@.len() >= nbytes@ && value@ == crate::variable_read_ops::le_weight(input@, nbytes@) && (^input)@ == input@[nbytes@..], Err(error) => input@.len() < nbytes@ && error.requested == nbytes && error.available@ == input@.len() && (^input)@ == input@ })]
pub fn try_get_uint_le(input: &mut &[u8], nbytes: usize) -> Result<u64, TryGetError> {
        let _slice_at = match 8usize.checked_sub(nbytes) {
            Some(slice_at) => slice_at,
            None => panic_does_not_fit(8, nbytes),
        };

        crate::variable_read_ops::read_le_u64(input, nbytes).ok_or(TryGetError {
            requested: nbytes,
            available: input.len(),
        })
    }
#[requires(nbytes <= 8usize && input@.len() >= nbytes@)]
#[ensures(result@ == crate::variable_read_ops::le_weight(input@, nbytes@) && (^input)@ == input@[nbytes@..])]
pub fn get_uint_le(input: &mut &[u8], nbytes: usize) -> u64 {
        match try_get_uint_le(input, nbytes) {
            Ok(value) => value,
            Err(error) => panic_advance(&error),
        }
    }
