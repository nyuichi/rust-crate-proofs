#[requires(crate::verification_model::valid_cursor(bytes@))]
#[ensures((^bytes)@.input == bytes@.input
    && (^bytes)@.end == bytes@.end)]
#[ensures((^bytes)@.cursor == crate::verification_newline::parse_newline_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).cursor)]
#[ensures((^bytes)@.mark == crate::verification_newline::parse_newline_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).mark)]
#[ensures(match result {
    Ok(Status::Complete(())) => crate::verification_newline::newline_result_is_complete(
        crate::verification_newline::parse_newline_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Ok(Status::Partial) => crate::verification_newline::newline_result_is_partial(
        crate::verification_newline::parse_newline_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(Error::NewLine) => crate::verification_newline::newline_result_is_error(
        crate::verification_newline::parse_newline_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(_) => false,
})]
#[inline]
fn parse_newline(bytes: &mut Bytes<'_>) -> Result<()> {
    newline!(bytes);
    Ok(Status::Complete(()))
}
