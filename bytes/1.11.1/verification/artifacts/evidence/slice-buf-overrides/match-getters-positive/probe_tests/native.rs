use bytes_slice_buf_overrides::{try_get_u8, try_get_u16, try_get_u32};
#[test]
fn checked_slice_overrides_preserve_short_input_and_error_metadata() {
    for len in 0..6 {
        let bytes = [1, 2, 3, 4, 5, 6];
        let original = &bytes[..len];
        let mut input = original;
        match try_get_u8(&mut input) {
            Ok(v) => { assert_eq!(v, 1); assert_eq!(input, &original[1..]); }
            Err(e) => { assert_eq!((e.requested,e.available),(1,len)); assert_eq!(input,original); }
        }
        let mut input = original;
        match try_get_u16(&mut input) {
            Ok(v) => { assert_eq!(v, 0x0102); assert_eq!(input, &original[2..]); }
            Err(e) => { assert_eq!((e.requested,e.available),(2,len)); assert_eq!(input,original); }
        }
        let mut input = original;
        match try_get_u32(&mut input) {
            Ok(v) => { assert_eq!(v, 0x01020304); assert_eq!(input, &original[4..]); }
            Err(e) => { assert_eq!((e.requested,e.available),(4,len)); assert_eq!(input,original); }
        }
    }
}

#[test]
fn fixed_width_overrides_cover_unsigned_signed_and_both_endians() {
    use bytes_slice_buf_overrides::*;
    macro_rules! check {
        ($read:ident, $getter:ident, $ty:ty, $width:expr, $decode:ident) => {
            for fill in [0, 1, 127, 128, 255] {
                let bytes: [u8; 18] = core::array::from_fn(|index| (fill as u8).wrapping_add(index as u8));
                for len in 0..=18 {
                    let original = &bytes[..len];
                    let mut input = original;
                    match $read(&mut input) {
                        Ok(value) => {
                            assert!(len >= $width);
                            assert_eq!(value, <$ty>::$decode(original[..$width].try_into().unwrap()));
                            assert_eq!(input, &original[$width..]);
                            let mut normal = original;
                            assert_eq!($getter(&mut normal), value);
                            assert_eq!(normal, input);
                            assert_eq!(remaining(&normal), normal.len());
                            assert_eq!(chunk(&normal), normal);
                        }
                        Err(error) => {
                            assert!(len < $width);
                            assert_eq!((error.requested, error.available), ($width, len));
                            assert_eq!(input, original);
                        }
                    }
                }
            }
        };
    }
    check!(try_get_u16, get_u16, u16, 2, from_be_bytes);
    check!(try_get_u16_le, get_u16_le, u16, 2, from_le_bytes);
    check!(try_get_u32, get_u32, u32, 4, from_be_bytes);
    check!(try_get_u32_le, get_u32_le, u32, 4, from_le_bytes);
    check!(try_get_u64, get_u64, u64, 8, from_be_bytes);
    check!(try_get_u64_le, get_u64_le, u64, 8, from_le_bytes);
    check!(try_get_u128, get_u128, u128, 16, from_be_bytes);
    check!(try_get_u128_le, get_u128_le, u128, 16, from_le_bytes);
    check!(try_get_i16, get_i16, i16, 2, from_be_bytes);
    check!(try_get_i16_le, get_i16_le, i16, 2, from_le_bytes);
    check!(try_get_i32, get_i32, i32, 4, from_be_bytes);
    check!(try_get_i32_le, get_i32_le, i32, 4, from_le_bytes);
    check!(try_get_i64, get_i64, i64, 8, from_be_bytes);
    check!(try_get_i64_le, get_i64_le, i64, 8, from_le_bytes);
    check!(try_get_i128, get_i128, i128, 16, from_be_bytes);
    check!(try_get_i128_le, get_i128_le, i128, 16, from_le_bytes);
}

#[test]
fn variable_width_overrides_cover_zero_width_short_and_long_input() {
    use bytes_slice_buf_overrides::{try_get_uint, try_get_uint_le};
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for width in 0..=8 {
        for len in 0..=10 {
            let original = &bytes[..len];
            let mut input = original;
            match try_get_uint(&mut input, width) {
                Ok(value) => {
                    let expected = original[..width].iter().fold(0u64, |word, &byte| (word << 8) | byte as u64);
                    assert_eq!(value, expected);
                    assert_eq!(input, &original[width..]);
                }
                Err(error) => {
                    assert_eq!((error.requested, error.available), (width, len));
                    assert_eq!(input, original);
                }
            }
            let mut input = original;
            match try_get_uint_le(&mut input, width) {
                Ok(value) => {
                    let expected = original[..width].iter().rev().fold(0u64, |word, &byte| (word << 8) | byte as u64);
                    assert_eq!(value, expected);
                    assert_eq!(input, &original[width..]);
                }
                Err(error) => {
                    assert_eq!((error.requested, error.available), (width, len));
                    assert_eq!(input, original);
                }
            }
        }
    }
}
