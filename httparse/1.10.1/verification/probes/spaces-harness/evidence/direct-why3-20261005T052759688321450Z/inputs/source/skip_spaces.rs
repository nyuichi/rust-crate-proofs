#[allow(unused_imports)]
use creusot_std::prelude::{ghost, invariant, snapshot, variant, DeepModel, Int};

#[requires(crate::verification_model::valid_cursor(bytes@))]
#[ensures((^bytes)@.input == bytes@.input
    && (^bytes)@.end == bytes@.end)]
#[ensures((^bytes)@.cursor == crate::verification_spaces::space_prefix_end(
    bytes@.input, bytes@.cursor, bytes@.end))]
#[ensures((^bytes)@.mark == crate::verification_spaces::skip_spaces_model(
    bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end).mark)]
#[ensures(match result {
    Ok(Status::Complete(())) => crate::verification_spaces::spaces_result_is_complete(
        crate::verification_spaces::skip_spaces_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Ok(Status::Partial) => crate::verification_spaces::spaces_result_is_partial(
        crate::verification_spaces::skip_spaces_model(
            bytes@.input, bytes@.mark, bytes@.cursor, bytes@.end)),
    Err(_) => false,
})]
#[inline]
fn skip_spaces(bytes: &mut Bytes<'_>) -> Result<()> {
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
    #[invariant(forall<i: Int> *entry_cursor <= i && i < bytes@.cursor
        ==> bytes@.input[i].deep_model() == 32)]
    #[invariant(bytes@.cursor <= crate::verification_spaces::space_prefix_end(
        bytes@.input, *entry_cursor, *entry_end))]
    #[variant(*entry_end - bytes@.cursor)]
    loop {
        #[cfg(creusot)]
        ghost! {
            crate::verification_spaces::space_prefix_at_cursor(
                snapshot!(bytes@.input),
                snapshot!(*entry_cursor),
                snapshot!(bytes@.cursor),
                snapshot!(*entry_end),
            );
        };

        let b = bytes.peek();
        match b {
            Some(b' ') => {
                // SAFETY: peeked and found ` `, so it's safe to bump 1 pos
                unsafe { bytes.bump() };
            }
            Some(..) => {
                bytes.slice();
                return Ok(Status::Complete(()));
            }
            None => return Ok(Status::Partial),
        }
    }
}
