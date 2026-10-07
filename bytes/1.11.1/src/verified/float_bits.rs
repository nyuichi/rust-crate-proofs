//! Exact IEEE 754 wire bit patterns for the verified modified API.
//!
//! The legacy `Buf::get_f32*` and `BufMut::put_f32*` methods interpret these
//! words as floating-point values. This API keeps them as integers instead,
//! so every encoded bit, including a NaN payload, remains observable. It does
//! not convert the words to `f32` or `f64`.

use creusot_std::prelude::*;

#[cfg(creusot)]
use super::writes::{append_model_holds, model_be_bytes, model_le_bytes};
use super::{cursor::Cursor, exclusive::ExclusiveBytes};

/// A binary32 wire word. Its view is exactly the stored `u32` bit pattern.
#[derive(creusot_std::prelude::Clone, Copy)]
pub struct Float32Bits {
    bits: u32,
}

impl View for Float32Bits {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.bits@ }
    }
}

impl Float32Bits {
    /// Wraps a binary32 bit pattern without materializing an `f32`.
    #[ensures(result@ == bits@)]
    pub fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    /// Returns the exact stored binary32 bit pattern.
    #[ensures(result@ == self@)]
    pub fn bits(self) -> u32 {
        self.bits
    }
}

/// A binary64 wire word. Its view is exactly the stored `u64` bit pattern.
#[derive(creusot_std::prelude::Clone, Copy)]
pub struct Float64Bits {
    bits: u64,
}

impl View for Float64Bits {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.bits@ }
    }
}

impl Float64Bits {
    /// Wraps a binary64 bit pattern without materializing an `f64`.
    #[ensures(result@ == bits@)]
    pub fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    /// Returns the exact stored binary64 bit pattern.
    #[ensures(result@ == self@)]
    pub fn bits(self) -> u64 {
        self.bits
    }
}

impl<'a> Cursor<'a> {
    /// Reads a big-endian binary32 bit pattern without converting it to `f32`.
    #[ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == self@[0]@ * 16_777_216
                + self@[1]@ * 65_536
                + self@[2]@ * 256
                + self@[3]@
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    })]
    pub fn try_get_f32_bits_be(&mut self) -> Option<Float32Bits> {
        match self.try_get_u32_be() {
            Some(bits) => Some(Float32Bits::from_bits(bits)),
            None => None,
        }
    }

    /// Reads a little-endian binary32 bit pattern without converting it to `f32`.
    #[ensures(match result {
        Some(value) => self@.len() >= 4
            && value@ == self@[0]@
                + self@[1]@ * 256
                + self@[2]@ * 65_536
                + self@[3]@ * 16_777_216
            && (^self)@ == self@[4..],
        None => self@.len() < 4 && (^self)@ == self@
    })]
    pub fn try_get_f32_bits_le(&mut self) -> Option<Float32Bits> {
        match self.try_get_u32_le() {
            Some(bits) => Some(Float32Bits::from_bits(bits)),
            None => None,
        }
    }

    /// Reads a native-endian binary32 bit pattern without converting it to `f32`.
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
    pub fn try_get_f32_bits_ne(&mut self) -> Option<Float32Bits> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_f32_bits_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_f32_bits_be()
        }
    }

    /// Reads a big-endian binary64 bit pattern without converting it to `f64`.
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
    pub fn try_get_f64_bits_be(&mut self) -> Option<Float64Bits> {
        match self.try_get_u64_be() {
            Some(bits) => Some(Float64Bits::from_bits(bits)),
            None => None,
        }
    }

    /// Reads a little-endian binary64 bit pattern without converting it to `f64`.
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
    pub fn try_get_f64_bits_le(&mut self) -> Option<Float64Bits> {
        match self.try_get_u64_le() {
            Some(bits) => Some(Float64Bits::from_bits(bits)),
            None => None,
        }
    }

    /// Reads a native-endian binary64 bit pattern without converting it to `f64`.
    #[cfg_attr(target_endian = "little", ensures(match result {
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
    }))]
    #[cfg_attr(target_endian = "big", ensures(match result {
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
    }))]
    pub fn try_get_f64_bits_ne(&mut self) -> Option<Float64Bits> {
        #[cfg(target_endian = "little")]
        {
            self.try_get_f64_bits_le()
        }
        #[cfg(target_endian = "big")]
        {
            self.try_get_f64_bits_be()
        }
    }
}

impl ExclusiveBytes {
    /// Appends a binary32 bit pattern in big-endian order.
    #[ensures(append_model_holds(
        self@,
        (^self)@,
        model_be_bytes(value@, 4)
    ))]
    pub fn write_f32_bits_be(&mut self, value: Float32Bits) {
        self.write_u32_be(value.bits);
    }

    /// Appends a binary32 bit pattern in little-endian order.
    #[ensures(append_model_holds(
        self@,
        (^self)@,
        model_le_bytes(value@, 4)
    ))]
    pub fn write_f32_bits_le(&mut self, value: Float32Bits) {
        self.write_u32_le(value.bits);
    }

    /// Appends a binary32 bit pattern in native-endian order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(
        self@,
        (^self)@,
        model_le_bytes(value@, 4)
    )))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(
        self@,
        (^self)@,
        model_be_bytes(value@, 4)
    )))]
    pub fn write_f32_bits_ne(&mut self, value: Float32Bits) {
        #[cfg(target_endian = "little")]
        self.write_f32_bits_le(value);
        #[cfg(target_endian = "big")]
        self.write_f32_bits_be(value);
    }

    /// Appends a binary64 bit pattern in big-endian order.
    #[ensures(append_model_holds(
        self@,
        (^self)@,
        model_be_bytes(value@, 8)
    ))]
    pub fn write_f64_bits_be(&mut self, value: Float64Bits) {
        self.write_u64_be(value.bits);
    }

    /// Appends a binary64 bit pattern in little-endian order.
    #[ensures(append_model_holds(
        self@,
        (^self)@,
        model_le_bytes(value@, 8)
    ))]
    pub fn write_f64_bits_le(&mut self, value: Float64Bits) {
        self.write_u64_le(value.bits);
    }

    /// Appends a binary64 bit pattern in native-endian order.
    #[cfg_attr(target_endian = "little", ensures(append_model_holds(
        self@,
        (^self)@,
        model_le_bytes(value@, 8)
    )))]
    #[cfg_attr(target_endian = "big", ensures(append_model_holds(
        self@,
        (^self)@,
        model_be_bytes(value@, 8)
    )))]
    pub fn write_f64_bits_ne(&mut self, value: Float64Bits) {
        #[cfg(target_endian = "little")]
        self.write_f64_bits_le(value);
        #[cfg(target_endian = "big")]
        self.write_f64_bits_be(value);
    }
}

/// Writes a binary32 bit pattern and reads it back in a scoped shared callback.
#[cfg(feature = "std")]
#[ensures(result@ == value@)]
pub fn scoped_f32_bits_roundtrip(value: Float32Bits) -> Float32Bits {
    let mut bytes = ExclusiveBytes::from_vec(alloc::vec::Vec::new());
    bytes.write_f32_bits_be(value);
    proof_assert!(
        model_be_bytes(value@, 4)[0] * 16_777_216
            + model_be_bytes(value@, 4)[1] * 65_536
            + model_be_bytes(value@, 4)[2] * 256
            + model_be_bytes(value@, 4)[3]
            == value@
    );
    let input = bytes.into_vec();
    let (decoded, ()) = super::with_shared_read(
        input,
        |input: &[u8]| {
            let mut cursor = Cursor::new(input);
            cursor.try_get_f32_bits_be().unwrap()
        },
        |_input: &[u8]| (),
    );
    decoded
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use alloc::vec::Vec;

    use super::{scoped_f32_bits_roundtrip, Cursor, ExclusiveBytes, Float32Bits, Float64Bits};

    fn check_f32_order(bits: u32, le: bool) {
        let mut bytes = ExclusiveBytes::from_vec(Vec::new());
        let value = Float32Bits::from_bits(bits);
        if le {
            bytes.write_f32_bits_le(value);
            assert_eq!(bytes.as_slice(), bits.to_le_bytes());
        } else {
            bytes.write_f32_bits_be(value);
            assert_eq!(bytes.as_slice(), bits.to_be_bytes());
        }
        let decoded = {
            let mut cursor = Cursor::new(bytes.as_slice());
            if le {
                cursor.try_get_f32_bits_le()
            } else {
                cursor.try_get_f32_bits_be()
            }
        };
        assert_eq!(decoded.map(Float32Bits::bits), Some(bits));
        bytes.close();
    }

    fn check_f64_order(bits: u64, le: bool) {
        let mut bytes = ExclusiveBytes::from_vec(Vec::new());
        let value = Float64Bits::from_bits(bits);
        if le {
            bytes.write_f64_bits_le(value);
            assert_eq!(bytes.as_slice(), bits.to_le_bytes());
        } else {
            bytes.write_f64_bits_be(value);
            assert_eq!(bytes.as_slice(), bits.to_be_bytes());
        }
        let decoded = {
            let mut cursor = Cursor::new(bytes.as_slice());
            if le {
                cursor.try_get_f64_bits_le()
            } else {
                cursor.try_get_f64_bits_be()
            }
        };
        assert_eq!(decoded.map(Float64Bits::bits), Some(bits));
        bytes.close();
    }

    #[test]
    fn binary32_words_round_trip_all_endian_forms_without_float_conversion() {
        for bits in [
            0x0000_0000u32,
            0x8000_0000,
            0x3fa0_0000,
            0x7f80_0000,
            0xff80_0000,
            0x7fc1_2345,
            0x7f81_2345,
        ] {
            check_f32_order(bits, false);
            check_f32_order(bits, true);

            let bytes = bits.to_ne_bytes();
            let mut cursor = Cursor::new(&bytes);
            assert_eq!(
                cursor.try_get_f32_bits_ne().map(Float32Bits::bits),
                Some(bits)
            );
            assert_eq!(cursor.remaining(), 0);

            let mut owner = ExclusiveBytes::from_vec(Vec::new());
            owner.write_f32_bits_ne(Float32Bits::from_bits(bits));
            assert_eq!(owner.as_slice(), bytes);
            owner.close();
        }

        let short = [0x3f, 0x80, 0x00];
        let mut cursor = Cursor::new(&short);
        assert!(cursor.try_get_f32_bits_be().is_none());
        assert_eq!(cursor.chunk(), &short);
    }

    #[test]
    fn binary64_words_round_trip_all_endian_forms_without_float_conversion() {
        for bits in [
            0x0000_0000_0000_0000u64,
            0x8000_0000_0000_0000,
            0x3ff4_0000_0000_0000,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_1234_5678_9abc,
            0x7ff0_1234_5678_9abc,
        ] {
            check_f64_order(bits, false);
            check_f64_order(bits, true);

            let bytes = bits.to_ne_bytes();
            let mut cursor = Cursor::new(&bytes);
            assert_eq!(
                cursor.try_get_f64_bits_ne().map(Float64Bits::bits),
                Some(bits)
            );
            assert_eq!(cursor.remaining(), 0);

            let mut owner = ExclusiveBytes::from_vec(Vec::new());
            owner.write_f64_bits_ne(Float64Bits::from_bits(bits));
            assert_eq!(owner.as_slice(), bytes);
            owner.close();
        }

        let short = [0x3f, 0xf0, 0, 0, 0, 0, 0];
        let mut cursor = Cursor::new(&short);
        assert!(cursor.try_get_f64_bits_be().is_none());
        assert_eq!(cursor.chunk(), &short);
    }

    #[test]
    fn binary32_word_survives_scoped_shared_read() {
        for bits in [0x0000_0000, 0x8000_0000, 0x7fc1_2345, 0x7f81_2345] {
            assert_eq!(
                scoped_f32_bits_roundtrip(Float32Bits::from_bits(bits)).bits(),
                bits
            );
        }
    }
}
