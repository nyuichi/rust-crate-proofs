//! Prove bounded pointer-metadata advancement without minting memory access.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;

use alloc::vec::Vec;
use creusot_std::prelude::*;

#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

use raw_vec::{deallocate_bound_vec, detach_vec};

/// Advance by zero, then free with the original B1 descriptor and capabilities.
#[ensures(result.0@ == input@.len())]
#[ensures(result.1@ >= result.0@)]
pub fn advance_zero_and_release(input: Vec<u8>) -> (usize, usize) {
    let (raw, len, capabilities) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let unchanged = bound.advance_within(0);
    proof_assert!(unchanged.invariant());

    // The advanced value is metadata only; B3 uses the original offset-zero
    // descriptor with the unique B1 recovery and full physical-region tokens.
    unsafe { deallocate_bound_vec(bound, capacity, capabilities) };
    (len, capacity)
}

/// Advance to the one-past position, then free with the original descriptor.
#[ensures(result.0@ == input@.len())]
#[ensures(result.1@ >= result.0@)]
pub fn advance_to_capacity_and_release(input: Vec<u8>) -> (usize, usize) {
    let (raw, len, capabilities) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let one_past = bound.advance_within(capacity);
    proof_assert!(one_past.invariant());

    // `one_past` remains only a pointer-sized descriptor. It carries no live
    // permission and is not used to dereference or deallocate the allocation.
    unsafe { deallocate_bound_vec(bound, capacity, capabilities) };
    (len, capacity)
}

/// Recover the original sealed base after advancing metadata to one-past.
/// Deallocation consumes the unique recovery and full region with that base.
pub fn advance_retreat_and_release(input: Vec<u8>) {
    let (raw, _len, capabilities) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let interior = bound.advance_within(capacity);
    let base = interior.retreat_within(capacity);
    unsafe { deallocate_bound_vec(base, capacity, capabilities) };
}

/// Negative control: advancing beyond the B1 allocation must fail the helper's
/// range precondition. The attempted pointer is never dereferenced.
#[cfg(feature = "negative_overshoot")]
pub fn negative_advance_past_capacity(input: Vec<u8>) {
    let (raw, _len, capabilities) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let overshoot = capacity + 1;
    let _invalid = bound.advance_within(overshoot);
    // This line is unreachable in a successful proof because the preceding
    // call violates `advance_within`'s bounded-offset precondition.
    unsafe { deallocate_bound_vec(bound, capacity, capabilities) };
}

/// Negative control: an interior pointer keeps the same native allocation
/// address lineage but does not satisfy B3's sealed offset-zero cleanup gate.
#[cfg(feature = "negative_interior_cleanup")]
#[requires(input@.len() >= 1)]
pub fn negative_deallocate_from_interior(input: Vec<u8>) {
    let (raw, _len, capabilities) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let interior = bound.advance_within(1);

    // SAFETY: intentionally rejected. B3 requires the original offset-zero
    // BoundPtr even when pointer bits/provenance and all physical tokens match.
    unsafe { deallocate_bound_vec(interior, capacity, capabilities) };
}

#[cfg(test)]
mod tests {
    use super::{advance_retreat_and_release, advance_to_capacity_and_release, advance_zero_and_release};
    use crate::raw_vec::{deallocate_bound_vec, detach_vec};
    use alloc::vec::Vec;

    fn check_native_offset(input: Vec<u8>, requested: Option<usize>) {
        let (raw, _len, capabilities) = detach_vec(input);
        let (bound, capacity) = raw.into_bound_ptr_at_zero();
        let count = requested.unwrap_or(capacity);
        assert!(count <= capacity);

        let start = bound.as_ptr();
        let advanced = bound.advance_within(count);
        assert_eq!(advanced.as_ptr(), start.wrapping_add(count));

        let restored = advanced.retreat_within(count);
        assert_eq!(restored.as_ptr(), start);
        // Recovery uses the derived sealed base and original full capabilities.
        unsafe { deallocate_bound_vec(restored, capacity, capabilities) };
    }

    #[test]
    fn zero_interior_and_one_past_offsets_preserve_native_pointer_arithmetic() {
        check_native_offset(Vec::new(), Some(0));
        check_native_offset(Vec::with_capacity(0), None);
        check_native_offset(vec![1, 2, 3], Some(0));
        check_native_offset(Vec::with_capacity(8), Some(1));
        check_native_offset(vec![1, 2, 3], None);

        for input in [Vec::new(), Vec::with_capacity(8), vec![0, 255, 3]] {
            advance_retreat_and_release(input);
        }

        // The public helper functions independently exercise zero and
        // one-past advancement before explicit full-allocation cleanup.
        for input in [Vec::new(), Vec::with_capacity(0), vec![1, 2, 3]] {
            let (len, capacity) = advance_zero_and_release(input);
            assert!(len <= capacity);
        }
        for input in [Vec::with_capacity(0), Vec::with_capacity(8), vec![1, 2, 3]] {
            let (len, capacity) = advance_to_capacity_and_release(input);
            assert!(len <= capacity);
        }
    }
}
