//! Supplementary checks of the actual unchanged bit_field 0.10.3 dependency.
//! Signed references are expressed in the corresponding unsigned bit pattern.
#[cfg(kani)]
mod checks {
    use bit_field::BitField;
    macro_rules! primitive_checks {
        ($ty:ty, $unsigned:ty, $bytes:expr, $bits:expr, $layout:ident, $body:ident) => {
            #[kani::proof]
            fn $layout() {
                assert_eq!(core::mem::size_of::<$ty>(), $bytes);
                assert_eq!(<$ty as BitField>::BIT_LENGTH, $bits);
            }
            #[kani::proof]
            fn $body() {
                let x: $ty = kani::any();
                let bit: usize = kani::any();
                let value: bool = kani::any();
                kani::assume(bit < $bits);
                let original_bits = x as $unsigned;
                // Logical right shift in the unsigned representation includes
                // every negative value and the sign-bit position for signed T.
                let expected_get = ((original_bits >> bit) & 1) != 0;
                assert_eq!(x.get_bit(bit), expected_get);
                let mask: $unsigned = (1 as $unsigned) << bit;
                let expected_bits = if value {
                    original_bits | mask
                } else {
                    original_bits & !mask
                };
                let mut y = x;
                let returned = y.set_bit(bit, value);
                assert_eq!(*returned as $unsigned, expected_bits);
                assert_eq!(returned.get_bit(bit), value);
                assert_eq!(((*returned as $unsigned) ^ original_bits) & !mask, 0);
                let replacement: $ty = kani::any();
                *returned = replacement;
                assert_eq!(y, replacement);
            }
        };
    }
    primitive_checks!(u8, u8, 1, 8, u8_layout, u8_get_set);
    primitive_checks!(u16, u16, 2, 16, u16_layout, u16_get_set);
    primitive_checks!(u32, u32, 4, 32, u32_layout, u32_get_set);
    primitive_checks!(u64, u64, 8, 64, u64_layout, u64_get_set);
    primitive_checks!(u128, u128, 16, 128, u128_layout, u128_get_set);
    primitive_checks!(usize, usize, 8, 64, usize_layout, usize_get_set);
    primitive_checks!(i8, u8, 1, 8, i8_layout, i8_get_set);
    primitive_checks!(i16, u16, 2, 16, i16_layout, i16_get_set);
    primitive_checks!(i32, u32, 4, 32, i32_layout, i32_get_set);
    primitive_checks!(i64, u64, 8, 64, i64_layout, i64_get_set);
    primitive_checks!(i128, u128, 16, 128, i128_layout, i128_get_set);
    primitive_checks!(isize, usize, 8, 64, isize_layout, isize_get_set);
}
