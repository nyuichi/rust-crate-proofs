use ascii::{AsciiChar, ToAsciiChar};

#[kani::proof]
fn transmute_model_on_valid_discriminants() {
    let byte: u8 = kani::any();
    kani::assume(byte < 128);
    let value: AsciiChar = unsafe { core::mem::transmute(byte) };
    assert_eq!(value.as_byte(), byte);
}

#[kani::proof]
fn checked_u8_full_domain() {
    let value: u8 = kani::any();
    match value.to_ascii_char() {
        Ok(c) => { assert!(value < 128); assert_eq!(c.as_byte(), value); }
        Err(_) => assert!(value >= 128),
    }
}

#[kani::proof]
fn checked_u32_full_domain() {
    let value: u32 = kani::any();
    match value.to_ascii_char() {
        Ok(c) => { assert!(value < 128); assert_eq!(u32::from(c.as_byte()), value); }
        Err(_) => assert!(value >= 128),
    }
}

#[kani::proof]
fn checked_i8_full_domain() {
    let value: i8 = kani::any();
    match value.to_ascii_char() {
        Ok(c) => { assert!(value >= 0); assert_eq!(c.as_byte(), value as u8); }
        Err(_) => assert!(value < 0),
    }
}

#[kani::proof]
fn checked_u16_full_domain() {
    let value: u16 = kani::any();
    match value.to_ascii_char() {
        Ok(c) => { assert!(value < 128); assert_eq!(u16::from(c.as_byte()), value); }
        Err(_) => assert!(value >= 128),
    }
}

#[kani::proof]
fn checked_char_full_domain() {
    let codepoint: u32 = kani::any();
    // Exactly the Rust char domain: every Unicode scalar value, no surrogate.
    kani::assume(codepoint <= 0x10ffff && (codepoint < 0xd800 || codepoint > 0xdfff));
    let value: char = char::from_u32(codepoint).unwrap();
    assert_eq!(value as u32, codepoint);
    match value.to_ascii_char() {
        Ok(c) => { assert!((value as u32) < 128); assert_eq!(u32::from(c.as_byte()), value as u32); }
        Err(_) => assert!((value as u32) >= 128),
    }
}
