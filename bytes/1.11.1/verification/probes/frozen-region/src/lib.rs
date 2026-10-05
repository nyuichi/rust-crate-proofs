#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
use creusot_std::{
    ghost::{lifetime_logic::{EndBorrow, FullBorrow, LifetimeToken}, GhostShared},
    prelude::*,
};
#[path = "../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path = "../../../../src/ownership_proof/bound_ptr.rs"] mod bound_ptr;
#[path = "../../../../src/ownership_proof/frozen_region.rs"] mod frozen_region;
use raw_vec::PhysicalRegion;
use frozen_region::{FrozenRegion, borrow_frozen};

/// Freeze a real Vec allocation, create three independently borrowed read
/// tickets, read overlapping views, return all tickets, and explicitly free.
#[requires(at@ <= input@.len())]
#[ensures(input@.len() > 0 ==> result.0 == input@[0])]
#[ensures(at@ < input@.len() ==> result.1 == input@[at@])]
#[ensures(input@.len() == 0 ==> result.0 == 0u8)]
#[ensures(at@ == input@.len() ==> result.1 == 0u8)]
pub fn freeze_three_views(input: Vec<u8>, at: usize, reverse: bool) -> (u8, u8) {
    let (base, len, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    let (recovery, region) = capabilities.split();
    let lifetime = LifetimeToken::new();
    let (full, end): (Ghost<FullBorrow<PhysicalRegion>>, Ghost<EndBorrow<PhysicalRegion>>) =
        FullBorrow::new(region, snapshot!(lifetime.lft()));
    let shared: Ghost<FrozenRegion> = ghost! { GhostShared::new(full).into_inner() };
    let (first, tail) = lifetime.split();
    let (second, third) = tail.split();
    let suffix = base.advance_within(at);
    let all = borrow_frozen(&base, len, shared, &first);
    let overlap = borrow_frozen(&suffix, len - at, shared, &second);
    let again = borrow_frozen(&base, len, shared, &third);
    proof_assert!(all@ == again@);
    proof_assert!(forall<i: Int> 0 <= i && i < (len - at)@ ==> overlap@[i] == all@[at@ + i]);
    let a = if len == 0 { 0 } else { all[0] };
    let b = if at == len { 0 } else { overlap[0] };
    let _ = (all, overlap, again);
    let lifetime = if reverse { third.join(first).join(second) } else { first.join(second).join(third) };
    proof_assert!(lifetime.frac().ext_eq(creusot_std::logic::real::PositiveReal::from_int(1)));
    let dead = lifetime.end();
    let recovered = ghost! { end.into_inner().get(dead) };
    let capabilities = ghost! { (recovery.into_inner(), recovered.into_inner()) };
    unsafe { raw_vec::deallocate_bound_vec(base, capacity, capabilities); }
    (a, b)
}

#[cfg(feature = "negative_missing_ticket")]
pub fn missing_ticket(input: Vec<u8>) {
    let (base, _len, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    let (recovery, region) = capabilities.split();
    let lifetime = LifetimeToken::new();
    let (full, end) = FullBorrow::new(region, snapshot!(lifetime.lft()));
    let _shared = ghost! { GhostShared::new(full).into_inner() };
    let (one, _outstanding) = lifetime.split();
    let dead = one.end();
    let recovered = ghost! { end.into_inner().get(dead) };
    unsafe { raw_vec::deallocate_bound_vec(base, capacity,
        ghost! { (recovery.into_inner(), recovered.into_inner()) }); }
}

#[cfg(feature = "negative_unknown_read")]
pub fn unknown_cannot_be_shared() {
    let input = Vec::<u8>::with_capacity(4);
    let (base, _len, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    let (_recovery, region) = capabilities.split();
    let lifetime = LifetimeToken::new();
    let (full, _end) = FullBorrow::new(region, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    if capacity > 0 {
        proof_assert!(shared.val().cur().slot(0int) == Some(None));
        let _invalid = borrow_frozen(&base, 1, shared, &lifetime);
    }
}

#[cfg(feature = "negative_live_reference")]
pub fn live_reference_blocks_recovery(input: Vec<u8>) -> u8 {
    let (base, len, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    let (recovery, region) = capabilities.split();
    let lifetime = LifetimeToken::new();
    let (full, end) = FullBorrow::new(region, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    let bytes = borrow_frozen(&base, len, shared, &lifetime);
    let dead = lifetime.end();
    let recovered = ghost! { end.into_inner().get(dead) };
    unsafe { raw_vec::deallocate_bound_vec(base, capacity,
        ghost! { (recovery.into_inner(), recovered.into_inner()) }); }
    if bytes.is_empty() { 0 } else { bytes[0] }
}

#[cfg(test)]
mod tests {
    #[test]
    fn three_overlapping_readers_and_empty_views() {
        for input in [vec![], vec![7], vec![1,2,3,4], Vec::with_capacity(8)] {
            for at in 0..=input.len() {
                for reverse in [false, true] {
                    let expected = (input.first().copied().unwrap_or(0), input.get(at).copied().unwrap_or(0));
                    assert_eq!(super::freeze_three_views(input.clone(), at, reverse), expected);
                }
            }
        }
    }
}
