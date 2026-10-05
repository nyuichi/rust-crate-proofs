#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]

extern crate alloc;

#[cfg(creusot)]
use creusot_std::prelude::*;

#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/vec_capacity.rs"]
mod vec_capacity;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

#[cfg(creusot)]
pub struct CapacityFields {
    ptr: raw_vec::BoundPtr,
    len: usize,
    cap: usize,
    unique_at_zero: Ghost<Option<(raw_vec::Recovery, raw_vec::PhysicalRegion)>>,
}

/// Compose the exact detached B1 values into the ownership-related fields
/// used by the offset-zero BytesMut constructor.
#[cfg(creusot)]
#[ensures(result.len@ == 0)]
#[ensures(result.cap@ >= requested@)]
#[ensures(match result.unique_at_zero.inner_logic() {
    Some((recovery, region)) =>
        result.ptr@ == Some((recovery.namespace(), result.cap@, 0int)) &&
        result.ptr.invariant() && recovery.invariant() && region.invariant() &&
        recovery.capacity() == result.cap@ && region.capacity() == result.cap@ &&
        recovery.namespace() == region.namespace() &&
        recovery.namespace() == region.resource_id() &&
        region.lo() == 0 && region.hi() == result.cap@ &&
        forall<index: Int> 0 <= index && index < result.cap@ ==>
            region.slot(index) == Some(None),
    None => false
})]
fn construct_capacity_fields(requested: usize) -> CapacityFields {
    let (ptr, len, cap, capabilities) = vec_capacity::with_capacity_bound(requested);
    let unique_at_zero = ghost! { Some(capabilities.into_inner()) };
    CapacityFields { ptr, len, cap, unique_at_zero }
}

/// The field construction retains the requested capacity and the complete B1
/// allocation facts for the same physical Vec allocation.
#[cfg(creusot)]
#[ensures(result.len@ == 0)]
#[ensures(result.cap@ >= requested@)]
pub fn requested_capacity_positive(requested: usize) -> CapacityFields {
    construct_capacity_fields(requested)
}

/// A detached buffer cannot claim the guarantee for a strictly larger request.
#[cfg(all(creusot, feature = "wrong_requested_capacity"))]
#[requires(requested@ < larger_request@)]
#[ensures(result.cap@ >= larger_request@)]
pub fn wrong_requested_capacity(requested: usize, larger_request: usize) -> CapacityFields {
    construct_capacity_fields(requested)
}

/// Native observation used by tests. The helper consumes the B1 authority to
/// free the exact allocation after reading its actual length and capacity.
#[cfg(not(creusot))]
pub fn native_allocate_and_release(requested: usize) -> (usize, usize) {
    vec_capacity::native_allocate_and_release(requested)
}
