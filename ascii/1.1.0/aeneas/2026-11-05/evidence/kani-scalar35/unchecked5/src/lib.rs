use ascii::ToAsciiChar;
#[kani::proof]
fn unchecked_u8(){let value:u8=kani::any();kani::assume(value<128);let result=unsafe{<u8 as ToAsciiChar>::to_ascii_char_unchecked(value)};assert_eq!(u32::from(result.as_byte()),value as u32);}
#[kani::proof]
fn unchecked_i8(){let value:i8=kani::any();kani::assume(value>=0);let result=unsafe{<i8 as ToAsciiChar>::to_ascii_char_unchecked(value)};assert_eq!(u32::from(result.as_byte()),value as u32);}
#[kani::proof]
fn unchecked_u16(){let value:u16=kani::any();kani::assume(value<128);let result=unsafe{<u16 as ToAsciiChar>::to_ascii_char_unchecked(value)};assert_eq!(u32::from(result.as_byte()),value as u32);}
#[kani::proof]
fn unchecked_u32(){let value:u32=kani::any();kani::assume(value<128);let result=unsafe{<u32 as ToAsciiChar>::to_ascii_char_unchecked(value)};assert_eq!(u32::from(result.as_byte()),value as u32);}
#[kani::proof]
fn unchecked_char(){let value_code:u32=kani::any();kani::assume(value_code<128);let value=char::from_u32(value_code).unwrap();let result=unsafe{<char as ToAsciiChar>::to_ascii_char_unchecked(value)};assert_eq!(u32::from(result.as_byte()),value_code);}
