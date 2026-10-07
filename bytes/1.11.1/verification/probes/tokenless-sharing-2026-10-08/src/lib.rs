#![recursion_limit="512"]
#![allow(unexpected_cfgs, dead_code, unused_imports, unused_variables)]
extern crate alloc;
#[cfg(creusot)] use creusot_std::{prelude::*, ghost::GhostShared};
#[cfg(creusot)] #[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[cfg(creusot)] #[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[cfg(creusot)] #[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[cfg(creusot)] use raw_vec::{Recovery, PhysicalRegion};

// A diagnostic helper, not a replacement Bytes API. It uses stock Copy only.
#[cfg(creusot)]
#[check(ghost)]
#[ensures(result == *shared)]
fn clone_shared_receiver(shared: &GhostShared<(Recovery, PhysicalRegion)>)
    -> GhostShared<(Recovery, PhysicalRegion)> {
    *shared
}

#[cfg(creusot)]
#[ensures(result.0@ == input@ && result.1@ == input@)]
pub fn actual_two_reads(input: alloc::vec::Vec<u8>) -> (alloc::vec::Vec<u8>, alloc::vec::Vec<u8>) {
    let len = input.len();
    let (raw, _, caps) = raw_vec::detach_vec(input);
    let (base, _) = raw.bound_ptr_at_zero();
    // Intentionally irrecoverable retained storage. No ordinary Vec owner remains.
    let shared = GhostShared::new(caps);
    let alias = ghost!(clone_shared_receiver(&*shared));
    let first = unsafe { raw_vec::borrow_bound(&base, len, ghost!(&shared.to_ref().1)) }.to_vec();
    let second = unsafe { raw_vec::borrow_bound(&base, len, ghost!(&alias.to_ref().1)) }.to_vec();
    (first, second)
}

#[cfg(all(creusot, feature="recover"))]
pub fn attempted_recovery(input: alloc::vec::Vec<u8>) {
    let (raw, _, caps) = raw_vec::detach_vec(input);
    let shared = GhostShared::new(caps);
    // Stock API deliberately cannot move affine authority out of this shared borrow.
    let recovered = ghost!(*shared.to_ref());
    unsafe { raw_vec::deallocate_vec(raw, recovered); }
}
