//! Safe prefix initialization operations for mutable `MaybeUninit<u8>` slices.
//!
//! These helpers assume the caller has already checked the requested size.
//! They initialize each consumed slot, return that prefix, and advance the
//! slice cursor to the untouched suffix.

use core::mem::MaybeUninit;
use creusot_std::prelude::*;

/// Copies an initialized source slice into a valid prefix of an uninitialized
/// slice cursor and returns the initialized prefix.
#[requires(source@.len() <= input@.len())]
#[ensures(result@.len() == source@.len())]
#[ensures(forall<i> 0 <= i && i < source@.len() ==> result@[i]@ == Some(source@[i]))]
#[ensures((^input)@ == input@[source@.len()..])]
#[inline]
pub(crate) fn initialize_prefix<'a>(
    input: &mut &'a mut [MaybeUninit<u8>],
    source: &[u8],
) -> &'a mut [MaybeUninit<u8>] {
    let (prefix, suffix) = core::mem::take(input).split_at_mut(source.len());
    proof_assert!(prefix@.len() == source@.len());
    let mut index = 0;
    #[invariant(index@ <= prefix@.len())]
    #[invariant(prefix@.len() == source@.len())]
    #[invariant(forall<i> 0 <= i && i < index@ ==> prefix@[i]@ == Some(source@[i]))]
    #[variant(prefix@.len() - index@)]
    while index < prefix.len() {
        prefix[index] = MaybeUninit::new(source[index]);
        index += 1;
    }
    *input = suffix;
    prefix
}

/// Fills a valid prefix of an uninitialized slice cursor with `value` and
/// returns the initialized prefix.
#[requires(count@ <= input@.len())]
#[ensures(result@.len() == count@)]
#[ensures(forall<i> 0 <= i && i < count@ ==> result@[i]@ == Some(value))]
#[ensures((^input)@ == input@[count@..])]
#[inline]
pub(crate) fn fill_prefix<'a>(
    input: &mut &'a mut [MaybeUninit<u8>],
    count: usize,
    value: u8,
) -> &'a mut [MaybeUninit<u8>] {
    let (prefix, suffix) = core::mem::take(input).split_at_mut(count);
    proof_assert!(prefix@.len() == count@);
    let mut index = 0;
    #[invariant(index@ <= prefix@.len())]
    #[invariant(prefix@.len() == count@)]
    #[invariant(forall<i> 0 <= i && i < index@ ==> prefix@[i]@ == Some(value))]
    #[variant(prefix@.len() - index@)]
    while index < prefix.len() {
        prefix[index] = MaybeUninit::new(value);
        index += 1;
    }
    *input = suffix;
    prefix
}
