//! Small arithmetic and slice helpers for `std::io::Cursor<T>` as a `Buf`.
//!
//! These operate on the cursor's real slice length and position. They do not
//! introduce a separate model for the bytes or their length.
use creusot_std::prelude::*;

use crate::arithmetic::{min_u64_usize, saturating_sub_usize_u64};

/// Number of bytes left from a cursor position, saturating at zero.
#[inline(always)]
#[ensures(result@ == if pos@ <= len@ { len@ - pos@ } else { 0 })]
pub(crate) fn cursor_remaining(len: usize, pos: u64) -> usize {
    saturating_sub_usize_u64(len, pos)
}

/// Slice index at which a cursor's remaining chunk starts.
#[inline(always)]
#[ensures(result@ == if pos@ < len@ { pos@ } else { len@ })]
pub(crate) fn cursor_chunk_start(pos: u64, len: usize) -> usize {
    min_u64_usize(pos, len)
}

/// Exact suffix returned for a cursor's remaining chunk.
#[inline(always)]
#[ensures(result@ == slice@.subsequence(
    if pos@ < slice@.len() { pos@ } else { slice@.len() },
    slice@.len()
))]
pub(crate) fn cursor_chunk<'a>(slice: &'a [u8], pos: u64) -> &'a [u8] {
    let start = cursor_chunk_start(pos, slice.len());
    &slice[start..]
}

/// New position after the caller has checked the standard Cursor advance bound.
///
/// The precondition captures the native `Cursor` rule: even past EOF, an
/// advance of zero is allowed; every positive advance must fit in the
/// saturating remaining length. This bound also proves `pos + cnt as u64` does
/// not overflow.
#[inline(always)]
#[requires(cnt@ <= (if pos@ <= len@ { len@ - pos@ } else { 0 }))]
#[ensures(result@ == pos@ + cnt@)]
#[ensures(pos@ + cnt@ <= u64::MAX@)]
pub(crate) fn cursor_position_after_advance(len: usize, pos: u64, cnt: usize) -> u64 {
    pos + cnt as u64
}
