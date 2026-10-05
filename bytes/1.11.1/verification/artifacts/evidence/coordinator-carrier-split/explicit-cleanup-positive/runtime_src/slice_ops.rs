//! Safe slice operations shared with the runtime `Buf for &[u8]` implementation.
//!
//! These helpers have valid-bounds preconditions. Callers retain responsibility
//! for their public bounds checks and panic behavior.

use creusot_std::prelude::*;

/// Advances a byte slice by `count` bytes.
///
/// The caller must establish that `count` is within the slice. The resulting
/// slice is exactly the original suffix after that prefix.
#[requires(count@ <= input@.len())]
#[ensures((^input)@ == input@[count@..])]
#[inline]
pub(crate) fn advance_slice(input: &mut &[u8], count: usize) {
    *input = &input[count..];
}

/// Copies the matching prefix into `dst` and advances the source by that size.
///
/// The caller must establish that the destination fits in the source. Every
/// copied byte and the remaining source suffix are specified exactly.
#[requires(dst@.len() <= input@.len())]
#[ensures((^dst)@ == input@[0..dst@.len()])]
#[ensures((^input)@ == input@[dst@.len()..])]
#[inline]
pub(crate) fn copy_to_slice(input: &mut &[u8], dst: &mut [u8]) {
    let count = dst.len();
    dst.copy_from_slice(&input[..count]);
    *input = &input[count..];
}
