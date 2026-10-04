//! Explicit physical retirement; no actual Shared, atomics, or Drop proof.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
use alloc::vec::Vec;
use creusot_std::prelude::*;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
use raw_vec::{
    PhysicalPool, PhysicalRegion, Recovery, borrow_mut, deallocate_bound_vec,
    deallocate_vec, detach_vec, resume_vec,
};

#[requires(split@ <= input@.len())]
pub fn retire_left_then_right(input: Vec<u8>, split: usize) {
    let (raw, _len, caps) = detach_vec(input);
    let full = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let boundary: Int = *Int::new(split as i128);
        let (left, right) = region.split_at(boundary);
        let pool = pool.retire(left);
        let pool = pool.retire(right);
        (recovery, pool.finish(0int))
    };
    unsafe { deallocate_vec(raw, full); }
}

#[requires(split@ <= input@.len())]
pub fn retire_right_then_left(input: Vec<u8>, split: usize) {
    let (raw, _len, caps) = detach_vec(input);
    let full = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let boundary: Int = *Int::new(split as i128);
        let (left, right) = region.split_at(boundary);
        let pool = pool.retire(right);
        let pool = pool.retire(left);
        (recovery, pool.finish(0int))
    };
    unsafe { deallocate_vec(raw, full); }
}

/// Split at the allocation capacity, returning the full left interval and an
/// empty right interval to the pool before explicit B3 deallocation.
pub fn retire_at_capacity(input: Vec<u8>) {
    let (raw, _len, caps) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let full = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let boundary: Int = *Int::new(capacity as i128);
        let (left, right) = region.split_at(boundary);
        let pool = pool.retire(right);
        let pool = pool.retire(left);
        (recovery, pool.finish(0int))
    };
    unsafe { deallocate_bound_vec(bound, capacity, full); }
}

/// B4 mutates two disjoint initialized fragments. Their updated affine
/// resources enter the pool, which reconstructs the full region for B2.
#[requires(input@.len() >= 4)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[1] == 0xA1u8)]
#[ensures(result@[2] == 0xB2u8)]
#[ensures(forall<index: Int> 0 <= index && index < input@.len() &&
    index != 1 && index != 2 ==> result@[index] == input@[index])]
pub fn mutate_retire_resume(input: Vec<u8>) -> Vec<u8> {
    let (raw, len, caps) = detach_vec(input);
    let fragments: Ghost<(Recovery, PhysicalPool, PhysicalRegion, PhysicalRegion)> = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let (left, right) = region.split_at(2int);
        (recovery, pool, left, right)
    };
    let (recovery, pool, left, right) = fragments.split();
    let mut left = left;
    let mut right = right;

    let left_bytes = unsafe { borrow_mut(&raw, 0, 2, ghost! { &mut *left }) };
    let right_bytes = unsafe { borrow_mut(&raw, 2, 2, ghost! { &mut *right }) };
    left_bytes[1] = 0xA1;
    right_bytes[0] = 0xB2;
    let _ = (left_bytes, right_bytes);

    let full: Ghost<(Recovery, PhysicalRegion)> = ghost! {
        let pool = pool.into_inner().retire(left.into_inner()).retire(right.into_inner());
        (recovery.into_inner(), pool.finish(0int))
    };
    unsafe { resume_vec(raw, len, full) }
}

/// A genuine B1 allocation split with one returned half deliberately omitted.
/// The call to finish must fail its exact full-domain requirement.
#[cfg(feature = "negative_half_finish")]
#[requires(input@.len() >= 2)]
pub fn reject_half_finish(input: Vec<u8>) {
    let (_raw, _len, caps) = detach_vec(input);
    let _half = ghost! {
        let (_recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let (left, _right) = region.split_at(1int);
        pool.retire(left).finish(0int)
    };
}

/// A negative API-contract check over sealed tokens from distinct B1
/// allocations. The mismatched namespace and resource id prevent retirement.
#[cfg(feature = "negative_wrong_namespace")]
#[requires(pool.inner_logic().invariant())]
#[requires(region.inner_logic().invariant())]
#[requires(pool.inner_logic().capacity() == region.inner_logic().capacity())]
#[requires(forall<index: Int> pool.inner_logic().contains(index) ==>
    !(region.inner_logic().lo() <= index && index < region.inner_logic().hi()))]
#[requires(pool.inner_logic().namespace() != region.inner_logic().namespace())]
#[requires(pool.inner_logic().resource_id() != region.inner_logic().resource_id())]
// This is an API-contract negative over assumed Ghost inputs of the sealed
// types. It confirms namespace and resource-id guards; it does not prove a
// two-allocation caller that constructs such a mismatch.
pub(crate) fn reject_wrong_namespace(
    pool: Ghost<PhysicalPool>,
    region: Ghost<PhysicalRegion>,
) {
    let _ = ghost! { pool.into_inner().retire(region.into_inner()) };
}

#[cfg(test)]
mod tests {
    use super::{
        mutate_retire_resume, retire_at_capacity, retire_left_then_right,
        retire_right_then_left,
    };

    #[test]
    fn retires_empty_zero_capacity_spare_capacity_and_two_nonempty_halves() {
        retire_left_then_right(Vec::new(), 0);
        retire_right_then_left(Vec::with_capacity(37), 0);
        retire_at_capacity(Vec::new());
        retire_at_capacity(Vec::with_capacity(19));
        retire_left_then_right(vec![1, 2, 3, 4], 2);
        retire_right_then_left(vec![1, 2, 3, 4], 2);
    }

    #[test]
    fn mutates_disjoint_regions_then_resumes_through_the_pool() {
        let result = mutate_retire_resume(vec![0x10, 0x20, 0x30, 0x40, 0x50]);
        assert_eq!(result, vec![0x10, 0xA1, 0xB2, 0x40, 0x50]);
    }
}
