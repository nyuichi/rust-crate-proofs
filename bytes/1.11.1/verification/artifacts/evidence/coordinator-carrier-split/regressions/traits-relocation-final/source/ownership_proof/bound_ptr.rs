//! B1 constructor helper for a proof-bound byte pointer.
//!
//! This module does not create an access permission. It composes the existing
//! Vec detachment boundary with the sealed `RawAllocation` conversion in
//! `raw_vec`, so callers receive a pointer descriptor tied to that exact B1
//! allocation together with its recovery and physical-region capabilities.

use alloc::vec::Vec;

use creusot_std::prelude::*;

use super::raw_vec::{detach_vec, BoundPtr, PhysicalRegion, Recovery};

/// Detach a Vec and return a pointer descriptor bound to its allocation.
///
/// The returned `BoundPtr` is positioned at offset zero. Its one-word native
/// representation is the allocation pointer; its Creusot-only binding records
/// the B1 namespace and allocation capacity. The returned capabilities carry
/// the unique recovery marker and the full `[0, capacity)` slot region.
///
/// This helper establishes metadata correspondence only. It does not grant
/// permission to read or write through the pointer.
#[ensures(result.0@ == Some((
    result.3.inner_logic().0.namespace(),
    result.2@,
    0int,
)))]
#[ensures(result.0.invariant())]
#[ensures(result.1@ == input@.len())]
#[ensures(result.2@ == result.3.inner_logic().0.capacity())]
#[ensures(result.2@ == result.3.inner_logic().1.capacity())]
#[ensures(result.2@ >= result.1@)]
#[ensures(result.3.inner_logic().0.invariant())]
#[ensures(result.3.inner_logic().1.invariant())]
#[ensures(result.3.inner_logic().1.lo() == 0)]
#[ensures(result.3.inner_logic().1.hi() == result.2@)]
#[ensures(result.3.inner_logic().0.namespace() == result.3.inner_logic().1.namespace())]
#[ensures(result.3.inner_logic().0.namespace() == result.3.inner_logic().1.resource_id())]
#[ensures(forall<index: Int> 0 <= index && index < result.1@ ==>
    result.3.inner_logic().1.slot(index) == Some(Some(input@[index])))]
#[ensures(forall<index: Int> result.1@ <= index && index < result.2@ ==>
    result.3.inner_logic().1.slot(index) == Some(None))]
pub(crate) fn detach_bound_vec(
    input: Vec<u8>,
) -> (BoundPtr, usize, usize, Ghost<(Recovery, PhysicalRegion)>) {
    let (raw, len, capabilities) = detach_vec(input);
    let (pointer, capacity) = raw.into_bound_ptr_at_zero();
    (pointer, len, capacity, capabilities)
}
