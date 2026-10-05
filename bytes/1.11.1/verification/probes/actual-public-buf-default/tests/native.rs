use bytes_actual_public_buf_default::{generic_try_get_u8, slice_try_get_u8};

#[test]
fn extracted_public_default_reads_and_advances_one_byte() {
    let mut input: &[u8] = &[0xA7, 0x31];
    assert_eq!(generic_try_get_u8(&mut input).unwrap(), 0xA7);
    assert_eq!(input, &[0x31]);
}

#[test]
fn extracted_public_default_reports_empty_without_advancing() {
    let mut input: &[u8] = &[];
    let error = generic_try_get_u8(&mut input).unwrap_err();
    assert_eq!(error.requested, 1);
    assert_eq!(error.available, 0);
    assert!(input.is_empty());
}

#[test]
fn concrete_wrapper_dispatches_through_the_same_trait_default() {
    let mut input: &[u8] = &[0x5C, 0x44];
    assert_eq!(slice_try_get_u8(&mut input).unwrap(), 0x5C);
    assert_eq!(input, &[0x44]);
}
