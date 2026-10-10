#[allow(unused_imports)]
use creusot_std::prelude::{ghost, invariant, snapshot, variant, DeepModel, Int};

#[requires(crate::verification_model::valid_cursor(bytes@))]
#[ensures((^bytes)@.input == bytes@.input
    && (^bytes)@.end == bytes@.end)]
#[ensures((^bytes)@.mark == crate::verification_empty_lines::skip_empty_lines_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).mark)]
#[ensures((^bytes)@.cursor == crate::verification_empty_lines::skip_empty_lines_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).cursor)]
#[ensures(match result {
    Ok(Status::Complete(())) => crate::verification_empty_lines::empty_lines_is_complete(
        crate::verification_empty_lines::skip_empty_lines_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Ok(Status::Partial) => crate::verification_empty_lines::empty_lines_is_partial(
        crate::verification_empty_lines::skip_empty_lines_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(Error::NewLine) => crate::verification_empty_lines::empty_lines_is_newline_error(
        crate::verification_empty_lines::skip_empty_lines_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(_) => false,
})]
#[inline]
fn skip_empty_lines(bytes: &mut Bytes<'_>) -> Result<()> {
    #[cfg(creusot)]
    let entry_mark = snapshot!(bytes@.mark).into_ghost();
    #[cfg(creusot)]
    let entry_cursor = snapshot!(bytes@.cursor).into_ghost();
    #[cfg(creusot)]
    let entry_end = snapshot!(bytes@.end).into_ghost();
    #[cfg(creusot)]
    let entry_input = snapshot!(bytes@.input);

    #[invariant(crate::verification_model::valid_cursor(bytes@))]
    #[invariant(bytes@.input == *entry_input)]
    #[invariant(bytes@.mark == *entry_mark)]
    #[invariant(bytes@.end == *entry_end)]
    #[invariant(*entry_cursor <= bytes@.cursor)]
    #[invariant(crate::verification_empty_lines::complete_line_prefix(
        bytes@.input, *entry_cursor, bytes@.cursor))]
    #[invariant(bytes@.cursor
        <= crate::verification_empty_lines::complete_line_prefix_end(
            bytes@.input, *entry_cursor, *entry_end))]
    #[variant(*entry_end - bytes@.cursor)]
    loop {
        #[cfg(creusot)]
        ghost! {
            crate::verification_empty_lines::complete_line_prefix_at_cursor(
                snapshot!(bytes@.input),
                snapshot!(*entry_cursor),
                snapshot!(bytes@.cursor),
                snapshot!(*entry_end),
            );
        };

        let b = bytes.peek();
        match b {
            Some(b'\r') => {
                #[cfg(creusot)]
                let line_cursor = snapshot!(bytes@.cursor).into_ghost();
                // SAFETY: peeked and found `\r`, so it's safe to bump 1 pos
                unsafe { bytes.bump() };
                expect!(bytes.next() == b'\n' => Err(Error::NewLine));

                #[cfg(creusot)]
                ghost! {
                    crate::verification_empty_lines::complete_line_prefix_extend_crlf(
                        snapshot!(bytes@.input),
                        snapshot!(*entry_cursor),
                        snapshot!(*line_cursor),
                        snapshot!(*entry_end),
                    );
                    crate::verification_empty_lines::complete_line_prefix_at_cursor(
                        snapshot!(bytes@.input),
                        snapshot!(*entry_cursor),
                        snapshot!(bytes@.cursor),
                        snapshot!(*entry_end),
                    );
                };
            }
            Some(b'\n') => {
                #[cfg(creusot)]
                let line_cursor = snapshot!(bytes@.cursor).into_ghost();
                // SAFETY: peeked and found `\n`, so it's safe to bump 1 pos
                unsafe {
                    bytes.bump();
                }

                #[cfg(creusot)]
                ghost! {
                    crate::verification_empty_lines::complete_line_prefix_extend_lf(
                        snapshot!(bytes@.input),
                        snapshot!(*entry_cursor),
                        snapshot!(*line_cursor),
                        snapshot!(*entry_end),
                    );
                    crate::verification_empty_lines::complete_line_prefix_at_cursor(
                        snapshot!(bytes@.input),
                        snapshot!(*entry_cursor),
                        snapshot!(bytes@.cursor),
                        snapshot!(*entry_end),
                    );
                };
            }
            Some(..) => {
                bytes.slice();
                return Ok(Status::Complete(()));
            }
            None => return Ok(Status::Partial),
        }
    }
}
