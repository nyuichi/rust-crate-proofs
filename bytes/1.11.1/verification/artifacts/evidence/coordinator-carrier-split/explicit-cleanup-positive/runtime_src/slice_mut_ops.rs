//! Safe cursor operations for an initialized mutable byte slice.
//!
//! These helpers assume the caller has already checked the requested length.
//! They return the consumed prefix so callers and proofs can retain facts about
//! the bytes while the cursor advances to the disjoint suffix.

use creusot_std::prelude::*;

/// Advances the mutable-slice cursor and returns the unchanged consumed prefix.
#[requires(count@ <= input@.len())]
#[ensures(result@ == input@[0..count@])]
#[ensures((^input)@ == input@[count@..])]
#[ensures((^*input)@ == (^result)@.concat((^^input)@))]
#[inline]
pub(crate) fn advance_slice_mut<'a, T>(input: &mut &'a mut [T], count: usize) -> &'a mut [T] {
    let (prefix, suffix) = core::mem::take(input).split_at_mut(count);
    *input = suffix;
    prefix
}

/// Copies `source` into the consumed prefix, advances the cursor, and returns
/// the written prefix.
#[requires(source@.len() <= input@.len())]
#[ensures(result@ == source@)]
#[ensures((^input)@ == input@[source@.len()..])]
#[ensures(result@.concat((^input)@) == source@.concat(input@[source@.len()..]))]
#[ensures((^*input)@ == (^result)@.concat((^^input)@))]
#[inline]
pub(crate) fn copy_to_slice_mut<'a>(input: &mut &'a mut [u8], source: &[u8]) -> &'a mut [u8] {
    let (prefix, suffix) = core::mem::take(input).split_at_mut(source.len());
    prefix.copy_from_slice(source);
    *input = suffix;
    prefix
}

/// Fills the consumed prefix with `value`, advances the cursor, and returns
/// the initialized prefix.
#[requires(count@ <= input@.len())]
#[ensures(result@.len() == count@)]
#[ensures(forall<i> 0 <= i && i < count@ ==> result@[i] == value)]
#[ensures((^input)@ == input@[count@..])]
#[ensures((^*input)@ == (^result)@.concat((^^input)@))]
#[inline]
pub(crate) fn fill_slice_mut<'a>(input: &mut &'a mut [u8], count: usize, value: u8) -> &'a mut [u8] {
    let _original = snapshot!(input@);
    let bytes = core::mem::take(input);
    let _original_len = bytes.len();
    let mut index = 0;
    #[invariant(index@ <= count@)]
    #[invariant(count@ <= _original_len@)]
    #[invariant(bytes@.len() == _original_len@)]
    #[invariant(forall<i> 0 <= i && i < index@ ==> bytes@[i] == value)]
    #[invariant(forall<j> count@ <= j && j < _original_len@ ==> bytes@[j] == _original[j])]
    #[variant(count@ - index@)]
    while index < count {
        bytes[index] = value;
        index += 1;
    }
    let (prefix, suffix) = bytes.split_at_mut(count);
    *input = suffix;
    prefix
}
