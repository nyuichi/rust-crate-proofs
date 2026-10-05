use creusot_std::prelude::*;

/// Convert the raw index-table capacity into the number of primary entries it
/// can hold at the configured load factor.
#[inline]
#[ensures(result@ == capacity@ - capacity@ / 4)]
pub(super) fn usable_capacity(capacity: usize) -> usize {
    capacity - capacity / 4
}

/// Compute the raw table capacity before rounding to a power of two.
#[inline]
#[ensures(match result {
    Some(raw) => raw@ == requested@ + requested@ / 3,
    None => requested@ + requested@ / 3 > usize::MAX@,
})]
pub(super) fn checked_raw_capacity(requested: usize) -> Option<usize> {
    requested.checked_add(requested / 3)
}
