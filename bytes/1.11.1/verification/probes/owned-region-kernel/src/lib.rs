//! Pure resource-algebra probe for the sealed owned-region wrapper.
#![allow(unexpected_cfgs)]

use creusot_std::{
    ghost::resource::Resource,
    logic::{FMap, Int, ra::{RA, excl::Excl}},
    prelude::*,
};

#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;

use owned_region::{KernelRA, OwnedRegion, SlotMap};

/// Create a model-only four-slot ledger. This is deliberately not tied to an
/// allocation and cannot establish physical byte ownership.
#[check(ghost)]
#[ensures(result.inner_logic().invariant())]
#[ensures(result.inner_logic().lo() == 10usize@ && result.inner_logic().hi() == 14usize@)]
#[ensures(result.inner_logic().slot(10usize@) == Some(Some(0x21u8)))]
#[ensures(result.inner_logic().slot(11usize@) == Some(None))]
#[ensures(result.inner_logic().slot(12usize@) == Some(Some(0x43u8)))]
#[ensures(result.inner_logic().slot(13usize@) == Some(Some(0x54u8)))]
fn model_region() -> Ghost<OwnedRegion> {
    let slots: Snapshot<FMap<Int, Excl<Option<u8>>>> = snapshot! {
        FMap::empty()
            .insert(10, Excl(Some(0x21u8)))
            .insert(11, Excl(None))
            .insert(12, Excl(Some(0x43u8)))
            .insert(13, Excl(Some(0x54u8)))
    };
    let empty_slots: Snapshot<SlotMap> = snapshot!(FMap::empty());
    let recovery_value: Snapshot<KernelRA> = snapshot!((Some(Excl(())), FMap::empty()));
    let region_value: Snapshot<KernelRA> = snapshot!((None, *slots));
    let allocation_value: Snapshot<KernelRA> = snapshot!((Some(Excl(())), *slots));
    ghost! {
        proof_assert!(forall<index: Int>
            (*empty_slots).get(index).op((*slots).get(index)) == Some((*slots).get(index))
        );
        owned_region::map_compose_eq(empty_slots, slots, slots);
    };
    proof_assert!((*recovery_value).op(*region_value) == Some(*allocation_value));
    proof_assert!(KernelRA::incl_eq_op(*recovery_value, *region_value, *allocation_value));

    ghost! {
        let allocation = Resource::alloc(allocation_value).into_inner();
        let (_recovery, region_resource) = allocation.split(recovery_value, region_value);
        OwnedRegion::from_model_ledger(10int, 14int, region_resource)
    }
}

/// Return both halves from a helper, preserving their shared resource id.
#[check(ghost)]
#[ensures(result.inner_logic().0.invariant() && result.inner_logic().1.invariant())]
#[ensures(result.inner_logic().0.lo() == 10usize@ && result.inner_logic().0.hi() == 12usize@)]
#[ensures(result.inner_logic().1.lo() == 12usize@ && result.inner_logic().1.hi() == 14usize@)]
#[ensures(result.inner_logic().0.resource_id() == result.inner_logic().1.resource_id())]
#[ensures(result.inner_logic().0.slot(10usize@) == Some(Some(0x21u8)))]
#[ensures(result.inner_logic().0.slot(11usize@) == Some(None))]
#[ensures(result.inner_logic().1.slot(12usize@) == Some(Some(0x43u8)))]
#[ensures(result.inner_logic().1.slot(13usize@) == Some(Some(0x54u8)))]
fn split_in_helper() -> Ghost<(OwnedRegion, OwnedRegion)> {
    let region = model_region();
    ghost! { region.into_inner().split_at(12int) }
}

/// The caller observes each disjoint half independently, then rejoins them.
/// The postcondition checks exact domain and value preservation across the
/// helper boundary, including an Unknown slot.
#[check(ghost)]
#[ensures(result.inner_logic().invariant())]
#[ensures(result.inner_logic().lo() == 10usize@ && result.inner_logic().hi() == 14usize@)]
#[ensures(result.inner_logic().slot(10usize@) == Some(Some(0x21u8)))]
#[ensures(result.inner_logic().slot(11usize@) == Some(None))]
#[ensures(result.inner_logic().slot(12usize@) == Some(Some(0x43u8)))]
#[ensures(result.inner_logic().slot(13usize@) == Some(Some(0x54u8)))]
fn caller_rejoins_helper_regions() -> Ghost<OwnedRegion> {
    let halves = split_in_helper();
    ghost! {
        let (left, right) = halves.into_inner();
        proof_assert!(left.slot(10) == Some(Some(0x21u8)));
        proof_assert!(left.slot(11) == Some(None));
        proof_assert!(right.slot(12) == Some(Some(0x43u8)));
        proof_assert!(right.slot(13) == Some(Some(0x54u8)));
        left.join(right)
    }
}

#[cfg(feature = "wrong_overlap")]
/// Expected precondition failure: overlapping intervals cannot be joined.
#[check(ghost)]
#[requires(left.invariant() && right.invariant())]
#[requires(left.resource_id() == right.resource_id())]
#[requires(right.lo() < left.hi())]
fn rejected_overlap(left: OwnedRegion, right: OwnedRegion) -> OwnedRegion {
    left.join(right)
}
