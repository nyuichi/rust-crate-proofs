//! Shared bounded-buffer operations used by the runtime adapters.
#[cfg(creusot)]
use creusot_std::prelude::*;

#[inline]
#[cfg_attr(creusot, ensures(result@ == if available@ < limit@ { available@ } else { limit@ }))]
pub(crate) fn bounded_len(available: usize, limit: usize) -> usize {
    core::cmp::min(available, limit)
}

#[inline]
#[cfg_attr(creusot, ensures(result@ == bytes@.subsequence(0, if bytes@.len() < limit@ { bytes@.len() } else { limit@ })))]
pub(crate) fn bounded_chunk(bytes: &[u8], limit: usize) -> &[u8] {
    &bytes[..bounded_len(bytes.len(), limit)]
}

#[inline]
#[cfg_attr(creusot, requires(count@ <= (*limit)@))]
#[cfg_attr(creusot, ensures((^limit)@ == (*limit)@ - count@))]
pub(crate) fn decrease_limit(limit: &mut usize, count: usize) {
    *limit -= count;
}
