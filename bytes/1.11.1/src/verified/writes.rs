//! Concrete initialized writes for verified::exclusive::ExclusiveBytes.
//!
//! The append primitive lives in exclusive.rs to keep its Vec field private.

use super::exclusive::ExclusiveBytes;
use creusot_std::prelude::*;

/// The power table used by the integer codec contracts, for exponents 0..15.
#[logic(open(super))]
pub(super) fn byte_power(exponent: Int) -> Int {
    pearlite! {
        if exponent == 0 { 1 }
        else if exponent == 1 { 256 }
        else if exponent == 2 { 65_536 }
        else if exponent == 3 { 16_777_216 }
        else if exponent == 4 { 4_294_967_296 }
        else if exponent == 5 { 1_099_511_627_776 }
        else if exponent == 6 { 281_474_976_710_656 }
        else if exponent == 7 { 72_057_594_037_927_936 }
        else if exponent == 8 { 18_446_744_073_709_551_616 }
        else if exponent == 9 { 4_722_366_482_869_645_213_696 }
        else if exponent == 10 { 1_208_925_819_614_629_174_706_176 }
        else if exponent == 11 { 309_485_009_821_345_068_724_781_056 }
        else if exponent == 12 { 79_228_162_514_264_337_593_543_950_336 }
        else if exponent == 13 { 20_282_409_603_651_670_423_947_251_286_016 }
        else if exponent == 14 { 5_192_296_858_534_827_628_530_496_329_220_096 }
        else if exponent == 15 { 1_329_227_995_784_915_872_903_807_060_280_344_576 }
        else { 1 }
    }
}

#[logic(open(super))]
#[requires(0 <= value)]
#[requires(0 <= width && width <= 16)]
pub(super) fn model_be_bytes(value: Int, width: Int) -> Seq<Int> {
    pearlite! {
        Seq::create(width, |index: Int|
            value / byte_power(width - 1 - index) % 256)
    }
}

#[logic(open(super))]
#[requires(0 <= value)]
#[requires(0 <= width && width <= 16)]
pub(super) fn model_le_bytes(value: Int, width: Int) -> Seq<Int> {
    pearlite! {
        Seq::create(width, |index: Int|
            value / byte_power(index) % 256)
    }
}

/// Relates an old byte sequence to its appended integer-byte model. The
/// prefix is preserved, and every byte in the suffix has its exact value.
#[logic(open(super))]
pub(super) fn append_model_holds(
    old: Seq<u8>,
    new: Seq<u8>,
    suffix: Seq<Int>,
) -> bool {
    pearlite! {
        new.len() == old.len() + suffix.len()
            && (forall<i: Int> 0 <= i && i < old.len() ==> new[i] == old[i])
            && (forall<j: Int> 0 <= j && j < suffix.len() ==>
                new[old.len() + j]@ == suffix[j])
    }
}

impl ExclusiveBytes {
    /// Appends all bytes from `source`.
    #[ensures((^self)@ == self@.concat(source@))]
    pub fn extend_from_slice(&mut self, source: &[u8]) {
        self.append_initialized_slice(source);
    }

    /// Appends one byte.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 1)))]
    pub fn write_u8(&mut self, value: u8) {
        self.append_initialized_slice(&[value]);
    }

    /// Appends one two's-complement byte.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u8)@, 1)))]
    pub fn write_i8(&mut self, value: i8) {
        self.write_u8(value as u8);
    }

    /// Appends a `u16` in big-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 2)))]
    pub fn write_u16_be(&mut self, value: u16) {
        let encoded = crate::byte_codec_ops::encode_be_u16(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u16` in little-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 2)))]
    pub fn write_u16_le(&mut self, value: u16) {
        let encoded = crate::endian_ops::encode_le_u16(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u32` in big-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 4)))]
    pub fn write_u32_be(&mut self, value: u32) {
        let encoded = crate::byte_codec_ops::encode_be_u32(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u32` in little-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 4)))]
    pub fn write_u32_le(&mut self, value: u32) {
        let encoded = crate::endian_ops::encode_le_u32(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u64` in big-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 8)))]
    pub fn write_u64_be(&mut self, value: u64) {
        let encoded = crate::byte_codec_wide_ops::encode_be_u64(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u64` in little-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 8)))]
    pub fn write_u64_le(&mut self, value: u64) {
        let encoded = crate::byte_codec_wide_ops::encode_le_u64(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u128` in big-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 16)))]
    pub fn write_u128_be(&mut self, value: u128) {
        let encoded = crate::byte_codec_wide_ops::encode_be_u128(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends a `u128` in little-endian order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 16)))]
    pub fn write_u128_le(&mut self, value: u128) {
        let encoded = crate::byte_codec_wide_ops::encode_le_u128(value);
        self.append_initialized_slice(&encoded);
    }

    /// Appends an `i16` in big-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u16)@, 2)))]
    pub fn write_i16_be(&mut self, value: i16) {
        self.write_u16_be(value as u16);
    }

    /// Appends an `i16` in little-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u16)@, 2)))]
    pub fn write_i16_le(&mut self, value: i16) {
        self.write_u16_le(value as u16);
    }

    /// Appends an `i32` in big-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u32)@, 4)))]
    pub fn write_i32_be(&mut self, value: i32) {
        self.write_u32_be(value as u32);
    }

    /// Appends an `i32` in little-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u32)@, 4)))]
    pub fn write_i32_le(&mut self, value: i32) {
        self.write_u32_le(value as u32);
    }

    /// Appends an `i64` in big-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u64)@, 8)))]
    pub fn write_i64_be(&mut self, value: i64) {
        self.write_u64_be(value as u64);
    }

    /// Appends an `i64` in little-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u64)@, 8)))]
    pub fn write_i64_le(&mut self, value: i64) {
        self.write_u64_le(value as u64);
    }

    /// Appends an `i128` in big-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u128)@, 16)))]
    pub fn write_i128_be(&mut self, value: i128) {
        self.write_u128_be(value as u128);
    }

    /// Appends an `i128` in little-endian two's-complement order.
    #[ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u128)@, 16)))]
    pub fn write_i128_le(&mut self, value: i128) {
        self.write_u128_le(value as u128);
    }

    /// Appends a `u16` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 2))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 2))))]
    pub fn write_u16_ne(&mut self, value: u16) {
        #[cfg(target_endian = "little")]
        self.write_u16_le(value);
        #[cfg(target_endian = "big")]
        self.write_u16_be(value);
    }

    /// Appends a `u32` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 4))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 4))))]
    pub fn write_u32_ne(&mut self, value: u32) {
        #[cfg(target_endian = "little")]
        self.write_u32_le(value);
        #[cfg(target_endian = "big")]
        self.write_u32_be(value);
    }

    /// Appends a `u64` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 8))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 8))))]
    pub fn write_u64_ne(&mut self, value: u64) {
        #[cfg(target_endian = "little")]
        self.write_u64_le(value);
        #[cfg(target_endian = "big")]
        self.write_u64_be(value);
    }

    /// Appends a `u128` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes(value@, 16))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes(value@, 16))))]
    pub fn write_u128_ne(&mut self, value: u128) {
        #[cfg(target_endian = "little")]
        self.write_u128_le(value);
        #[cfg(target_endian = "big")]
        self.write_u128_be(value);
    }

    /// Appends an `i16` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u16)@, 2))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u16)@, 2))))]
    pub fn write_i16_ne(&mut self, value: i16) {
        #[cfg(target_endian = "little")]
        self.write_i16_le(value);
        #[cfg(target_endian = "big")]
        self.write_i16_be(value);
    }

    /// Appends an `i32` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u32)@, 4))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u32)@, 4))))]
    pub fn write_i32_ne(&mut self, value: i32) {
        #[cfg(target_endian = "little")]
        self.write_i32_le(value);
        #[cfg(target_endian = "big")]
        self.write_i32_be(value);
    }

    /// Appends an `i64` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u64)@, 8))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u64)@, 8))))]
    pub fn write_i64_ne(&mut self, value: i64) {
        #[cfg(target_endian = "little")]
        self.write_i64_le(value);
        #[cfg(target_endian = "big")]
        self.write_i64_be(value);
    }

    /// Appends an `i128` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(self@, (^self)@, model_le_bytes((value as u128)@, 16))))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(self@, (^self)@, model_be_bytes((value as u128)@, 16))))]
    pub fn write_i128_ne(&mut self, value: i128) {
        #[cfg(target_endian = "little")]
        self.write_i128_le(value);
        #[cfg(target_endian = "big")]
        self.write_i128_be(value);
    }

    /// Appends the low `nbytes` of `value` in big-endian order.
    /// Widths above eight are rejected without changing the byte sequence.
    #[ensures(result == (nbytes@ <= 8))]
    #[ensures(if result {
        append_model_holds(self@, (^self)@, model_be_bytes(value@, nbytes@))
    } else {
        (^self)@ == self@
    })]
    pub fn try_write_uint_be(&mut self, value: u64, nbytes: usize) -> bool {
        if nbytes > 8 {
            false
        } else {
            let encoded_array = crate::byte_codec_wide_ops::encode_be_u64(value);
            let encoded: &[u8] = &encoded_array;
            match nbytes {
                0 => {}
                1 => self.append_initialized_slice(&encoded[7..8]),
                2 => self.append_initialized_slice(&encoded[6..8]),
                3 => self.append_initialized_slice(&encoded[5..8]),
                4 => self.append_initialized_slice(&encoded[4..8]),
                5 => self.append_initialized_slice(&encoded[3..8]),
                6 => self.append_initialized_slice(&encoded[2..8]),
                7 => self.append_initialized_slice(&encoded[1..8]),
                _ => self.append_initialized_slice(&encoded[0..8]),
            }
            true
        }
    }

    /// Appends the low `nbytes` of `value` in little-endian order.
    /// Widths above eight are rejected without changing the byte sequence.
    #[ensures(result == (nbytes@ <= 8))]
    #[ensures(if result {
        append_model_holds(self@, (^self)@, model_le_bytes(value@, nbytes@))
    } else {
        (^self)@ == self@
    })]
    pub fn try_write_uint_le(&mut self, value: u64, nbytes: usize) -> bool {
        if nbytes > 8 {
            false
        } else {
            let encoded_array = crate::byte_codec_wide_ops::encode_le_u64(value);
            let encoded: &[u8] = &encoded_array;
            match nbytes {
                0 => {}
                1 => self.append_initialized_slice(&encoded[0..1]),
                2 => self.append_initialized_slice(&encoded[0..2]),
                3 => self.append_initialized_slice(&encoded[0..3]),
                4 => self.append_initialized_slice(&encoded[0..4]),
                5 => self.append_initialized_slice(&encoded[0..5]),
                6 => self.append_initialized_slice(&encoded[0..6]),
                7 => self.append_initialized_slice(&encoded[0..7]),
                _ => self.append_initialized_slice(&encoded[0..8]),
            }
            true
        }
    }

    /// Appends `nbytes` low bytes of a signed value in big-endian order.
    /// Widths above eight are rejected without changing the byte sequence.
    #[ensures(result == (nbytes@ <= 8))]
    #[ensures(if result {
        append_model_holds(self@, (^self)@, model_be_bytes((value as u64)@, nbytes@))
    } else {
        (^self)@ == self@
    })]
    pub fn try_write_int_be(&mut self, value: i64, nbytes: usize) -> bool {
        self.try_write_uint_be(value as u64, nbytes)
    }

    /// Appends `nbytes` low bytes of a signed value in little-endian order.
    /// Widths above eight are rejected without changing the byte sequence.
    #[ensures(result == (nbytes@ <= 8))]
    #[ensures(if result {
        append_model_holds(self@, (^self)@, model_le_bytes((value as u64)@, nbytes@))
    } else {
        (^self)@ == self@
    })]
    pub fn try_write_int_le(&mut self, value: i64, nbytes: usize) -> bool {
        self.try_write_uint_le(value as u64, nbytes)
    }

    /// Appends `nbytes` low unsigned bytes in the target's native byte order.
    /// Widths above eight are rejected without changing the byte sequence.
    #[cfg_attr(target_endian = "little", ensures(result == (nbytes@ <= 8)))]
    #[cfg_attr(target_endian = "little", ensures(if result {
        append_model_holds(self@, (^self)@, model_le_bytes(value@, nbytes@))
    } else { (^self)@ == self@ }))]
    #[cfg_attr(target_endian = "big", ensures(result == (nbytes@ <= 8)))]
    #[cfg_attr(target_endian = "big", ensures(if result {
        append_model_holds(self@, (^self)@, model_be_bytes(value@, nbytes@))
    } else { (^self)@ == self@ }))]
    pub fn try_write_uint_ne(&mut self, value: u64, nbytes: usize) -> bool {
        #[cfg(target_endian = "little")]
        { self.try_write_uint_le(value, nbytes) }
        #[cfg(target_endian = "big")]
        { self.try_write_uint_be(value, nbytes) }
    }

    /// Appends `nbytes` low signed bytes in the target's native byte order.
    /// Widths above eight are rejected without changing the byte sequence.
    #[cfg_attr(target_endian = "little", ensures(result == (nbytes@ <= 8)))]
    #[cfg_attr(target_endian = "little", ensures(if result {
        append_model_holds(self@, (^self)@, model_le_bytes((value as u64)@, nbytes@))
    } else { (^self)@ == self@ }))]
    #[cfg_attr(target_endian = "big", ensures(result == (nbytes@ <= 8)))]
    #[cfg_attr(target_endian = "big", ensures(if result {
        append_model_holds(self@, (^self)@, model_be_bytes((value as u64)@, nbytes@))
    } else { (^self)@ == self@ }))]
    pub fn try_write_int_ne(&mut self, value: i64, nbytes: usize) -> bool {
        #[cfg(target_endian = "little")]
        { self.try_write_int_le(value, nbytes) }
        #[cfg(target_endian = "big")]
        { self.try_write_int_be(value, nbytes) }
    }
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::ExclusiveBytes;
    use crate::verified::cursor::Cursor;
    use std::{vec, vec::Vec};

    fn check_u16(value: u16) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_u16_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u16_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_u16_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u16_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_u16_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u16_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_u32(value: u32) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_u32_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u32_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_u32_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u32_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_u32_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u32_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_u64(value: u64) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_u64_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u64_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_u64_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u64_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_u64_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u64_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_u128(value: u128) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_u128_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u128_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_u128_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u128_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_u128_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_u128_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_i16(value: i16) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_i16_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i16_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_i16_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i16_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_i16_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i16_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_i32(value: i32) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_i32_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i32_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_i32_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i32_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_i32_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i32_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_i64(value: i64) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_i64_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i64_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_i64_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i64_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_i64_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i64_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn check_i128(value: i128) {
        let mut be = ExclusiveBytes::from_vec(Vec::new());
        be.write_i128_be(value);
        let written = be.as_slice();
        assert_eq!(written, value.to_be_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i128_be(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut le = ExclusiveBytes::from_vec(Vec::new());
        le.write_i128_le(value);
        let written = le.as_slice();
        assert_eq!(written, value.to_le_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i128_le(), Some(value));
        assert_eq!(cursor.remaining(), 0);

        let mut native = ExclusiveBytes::from_vec(Vec::new());
        native.write_i128_ne(value);
        let written = native.as_slice();
        assert_eq!(written, value.to_ne_bytes());
        let mut cursor = Cursor::new(written);
        assert_eq!(cursor.try_get_i128_ne(), Some(value));
        assert_eq!(cursor.remaining(), 0);
    }

    fn signed_low_width(value: i64, width: usize) -> i64 {
        if width == 0 {
            0
        } else if width == 8 {
            value
        } else {
            let bits = width * 8;
            let mask = (1u64 << bits) - 1;
            let word = (value as u64) & mask;
            let sign = 1u64 << (bits - 1);
            if word & sign == 0 {
                word as i64
            } else {
                (word | !mask) as i64
            }
        }
    }

    fn unsigned_low_width(value: u64, width: usize) -> u64 {
        if width == 0 {
            0
        } else if width == 8 {
            value
        } else {
            value & ((1u64 << (width * 8)) - 1)
        }
    }

    fn decode_be(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0, |value, byte| (value << 8) | u64::from(*byte))
    }

    fn decode_le(bytes: &[u8]) -> u64 {
        bytes.iter().enumerate().fold(0, |value, (index, byte)| {
            value | (u64::from(*byte) << (8 * index))
        })
    }

    #[test]
    fn extend_from_slice_appends_initialized_bytes_and_cursor_reads_them() {
        let mut bytes = ExclusiveBytes::from_vec(vec![0x12, 0x34]);
        bytes.extend_from_slice(&[0x56, 0x78]);
        assert_eq!(bytes.as_slice(), &[0x12, 0x34, 0x56, 0x78]);
        let mut cursor = Cursor::new(bytes.as_slice());
        assert_eq!(cursor.try_get_u32_be(), Some(0x1234_5678));
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn fixed_width_endian_writes_round_trip_extrema() {
        for value in [0, u16::MAX] { check_u16(value); }
        for value in [0, u32::MAX] { check_u32(value); }
        for value in [0, u64::MAX] { check_u64(value); }
        for value in [0, u128::MAX] { check_u128(value); }
    }

    #[test]
    fn one_byte_unsigned_and_signed_writes_round_trip() {
        for value in [0, u8::MAX] {
            let mut bytes = ExclusiveBytes::from_vec(Vec::new());
            bytes.write_u8(value);
            assert_eq!(bytes.as_slice(), &[value]);
            let mut cursor = Cursor::new(bytes.as_slice());
            assert_eq!(cursor.try_get_u8(), Some(value));
            assert_eq!(cursor.remaining(), 0);
        }
        for value in [i8::MIN, -1, 0, i8::MAX] {
            let mut bytes = ExclusiveBytes::from_vec(Vec::new());
            bytes.write_i8(value);
            assert_eq!(bytes.as_slice(), &value.to_be_bytes());
            let mut cursor = Cursor::new(bytes.as_slice());
            assert_eq!(cursor.try_get_i8(), Some(value));
            assert_eq!(cursor.remaining(), 0);
        }
    }

    #[test]
    fn signed_fixed_endian_and_native_writes_round_trip_extrema() {
        for value in [i16::MIN, -1, 0, i16::MAX] { check_i16(value); }
        for value in [i32::MIN, -1, 0, i32::MAX] { check_i32(value); }
        for value in [i64::MIN, -1, 0, i64::MAX] { check_i64(value); }
        for value in [i128::MIN, -1, 0, i128::MAX] { check_i128(value); }
    }

    #[test]
    fn written_u16_is_read_after_real_shared_close_lifecycle() {
        for value in [0, u16::MAX] {
            for reverse in [false, true] {
                let (read, remaining, first_last, second_last) =
                    crate::verified::scoped_write_read_u16_be(value, reverse);
                assert_eq!(read, Some(value));
                assert_eq!(remaining, 0);
                assert_ne!(first_last, second_last);
            }
        }
    }

    #[test]
    fn variable_width_writes_cover_zero_through_nine_and_read_back() {
        let value = 0x0123_4567_89ab_cdefu64;
        let mut be_full = ExclusiveBytes::from_vec(Vec::new());
        assert!(be_full.try_write_uint_be(value, 8));
        let be_bytes = be_full.as_slice().to_vec();
        let mut le_full = ExclusiveBytes::from_vec(Vec::new());
        assert!(le_full.try_write_uint_le(value, 8));
        let le_bytes = le_full.as_slice().to_vec();

        for width in 0..=8 {
            let mut be = ExclusiveBytes::from_vec(vec![0xaa]);
            assert!(be.try_write_uint_be(value, width));
            let expected = &be_bytes[8 - width..];
            assert_eq!(&be.as_slice()[1..], expected);
            let expected_value = decode_be(expected);
            let mut cursor = Cursor::new(&be.as_slice()[1..]);
            assert_eq!(cursor.try_get_uint_be(width), Some(expected_value));
            assert_eq!(cursor.remaining(), 0);

            let mut le = ExclusiveBytes::from_vec(vec![0xaa]);
            assert!(le.try_write_uint_le(value, width));
            let expected = &le_bytes[..width];
            assert_eq!(&le.as_slice()[1..], expected);
            let expected_value = decode_le(expected);
            let mut cursor = Cursor::new(&le.as_slice()[1..]);
            assert_eq!(cursor.try_get_uint_le(width), Some(expected_value));
            assert_eq!(cursor.remaining(), 0);

            let mut native = ExclusiveBytes::from_vec(Vec::new());
            assert!(native.try_write_uint_ne(value, width));
            let big_native = cfg!(target_endian = "big");
            let native_expected = if big_native {
                &be_bytes[8 - width..]
            } else {
                &le_bytes[..width]
            };
            assert_eq!(native.as_slice(), native_expected);
            let mut cursor = Cursor::new(native.as_slice());
            assert_eq!(cursor.try_get_uint_ne(width), Some(unsigned_low_width(value, width)));
            assert_eq!(cursor.remaining(), 0);
        }

        let mut invalid_be = ExclusiveBytes::from_vec(vec![0xaa]);
        assert!(!invalid_be.try_write_uint_be(value, 9));
        assert_eq!(invalid_be.as_slice(), &[0xaa]);
        let mut invalid_le = ExclusiveBytes::from_vec(vec![0xaa]);
        assert!(!invalid_le.try_write_uint_le(value, 9));
        assert_eq!(invalid_le.as_slice(), &[0xaa]);
    }

    #[test]
    fn variable_signed_writes_cover_all_widths_and_endian_aliases() {
        for value in [i64::MIN, -1, 0, i64::MAX] {
            let big = value.to_be_bytes();
            let little = value.to_le_bytes();
            for width in 0..=8 {
                let mut be = ExclusiveBytes::from_vec(Vec::new());
                assert!(be.try_write_int_be(value, width));
                assert_eq!(be.as_slice(), &big[8 - width..]);
                let mut cursor = Cursor::new(be.as_slice());
                assert_eq!(cursor.try_get_int_be(width), Some(signed_low_width(value, width)));
                assert_eq!(cursor.remaining(), 0);

                let mut le = ExclusiveBytes::from_vec(Vec::new());
                assert!(le.try_write_int_le(value, width));
                assert_eq!(le.as_slice(), &little[..width]);
                let mut cursor = Cursor::new(le.as_slice());
                assert_eq!(cursor.try_get_int_le(width), Some(signed_low_width(value, width)));
                assert_eq!(cursor.remaining(), 0);

                let mut native = ExclusiveBytes::from_vec(Vec::new());
                assert!(native.try_write_int_ne(value, width));
                let native_expected = if cfg!(target_endian = "big") {
                    &big[8 - width..]
                } else {
                    &little[..width]
                };
                assert_eq!(native.as_slice(), native_expected);
                let mut cursor = Cursor::new(native.as_slice());
                assert_eq!(cursor.try_get_int_ne(width), Some(signed_low_width(value, width)));
                assert_eq!(cursor.remaining(), 0);
            }
        }

        let mut invalid_be = ExclusiveBytes::from_vec(vec![0xaa]);
        assert!(!invalid_be.try_write_int_be(-1, 9));
        assert_eq!(invalid_be.as_slice(), &[0xaa]);
        let mut invalid_le = ExclusiveBytes::from_vec(vec![0xaa]);
        assert!(!invalid_le.try_write_int_le(-1, 9));
        assert_eq!(invalid_le.as_slice(), &[0xaa]);
        let mut invalid_native = ExclusiveBytes::from_vec(vec![0xaa]);
        assert!(!invalid_native.try_write_int_ne(-1, 9));
        assert_eq!(invalid_native.as_slice(), &[0xaa]);
    }
}
