//! Prefix writes to caller-owned `MaybeUninit<u8>` storage.
//!
//! These helpers preserve the destination slice and its length. They make no
//! allocation and initialize only the requested prefix.

use core::mem::MaybeUninit;
use creusot_std::prelude::*;

/// Typed effect boundary for the native memcpy used by BytesMut::extend_from_slice
/// and unique Vec prefix reclaim.
///
/// Temporary generic trust: the pinned pointer contracts do not describe
/// copy_nonoverlapping's effect on a borrowed MaybeUninit slice. This leaf
/// assumes only that effect, never allocation identity, liveness or refcounts.
/// TODO: remove trust when the pointer-copy effect can be proved; the
/// body-proved copy_to_uninit_prefix below supplies the same functional contract.
/// This must remain an ordinary program operation, not an erased ghost write.
///
/// # Safety
/// src.len() must not exceed dst.len(). The typed borrows provide valid,
/// disjoint source and destination ranges, including empty ranges.
#[trusted]
#[requires(src@.len() <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < src@.len() ==> (^dst)@[i]@ == Some(src@[i]))]
#[ensures(forall<i> src@.len() <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
#[inline]
pub(crate) unsafe fn copy_to_uninit_prefix_raw(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr().cast(), src.len()) };
}

/// Typed effect boundary for the native memset used by `BytesMut::resize`.
///
/// This generic boundary describes only the initialized prefix written into a
/// caller-owned uninitialized slice. It does not grant allocation identity,
/// liveness, Vec ownership, refcount, or Bytes-specific authority.
/// The pinned standard-library pointer contract does not model this memory
/// effect yet. Replace this boundary when `ptr::write_bytes` has a verified
/// initialized-prefix contract that preserves the rest of the borrowed slice.
///
/// # Safety
/// `count` must not exceed `dst.len()`. The mutable slice provides a valid,
/// properly aligned destination, including when `count` is zero.
#[trusted]
#[requires(count@ <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < count@ ==> (^dst)@[i]@ == Some(value))]
#[ensures(forall<i> count@ <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
#[inline]
pub(crate) unsafe fn fill_uninit_prefix_raw(
    dst: &mut [MaybeUninit<u8>], count: usize, value: u8,
) {
    unsafe { core::ptr::write_bytes(dst.as_mut_ptr().cast::<u8>(), value, count) };
}

/// Fill the first `count` destination slots, leaving the suffix unchanged.
#[requires(count@ <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < count@ ==> (^dst)@[i]@ == Some(value))]
#[ensures(forall<i> count@ <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
#[inline]
pub(crate) fn fill_uninit_prefix(dst: &mut [MaybeUninit<u8>], count: usize, value: u8) {
    #[cfg(creusot)]
    let original = snapshot!(dst@);
    let mut index = 0;
    #[invariant(index@ <= count@)]
    #[invariant(dst@.len() == original.len())]
    #[invariant(forall<i> 0 <= i && i < index@ ==> dst@[i]@ == Some(value))]
    #[invariant(forall<i> count@ <= i && i < original.len() ==> dst@[i]@ == original[i]@)]
    #[variant(count@ - index@)]
    while index < count {
        dst[index] = MaybeUninit::new(value);
        index += 1;
    }
}

/// Copy `src` into the destination prefix, leaving the remaining suffix unchanged.
#[requires(src@.len() <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < src@.len() ==> (^dst)@[i]@ == Some(src@[i]))]
#[ensures(forall<i> src@.len() <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
#[inline]
pub(crate) fn copy_to_uninit_prefix(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    #[cfg(creusot)]
    let original = snapshot!(dst@);
    let mut index = 0;
    #[invariant(index@ <= src@.len())]
    #[invariant(dst@.len() == original.len())]
    #[invariant(forall<i> 0 <= i && i < index@ ==> dst@[i]@ == Some(src@[i]))]
    #[invariant(forall<i> src@.len() <= i && i < original.len() ==> dst@[i]@ == original[i]@)]
    #[variant(src@.len() - index@)]
    while index < src.len() {
        dst[index] = MaybeUninit::new(src[index]);
        index += 1;
    }
}
