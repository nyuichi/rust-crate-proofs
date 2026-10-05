//! Feasibility probe for an authority-backed affine ticket inventory.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;

#[cfg(not(ticket_laws_only))]
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[cfg(not(ticket_laws_only))]
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[cfg(not(ticket_laws_only))]
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/scalable_tickets.rs"]
mod scalable_tickets;

#[cfg(not(ticket_laws_only))]
pub(crate) mod ownership_proof {
    pub(crate) use crate::{owned_region, raw_vec};
}

#[cfg(not(ticket_laws_only))]
use creusot_std::prelude::*;
#[cfg(not(ticket_laws_only))]
use raw_vec::deallocate_bound_vec;

/// Start with one full-range ticket, replace it at zero, then replace the
/// nonempty ticket at its far endpoint. Three registrations remain, two of
/// which own empty intervals. The empty registrations must be returned before
/// final allocation recovery is available.
#[cfg(not(ticket_laws_only))]
pub fn replace_twice_and_return_all(input: alloc::vec::Vec<u8>) {
    let (bound, capacity, initialized) = scalable_tickets::initialize(input);
    let (mut coordinator, root) = initialized.split();
    let (empty_left, full_right) = scalable_tickets::replace(
        ghost!(&mut *coordinator), root, 0,
    ).split();
    let (full_left, empty_right) = scalable_tickets::replace(
        ghost!(&mut *coordinator), full_right, capacity,
    ).split();

    proof_assert!((*coordinator.public().pending).len() == 3);
    proof_assert!(empty_left.inner_logic().1.lo() == empty_left.inner_logic().1.hi());
    proof_assert!(empty_right.inner_logic().1.lo() == empty_right.inner_logic().1.hi());
    scalable_tickets::retire(ghost!(&mut *coordinator), full_left);
    proof_assert!((*coordinator.public().pending).len() == 2);
    scalable_tickets::retire(ghost!(&mut *coordinator), empty_left);
    scalable_tickets::retire(ghost!(&mut *coordinator), empty_right);

    let capabilities = scalable_tickets::finish(coordinator);
    // SAFETY: finish requires every affine ticket returned and recovers full
    // B1 physical coverage plus the matching recovery marker.
    unsafe { deallocate_bound_vec(bound, capacity, capabilities) };
}

/// Expected precondition rejection: all bytes have returned, but two empty
/// interval tickets remain outstanding.
#[cfg(feature = "negative_premature_empty_tickets")]
#[cfg(not(ticket_laws_only))]
pub fn reject_finish_with_empty_tickets(input: alloc::vec::Vec<u8>) {
    let (bound, capacity, initialized) = scalable_tickets::initialize(input);
    let (mut coordinator, root) = initialized.split();
    let (_empty_left, full_right) = scalable_tickets::replace(
        ghost!(&mut *coordinator), root, 0,
    ).split();
    let (full_left, _empty_right) = scalable_tickets::replace(
        ghost!(&mut *coordinator), full_right, capacity,
    ).split();
    scalable_tickets::retire(ghost!(&mut *coordinator), full_left);
    proof_assert!((*coordinator.public().pending).len() == 2);
    let _forbidden = scalable_tickets::finish(coordinator);
    let _ = (bound, capacity);
}
