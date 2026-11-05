use ascii::AsciiChar;
fn original_input(code: u8) -> AsciiChar {
    // Exactly the complete domain of valid AsciiChar discriminants, not a
    // restriction on target API inputs: all 128 enum variants are represented.
    kani::assume(code < 128);
    let value = AsciiChar::new(char::from(code));
    assert_eq!(value.as_byte(), code);
    value
}
#[kani::proof]
fn from_ascii_char_to_u8_all_variants() {
    let code: u8 = kani::any();
    let value = original_input(code);
    let result = <u8 as From<AsciiChar>>::from(value);
    assert_eq!(result, code);
}
#[kani::proof]
fn from_ascii_char_to_char_all_variants() {
    let code: u8 = kani::any();
    let value = original_input(code);
    let result = <char as From<AsciiChar>>::from(value);
    assert_eq!(result as u32, u32::from(code));
}
