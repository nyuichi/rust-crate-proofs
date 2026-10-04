//! Feasibility probe for shared borrows of a real boxed control block.
//!
//! This checks only the existing Creusot lifetime-token, FullBorrow, and
//! pointer-permission interfaces. It makes no claim about Bytes/BytesMut,
//! atomics, reference counts, or drop glue.
#![allow(unexpected_cfgs)]

use creusot_std::{
    ghost::{
        lifetime_logic::{EndBorrow, FullBorrow, LifetimeToken},
        perm::Perm,
        GhostShared,
    },
    prelude::*,
};

#[derive(DeepModel, creusot_std::std::cmp::PartialEq, ::core::cmp::Eq)]
pub struct Control {
    pub metadata: u64,
}

type OwnedControl = Box<Perm<*const Control>>;
type SharedControlBorrow = FullBorrow<OwnedControl>;

/// Recover the immutable pointer permission through one ticket's lifetime
/// fraction. The returned reference is scoped by the Rust borrow of `ticket`.
#[requires(pointer as *const Control == *shared.val().cur().ward())]
#[requires(shared.val().lft() == ticket.lft())]
#[ensures(*result == *shared.val().cur().val())]
fn borrow_control<'a>(
    pointer: *mut Control,
    shared: Ghost<GhostShared<SharedControlBorrow>>,
    ticket: &'a LifetimeToken,
) -> &'a Control {
    let permission: Ghost<&'a Perm<*const Control>> = ghost! {
        let full: &'a SharedControlBorrow = (*shared).to_ref();
        let owner: &'a OwnedControl = full.borrow(ticket);
        &**owner
    };
    unsafe { Perm::as_ref(pointer, permission) }
}

/// Borrow the actual Box through two independent lifetime fractions, end the
/// scoped borrows in first-then-second order, rejoin the fractions, then
/// recover the same Box allocation and metadata.
#[ensures(result.metadata == input.metadata)]
pub fn recover_first_then_second(input: Box<Control>) -> Box<Control> {
    let (pointer, owner) = Perm::from_box(input);
    let original_metadata = unsafe {
        (*Perm::as_ref(pointer, ghost! { &**owner })).metadata
    };

    let lifetime = LifetimeToken::new();
    let (full, end_borrow): (Ghost<SharedControlBorrow>, Ghost<EndBorrow<OwnedControl>>) =
        FullBorrow::new(owner, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    let (first, second) = lifetime.split();

    {
        let borrowed = borrow_control(pointer, shared, &first);
        proof_assert!(borrowed.metadata == original_metadata);
        let _observed = borrowed.metadata;
    }
    {
        let borrowed = borrow_control(pointer, shared, &second);
        proof_assert!(borrowed.metadata == original_metadata);
        let _observed = borrowed.metadata;
    }

    let dead = first.join(second).end();
    let recovered_owner = ghost! { end_borrow.into_inner().get(dead) };
    let recovered = unsafe { Perm::to_box(pointer, recovered_owner) };
    proof_assert!(recovered.metadata == original_metadata);
    recovered
}

/// Exercise the opposite order for the two scoped immutable borrows.
#[ensures(result.metadata == input.metadata)]
pub fn recover_second_then_first(input: Box<Control>) -> Box<Control> {
    let (pointer, owner) = Perm::from_box(input);
    let original_metadata = unsafe {
        (*Perm::as_ref(pointer, ghost! { &**owner })).metadata
    };

    let lifetime = LifetimeToken::new();
    let (full, end_borrow): (Ghost<SharedControlBorrow>, Ghost<EndBorrow<OwnedControl>>) =
        FullBorrow::new(owner, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    let (first, second) = lifetime.split();

    {
        let borrowed = borrow_control(pointer, shared, &second);
        proof_assert!(borrowed.metadata == original_metadata);
        let _observed = borrowed.metadata;
    }
    {
        let borrowed = borrow_control(pointer, shared, &first);
        proof_assert!(borrowed.metadata == original_metadata);
        let _observed = borrowed.metadata;
    }

    let dead = second.join(first).end();
    let recovered_owner = ghost! { end_borrow.into_inner().get(dead) };
    let recovered = unsafe { Perm::to_box(pointer, recovered_owner) };
    proof_assert!(recovered.metadata == original_metadata);
    recovered
}

/// One half of a split lifetime does not own enough fraction to end it.
#[cfg(feature = "negative_one_fraction")]
pub fn negative_one_fraction_cannot_end() {
    let lifetime = LifetimeToken::new();
    let (one_half, _other_half) = lifetime.split();
    let _dead = one_half.end();
}

/// An outstanding borrow ticket retains half the lifetime fraction, so the
/// other ticket cannot end the lifetime or release the boxed owner.
#[cfg(feature = "negative_outstanding_borrow")]
pub fn negative_outstanding_borrow_cannot_recover(input: Box<Control>) -> Box<Control> {
    let (pointer, owner) = Perm::from_box(input);
    let lifetime = LifetimeToken::new();
    let (full, end_borrow): (Ghost<SharedControlBorrow>, Ghost<EndBorrow<OwnedControl>>) =
        FullBorrow::new(owner, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    let (borrow_ticket, coordinator_ticket) = lifetime.split();

    let borrowed = borrow_control(pointer, shared, &borrow_ticket);
    // This must fail: the live borrow ticket still owns the other half.
    let dead = coordinator_ticket.end();
    let recovered_owner = ghost! { end_borrow.into_inner().get(dead) };
    let recovered = unsafe { Perm::to_box(pointer, recovered_owner) };
    proof_assert!(borrowed.metadata == recovered.metadata);
    recovered
}

/// Diagnostic only: try to keep an ordinary reference live across ending its
/// token, recovering the Box, and dropping that Box, then read through the
/// reference. A Rust borrow-check rejection is expected; this is not a proof
/// contract and must never be treated as an accepted memory-safety proof.
#[cfg(feature = "negative_ended_reference")]
pub fn negative_ended_reference(input: Box<Control>) -> u64 {
    let (pointer, owner) = Perm::from_box(input);
    let lifetime = LifetimeToken::new();
    let (full, end_borrow): (Ghost<SharedControlBorrow>, Ghost<EndBorrow<OwnedControl>>) =
        FullBorrow::new(owner, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };

    let borrowed = borrow_control(pointer, shared, &lifetime);
    let dead = lifetime.end();
    let recovered_owner = ghost! { end_borrow.into_inner().get(dead) };
    let recovered = unsafe { Perm::to_box(pointer, recovered_owner) };
    drop(recovered);
    borrowed.metadata
}
