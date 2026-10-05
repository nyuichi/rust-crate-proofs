//! Body proof and native checks for disjoint unique in-place reclamation.
#![allow(unexpected_cfgs, dead_code)]
#![recursion_limit = "512"]
extern crate alloc;
use alloc::vec::Vec;
use creusot_std::prelude::*;

#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../src/storage_ops.rs"]
mod storage_ops;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/unique_reclaim.rs"]
mod unique_reclaim;

#[requires(len <= offset)]
#[requires(offset@ + len@ <= input@.len())]
pub fn check_reclaim(input: Vec<u8>, offset: usize, len: usize) {
    #[cfg(not(creusot))]
    let expected = input[offset..offset + len].to_vec();
    let original = snapshot!(input@);
    let (base, _, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    let (recovery, region) = capabilities.split();
    let view = base.advance_within(offset);
    let (reclaimed, region) = unique_reclaim::reclaim(view, offset, len, region);
    #[cfg(not(creusot))]
    assert_eq!(base.as_ptr(), reclaimed.as_ptr());
    {
        let content = unsafe { raw_vec::borrow_bound(&reclaimed, len, region.borrow()) };
        assert!(content.len() == len);
        proof_assert!(forall<i: Int> 0 <= i && i < len@ ==> content@[i] == original[offset@ + i]);
        #[cfg(not(creusot))]
        assert_eq!(content, expected.as_slice());
    }
    // Full region recombination remains sufficient for existing B3 cleanup.
    unsafe { raw_vec::deallocate_bound_vec(reclaimed, capacity, ghost! { (recovery.into_inner(), region.into_inner()) }); }
}

#[cfg(feature = "negative_overlap")]
pub fn reject_overlap() {
    let input = alloc::vec![1u8, 2u8, 3u8];
    let (base, _, _, capabilities) = bound_ptr::detach_bound_vec(input);
    let region = ghost! { capabilities.into_inner().1 };
    let view = base.advance_within(1);
    let _ = unique_reclaim::reclaim(view, 1, 2, region);
}

#[cfg(feature = "negative_unknown_source")]
pub fn reject_unknown_source() {
    let input: Vec<u8> = Vec::with_capacity(4);
    let (base, _, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    if capacity >= 3 {
        let region = ghost! { capabilities.into_inner().1 };
        let view = base.advance_within(2);
        let _ = unique_reclaim::reclaim(view, 2, 1, region);
    }
}
