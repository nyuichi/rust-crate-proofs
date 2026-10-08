//! Local wrappers for the source-gated probe.
//!
//! The path dependency already imports the repository's one generic extern
//! spec for u8 `wrapping_add`; this module deliberately contains no duplicate
//! extern specification.

#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(creusot, ensures(result == ptr.addr_logic()))]
#[cfg_attr(creusot, check(ghost))]
pub(crate) fn pointer_addr<T>(ptr: *const T) -> usize {
    ptr.addr()
}

#[cfg_attr(creusot, ensures(
    result.addr_logic()@ == (ptr.addr_logic()@ + offset@) % (usize::MAX@ + 1)
))]
pub(crate) fn wrapping_offset(ptr: *mut u8, offset: usize) -> *mut u8 {
    ptr.wrapping_add(offset)
}
