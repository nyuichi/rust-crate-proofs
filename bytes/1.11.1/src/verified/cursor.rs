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
    fn fixed_width_reads_match_native_endian_decoding() {
        let mut cursor = Cursor::new(&[
            0x12, 0x34, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x78, 0x56, 0x34, 0x12,
        ]);
        assert_eq!(cursor.try_get_u16_be(), Some(u16::from_be_bytes([0x12, 0x34])));
        assert_eq!(cursor.try_get_u16_le(), Some(u16::from_le_bytes([0x12, 0x34])));
        assert_eq!(cursor.try_get_u32_be(), Some(u32::from_be_bytes([0x12, 0x34, 0x56, 0x78])));
        assert_eq!(cursor.try_get_u32_le(), Some(u32::from_le_bytes([0x78, 0x56, 0x34, 0x12])));
        assert_eq!(cursor.remaining(), 0);

        let wide = [
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let mut cursor = Cursor::new(&wide);
        assert_eq!(cursor.try_get_u64_be(), Some(u64::from_be_bytes(wide[..8].try_into().unwrap())));
        let mut cursor = Cursor::new(&wide);
        assert_eq!(cursor.try_get_u128_be(), Some(u128::from_be_bytes(wide.try_into().unwrap())));
        let mut cursor = Cursor::new(&wide);
        assert_eq!(cursor.try_get_i128_be(), Some(i128::from_be_bytes(wide.try_into().unwrap())));

        let wide_le = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let mut cursor = Cursor::new(&wide_le);
        assert_eq!(cursor.try_get_u64_le(), Some(u64::from_le_bytes(wide_le[..8].try_into().unwrap())));
        let mut cursor = Cursor::new(&wide_le);
        assert_eq!(cursor.try_get_u128_le(), Some(u128::from_le_bytes(wide_le.try_into().unwrap())));
        let mut cursor = Cursor::new(&wide_le);
        assert_eq!(cursor.try_get_i128_le(), Some(i128::from_le_bytes(wide_le.try_into().unwrap())));

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
        assert_eq!(cursor.try_get_uint_le(5), Some(u64::from_le_bytes([0x78, 0x9a, 0xbc, 0xde, 0xf0, 0, 0, 0])));
        assert_eq!(cursor.remaining(), 0);

        let mut short = Cursor::new(&[0x12, 0x34]);
        assert_eq!(short.try_get_uint_be(8), None);
        assert_eq!(short.chunk(), &[0x12, 0x34]);
        assert_eq!(short.try_get_uint_le(9), None);
        assert_eq!(short.chunk(), &[0x12, 0x34]);
    }
}
