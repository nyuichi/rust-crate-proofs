//! Byte-slice comparison shared by the runtime buffer representations.
#[cfg(creusot)]
use creusot_std::prelude::*;

#[inline]
#[cfg_attr(creusot, ensures(result == (left.deep_model() == right.deep_model())))]
pub(crate) fn equal(left: &[u8], right: &[u8]) -> bool {
    left == right
}

#[inline]
#[cfg_attr(creusot, ensures(result == left.deep_model().cmp_log(right.deep_model())))]
pub(crate) fn compare(left: &[u8], right: &[u8]) -> core::cmp::Ordering {
    left.cmp(right)
}
