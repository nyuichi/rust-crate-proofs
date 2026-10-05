//! Prefix writes to caller-owned `MaybeUninit<u8>` storage.
//!
//! These helpers preserve the destination slice and its length. They make no
//! allocation and initialize only the requested prefix.

use core::mem::MaybeUninit;
use creusot_std::prelude::*;

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
