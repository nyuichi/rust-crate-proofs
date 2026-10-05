//! Arithmetic helpers shared by the runtime `Chain` adapter.

use creusot_std::prelude::*;

/// Adds two buffer lengths, saturating at the largest representable length.
#[inline]
#[ensures(result@ == if a@ + b@ <= usize::MAX@ { a@ + b@ } else { usize::MAX@ })]
pub(crate) fn saturating_sum(a: usize, b: usize) -> usize {
    a.saturating_add(b)
}

/// Splits a requested count between the first and second buffers.
///
/// `first` is the portion consumed from the first buffer, and `second` is the
/// rest of the request. This is arithmetic only; callers retain control over
/// the order and conditions of buffer operations.
#[inline]
#[ensures(result.0@ == if a_remaining@ <= count@ { a_remaining@ } else { count@ })]
#[ensures(result.1@ == if a_remaining@ < count@ { count@ - a_remaining@ } else { 0 })]
#[ensures(result.0@ + result.1@ == count@)]
pub(crate) fn split_count(a_remaining: usize, count: usize) -> (usize, usize) {
    let first = if a_remaining <= count {
        a_remaining
    } else {
        count
    };
    let second = count - first;
    (first, second)
}
