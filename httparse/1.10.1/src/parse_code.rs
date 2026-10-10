// The exact `parse_code` implementation included at crate root.

#[requires(crate::verification_model::valid_cursor(bytes@))]
#[ensures((^bytes)@.input == bytes@.input
    && (^bytes)@.mark == bytes@.mark
    && (^bytes)@.end == bytes@.end)]
#[ensures((^bytes)@.cursor == crate::verification_code::parse_code_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).cursor)]
#[ensures(match result {
    Ok(Status::Complete(value)) =>
        crate::verification_code::code_result_is_complete(
            crate::verification_code::parse_code_model(
                bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end),
            value@),
    Ok(Status::Partial) =>
        crate::verification_code::code_result_is_partial(
            crate::verification_code::parse_code_model(
                bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(Error::Status) =>
        crate::verification_code::code_result_is_error(
            crate::verification_code::parse_code_model(
                bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(_) => false,
})]
#[inline]
fn parse_code(bytes: &mut Bytes<'_>) -> Result<u16> {
    let hundreds = expect!(bytes.next() == b'0'..=b'9' => Err(Error::Status));
    let tens = expect!(bytes.next() == b'0'..=b'9' => Err(Error::Status));
    let ones = expect!(bytes.next() == b'0'..=b'9' => Err(Error::Status));

    Ok(Status::Complete((hundreds - b'0') as u16 * 100 +
        (tens - b'0') as u16 * 10 +
        (ones - b'0') as u16))
}
