//! A checked, immutable byte cursor for the modified verified API.
//!
//! Reads consume bytes only when the full requested value is available. A
//! failed read leaves the cursor at exactly the same byte sequence.

use creusot_std::prelude::*;

/// A view into an immutable byte sequence with a checked read position.
pub struct Cursor<'a> {
    remaining: &'a [u8],
}

impl<'a> View for Cursor<'a> {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.remaining@ }
    }
}

#[logic(open)]
pub fn variable_width_modulus(nbytes: Int) -> Int {
    if nbytes <= 0 {
        1
    } else if nbytes == 1 {
        256
    } else if nbytes == 2 {
        65_536
    } else if nbytes == 3 {
        16_777_216
    } else if nbytes == 4 {
        4_294_967_296
    } else if nbytes == 5 {
        1_099_511_627_776
    } else if nbytes == 6 {
        281_474_976_710_656
    } else if nbytes == 7 {
        72_057_594_037_927_936
    } else {
        18_446_744_073_709_551_616
    }
}

#[logic(open)]
pub fn signed_variable_value(word: Int, nbytes: Int) -> Int {
    if nbytes <= 0 {
        0
    } else if word < variable_width_modulus(nbytes) / 2 {
        word
    } else {
        word - variable_width_modulus(nbytes)
    }
}

/// Gives an already-read unsigned word its signed two's-complement value.
#[requires(nbytes@ <= 8)]
#[requires(word@ < variable_width_modulus(nbytes@))]
#[ensures(result@ == signed_variable_value(word@, nbytes@))]
fn signed_from_variable_word(word: u64, nbytes: usize) -> i64 {
    let signed_word = crate::signed_wide_ops::signed_from_u64(word);
    match nbytes {
        0 => 0,
        1 => {
            if word < 128 {
                signed_word
            } else {
                signed_word - 256
            }
        }
        2 => {
            if word < 32_768 {
                signed_word
            } else {
                signed_word - 65_536
            }
        }
        3 => {
            if word < 8_388_608 {
                signed_word
            } else {
                signed_word - 16_777_216
            }
        }
        4 => {
            if word < 2_147_483_648 {
                signed_word
            } else {
                signed_word - 4_294_967_296
            }
        }
        5 => {
            if word < 549_755_813_888 {
                signed_word
            } else {
                signed_word - 1_099_511_627_776
            }
        }
        6 => {
            if word < 140_737_488_355_328 {
                signed_word
            } else {
                signed_word - 281_474_976_710_656
            }
        }
        7 => {
            if word < 36_028_797_018_963_968 {
                signed_word
            } else {
                signed_word - 72_057_594_037_927_936
            }
        }
        _ => signed_word,
    }
}

#[logic(open)]
pub fn be_u128_weight(bytes: Seq<u8>) -> Int {
    pearlite! {
        bytes[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
            + bytes[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096
            + bytes[2]@ * 20_282_409_603_651_670_423_947_251_286_016
            + bytes[3]@ * 79_228_162_514_264_337_593_543_950_336
            + bytes[4]@ * 309_485_009_821_345_068_724_781_056
            + bytes[5]@ * 1_208_925_819_614_629_174_706_176
            + bytes[6]@ * 4_722_366_482_869_645_213_696
            + bytes[7]@ * 18_446_744_073_709_551_616
            + bytes[8]@ * 72_057_594_037_927_936
            + bytes[9]@ * 281_474_976_710_656
            + bytes[10]@ * 1_099_511_627_776
            + bytes[11]@ * 4_294_967_296
            + bytes[12]@ * 16_777_216
            + bytes[13]@ * 65_536
            + bytes[14]@ * 256
            + bytes[15]@
    }
}

#[logic(open)]
pub fn le_u128_weight(bytes: Seq<u8>) -> Int {
    pearlite! {
        bytes[0]@
            + bytes[1]@ * 256
            + bytes[2]@ * 65_536
            + bytes[3]@ * 16_777_216
            + bytes[4]@ * 4_294_967_296
            + bytes[5]@ * 1_099_511_627_776
            + bytes[6]@ * 281_474_976_710_656
            + bytes[7]@ * 72_057_594_037_927_936
            + bytes[8]@ * 18_446_744_073_709_551_616
            + bytes[9]@ * 4_722_366_482_869_645_213_696
            + bytes[10]@ * 1_208_925_819_614_629_174_706_176
            + bytes[11]@ * 309_485_009_821_345_068_724_781_056
            + bytes[12]@ * 79_228_162_514_264_337_593_543_950_336
            + bytes[13]@ * 20_282_409_603_651_670_423_947_251_286_016
            + bytes[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096
            + bytes[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
    }
}

impl<'a> Cursor<'a> {
    /// Creates a cursor over the complete input slice.
    #[ensures(result@ == input@)]
    pub fn new(input: &'a [u8]) -> Self {
        Self { remaining: input }
    }

    /// Returns the number of bytes left to read.
    #[ensures(result@ == self@.len())]
    pub fn remaining(&self) -> usize {
        self.remaining.len()
    }

    /// Returns the complete unread suffix.
    #[ensures(result@ == self@)]
    pub fn chunk(&self) -> &'a [u8] {
        self.remaining
    }

    /// Advances by `count` bytes, returning false without changes if it is too large.
    #[ensures(result == (count@ <= self@.len()))]
    #[ensures(if result {
        (^self)@ == self@[count@..]
    } else {
        (^self)@ == self@
    })]
    pub fn advance(&mut self, count: usize) -> bool {
        if count > self.remaining.len() {
            false
        } else {
            crate::slice_ops::advance_slice(&mut self.remaining, count);
            true
        }
    }

    /// Copies the unread prefix into `dst` and advances on success.
    ///
    /// If `dst` is longer than the unread input, both the cursor and `dst` are
    /// left unchanged.
    #[ensures(match result {
        true => (^dst)@ == self@[0..dst@.len()]
            && (^self)@ == self@[dst@.len()..],
        false => (^dst)@ == dst@ && (^self)@ == self@,
    })]
    #[ensures(result == (dst@.len() <= self@.len()))]
    pub fn copy_to_slice(&mut self, dst: &mut [u8]) -> bool {
        if dst.len() > self.remaining.len() {
            false
        } else {
            crate::slice_ops::copy_to_slice(&mut self.remaining, dst);
            true
        }
    }

    /// Reads as much of the unread prefix as fits in `dst` and returns its length.
    #[ensures(result@ == if dst@.len() < self@.len() { dst@.len() } else { self@.len() })]
    #[ensures((^self)@ == self@[result@..])]
    #[ensures(forall<i: Int> 0 <= i && i < result@ ==> (^dst)@[i] == self@[i])]
    #[ensures(forall<i: Int> result@ <= i && i < dst@.len() ==> (^dst)@[i] == dst@[i])]
    pub fn read_prefix(&mut self, dst: &mut [u8]) -> usize {
        let count = core::cmp::min(dst.len(), self.remaining.len());
        self.copy_to_slice(&mut dst[..count]);
        count
    }

    /// Reads one byte, leaving the cursor unchanged when it is empty.
    #[ensures(match result {
        Some(value) => self@.len() >= 1
            && value@ == self@[0]@
            && (^self)@ == self@[1..],
        None => self@.len() == 0 && (^self)@ == self@
    })]
    pub fn try_get_u8(&mut self) -> Option<u8> {
        crate::slice_read_ops::read_u8(&mut self.remaining)
    }

    /// Reads a signed byte, leaving an empty cursor unchanged.
    #[ensures(match result {
        Some(value) => self@.len() >= 1
            && value@ == if self@[0]@ <= 127 { self@[0]@ } else { self@[0]@ - 256 }
            && (^self)@ == self@[1..],
        None => self@.len() == 0 && (^self)@ == self@
    })]
    pub fn try_get_i8(&mut self) -> Option<i8> {
        let byte = self.try_get_u8()?;
        let signed = if byte <= i8::MAX as u8 {
            byte as i8
        } else {
            (byte - 128) as i8 + i8::MIN
        };
        Some(signed)
    }

    /// Reads a big-endian `u16`.
    #[ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == self@[0]@ * 256 + self@[1]@
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    })]
    pub fn try_get_u16_be(&mut self) -> Option<u16> {
        crate::slice_read_ops::read_be_u16(&mut self.remaining)
    }

    /// Reads a little-endian `u16`.
    #[ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == self@[0]@ + self@[1]@ * 256
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    })]
    pub fn try_get_u16_le(&mut self) -> Option<u16> {
        crate::endian_ops::read_le_u16(&mut self.remaining)
    }

    /// Reads a big-endian `u32`.
    #[ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == self@[0]@ * 16_777_216
                + self@[1]@ * 65_536
                + self@[2]@ * 256
                + self@[3]@
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    })]
    pub fn try_get_u32_be(&mut self) -> Option<u32> {
        crate::slice_read_ops::read_be_u32(&mut self.remaining)
    }

    /// Reads a little-endian `u32`.
    #[ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == self@[0]@
                + self@[1]@ * 256
                + self@[2]@ * 65_536
                + self@[3]@ * 16_777_216
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    })]
    pub fn try_get_u32_le(&mut self) -> Option<u32> {
        crate::endian_ops::read_le_u32(&mut self.remaining)
    }

    /// Reads a big-endian `u64`.
    #[ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == (self@[0]@ * 72_057_594_037_927_936
                + self@[1]@ * 281_474_976_710_656
                + self@[2]@ * 1_099_511_627_776
                + self@[3]@ * 4_294_967_296
                + self@[4]@ * 16_777_216
                + self@[5]@ * 65_536
                + self@[6]@ * 256
                + self@[7]@)
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    })]
    pub fn try_get_u64_be(&mut self) -> Option<u64> {
        crate::slice_wide_read_ops::read_be_u64(&mut self.remaining)
    }

    /// Reads a little-endian `u64`.
    #[ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == (self@[0]@
                + self@[1]@ * 256
                + self@[2]@ * 65_536
                + self@[3]@ * 16_777_216
                + self@[4]@ * 4_294_967_296
                + self@[5]@ * 1_099_511_627_776
                + self@[6]@ * 281_474_976_710_656
                + self@[7]@ * 72_057_594_037_927_936)
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    })]
    pub fn try_get_u64_le(&mut self) -> Option<u64> {
        crate::slice_wide_read_ops::read_le_u64(&mut self.remaining)
    }

    /// Reads a big-endian `u128`.
    #[ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == (self@[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
                + self@[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096
                + self@[2]@ * 20_282_409_603_651_670_423_947_251_286_016
                + self@[3]@ * 79_228_162_514_264_337_593_543_950_336
                + self@[4]@ * 309_485_009_821_345_068_724_781_056
                + self@[5]@ * 1_208_925_819_614_629_174_706_176
                + self@[6]@ * 4_722_366_482_869_645_213_696
                + self@[7]@ * 18_446_744_073_709_551_616
                + self@[8]@ * 72_057_594_037_927_936
                + self@[9]@ * 281_474_976_710_656
                + self@[10]@ * 1_099_511_627_776
                + self@[11]@ * 4_294_967_296
                + self@[12]@ * 16_777_216
                + self@[13]@ * 65_536
                + self@[14]@ * 256
                + self@[15]@)
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    })]
    pub fn try_get_u128_be(&mut self) -> Option<u128> {
        crate::slice_wide_read_ops::read_be_u128(&mut self.remaining)
    }

    /// Reads a little-endian `u128`.
    #[ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == (self@[0]@
                + self@[1]@ * 256
                + self@[2]@ * 65_536
                + self@[3]@ * 16_777_216
                + self@[4]@ * 4_294_967_296
                + self@[5]@ * 1_099_511_627_776
                + self@[6]@ * 281_474_976_710_656
                + self@[7]@ * 72_057_594_037_927_936
                + self@[8]@ * 18_446_744_073_709_551_616
                + self@[9]@ * 4_722_366_482_869_645_213_696
                + self@[10]@ * 1_208_925_819_614_629_174_706_176
                + self@[11]@ * 309_485_009_821_345_068_724_781_056
                + self@[12]@ * 79_228_162_514_264_337_593_543_950_336
                + self@[13]@ * 20_282_409_603_651_670_423_947_251_286_016
                + self@[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096
                + self@[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576)
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    })]
    pub fn try_get_u128_le(&mut self) -> Option<u128> {
        crate::slice_wide_read_ops::read_le_u128(&mut self.remaining)
    }

    /// Reads a big-endian two's-complement `i16`.
    #[ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == crate::endian_ops::signed_u16(
                self@[0]@ * 256 + self@[1]@
            )
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    })]
    pub fn try_get_i16_be(&mut self) -> Option<i16> {
        crate::endian_ops::read_be_i16(&mut self.remaining)
    }

    /// Reads a little-endian two's-complement `i16`.
    #[ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == crate::endian_ops::signed_u16(
                self@[0]@ + self@[1]@ * 256
            )
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    })]
    pub fn try_get_i16_le(&mut self) -> Option<i16> {
        crate::endian_ops::read_le_i16(&mut self.remaining)
    }

    /// Reads a big-endian two's-complement `i32`.
    #[ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == crate::endian_ops::signed_u32(
                self@[0]@ * 16_777_216
                    + self@[1]@ * 65_536
                    + self@[2]@ * 256
                    + self@[3]@
            )
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    })]
    pub fn try_get_i32_be(&mut self) -> Option<i32> {
        crate::endian_ops::read_be_i32(&mut self.remaining)
    }

    /// Reads a little-endian two's-complement `i32`.
    #[ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == crate::endian_ops::signed_u32(
                self@[0]@
                    + self@[1]@ * 256
                    + self@[2]@ * 65_536
                    + self@[3]@ * 16_777_216
            )
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    })]
    pub fn try_get_i32_le(&mut self) -> Option<i32> {
        crate::endian_ops::read_le_i32(&mut self.remaining)
    }

    /// Reads a big-endian two's-complement `i64`.
    #[ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == crate::signed_wide_ops::signed_u64(
                self@[0]@ * 72_057_594_037_927_936
                    + self@[1]@ * 281_474_976_710_656
                    + self@[2]@ * 1_099_511_627_776
                    + self@[3]@ * 4_294_967_296
                    + self@[4]@ * 16_777_216
                    + self@[5]@ * 65_536
                    + self@[6]@ * 256
                    + self@[7]@
            )
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    })]
    pub fn try_get_i64_be(&mut self) -> Option<i64> {
        crate::signed_wide_ops::read_be_i64(&mut self.remaining)
    }

    /// Reads a little-endian two's-complement `i64`.
    #[ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == crate::signed_wide_ops::signed_u64(
                self@[0]@
                    + self@[1]@ * 256
                    + self@[2]@ * 65_536
                    + self@[3]@ * 16_777_216
                    + self@[4]@ * 4_294_967_296
                    + self@[5]@ * 1_099_511_627_776
                    + self@[6]@ * 281_474_976_710_656
                    + self@[7]@ * 72_057_594_037_927_936
            )
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    })]
    pub fn try_get_i64_le(&mut self) -> Option<i64> {
        crate::signed_wide_ops::read_le_i64(&mut self.remaining)
    }

    /// Reads a big-endian two's-complement `i128`.
    #[ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == crate::signed_wide_ops::signed_u128(
                self@[0]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
                    + self@[1]@ * 5_192_296_858_534_827_628_530_496_329_220_096
                    + self@[2]@ * 20_282_409_603_651_670_423_947_251_286_016
                    + self@[3]@ * 79_228_162_514_264_337_593_543_950_336
                    + self@[4]@ * 309_485_009_821_345_068_724_781_056
                    + self@[5]@ * 1_208_925_819_614_629_174_706_176
                    + self@[6]@ * 4_722_366_482_869_645_213_696
                    + self@[7]@ * 18_446_744_073_709_551_616
                    + self@[8]@ * 72_057_594_037_927_936
                    + self@[9]@ * 281_474_976_710_656
                    + self@[10]@ * 1_099_511_627_776
                    + self@[11]@ * 4_294_967_296
                    + self@[12]@ * 16_777_216
                    + self@[13]@ * 65_536
                    + self@[14]@ * 256
                    + self@[15]@
            )
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    })]
    pub fn try_get_i128_be(&mut self) -> Option<i128> {
        crate::signed_wide_ops::read_be_i128(&mut self.remaining)
    }

    /// Reads a little-endian two's-complement `i128`.
    #[ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == crate::signed_wide_ops::signed_u128(
                self@[0]@
                    + self@[1]@ * 256
                    + self@[2]@ * 65_536
                    + self@[3]@ * 16_777_216
                    + self@[4]@ * 4_294_967_296
                    + self@[5]@ * 1_099_511_627_776
                    + self@[6]@ * 281_474_976_710_656
                    + self@[7]@ * 72_057_594_037_927_936
                    + self@[8]@ * 18_446_744_073_709_551_616
                    + self@[9]@ * 4_722_366_482_869_645_213_696
                    + self@[10]@ * 1_208_925_819_614_629_174_706_176
                    + self@[11]@ * 309_485_009_821_345_068_724_781_056
                    + self@[12]@ * 79_228_162_514_264_337_593_543_950_336
                    + self@[13]@ * 20_282_409_603_651_670_423_947_251_286_016
                    + self@[14]@ * 5_192_296_858_534_827_628_530_496_329_220_096
                    + self@[15]@ * 1_329_227_995_784_915_872_903_807_060_280_344_576
            )
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    })]
    pub fn try_get_i128_le(&mut self) -> Option<i128> {
        crate::signed_wide_ops::read_le_i128(&mut self.remaining)
    }

    /// Reads an unsigned big-endian integer of at most eight bytes.
    ///
    /// A width greater than eight is rejected without changing the cursor.
    #[ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == crate::variable_read_ops::be_weight(self@, nbytes@)
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@)
            && (^self)@ == self@
    })]
    pub fn try_get_uint_be(&mut self, nbytes: usize) -> Option<u64> {
        if nbytes > 8 {
            None
        } else {
            crate::variable_read_ops::read_be_u64(&mut self.remaining, nbytes)
        }
    }

    /// Reads an unsigned little-endian integer of at most eight bytes.
    ///
    /// A width greater than eight is rejected without changing the cursor.
    #[ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == crate::variable_read_ops::le_weight(self@, nbytes@)
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@)
            && (^self)@ == self@
    })]
    pub fn try_get_uint_le(&mut self, nbytes: usize) -> Option<u64> {
        if nbytes > 8 {
            None
        } else {
            crate::variable_read_ops::read_le_u64(&mut self.remaining, nbytes)
        }
    }

    /// Reads a signed big-endian integer of at most eight bytes.
    #[ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == signed_variable_value(
                crate::variable_read_ops::be_weight(self@, nbytes@), nbytes@
            )
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@)
            && (^self)@ == self@
    })]
    pub fn try_get_int_be(&mut self, nbytes: usize) -> Option<i64> {
        let word = self.try_get_uint_be(nbytes)?;
        Some(signed_from_variable_word(word, nbytes))
    }

    /// Reads a signed little-endian integer of at most eight bytes.
    #[ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == signed_variable_value(
                crate::variable_read_ops::le_weight(self@, nbytes@), nbytes@
            )
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@)
            && (^self)@ == self@
    })]
    pub fn try_get_int_le(&mut self, nbytes: usize) -> Option<i64> {
        let word = self.try_get_uint_le(nbytes)?;
        Some(signed_from_variable_word(word, nbytes))
    }

    /// Reads an unsigned integer in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == crate::variable_read_ops::le_weight(self@, nbytes@)
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@) && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == crate::variable_read_ops::be_weight(self@, nbytes@)
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@) && (^self)@ == self@
    }))]
    pub fn try_get_uint_ne(&mut self, nbytes: usize) -> Option<u64> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_uint_le(nbytes)
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_uint_be(nbytes)
        }
    }

    /// Reads a signed integer in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == signed_variable_value(
                crate::variable_read_ops::le_weight(self@, nbytes@), nbytes@
            )
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@) && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => nbytes@ <= 8
            && self@.len() >= nbytes@
            && value@ == signed_variable_value(
                crate::variable_read_ops::be_weight(self@, nbytes@), nbytes@
            )
            && (^self)@ == self@[nbytes@..],
        None => (nbytes@ > 8 || self@.len() < nbytes@) && (^self)@ == self@
    }))]
    pub fn try_get_int_ne(&mut self, nbytes: usize) -> Option<i64> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_int_le(nbytes)
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_int_be(nbytes)
        }
    }

    /// Reads a `u16` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == self@[0]@ + self@[1]@ * 256
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == self@[0]@ * 256 + self@[1]@
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    }))]
    pub fn try_get_u16_ne(&mut self) -> Option<u16> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_u16_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_u16_be()
        }
    }

    /// Reads a `u32` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == self@[0]@
                + self@[1]@ * 256
                + self@[2]@ * 65_536
                + self@[3]@ * 16_777_216
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == self@[0]@ * 16_777_216
                + self@[1]@ * 65_536
                + self@[2]@ * 256
                + self@[3]@
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    }))]
    pub fn try_get_u32_ne(&mut self) -> Option<u32> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_u32_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_u32_be()
        }
    }

    /// Reads a `u64` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == crate::variable_read_ops::le_weight(self@, 8)
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == crate::variable_read_ops::be_weight(self@, 8)
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    }))]
    pub fn try_get_u64_ne(&mut self) -> Option<u64> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_u64_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_u64_be()
        }
    }

    /// Reads a `u128` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == le_u128_weight(self@)
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == be_u128_weight(self@)
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    }))]
    pub fn try_get_u128_ne(&mut self) -> Option<u128> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_u128_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_u128_be()
        }
    }

    /// Reads an `i16` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == crate::endian_ops::signed_u16(
                self@[0]@ + self@[1]@ * 256
            )
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 2
            && value@ == crate::endian_ops::signed_u16(
                self@[0]@ * 256 + self@[1]@
            )
            && (^self)@ == self@[2..],
        None => self@.len() < 2 && (^self)@ == self@
    }))]
    pub fn try_get_i16_ne(&mut self) -> Option<i16> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_i16_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_i16_be()
        }
    }

    /// Reads an `i32` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == crate::endian_ops::signed_u32(
                self@[0]@
                    + self@[1]@ * 256
                    + self@[2]@ * 65_536
                    + self@[3]@ * 16_777_216
            )
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == crate::endian_ops::signed_u32(
                self@[0]@ * 16_777_216
                    + self@[1]@ * 65_536
                    + self@[2]@ * 256
                    + self@[3]@
            )
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    }))]
    pub fn try_get_i32_ne(&mut self) -> Option<i32> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_i32_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_i32_be()
        }
    }

    /// Reads an `i64` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == crate::signed_wide_ops::signed_u64(
                crate::variable_read_ops::le_weight(self@, 8)
            )
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 8
            && value@ == crate::signed_wide_ops::signed_u64(
                crate::variable_read_ops::be_weight(self@, 8)
            )
            && (^self)@ == self@[8..],
        None => self@.len() < 8 && (^self)@ == self@
    }))]
    pub fn try_get_i64_ne(&mut self) -> Option<i64> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_i64_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_i64_be()
        }
    }

    /// Reads an `i128` in the target's native byte order.
    #[cfg_attr(target_endian = "little", ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == crate::signed_wide_ops::signed_u128(le_u128_weight(self@))
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
        Some(value) => self@.len() >= 16
            && value@ == crate::signed_wide_ops::signed_u128(be_u128_weight(self@))
            && (^self)@ == self@[16..],
        None => self@.len() < 16 && (^self)@ == self@
    }))]
    pub fn try_get_i128_ne(&mut self) -> Option<i128> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_i128_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_i128_be()
        }
    }
}

#[cfg(feature = "std")]
impl std::io::Read for Cursor<'_> {
    #[ensures(match result {
        Ok(count) => count@ == if dst@.len() < self@.len() { dst@.len() } else { self@.len() }
            && (^self)@ == self@[count@..]
            && (forall<i: Int> 0 <= i && i < count@ ==> (^dst)@[i] == self@[i])
            && (forall<i: Int> count@ <= i && i < dst@.len() ==> (^dst)@[i] == dst@[i]),
        Err(_) => false,
    })]
    fn read(&mut self, dst: &mut [u8]) -> std::io::Result<usize> {
        Ok(self.read_prefix(dst))
    }
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::Cursor;

    #[test]
    fn cursor_basics_and_nonconsuming_failures() {
        let input = [10, 20, 30];
        let mut cursor = Cursor::new(&input);
        assert_eq!(cursor.remaining(), 3);
        assert_eq!(cursor.chunk(), &input);
        assert!(!cursor.advance(4));
        assert_eq!(cursor.chunk(), &input);
        assert!(cursor.advance(1));
        assert_eq!(cursor.chunk(), &[20, 30]);
        assert_eq!(cursor.try_get_u32_be(), None);
        assert_eq!(cursor.chunk(), &[20, 30]);

        let mut destination = [0xaa; 3];
        assert!(!cursor.copy_to_slice(&mut destination));
        assert_eq!(destination, [0xaa; 3]);
        assert_eq!(cursor.chunk(), &[20, 30]);

        let mut short_destination = [0; 1];
        assert!(cursor.copy_to_slice(&mut short_destination));
        assert_eq!(short_destination, [20]);
        assert_eq!(cursor.chunk(), &[30]);
    }

    #[test]
    fn partial_prefix_reads_preserve_unwritten_destination_suffix() {
        let input = [1, 2];
        let mut cursor = Cursor::new(&input);
        let mut dst = [0xaa; 4];
        assert_eq!(cursor.read_prefix(&mut dst), 2);
        assert_eq!(dst, [1, 2, 0xaa, 0xaa]);
        assert_eq!(cursor.chunk(), &[]);

        let mut empty = Cursor::new(&[]);
        let mut untouched = [0xbb; 2];
        assert_eq!(empty.read_prefix(&mut untouched), 0);
        assert_eq!(untouched, [0xbb; 2]);

        let mut cursor = Cursor::new(&[9, 8]);
        let mut empty_dst = [];
        assert_eq!(cursor.read_prefix(&mut empty_dst), 0);
        assert_eq!(cursor.chunk(), &[9, 8]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn standard_read_uses_the_checked_partial_prefix_body() {
        use std::io::Read;

        let input = [4, 5];
        let mut cursor = Cursor::new(&input);
        let mut dst = [0xaa; 3];
        assert_eq!(Read::read(&mut cursor, &mut dst).unwrap(), 2);
        assert_eq!(dst, [4, 5, 0xaa]);
        assert_eq!(cursor.chunk(), &[]);
    }

    #[test]
    fn fixed_width_reads_match_native_endian_decoding() {
        let mut cursor = Cursor::new(&[
            0x12, 0x34, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x78, 0x56, 0x34, 0x12,
        ]);
        assert_eq!(
            cursor.try_get_u16_be(),
            Some(u16::from_be_bytes([0x12, 0x34]))
        );
        assert_eq!(
            cursor.try_get_u16_le(),
            Some(u16::from_le_bytes([0x12, 0x34]))
        );
        assert_eq!(
            cursor.try_get_u32_be(),
            Some(u32::from_be_bytes([0x12, 0x34, 0x56, 0x78]))
        );
        assert_eq!(
            cursor.try_get_u32_le(),
            Some(u32::from_le_bytes([0x78, 0x56, 0x34, 0x12]))
        );
        assert_eq!(cursor.remaining(), 0);

        let wide = [
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ];
        let mut cursor = Cursor::new(&wide);
        assert_eq!(
            cursor.try_get_u64_be(),
            Some(u64::from_be_bytes(wide[..8].try_into().unwrap()))
        );
        let mut cursor = Cursor::new(&wide);
        assert_eq!(
            cursor.try_get_u128_be(),
            Some(u128::from_be_bytes(wide.try_into().unwrap()))
        );
        let mut cursor = Cursor::new(&wide);
        assert_eq!(
            cursor.try_get_i128_be(),
            Some(i128::from_be_bytes(wide.try_into().unwrap()))
        );

        let wide_le = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ];
        let mut cursor = Cursor::new(&wide_le);
        assert_eq!(
            cursor.try_get_u64_le(),
            Some(u64::from_le_bytes(wide_le[..8].try_into().unwrap()))
        );
        let mut cursor = Cursor::new(&wide_le);
        assert_eq!(
            cursor.try_get_u128_le(),
            Some(u128::from_le_bytes(wide_le.try_into().unwrap()))
        );
        let mut cursor = Cursor::new(&wide_le);
        assert_eq!(
            cursor.try_get_i128_le(),
            Some(i128::from_le_bytes(wide_le.try_into().unwrap()))
        );

        let mut signed = Cursor::new(&[0x80, 0x00, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(signed.try_get_i16_be(), Some(i16::MIN));
        assert_eq!(signed.try_get_i32_be(), Some(-1));
        assert_eq!(signed.remaining(), 0);

        let mut signed_le = Cursor::new(&[0x00, 0x80, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(signed_le.try_get_i16_le(), Some(i16::MIN));
        assert_eq!(signed_le.try_get_i32_le(), Some(-1));
        assert_eq!(signed_le.remaining(), 0);

        let mut signed_wide = Cursor::new(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(signed_wide.try_get_i64_be(), Some(-1));
        let mut signed_wide_le = Cursor::new(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(signed_wide_le.try_get_i64_le(), Some(-1));
    }

    #[test]
    fn variable_width_reads_check_width_and_preserve_short_input() {
        let bytes = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let mut cursor = Cursor::new(&bytes);
        assert_eq!(cursor.try_get_uint_be(0), Some(0));
        assert_eq!(cursor.chunk(), &bytes);
        assert_eq!(cursor.try_get_uint_be(3), Some(0x12_34_56));
        assert_eq!(cursor.chunk(), &bytes[3..]);
        assert_eq!(
            cursor.try_get_uint_le(5),
            Some(u64::from_le_bytes([0x78, 0x9a, 0xbc, 0xde, 0xf0, 0, 0, 0]))
        );
        assert_eq!(cursor.remaining(), 0);

        let mut short = Cursor::new(&[0x12, 0x34]);
        assert_eq!(short.try_get_uint_be(8), None);
        assert_eq!(short.chunk(), &[0x12, 0x34]);
        assert_eq!(short.try_get_uint_le(9), None);
        assert_eq!(short.chunk(), &[0x12, 0x34]);
    }

    #[test]
    fn signed_byte_variable_and_native_endian_reads() {
        let signed_bytes = [0x7f, 0x80, 0xff];
        let mut signed = Cursor::new(&signed_bytes);
        assert_eq!(signed.try_get_i8(), Some(127));
        assert_eq!(signed.try_get_i8(), Some(i8::MIN));
        assert_eq!(signed.try_get_i8(), Some(-1));
        assert_eq!(signed.try_get_i8(), None);
        assert_eq!(signed.chunk(), &[]);

        let u16_bytes = 0x8123u16.to_ne_bytes();
        let mut u16_cursor = Cursor::new(&u16_bytes);
        assert_eq!(
            u16_cursor.try_get_u16_ne(),
            Some(u16::from_ne_bytes(u16_bytes))
        );
        let mut i16_cursor = Cursor::new(&u16_bytes);
        assert_eq!(
            i16_cursor.try_get_i16_ne(),
            Some(i16::from_ne_bytes(u16_bytes))
        );

        let u32_bytes = 0x8123_4567u32.to_ne_bytes();
        let mut u32_cursor = Cursor::new(&u32_bytes);
        assert_eq!(
            u32_cursor.try_get_u32_ne(),
            Some(u32::from_ne_bytes(u32_bytes))
        );
        let mut i32_cursor = Cursor::new(&u32_bytes);
        assert_eq!(
            i32_cursor.try_get_i32_ne(),
            Some(i32::from_ne_bytes(u32_bytes))
        );

        let u64_bytes = 0x8123_4567_89ab_cdefu64.to_ne_bytes();
        let mut u64_cursor = Cursor::new(&u64_bytes);
        assert_eq!(
            u64_cursor.try_get_u64_ne(),
            Some(u64::from_ne_bytes(u64_bytes))
        );
        let mut i64_cursor = Cursor::new(&u64_bytes);
        assert_eq!(
            i64_cursor.try_get_i64_ne(),
            Some(i64::from_ne_bytes(u64_bytes))
        );

        let u128_bytes = 0x8123_4567_89ab_cdef_0123_4567_89ab_cdefu128.to_ne_bytes();
        let mut u128_cursor = Cursor::new(&u128_bytes);
        assert_eq!(
            u128_cursor.try_get_u128_ne(),
            Some(u128::from_ne_bytes(u128_bytes))
        );
        let mut i128_cursor = Cursor::new(&u128_bytes);
        assert_eq!(
            i128_cursor.try_get_i128_ne(),
            Some(i128::from_ne_bytes(u128_bytes))
        );

        let mut variable_be = Cursor::new(&[0xff, 0xff, 0xfe]);
        assert_eq!(variable_be.try_get_int_be(3), Some(-2));
        assert_eq!(variable_be.remaining(), 0);
        let mut variable_le = Cursor::new(&[0x00, 0x80]);
        assert_eq!(variable_le.try_get_int_le(2), Some(i16::MIN.into()));
        assert_eq!(variable_le.remaining(), 0);
        let mut zero_width = Cursor::new(&[0x80]);
        assert_eq!(zero_width.try_get_int_be(0), Some(0));
        assert_eq!(zero_width.chunk(), &[0x80]);

        let mut short = Cursor::new(&[0x80]);
        assert_eq!(short.try_get_int_be(2), None);
        assert_eq!(short.chunk(), &[0x80]);
        assert_eq!(short.try_get_int_le(9), None);
        assert_eq!(short.try_get_int_ne(9), None);
        assert_eq!(short.chunk(), &[0x80]);

        let native_bytes = [0x34, 0x92];
        let mut native_uint = Cursor::new(&native_bytes);
        assert_eq!(
            native_uint.try_get_uint_ne(2),
            Some(u16::from_ne_bytes(native_bytes).into())
        );
        let mut native_int = Cursor::new(&native_bytes);
        assert_eq!(
            native_int.try_get_int_ne(2),
            Some(i16::from_ne_bytes(native_bytes).into())
        );
    }
}
