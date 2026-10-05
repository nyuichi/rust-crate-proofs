#![allow(non_snake_case)]

macro_rules! test {
    ($($name:ident($value:expr, $expected:expr))*) => {
        $(
            #[test]
            fn $name() {
                let mut buffer = itoa::Buffer::new();
                let s = buffer.format($value);
                assert_eq!(s, $expected);
            }
        )*
    }
}

test! {
    test_u64_0(0u64, "0")
    test_u64_half(u64::from(u32::MAX), "4294967295")
    test_u64_max(u64::MAX, "18446744073709551615")
    test_i64_min(i64::MIN, "-9223372036854775808")

    test_i16_0(0i16, "0")
    test_i16_min(i16::MIN, "-32768")

    test_u128_0(0u128, "0")
    test_u128_max(u128::MAX, "340282366920938463463374607431768211455")
    test_i128_min(i128::MIN, "-170141183460469231731687303715884105728")
    test_i128_max(i128::MAX, "170141183460469231731687303715884105727")
}

#[test]
fn test_max_str_len() {
    use itoa::Integer as _;

    assert_eq!(i8::MAX_STR_LEN, 4);
    assert_eq!(u8::MAX_STR_LEN, 3);
    assert_eq!(i16::MAX_STR_LEN, 6);
    assert_eq!(u16::MAX_STR_LEN, 5);
    assert_eq!(i32::MAX_STR_LEN, 11);
    assert_eq!(u32::MAX_STR_LEN, 10);
    assert_eq!(i64::MAX_STR_LEN, 20);
    assert_eq!(u64::MAX_STR_LEN, 20);
    assert_eq!(i128::MAX_STR_LEN, 40);
    assert_eq!(u128::MAX_STR_LEN, 39);
}

#[test]
fn test_reused_buffer_across_integer_types() {
    let mut buffer = itoa::Buffer::new();
    macro_rules! check {
        ($($value:expr),* $(,)?) => {$(
            let expected = $value.to_string();
            assert_eq!(buffer.format($value), expected);
        )*};
    }

    check!(
        u8::MIN, 0u8, 9u8, 10u8, 99u8, 100u8, u8::MAX,
        u16::MIN, 9u16, 10u16, 999u16, 1_000u16, 9_999u16, 10_000u16, u16::MAX,
        u32::MIN, 999u32, 1_000u32, 99_999u32, 100_000u32, u32::MAX,
        u64::MIN, 9_999u64, 10_000u64, 999_999_999u64, 1_000_000_000u64, u64::MAX,
        u128::MIN, 9_999_999_999_999_999u128, 10_000_000_000_000_000u128,
        u128::MAX,
        i8::MIN, -10i8, -1i8, 0i8, 1i8, 10i8, i8::MAX,
        i16::MIN, -10i16, -1i16, 0i16, 1i16, 10i16, i16::MAX,
        i32::MIN, -10i32, -1i32, 0i32, 1i32, 10i32, i32::MAX,
        i64::MIN, -10i64, -1i64, 0i64, 1i64, 10i64, i64::MAX,
        i128::MIN, -10i128, -1i128, 0i128, 1i128, 10i128, i128::MAX,
        usize::MIN, 9usize, 10usize, 1_000usize, usize::MAX,
        isize::MIN, -10isize, -1isize, 0isize, 1isize, 10isize, isize::MAX,
    );

    for value in u8::MIN..=u8::MAX {
        assert_eq!(buffer.format(value), value.to_string());
    }
    for value in i8::MIN..=i8::MAX {
        assert_eq!(buffer.format(value), value.to_string());
    }
}
