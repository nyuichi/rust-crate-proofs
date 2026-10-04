//! B1/B2 probe: detach, split in a helper, rejoin, and resume a Vec.
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
    PhysicalRegion, RawAllocation, Recovery, borrow_mut, deallocate_vec, detach_vec, resume_vec,
};

/// Detach the Vec and return both owned intervals from a helper.
///
/// This only tests the B1 binding and proved resource-algebra split; no native
/// bytes are mutated here.
#[requires(split@ <= input@.len())]
#[ensures(result.1@ == input@.len())]
#[ensures(result.0.invariant())]
#[ensures(result.0.capacity() >= input@.len())]
#[ensures(result.2.inner_logic().0.invariant())]
#[ensures(result.2.inner_logic().1.invariant() && result.2.inner_logic().2.invariant())]
#[ensures(result.2.inner_logic().0.capacity() == result.0.capacity())]
#[ensures(result.2.inner_logic().1.capacity() == result.0.capacity())]
#[ensures(result.2.inner_logic().2.capacity() == result.0.capacity())]
#[ensures(result.2.inner_logic().0.namespace() == result.2.inner_logic().1.resource_id())]
#[ensures(result.2.inner_logic().0.namespace() == result.2.inner_logic().2.resource_id())]
#[ensures(result.2.inner_logic().1.resource_id() == result.2.inner_logic().2.resource_id())]
#[ensures(result.2.inner_logic().0.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().1.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().2.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().1.lo() == 0 && result.2.inner_logic().1.hi() == split@)]
#[ensures(result.2.inner_logic().2.lo() == split@ &&
          result.2.inner_logic().2.hi() == result.0.capacity())]
#[ensures(forall<index: Int> 0 <= index && index < split@ ==>
    result.2.inner_logic().1.slot(index) == Some(Some(input@[index])))]
#[ensures(forall<index: Int> split@ <= index && index < input@.len() ==>
    result.2.inner_logic().2.slot(index) == Some(Some(input@[index])))]
#[ensures(forall<index: Int> input@.len() <= index && index < result.0.capacity() ==>
    result.2.inner_logic().2.slot(index) == Some(None))]
fn split_in_helper(
    input: Vec<u8>,
    split: usize,
) -> (
    RawAllocation,
    usize,
    Ghost<(Recovery, PhysicalRegion, PhysicalRegion)>,
) {
    let (raw, len, capabilities) = detach_vec(input);
    let fragments: Ghost<(Recovery, PhysicalRegion, PhysicalRegion)> = ghost! {
        let (recovery, region) = capabilities.into_inner();
        let boundary: Int = *Int::new(split as i128);
        let (left, right) = region.split_at(boundary);
        (recovery, left, right)
    };
    (raw, len, fragments)
}

/// The caller rejoins the helper's exact fragments and resumes the same Vec.
#[requires(split@ <= input@.len())]
#[ensures(result@ == input@)]
fn detach_split_join_resume(input: Vec<u8>, split: usize) -> Vec<u8> {
    let (raw, len, fragments) = split_in_helper(input, split);
    let capabilities: Ghost<(Recovery, PhysicalRegion)> = ghost! {
        let (recovery, left, right) = fragments.into_inner();
        let region = left.join(right);
        (recovery, region)
    };
    // SAFETY: B1 minted this raw descriptor and its matching Recovery/region;
    // split/join preserved full [0, capacity) coverage and all known values.
    unsafe { resume_vec(raw, len, capabilities) }
}

/// Borrow and mutate disjoint initialized prefixes of independently owned
/// fragments, then rejoin the updated ledgers and resume the allocation.
#[requires(input@.len() >= 4)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[1] == 0xA1u8)]
#[ensures(result@[2] == 0xB2u8)]
#[ensures(forall<index: Int> 0 <= index && index < input@.len() &&
    index != 1 && index != 2 ==> result@[index] == input@[index])]
fn detach_mutate_disjoint_fragments(input: Vec<u8>) -> Vec<u8> {
    let (raw, len, fragments) = split_in_helper(input, 2);
    let (recovery, left, right) = fragments.split();
    let mut left = left;
    let mut right = right;

    // Each borrow is tied to a distinct physical interval capability. The
    // pointer is derived from the same sealed B1 descriptor in both calls.
    let left_bytes = unsafe { borrow_mut(&raw, 0, 2, ghost! { &mut *left }) };
    let right_bytes = unsafe { borrow_mut(&raw, 2, 2, ghost! { &mut *right }) };
    left_bytes[1] = 0xA1;
    right_bytes[0] = 0xB2;
    let _ = (left_bytes, right_bytes);

    let capabilities: Ghost<(Recovery, PhysicalRegion)> = ghost! {
        let region = left.into_inner().join(right.into_inner());
        (recovery.into_inner(), region)
    };
    // SAFETY: the two borrows were disjoint, have ended, and join preserves
    // every updated byte plus full allocation coverage.
    unsafe { resume_vec(raw, len, capabilities) }
}

/// Return the split intervals in reverse tuple order.
#[requires(split@ <= input@.len())]
#[ensures(result.1@ == input@.len())]
#[ensures(result.0.invariant())]
#[ensures(result.0.capacity() >= input@.len())]
#[ensures(result.2.inner_logic().0.invariant())]
#[ensures(result.2.inner_logic().1.invariant() && result.2.inner_logic().2.invariant())]
#[ensures(result.2.inner_logic().0.capacity() == result.0.capacity())]
#[ensures(result.2.inner_logic().1.capacity() == result.0.capacity())]
#[ensures(result.2.inner_logic().2.capacity() == result.0.capacity())]
#[ensures(result.2.inner_logic().0.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().1.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().2.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().1.resource_id() == result.0.namespace())]
#[ensures(result.2.inner_logic().2.resource_id() == result.0.namespace())]
#[ensures(result.2.inner_logic().1.lo() == split@)]
#[ensures(result.2.inner_logic().1.hi() == result.0.capacity())]
#[ensures(result.2.inner_logic().2.lo() == 0)]
#[ensures(result.2.inner_logic().2.hi() == split@)]
fn split_in_reverse_helper(
    input: Vec<u8>,
    split: usize,
) -> (
    RawAllocation,
    usize,
    Ghost<(Recovery, PhysicalRegion, PhysicalRegion)>,
) {
    let (raw, len, fragments) = split_in_helper(input, split);
    let reversed: Ghost<(Recovery, PhysicalRegion, PhysicalRegion)> = ghost! {
        let (recovery, left, right) = fragments.into_inner();
        (recovery, right, left)
    };
    (raw, len, reversed)
}

/// Rejoin fragments returned in their spatial order, then explicitly free.
#[requires(split@ <= input@.len())]
fn deallocate_split_normal_tuple(input: Vec<u8>, split: usize) {
    let (raw, _len, fragments) = split_in_helper(input, split);
    let capabilities: Ghost<(Recovery, PhysicalRegion)> = ghost! {
        let (recovery, left, right) = fragments.into_inner();
        (recovery, left.join(right))
    };
    // SAFETY: B1's recovery plus the rejoined full interval are consumed by B3.
    unsafe { deallocate_vec(raw, capabilities) }
}

/// Reorder a reversed fragment tuple spatially, rejoin, and explicitly free.
#[requires(split@ <= input@.len())]
fn deallocate_split_reversed_tuple(input: Vec<u8>, split: usize) {
    let (raw, _len, fragments) = split_in_reverse_helper(input, split);
    let capabilities: Ghost<(Recovery, PhysicalRegion)> = ghost! {
        // The helper returned `(recovery, right_interval, left_interval)`.
        let (recovery, right, left) = fragments.into_inner();
        (recovery, left.join(right))
    };
    // SAFETY: the caller restores contiguous interval order before B3 consumes
    // the recovery marker and complete physical region.
    unsafe { deallocate_vec(raw, capabilities) }
}

#[cfg(test)]
mod tests {
    use super::{
        deallocate_split_normal_tuple, deallocate_split_reversed_tuple,
        detach_mutate_disjoint_fragments, detach_split_join_resume,
    };

    #[test]
    fn resumes_vec_after_helper_split_and_join() {
        let input = vec![0x11, 0x22, 0x33, 0x44];
        let result = detach_split_join_resume(input.clone(), 2);
        assert_eq!(result, input);
    }

    #[test]
    fn mutates_disjoint_fragments_then_resumes_vec() {
        let input = vec![0x10, 0x20, 0x30, 0x40, 0x50];
        let result = detach_mutate_disjoint_fragments(input);
        assert_eq!(result, vec![0x10, 0xA1, 0xB2, 0x40, 0x50]);
    }

    #[test]
    fn explicitly_frees_vec_with_normal_fragment_tuple() {
        deallocate_split_normal_tuple(Vec::new(), 0);
        deallocate_split_normal_tuple(Vec::with_capacity(37), 0);
        deallocate_split_normal_tuple(vec![1, 2, 3, 4], 2);
    }

    #[test]
    fn explicitly_frees_vec_with_reversed_fragment_tuple() {
        deallocate_split_reversed_tuple(Vec::new(), 0);
        deallocate_split_reversed_tuple(Vec::with_capacity(19), 0);
        deallocate_split_reversed_tuple(vec![1, 2, 3, 4], 2);
    }
}
