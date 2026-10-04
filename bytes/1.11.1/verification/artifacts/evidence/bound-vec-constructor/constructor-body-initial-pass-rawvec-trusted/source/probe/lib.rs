//! Probe the shared bound Vec constructor helper and explicit B3 cleanup.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;

use alloc::vec::Vec;
use creusot_std::prelude::*;

#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;

mod ownership_proof {
    pub(crate) use crate::{bound_ptr, owned_region, raw_vec};
}

#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

#[cfg(feature = "actual_from_vec")]
mod actual_from_vec;

use bound_ptr::detach_bound_vec;
use raw_vec::deallocate_bound_vec;
#[cfg(feature = "negative_unbound_cleanup")]
use raw_vec::BoundPtr;

/// Use the shared helper, then free the exact B1 allocation with all of its
/// recovery and region capabilities. This is explicit cleanup, not a Drop
/// translation claim.
#[ensures(result@ == input@.len())]
pub fn detach_then_explicitly_deallocate(input: Vec<u8>) -> usize {
    let (bound, len, capacity, capabilities) = detach_bound_vec(input);
    proof_assert!(bound@ != None);
    proof_assert!(bound.invariant());
    proof_assert!(capacity@ == capabilities.inner_logic().0.capacity());
    proof_assert!(capacity@ == capabilities.inner_logic().1.capacity());
    proof_assert!(capabilities.inner_logic().1.lo() == 0);
    proof_assert!(capabilities.inner_logic().1.hi() == capacity@);

    // SAFETY: the shared B1 helper returned the matching full recovery and
    // physical-region capabilities for this bound pointer and capacity.
    unsafe { deallocate_bound_vec(bound, capacity, capabilities) };
    len
}

/// Negative control: the pointer bits survive, but the sealed B1 binding is
/// deliberately replaced with `None`. They must not authorize deallocation.
#[cfg(feature = "negative_unbound_cleanup")]
pub unsafe fn negative_unbound_cleanup(input: Vec<u8>) {
    let (bound, _len, capacity, capabilities) = detach_bound_vec(input);
    let unbound = BoundPtr::unbound(bound.as_non_null());

    // SAFETY: intentionally violates B3 by supplying an unbound pointer.
    unsafe { deallocate_bound_vec(unbound, capacity, capabilities) };
}

/// Exercise the exact extracted runtime `BytesMut::from_vec` body and its
/// actual explicit unique-at-zero release method. This is not a Drop proof.
#[cfg(all(feature = "actual_from_vec", creusot))]
#[ensures(result@ == input@.len())]
pub fn actual_from_vec_then_release(input: Vec<u8>) -> usize {
    actual_from_vec::constructor_then_release(input)
}
