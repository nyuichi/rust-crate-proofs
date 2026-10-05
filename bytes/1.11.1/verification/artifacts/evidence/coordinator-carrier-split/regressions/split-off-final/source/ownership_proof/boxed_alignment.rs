//! Generic physical fact missing from Creusot 0.13's typed Box permission.
//!
//! TCB: converting an allocated Box preserves its native pointee alignment.
//! This adds no reference-count, tagging, handle, or recovery-protocol fact.
use alloc::boxed::Box;
use creusot_std::{ghost::perm::Perm, prelude::*};

#[trusted]
#[check(terminates)]
#[ensures(*result.1.ward() == result.0)]
#[ensures(*result.1.val() == *value)]
#[ensures(result.0.is_aligned_logic())]
pub(crate) fn into_raw_aligned<T>(value: Box<T>) -> (*mut T, Ghost<Box<Perm<*const T>>>) {
    Perm::from_box(value)
}

// A body-proved bit fact, separate from the physical allocation boundary.
#[bitwise_proof]
#[requires(alignment >= 2usize)]
#[requires(alignment & (alignment - 1usize) == 0usize)]
#[requires(address & (alignment - 1usize) == 0usize)]
#[ensures(address & 1usize == 0usize)]
pub(crate) fn aligned_address_has_clear_low_bit(address: usize, alignment: usize) {}
