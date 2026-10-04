//! Integer helpers used by the runtime Buf implementations.
use creusot_std::prelude::*;

#[inline(always)]
#[ensures(result@ == if b@ <= a@ { a@ - b@ } else { 0 })]
pub(crate) fn saturating_sub_usize_u64(a: usize, b: u64) -> usize {
    match usize::try_from(b) {
        Ok(b) => a.saturating_sub(b),
        Err(_) => 0,
    }
}

#[inline(always)]
#[ensures(result@ == if a@ <= usize::MAX@ { if a@ < b@ { a@ } else { b@ } } else { b@ })]
pub(crate) fn min_u64_usize(a: u64, b: usize) -> usize {
    match usize::try_from(a) {
        Ok(a) => usize::min(a, b),
        Err(_) => b,
    }
}

