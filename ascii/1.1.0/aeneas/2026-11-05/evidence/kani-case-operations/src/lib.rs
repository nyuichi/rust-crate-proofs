use ascii::AsciiChar;

// The finite enum input domain is complete: each code 0..127 is constructed
// with the original safe constructor, and its original public observation is
// checked before invoking the target API. No invalid enum is constructed.
fn input_ascii(code: u8) -> AsciiChar {
    kani::assume(code < 128);
    let value = AsciiChar::new(char::from(code));
    assert_eq!(value.as_byte(), code);
    value
}
fn upper_code(code: u8) -> u8 {
    if code >= b'a' && code <= b'z' { code - 32 } else { code }
}
fn lower_code(code: u8) -> u8 {
    if code >= b'A' && code <= b'Z' { code + 32 } else { code }
}
#[kani::proof]
fn uppercase_full_ascii_domain() {
    let code: u8 = kani::any();
    let value = input_ascii(code);
    assert_eq!(value.to_ascii_uppercase().as_byte(), upper_code(code));
}
#[kani::proof]
fn lowercase_full_ascii_domain() {
    let code: u8 = kani::any();
    let value = input_ascii(code);
    assert_eq!(value.to_ascii_lowercase().as_byte(), lower_code(code));
}
#[kani::proof]
fn make_uppercase_full_ascii_writeback() {
    let code: u8 = kani::any();
    let mut value = input_ascii(code);
    value.make_ascii_uppercase();
    assert_eq!(value.as_byte(), upper_code(code));
}
#[kani::proof]
fn make_lowercase_full_ascii_writeback() {
    let code: u8 = kani::any();
    let mut value = input_ascii(code);
    value.make_ascii_lowercase();
    assert_eq!(value.as_byte(), lower_code(code));
}
#[kani::proof]
fn as_char_full_ascii_domain() {
    let code: u8 = kani::any();
    let value = input_ascii(code);
    assert_eq!(value.as_char() as u32, u32::from(code));
}
#[kani::proof]
fn case_insensitive_equality_full_pair_domain() {
    let left: u8 = kani::any();
    let right: u8 = kani::any();
    let a = input_ascii(left);
    let b = input_ascii(right);
    assert_eq!(a.eq_ignore_ascii_case(&b), lower_code(left) == lower_code(right));
}
#[kani::proof]
fn printable_picture_full_ascii_domain() {
    let code: u8 = kani::any();
    let value = input_ascii(code);
    let expected: u32 = if code < 32 { 0x2400 + u32::from(code) }
        else if code == 127 { 0x2421 } else { u32::from(code) };
    assert_eq!(value.as_printable_char() as u32, expected);
}
