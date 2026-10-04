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
    PhysicalRegion, RawAllocation, Recovery, detach_vec, resume_vec,
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

#[cfg(test)]
mod tests {
    use super::detach_split_join_resume;

    #[test]
    fn resumes_vec_after_helper_split_and_join() {
        let input = vec![0x11, 0x22, 0x33, 0x44];
        let result = detach_split_join_resume(input.clone(), 2);
        assert_eq!(result, input);
    }
}
