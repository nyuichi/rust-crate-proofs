#[cfg(creusot)]
use creusot_std::prelude::*;

pub(crate) const MAX_REF_COUNT: usize = usize::MAX / 2;

/// Pure CAS update: refusal never performs an increment.
#[cfg_attr(creusot, ensures(match result {
    Some(_) => old <= MAX_REF_COUNT,
    None => old > MAX_REF_COUNT,
}))]
#[cfg_attr(creusot, ensures(match result {
    Some(next) => next@ == old@ + 1 && next@ <= MAX_REF_COUNT@ + 1,
    None => old > MAX_REF_COUNT,
}))]
#[inline]
pub(crate) fn next_ref_count(old: usize) -> Option<usize> {
    if old > MAX_REF_COUNT {
        None
    } else {
        Some(old + 1)
    }
}
