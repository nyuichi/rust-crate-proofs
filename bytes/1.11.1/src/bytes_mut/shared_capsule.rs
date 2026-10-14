//! Proof-only ownership capsule for a `BytesMut` shared control block.
//!
//! This module does not change the native `Shared` layout. Its generic event
//! invariant owns the one counter permission; the payload owns header/storage
//! authority, the all-reference ledger, pool, and phase state.
//! Per-handle tickets and physical regions remain outside the control block.

use alloc::boxed::Box;
use creusot_std::{
    ghost::{
        invariant::Protocol,
        perm::Perm,
        resource::Authority,
        Ghost, GhostShared,
    },
    logic::{ra::{excl::Excl, RA}, Id},
    prelude::*,
};

use crate::{
    ownership_proof::{
        raw_vec::{BoundPtr, PhysicalPool, PhysicalRegion, Recovery},
        ref_count_ledger::{
            active_mutable_contains, sole_active_id, ActiveReferences,
            AllocationBinding, OwnerLedger, prove_no_active_mutable_on_empty,
            prove_owner_reclassification_effect, OwnerTicket, RefKind,
            RetirePermit,
        },
        shared_context::BorrowedSharedContext,
        ready_registry::{ReadyCap, ReadyKey},
        terminal_phase::{PhaseAuthority, TerminalAction},
    },
    ref_count_ops::{event_namespace, CounterProtocol, EventBoundInvariant, NativeCounter},
    vec_storage::{StoredAuthority, VecStorage},
};

use super::Shared;

/// Payload for the generic counter event invariant. The invariant owns the
/// sole native counter permission outside this value; this payload records
/// only its opaque model and the shared storage/reference ledgers.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedPayload {
    header_id: *mut Shared,
    counter_model: Snapshot<NativeCounter>,
    allocation_binding: Snapshot<AllocationBinding>,
    header_vec: Snapshot<VecStorage>,
    original_capacity_repr: Snapshot<usize>,
    ledger_id: Id,
    phase_id: Id,
    lease_generation: Int,
    lease_key: ReadyKey,
    lease: HeaderLeaseAuthority,
    root: Option<Box<Perm<*const Shared>>>,
    storage: Option<StoredAuthority>,
    pool: Option<PhysicalPool>,
    ledger: Option<OwnerLedger>,
    retire: Option<RetirePermit>,
    phase: PhaseAuthority,
    terminal_action: Option<TerminalAction>,
    closed: bool,
}

pub(crate) type HeaderLeaseStamp = (
    Int,
    *mut (),
    NativeCounter,
    Id,
    Id,
    StoredAuthority,
    VecStorage,
    usize,
    ReadyKey,
);
/// One exclusive value distinguishes idle from every active generation. In
/// particular, an idle capability cannot compose with an active authority by
/// treating an absent second tuple component as the unit of a product RA.
pub(crate) type HeaderLeaseState = Option<Excl<(Id, Option<HeaderLeaseStamp>)>>;

/// The authority retains only the real RA authority. Its one exclusive cap is
/// moved into the caller context while idle and into the read loan while
/// active; the resource does not count as a native reference.
#[allow(missing_debug_implementations)]
struct HeaderLeaseAuthority {
    resource_id: Id,
    header_id: *mut Shared,
    authority: Authority<HeaderLeaseState>,
}

#[logic(open(self))]
fn header_lease_stamp(
    generation: Int,
    header_id: *mut Shared,
    counter_model: NativeCounter,
    ledger_id: Id,
    phase_id: Id,
    storage: StoredAuthority,
    header_vec: VecStorage,
    original_capacity_repr: usize,
    lease_key: ReadyKey,
) -> HeaderLeaseStamp {
    (
        generation,
        header_id as *mut (),
        counter_model,
        ledger_id,
        phase_id,
        storage,
        header_vec,
        original_capacity_repr,
        lease_key,
    )
}

#[logic(open(self))]
fn header_lease_stamp_matches_key(
    stamp: HeaderLeaseStamp,
    key: ReadyKey,
) -> bool {
    pearlite! { stamp.8 == key }
}

/// Exact ticket portion of the split relation. Kept opaque with its full
/// equation so clients can use the contract without opening ticket fields.
#[logic(opaque)]
#[ensures(result == (
    split.0.ledger_id() == (*before).ledger_id() &&
    split.1.ledger_id() == (*before).ledger_id() &&
    split.0.fragment_resource_id() == (*before).fragment_resource_id() &&
    split.1.fragment_resource_id() == (*before).fragment_resource_id() &&
    split.0.allocation_binding() == (*before).allocation_binding() &&
    split.1.allocation_binding() == (*before).allocation_binding() &&
    split.0.class() == RefKind::Mutable && split.1.class() == RefKind::Mutable &&
    split.0.owner_id() == (*before).owner_id() &&
    split.1.owner_id() != (*before).owner_id() &&
    split.0.owned_lo() == (*before).owned_lo() &&
    split.0.owned_hi() == *cut && split.1.owned_lo() == *cut &&
    split.1.owned_hi() == (*before).owned_hi()
))]
pub(crate) fn mutable_owner_ticket_split_result(
    before: Snapshot<OwnerTicket>,
    cut: Snapshot<Int>,
    split: (OwnerTicket, OwnerTicket),
) -> bool {
    pearlite! {
        split.0.ledger_id() == (*before).ledger_id() &&
        split.1.ledger_id() == (*before).ledger_id() &&
        split.0.fragment_resource_id() == (*before).fragment_resource_id() &&
        split.1.fragment_resource_id() == (*before).fragment_resource_id() &&
        split.0.allocation_binding() == (*before).allocation_binding() &&
        split.1.allocation_binding() == (*before).allocation_binding() &&
        split.0.class() == RefKind::Mutable && split.1.class() == RefKind::Mutable &&
        split.0.owner_id() == (*before).owner_id() &&
        split.1.owner_id() != (*before).owner_id() &&
        split.0.owned_lo() == (*before).owned_lo() &&
        split.0.owned_hi() == *cut && split.1.owned_lo() == *cut &&
        split.1.owned_hi() == (*before).owned_hi()
    }
}

/// Exact namespace, allocation, and interval portion of the physical split.
#[logic(opaque)]
#[ensures(result == (
    split.0.namespace() == (*before).namespace() &&
    split.1.namespace() == (*before).namespace() &&
    split.0.capacity() == (*before).capacity() &&
    split.1.capacity() == (*before).capacity() &&
    split.0.resource_id() == (*before).resource_id() &&
    split.1.resource_id() == (*before).resource_id() &&
    split.0.lo() == (*before).lo() && split.0.hi() == *cut &&
    split.1.lo() == *cut && split.1.hi() == (*before).hi()
))]
pub(crate) fn mutable_owner_region_split_bounds_result(
    before: Snapshot<PhysicalRegion>,
    cut: Snapshot<Int>,
    split: (PhysicalRegion, PhysicalRegion),
) -> bool {
    pearlite! {
        split.0.namespace() == (*before).namespace() &&
        split.1.namespace() == (*before).namespace() &&
        split.0.capacity() == (*before).capacity() &&
        split.1.capacity() == (*before).capacity() &&
        split.0.resource_id() == (*before).resource_id() &&
        split.1.resource_id() == (*before).resource_id() &&
        split.0.lo() == (*before).lo() && split.0.hi() == *cut &&
        split.1.lo() == *cut && split.1.hi() == (*before).hi()
    }
}

/// Pointwise byte-slot frame for both physical halves.
#[logic(opaque)]
#[ensures(result == (
    (forall<i: Int> split.0.slot(i) ==
        if (*before).lo() <= i && i < *cut { (*before).slot(i) } else { None }) &&
    (forall<i: Int> split.1.slot(i) ==
        if *cut <= i && i < (*before).hi() { (*before).slot(i) } else { None })
))]
pub(crate) fn mutable_owner_region_split_slots_result(
    before: Snapshot<PhysicalRegion>,
    cut: Snapshot<Int>,
    split: (PhysicalRegion, PhysicalRegion),
) -> bool {
    pearlite! {
        (forall<i: Int> split.0.slot(i) ==
            if (*before).lo() <= i && i < *cut { (*before).slot(i) } else { None }) &&
        (forall<i: Int> split.1.slot(i) ==
            if *cut <= i && i < (*before).hi() { (*before).slot(i) } else { None })
    }
}

/// Exact result relation shared by the payload callback and packet builder.
/// Its component equations are body-checked separately to keep each contract
/// small enough for modular clients.
#[logic(opaque)]
#[ensures(result == (
    mutable_owner_ticket_split_result(before_ticket, cut, (split.0, split.1)) &&
    mutable_owner_region_split_bounds_result(before_region, cut, (split.2, split.3)) &&
    mutable_owner_region_split_slots_result(before_region, cut, (split.2, split.3))
))]
pub(crate) fn mutable_owner_split_result(
    before_ticket: Snapshot<OwnerTicket>,
    before_region: Snapshot<PhysicalRegion>,
    cut: Snapshot<Int>,
    split: (OwnerTicket, OwnerTicket, PhysicalRegion, PhysicalRegion),
) -> bool {
    pearlite! {
        mutable_owner_ticket_split_result(before_ticket, cut, (split.0, split.1)) &&
        mutable_owner_region_split_bounds_result(before_region, cut, (split.2, split.3)) &&
        mutable_owner_region_split_slots_result(before_region, cut, (split.2, split.3))
    }
}

/// Convert the exact split slot relation and the old visible-prefix fact into
/// a pointwise known-slot fact for the right child. Keeping this arithmetic
/// proof next to the region relation avoids asking the packet constructor's
/// larger callback VC to rediscover the quantified interval bounds.
#[check(ghost)]
#[requires(mutable_owner_split_result(
    before_ticket,
    before_region,
    cut,
    (*parent_ticket, *child_ticket, *left_region, *right_region)
))]
#[requires(*cut >= *old_cursor)]
#[requires(*right_len == 0 || *cut + *right_len <= *old_cursor + *old_len)]
#[requires(*old_cursor + *old_len <= (*before_region).hi())]
#[requires(forall<i: Int> *old_cursor <= i && i < *old_cursor + *old_len ==>
    crate::ownership_proof::raw_vec::slot_known((*before_region).slot(i)))]
#[ensures(forall<i: Int> 0 <= i && i < *right_len ==>
    crate::ownership_proof::raw_vec::slot_known(
        (*right_region).slot(*cut + i)))]
pub(crate) fn prove_mutable_split_right_slots_known(
    before_ticket: Snapshot<OwnerTicket>,
    before_region: Snapshot<PhysicalRegion>,
    cut: Snapshot<Int>,
    parent_ticket: Snapshot<OwnerTicket>,
    child_ticket: Snapshot<OwnerTicket>,
    left_region: Snapshot<PhysicalRegion>,
    right_region: Snapshot<PhysicalRegion>,
    old_cursor: Snapshot<Int>,
    old_len: Snapshot<Int>,
    right_len: Snapshot<Int>,
) {
    proof_assert!(forall<i: Int> 0 <= i && i < *right_len ==>
        crate::ownership_proof::raw_vec::slot_known(
            (*right_region).slot(*cut + i)));
}

impl HeaderLeaseAuthority {
    #[logic(open(self))]
    fn resource_id(self) -> Id {
        self.resource_id
    }

    #[logic(open(self))]
    fn header_id(self) -> *mut Shared {
        self.header_id
    }

    #[logic(open(self), prophetic)]
    fn is_idle(self) -> bool {
        pearlite! { self.authority@ == Some(Excl((self.resource_id, None))) }
    }

    #[logic(open(self), prophetic)]
    fn is_active(self, stamp: HeaderLeaseStamp) -> bool {
        pearlite! { self.authority@ == Some(Excl((self.resource_id, Some(stamp)))) }
    }

    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[ensures(result.inner_logic().0.invariant())]
    #[ensures(result.inner_logic().0.is_idle())]
    #[ensures(result.inner_logic().0.resource_id() == result.inner_logic().1.resource_id())]
    #[ensures(result.inner_logic().0.header_id() == header_id)]
    #[ensures(result.inner_logic().1.slot() >= 0)]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures(context.inner_logic().registered_cap(
        (^context.inner_logic()),
        result.inner_logic().1,
        Some(Excl((result.inner_logic().1.resource_id(), None)))
    ))]
    pub(crate) fn new(
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        header_id: *mut Shared,
    ) -> Ghost<(Self, ReadyKey)> {
        ghost! {
            let mut authority: Authority<HeaderLeaseState> = Authority::alloc().into_inner();
            let resource_id = authority.id_ghost();
            let idle: HeaderLeaseState = Some(Excl((resource_id, None)));
            let fragment = authority.add_fragment(snapshot!(idle));
            let lease = Self {
                resource_id,
                header_id,
                authority,
            };
            let key = BorrowedSharedContext::register(context, Ghost::new(fragment)).into_inner();
            proof_assert!(lease.invariant() && lease.is_idle());
            proof_assert!(key.resource_id() == resource_id);
            (lease, key)
        }
    }

    #[check(ghost)]
    #[requires(authority.inner_logic().invariant() && authority.inner_logic().is_idle())]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key().resource_id() == authority.inner_logic().resource_id())]
    #[requires(header_lease_stamp_matches_key(
        *stamp,
        cap.inner_logic().key(),
    ))]
    #[requires(cap.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[requires(cap.inner_logic().fragment_value() ==
        Some(Excl((authority.inner_logic().resource_id(), None))))]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_active(*stamp))]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures((^authority.inner_logic()).header_id() ==
        authority.inner_logic().header_id())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == cap.inner_logic().key())]
    #[ensures(result.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[ensures(result.inner_logic().fragment_value() ==
        Some(Excl((authority.inner_logic().resource_id(), Some(*stamp)))))]
    fn activate(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let authority = authority.into_inner();
            let mut cap = cap.into_inner();
            let idle: HeaderLeaseState = Some(Excl((authority.resource_id, None)));
            let resource_id = authority.resource_id;
            let fragment = cap.fragment_ref().into_inner();
            authority.authority.frag_lemma(fragment);
            proof_assert!(fragment@ == idle);
            proof_assert!(authority.authority@ == idle);
            cap.update(&mut authority.authority, snapshot!((
                Some(Excl((resource_id, Some(*stamp)))),
                Some(Excl((resource_id, Some(*stamp)))),
            )));
            proof_assert!(cap.fragment_value() ==
                Some(Excl((resource_id, Some(*stamp)))));
            proof_assert!(authority.authority@ ==
                Some(Excl((resource_id, Some(*stamp)))));
            cap
        }
    }

    #[check(ghost)]
    #[requires(authority.inner_logic().invariant() &&
        authority.inner_logic().is_active(*stamp))]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key().resource_id() == authority.inner_logic().resource_id())]
    #[requires(header_lease_stamp_matches_key(
        *stamp,
        cap.inner_logic().key(),
    ))]
    #[requires(cap.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[requires(cap.inner_logic().fragment_value() ==
        Some(Excl((authority.inner_logic().resource_id(), Some(*stamp)))))]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_idle())]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures((^authority.inner_logic()).header_id() ==
        authority.inner_logic().header_id())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == cap.inner_logic().key())]
    #[ensures(result.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[ensures(result.inner_logic().fragment_value() ==
        Some(Excl((authority.inner_logic().resource_id(), None))))]
    fn deactivate(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let authority = authority.into_inner();
            let mut cap = cap.into_inner();
            let idle: HeaderLeaseState = Some(Excl((authority.resource_id, None)));
            let resource_id = authority.resource_id;
            let fragment = cap.fragment_ref().into_inner();
            authority.authority.frag_lemma(fragment);
            proof_assert!(fragment@.incl_eq(authority.authority@));
            proof_assert!(fragment@ ==
                Some(Excl((authority.resource_id, Some(*stamp)))));
            proof_assert!(authority.authority@ ==
                Some(Excl((authority.resource_id, Some(*stamp)))));
            cap.update(&mut authority.authority, snapshot!((
                Some(Excl((resource_id, None))),
                Some(Excl((resource_id, None))),
            )));
            proof_assert!(authority.authority@ == idle && cap.fragment_value() == idle);
            cap
        }
    }

    /// The current authoritative lease must agree with the caller's actual
    /// affine cap. The persistent exclusive header identity prevents a unit
    /// capability from standing in for readiness.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(cap.invariant())]
    #[requires(cap.key().resource_id() == self.resource_id())]
    #[requires(header_lease_stamp_matches_key(
        *stamp, cap.key(),
    ))]
    #[requires(cap.fragment_id() == self.resource_id())]
    #[requires(cap.fragment_value() == Some(Excl((self.resource_id, Some(*stamp)))))]
    #[ensures(self.is_active(*stamp))]
    fn require_active_fragment(
        &self,
        stamp: Snapshot<HeaderLeaseStamp>,
        cap: &ReadyCap<HeaderLeaseState>,
    ) {
        let _ = ghost! {
            let fragment = cap.fragment_ref().into_inner();
            self.authority.frag_lemma(fragment);
            proof_assert!(fragment@.incl_eq(self.authority@));
            proof_assert!(self.authority@ == Some(Excl((self.resource_id, Some(*stamp)))));
            proof_assert!(self.is_active(*stamp));
        };
    }

    /// Tie an idle capability borrowed from the context to this exact
    /// authority. The caller cannot infer readiness from the authority view
    /// or a copied key alone.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(cap.invariant())]
    #[requires(cap.key().resource_id() == self.resource_id())]
    #[requires(cap.fragment_id() == self.resource_id())]
    #[requires(cap.fragment_value() == Some(Excl((self.resource_id(), None))))]
    #[ensures(self.is_idle())]
    fn require_idle_cap(&self, cap: &ReadyCap<HeaderLeaseState>) {
        let _ = ghost! {
            let fragment = cap.fragment_ref().into_inner();
            self.authority.frag_lemma(fragment);
            proof_assert!(fragment@.incl_eq(self.authority@));
            proof_assert!(self.authority@ == Some(Excl((self.resource_id, None))));
            proof_assert!(self.is_idle());
        };
    }
}

impl Invariant for HeaderLeaseAuthority {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.authority.invariant() && self.authority.id() == self.resource_id
        }
    }
}

#[logic(open)]
fn same_stored_authority(left: StoredAuthority, right: StoredAuthority) -> bool {
    pearlite! { left == right }
}

impl Protocol for SharedPayload {
    type Public = (*mut (), NativeCounter, Id, Id, Id, AllocationBinding, ReadyKey);

    #[logic(open(self))]
    fn public(self) -> Self::Public {
        (
            self.header_id as *mut (),
            *self.counter_model,
            self.ledger_id,
            self.phase_id,
            self.lease.resource_id(),
            *self.allocation_binding,
            self.lease_key,
        )
    }

    #[logic(open(self), prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
        self.invariant() &&
        self.phase.invariant() && self.phase.resource_id() == self.phase_id &&
        self.lease.invariant() && self.lease.header_id == self.header_id &&
        self.lease_key.resource_id() == self.lease.resource_id() &&
        self.lease_key.slot() >= 0 && self.lease_generation >= 0 &&
        match (self.root, self.storage, self.pool, self.ledger,
            self.retire, self.terminal_action) {
            (Some(root), Some(storage), Some(pool), Some(ledger), None, Some(action)) => {
                let header = root.val();
                !self.closed && self.phase.is_open() &&
                self.lease.is_idle() &&
                root.invariant() && storage.invariant() && pool.invariant() &&
                ledger.invariant() && ledger.is_live() && ledger.count() > 0 &&
                ledger.ledger_id() == self.ledger_id &&
                action.invariant() &&
                action.resource_id() == self.phase_id &&
                *root.ward() == self.header_id as *const Shared &&
                header.ref_count.counter_model() == *self.counter_model &&
                header.vec@ != None &&
                header.vec_model() == *self.header_vec &&
                header.original_capacity_repr == *self.original_capacity_repr &&
                !header.ref_count.has_event_invariant() &&
                ledger.allocation_binding() == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                *self.allocation_binding == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                pool.capacity() == storage.capacity() &&
                pool.namespace() == storage.binding() &&
                pool.resource_id() == storage.binding() &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < storage.capacity() &&
                    !active_mutable_contains(ledger.active_ids(), index)
                )) &&
                header.vec@ != None && header.vec_model().matches_authority(storage)
            },
            (Some(root), Some(storage), Some(pool), Some(ledger), Some(retire), None) => {
                let header = root.val();
                !self.closed && self.phase.is_open() &&
                self.lease.is_idle() &&
                root.invariant() && storage.invariant() && pool.invariant() &&
                ledger.invariant() && !ledger.is_live() && ledger.count() == 0 &&
                retire.invariant() && retire.ledger_id() == self.ledger_id &&
                retire.active_ids().is_empty() &&
                ledger.ledger_id() == self.ledger_id &&
                ledger.allocation_binding() == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                *self.allocation_binding == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                *root.ward() == self.header_id as *const Shared &&
                header.ref_count.counter_model() == *self.counter_model &&
                header.vec@ != None &&
                header.vec_model() == *self.header_vec &&
                header.original_capacity_repr == *self.original_capacity_repr &&
                !header.ref_count.has_event_invariant() &&
                pool.capacity() == storage.capacity() &&
                pool.namespace() == storage.binding() &&
                pool.resource_id() == storage.binding() &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < storage.capacity()
                )) &&
                header.vec@ != None && header.vec_model().matches_authority(storage)
            },
            (None, Some(storage), Some(pool), Some(ledger), None, Some(action)) => {
                !self.closed && self.phase.is_open() &&
                self.lease.is_active(header_lease_stamp(
                    self.lease_generation,
                    self.header_id,
                    *self.counter_model,
                    self.ledger_id,
                    self.phase_id,
                    storage,
                    *self.header_vec,
                    *self.original_capacity_repr,
                    self.lease_key,
                )) &&
                storage.invariant() && pool.invariant() &&
                ledger.invariant() && ledger.is_live() && ledger.count() > 0 &&
                ledger.ledger_id() == self.ledger_id &&
                action.invariant() && action.resource_id() == self.phase_id &&
                ledger.allocation_binding() == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                *self.allocation_binding == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                self.header_vec.matches_authority(storage) &&
                pool.capacity() == storage.capacity() &&
                pool.namespace() == storage.binding() &&
                pool.resource_id() == storage.binding() &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < storage.capacity() &&
                    !active_mutable_contains(ledger.active_ids(), index)
                ))
            },
            (None, Some(storage), Some(pool), Some(ledger), Some(retire), None) => {
                !self.closed && self.phase.is_open() &&
                self.lease.is_active(header_lease_stamp(
                    self.lease_generation,
                    self.header_id,
                    *self.counter_model,
                    self.ledger_id,
                    self.phase_id,
                    storage,
                    *self.header_vec,
                    *self.original_capacity_repr,
                    self.lease_key,
                )) &&
                storage.invariant() && pool.invariant() &&
                ledger.invariant() && !ledger.is_live() && ledger.count() == 0 &&
                retire.invariant() && retire.ledger_id() == self.ledger_id &&
                retire.active_ids().is_empty() && ledger.ledger_id() == self.ledger_id &&
                ledger.allocation_binding() == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                *self.allocation_binding == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                self.header_vec.matches_authority(storage) &&
                pool.capacity() == storage.capacity() &&
                pool.namespace() == storage.binding() &&
                pool.resource_id() == storage.binding() &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < storage.capacity()
                ))
            },
            (None, None, None, None, Some(retire), None) =>
                self.closed && self.phase.is_closed() && self.lease.is_idle() && retire.invariant() &&
                retire.ledger_id() == self.ledger_id && retire.active_ids().is_empty(),
            _ => false,
        }
        }
    }
}

impl CounterProtocol for SharedPayload {
    #[logic(open(self), prophetic)]
    fn counter_model(self) -> NativeCounter {
        *self.counter_model
    }

    #[logic(open(self), prophetic)]
    fn agrees_with_value(self, value: usize) -> bool {
        pearlite! {
            match (self.ledger, self.retire) {
                (Some(ledger), None) => ledger.invariant() && ledger.agrees_with_value(value@),
                (Some(ledger), Some(retire)) => ledger.invariant() && !ledger.is_live() &&
                    ledger.count() == 0 && retire.invariant() &&
                    retire.ledger_id() == self.ledger_id && retire.active_ids().is_empty() &&
                    value@ == 0,
                (None, Some(retire)) => retire.invariant() &&
                    retire.ledger_id() == self.ledger_id && retire.active_ids().is_empty() &&
                    value@ == 0,
                _ => false,
            }
        }
    }
}

impl Invariant for SharedPayload {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        inv(self.root) && inv(self.storage) &&
        inv(self.pool) && inv(self.ledger) && inv(self.retire) &&
        self.phase.invariant() && inv(self.terminal_action) &&
        inv(self.header_vec) && inv(self.original_capacity_repr) &&
        self.lease.invariant()
    }
}

/// Affine, read-only access to a Shared header while its root permission is
/// temporarily outside the event payload. It carries the exact root and lease
/// capability; it cannot expose a mutable header borrow or physical authority.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedHeaderReadLoan {
    root: Box<Perm<*const Shared>>,
    header_id: *mut Shared,
    counter_model: Snapshot<NativeCounter>,
    header_vec: Snapshot<VecStorage>,
    original_capacity_repr: Snapshot<usize>,
    storage_model: Snapshot<StoredAuthority>,
    ledger_id: Id,
    phase_id: Id,
    generation: Int,
    lease_id: Id,
    lease_key: ReadyKey,
    lease_cap: ReadyCap<HeaderLeaseState>,
}

impl SharedHeaderReadLoan {
    #[logic(open(self), prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.root.invariant() &&
            *self.root.ward() == self.header_id as *const Shared &&
            self.root.val().ref_count.counter_model() == *self.counter_model &&
            !self.root.val().ref_count.has_event_invariant() &&
            self.root.val().vec@ != None &&
            self.root.val().vec_model() == *self.header_vec &&
            self.root.val().original_capacity_repr == *self.original_capacity_repr &&
            self.root.val().vec_model().matches_authority(*self.storage_model) &&
            self.generation > 0 &&
            self.lease_cap.invariant() &&
            self.lease_cap.key() == self.lease_key &&
            self.lease_key.resource_id() == self.lease_id &&
            self.lease_cap.fragment_id() == self.lease_id &&
            self.lease_cap.fragment_value() ==
                Some(Excl((self.lease_id, Some(header_lease_stamp(
                    self.generation,
                    self.header_id,
                    *self.counter_model,
                    self.ledger_id,
                    self.phase_id,
                    *self.storage_model,
                    *self.header_vec,
                    *self.original_capacity_repr,
                    self.lease_key,
                )))))
        }
    }
}

impl Invariant for SharedHeaderReadLoan {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

impl SharedPayload {
    /// Relate the payload's private lease identity to its public capsule view.
    /// This body-proved projection is used inside event callbacks, where the
    /// invariant open exposes only the public tuple to the caller.
    #[check(ghost)]
    #[requires(self.public().4 == *expected)]
    #[ensures(self.lease.resource_id() == *expected)]
    fn prove_public_lease_resource_id(&self, expected: Snapshot<Id>) {
        proof_assert!(self.lease.resource_id() == *expected);
    }

    #[check(ghost)]
    #[requires(self.public().6 == *expected)]
    #[ensures(self.lease_key == *expected)]
    fn prove_public_lease_key(&self, expected: Snapshot<ReadyKey>) {
        proof_assert!(self.lease_key == *expected);
    }

    /// Project the capsule's public phase identity back to the exact private
    /// phase identifier. Keep this fact in a small body proof so callers do
    /// not have to unfold the payload protocol while changing ledger state.
    #[check(ghost)]
    #[requires(self.public().3 == *expected)]
    #[ensures(self.phase_id == *expected)]
    fn prove_public_phase_id(&self, expected: Snapshot<Id>) {
        proof_assert!(self.phase_id == *expected);
    }

    /// A present root is available only in an idle-lease branch of the
    /// payload protocol. Keep the implication explicit for quantified event
    /// callbacks.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol() && self.root != None)]
    #[ensures(self.lease.is_idle())]
    fn prove_root_some_implies_lease_idle(&self) {
        proof_assert!(self.lease.is_idle());
    }

    /// Reconcile the packet's real owner fragment with the current payload.
    /// Every retired or terminal branch contradicts this live ticket, so a
    /// successful proof leaves the ledger live and the exact ticket active.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[ensures(self.ledger != None && self.ledger.unwrap_logic().is_live())]
    #[ensures(self.ledger.unwrap_logic().has_ticket(*ticket))]
    #[ensures(self.ledger.unwrap_logic().allocation_binding() ==
        ticket.allocation_binding())]
    #[ensures(self.retire == None)]
    fn prove_live_owner_ticket(&self, ticket: &OwnerTicket) {
        match (&self.ledger, &self.retire) {
            (Some(ledger), None) => ledger.prove_ticket_is_active(ticket),
            (Some(_), Some(retire)) => retire.contradicts_live_ticket(ticket),
            (None, Some(retire)) => retire.contradicts_live_ticket(ticket),
            (None, None) => proof_assert!(false),
        }
    }

    /// A real active owner ticket excludes retired and zero-owner states.
    /// Combined with the event's counter agreement, it binds the mathematical
    /// count to this native observation and proves the old count is nonzero.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(self.agrees_with_value(value))]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[ensures(self.ledger != None && self.retire == None)]
    #[ensures(self.ledger.unwrap_logic().count() == value@)]
    #[ensures(self.ledger.unwrap_logic().allocation_binding() ==
        ticket.allocation_binding())]
    #[ensures(value@ > 0)]
    fn prove_live_owner_matches_counter(&self, ticket: &OwnerTicket, value: usize) {
        self.prove_live_owner_ticket(ticket);
        proof_assert!(self.ledger.unwrap_logic().count() == value@);
        proof_assert!(value@ > 0);
    }

    /// Split one live Mutable ticket and its exact physical region after the
    /// native relaxed-CAS increment has succeeded. The caller supplies the
    /// actual ticket and region held by its packet; the payload's ledger and
    /// pool remain the authorities for reference count and free-space
    /// coverage.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().agrees_with_value(old))]
    #[requires(old@ <= crate::ref_count_limit::MAX_REF_COUNT@)]
    #[requires(new@ == old@ + 1)]
    #[requires(ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id)]
    #[requires(ticket.inner_logic().class() == RefKind::Mutable)]
    #[requires(region.inner_logic().invariant())]
    #[requires(ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
        ticket.inner_logic().owned_hi() == region.inner_logic().hi())]
    #[requires(ticket.inner_logic().allocation_binding().0 == region.inner_logic().namespace())]
    #[requires(ticket.inner_logic().allocation_binding().2 == region.inner_logic().capacity())]
    #[requires(ticket.inner_logic().owned_lo() <= *cut &&
        *cut <= ticket.inner_logic().owned_hi())]
    #[ensures((^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol())]
    #[ensures((^payload.inner_logic()).agrees_with_value(new))]
    #[ensures((^payload.inner_logic()).public() == payload.inner_logic().public())]
    #[ensures((^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model())]
    #[ensures((^payload.inner_logic()).ledger_id == payload.inner_logic().ledger_id)]
    #[ensures((^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding)]
    #[ensures((^payload.inner_logic()).storage == payload.inner_logic().storage)]
    #[ensures((^payload.inner_logic()).pool == payload.inner_logic().pool)]
    #[ensures((^payload.inner_logic()).root == payload.inner_logic().root)]
    #[ensures((^payload.inner_logic()).phase == payload.inner_logic().phase)]
    #[ensures((^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action)]
    #[ensures((^payload.inner_logic()).lease_key == payload.inner_logic().lease_key)]
    #[ensures((^payload.inner_logic()).lease.invariant())]
    #[ensures((^payload.inner_logic()).lease.resource_id() ==
        payload.inner_logic().lease.resource_id())]
    #[ensures((^payload.inner_logic()).lease.header_id() ==
        payload.inner_logic().lease.header_id())]
    #[ensures((^payload.inner_logic()).lease.authority@ ==
        payload.inner_logic().lease.authority@)]
    #[ensures((^payload.inner_logic()).lease_generation ==
        payload.inner_logic().lease_generation)]
    #[ensures((^payload.inner_logic()).ledger.unwrap_logic().count() == new@)]
    #[ensures((^payload.inner_logic()).ledger.unwrap_logic().has_ticket(
        result.inner_logic().0))]
    #[ensures((^payload.inner_logic()).ledger.unwrap_logic().has_ticket(
        result.inner_logic().1))]
    #[ensures(forall<index: Int>
        active_mutable_contains((^payload.inner_logic()).ledger.unwrap_logic().active_ids(), index) ==
            active_mutable_contains(payload.inner_logic().ledger.unwrap_logic().active_ids(), index))]
    #[ensures(*result.inner_logic().4 == ticket.inner_logic())]
    #[ensures(*result.inner_logic().5 == region.inner_logic())]
    #[ensures(*result.inner_logic().6 == *cut)]
    #[ensures(mutable_owner_split_result(
        result.inner_logic().4, result.inner_logic().5,
        result.inner_logic().6,
        (result.inner_logic().0, result.inner_logic().1,
            result.inner_logic().2, result.inner_logic().3)
    ))]
    fn split_mutable_owner_after_increment(
        payload: Ghost<&mut SharedPayload>,
        ticket: Ghost<OwnerTicket>,
        region: Ghost<PhysicalRegion>,
        cut: Snapshot<Int>,
        old: usize,
        new: usize,
    ) -> Ghost<(
        OwnerTicket, OwnerTicket, PhysicalRegion, PhysicalRegion,
        Snapshot<OwnerTicket>, Snapshot<PhysicalRegion>, Snapshot<Int>,
    )> {
        ghost! {
            let payload = payload.into_inner();
            let mut ticket = ticket.into_inner();
            let region = region.into_inner();
            let cut_value = cut.into_ghost().into_inner();
            let ticket_snapshot: Snapshot<OwnerTicket> = snapshot!(ticket);
            let region_snapshot: Snapshot<PhysicalRegion> = snapshot!(region);
            let cut_snapshot: Snapshot<Int> = snapshot!(cut_value);

            payload.prove_live_owner_matches_counter(&ticket, old);
            let before_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let mut ledger = payload.ledger.take().unwrap();
            let child = OwnerLedger::split_owner(
                Ghost::new(&mut ledger), Ghost::new(&mut ticket), snapshot!(cut_value),
            ).into_inner();
            payload.ledger = Some(ledger);

            let (left_region, right_region) = region.split_at(cut_value);
            proof_assert!(new@ == old@ + 1);
            proof_assert!(payload.invariant() && payload.protocol());
            proof_assert!(payload.agrees_with_value(new));
            proof_assert!(payload.ledger.unwrap_logic().count() == new@);
            let after_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            proof_assert!(forall<index: Int>
                active_mutable_contains(*after_owners, index) ==
                    active_mutable_contains(*before_owners, index));
            (ticket, child, left_region, right_region,
                ticket_snapshot, region_snapshot, cut_snapshot)
        }
    }

    /// Freeze one actual Mutable owner without changing the native strong
    /// count. The exact ticket is reclassified to its visible Readonly span,
    /// while the same affine physical region is consumed into the pool. The
    /// pool/ledger complement and all shared-header metadata are preserved by
    /// this source-side transition; no authority is reconstructed from the
    /// snapshots used to state its frames.
    #[check(ghost)]
    #[requires(
        payload.inner_logic().invariant() && payload.inner_logic().protocol() &&
        ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id &&
        ticket.inner_logic().class() == RefKind::Mutable &&
        ticket.inner_logic().allocation_binding() == *payload.inner_logic().allocation_binding &&
        region.inner_logic().invariant() &&
        ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
        ticket.inner_logic().owned_hi() == region.inner_logic().hi() &&
        ticket.inner_logic().allocation_binding().0 == region.inner_logic().namespace() &&
        ticket.inner_logic().allocation_binding().2 == region.inner_logic().capacity() &&
        ticket.inner_logic().owned_lo() <= *visible_lo &&
        *visible_lo <= *visible_hi && *visible_hi <= ticket.inner_logic().owned_hi() &&
        payload.inner_logic().ledger != None &&
        payload.inner_logic().ledger.unwrap_logic().has_ticket(ticket.inner_logic()) &&
        payload.inner_logic().pool != None
    )]
    #[ensures(
        (^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).public() == payload.inner_logic().public() &&
        (^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model() &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).root == payload.inner_logic().root &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).retire == payload.inner_logic().retire &&
        (^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action &&
        (^payload.inner_logic()).lease == payload.inner_logic().lease &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed
    )]
    #[ensures(
        (^payload.inner_logic()).ledger.unwrap_logic().count() ==
            payload.inner_logic().ledger.unwrap_logic().count() &&
        (^payload.inner_logic()).ledger.unwrap_logic().next_id() ==
            payload.inner_logic().ledger.unwrap_logic().next_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().ledger_id() ==
            payload.inner_logic().ledger.unwrap_logic().ledger_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().allocation_binding() ==
            payload.inner_logic().ledger.unwrap_logic().allocation_binding() &&
        (^payload.inner_logic()).ledger.unwrap_logic().active_ids() ==
            payload.inner_logic().ledger.unwrap_logic().active_ids().insert(
                ticket.inner_logic().owner_id(), Excl((
                    ticket.inner_logic().allocation_binding(), RefKind::Readonly,
                    *visible_lo, *visible_hi,
                ))) &&
        (^payload.inner_logic()).ledger.unwrap_logic().has_ticket(result.inner_logic())
    )]
    #[ensures(
        result.inner_logic().invariant() &&
        result.inner_logic().ledger_id() == ticket.inner_logic().ledger_id() &&
        result.inner_logic().owner_id() == ticket.inner_logic().owner_id() &&
        result.inner_logic().fragment_resource_id() == ticket.inner_logic().fragment_resource_id() &&
        result.inner_logic().class() == RefKind::Readonly &&
        result.inner_logic().allocation_binding() == ticket.inner_logic().allocation_binding() &&
        result.inner_logic().owned_lo() == *visible_lo &&
        result.inner_logic().owned_hi() == *visible_hi
    )]
    #[ensures(
        (^payload.inner_logic()).pool.unwrap_logic().capacity() ==
            payload.inner_logic().pool.unwrap_logic().capacity() &&
        (^payload.inner_logic()).pool.unwrap_logic().namespace() ==
            payload.inner_logic().pool.unwrap_logic().namespace() &&
        (^payload.inner_logic()).pool.unwrap_logic().resource_id() ==
            payload.inner_logic().pool.unwrap_logic().resource_id() &&
        (forall<index: Int>
            (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
                (payload.inner_logic().pool.unwrap_logic().contains(index) ||
                    region.inner_logic().lo() <= index && index < region.inner_logic().hi())) &&
        (forall<index: Int>
            (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
                (if payload.inner_logic().pool.unwrap_logic().contains(index) {
                    payload.inner_logic().pool.unwrap_logic().slot(index)
                } else if region.inner_logic().lo() <= index && index < region.inner_logic().hi() {
                    region.inner_logic().slot(index)
                } else {
                    None
                })) &&
        (forall<index: Int>
            (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
                (0 <= index && index < (^payload.inner_logic()).pool.unwrap_logic().capacity() &&
                    !active_mutable_contains(
                        (^payload.inner_logic()).ledger.unwrap_logic().active_ids(), index)))
    )]
    fn freeze_mutable_owner(
        payload: Ghost<&mut SharedPayload>,
        ticket: Ghost<OwnerTicket>,
        region: Ghost<PhysicalRegion>,
        visible_lo: Snapshot<Int>,
        visible_hi: Snapshot<Int>,
    ) -> Ghost<OwnerTicket> {
        ghost! {
            let payload = payload.into_inner();
            let mut ticket = ticket.into_inner();
            let region = region.into_inner();
            payload.prove_live_owner_ticket(&ticket);
            payload.prove_owner_region_excluded_from_pool(&ticket, &region);

            let ticket_before: Snapshot<OwnerTicket> = snapshot!(ticket);
            let region_before: Snapshot<PhysicalRegion> = snapshot!(region);
            let before_owners: Snapshot<ActiveReferences> = snapshot!(
                payload.ledger.unwrap_logic().active_ids()
            );
            let before_pool: Snapshot<PhysicalPool> = snapshot!(
                payload.pool.unwrap_logic()
            );
            let new_binding = snapshot!((
                ticket.allocation_binding(), RefKind::Readonly, *visible_lo, *visible_hi,
            ));
            match &payload.ledger {
                Some(ledger) => ledger.prove_readonly_binding_compatible(
                    &ticket, visible_lo, visible_hi,
                ),
                None => proof_assert!(false),
            }

            let mut ledger = payload.ledger.take().unwrap();
            OwnerLedger::reclassify(
                Ghost::new(&mut ledger),
                Ghost::new(&mut ticket),
                new_binding,
            );
            payload.ledger = Some(ledger);
            let after_owners: Snapshot<ActiveReferences> = snapshot!(
                payload.ledger.unwrap_logic().active_ids()
            );
            prove_owner_reclassification_effect(
                before_owners,
                after_owners,
                ticket_before,
                visible_lo,
                visible_hi,
            );

            let mut pool = payload.pool.take().unwrap();
            pool = pool.retire(region);
            payload.pool = Some(pool);
            let after_pool: Snapshot<PhysicalPool> = snapshot!(
                payload.pool.unwrap_logic()
            );
            SharedPayload::prove_pool_complement_after_mutable_release(
                before_pool,
                after_pool,
                before_owners,
                after_owners,
                region_before,
            );
            proof_assert!(payload.invariant() && payload.protocol());
            ticket
        }
    }

    /// Establish disjointness before consuming the ticket. The pool excludes
    /// every live Mutable span, and this actual ticket names the exact region
    /// that will be returned to that pool.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[requires(ticket.class() == RefKind::Mutable)]
    #[requires(region.invariant())]
    #[requires(ticket.owned_lo() == region.lo() && ticket.owned_hi() == region.hi())]
    #[requires(region.namespace() == self.ledger.unwrap_logic().allocation_binding().0)]
    #[requires(region.capacity() == self.ledger.unwrap_logic().allocation_binding().2)]
    #[ensures(self.pool.unwrap_logic().namespace() == region.namespace())]
    #[ensures(self.pool.unwrap_logic().capacity() == region.capacity())]
    #[ensures(self.pool.unwrap_logic().resource_id() == region.resource_id())]
    #[ensures(forall<index: Int>
        self.pool.unwrap_logic().contains(index) ==>
            !(region.lo() <= index && index < region.hi()))]
    fn prove_owner_region_excluded_from_pool(
        &self,
        ticket: &OwnerTicket,
        region: &PhysicalRegion,
    ) {
        self.prove_live_owner_ticket(ticket);
        let ledger = match &self.ledger {
            Some(ledger) => ledger,
            None => {
                proof_assert!(false);
                return;
            }
        };
        ledger.prove_mutable_ticket_covers_interval(ticket);
        proof_assert!(forall<index: Int>
            self.pool.unwrap_logic().contains(index) ==>
                !active_mutable_contains(
                    self.ledger.unwrap_logic().active_ids(), index));
        proof_assert!(forall<index: Int>
            self.pool.unwrap_logic().contains(index) ==>
                !(region.lo() <= index && index < region.hi()));
    }

    /// Re-establish the exact pool/owner complement after one Mutable owner
    /// returns its actual region. This isolates the set algebra from the
    /// surrounding payload protocol and count transition.
    #[check(ghost)]
    #[requires(before_pool.invariant() && after_pool.invariant())]
    #[requires((*before_pool).capacity() == (*after_pool).capacity())]
    #[requires(region.invariant())]
    #[requires(region.lo() >= 0 && region.lo() <= region.hi() &&
        region.hi() <= (*before_pool).capacity())]
    #[requires(region.capacity() == (*before_pool).capacity())]
    #[requires(forall<index: Int>
        (*before_pool).contains(index) ==
            (0 <= index && index < (*before_pool).capacity() &&
                !active_mutable_contains(*before_owners, index)))]
    #[requires(forall<index: Int>
        (*after_pool).contains(index) ==
            ((*before_pool).contains(index) ||
                region.lo() <= index && index < region.hi()))]
    #[requires(forall<index: Int>
        active_mutable_contains(*after_owners, index) ==
            (active_mutable_contains(*before_owners, index) &&
                !(region.lo() <= index && index < region.hi())))]
    #[ensures(forall<index: Int>
        (*after_pool).contains(index) ==
            (0 <= index && index < (*after_pool).capacity() &&
                !active_mutable_contains(*after_owners, index)))]
    fn prove_pool_complement_after_mutable_release(
        before_pool: Snapshot<PhysicalPool>,
        after_pool: Snapshot<PhysicalPool>,
        before_owners: Snapshot<ActiveReferences>,
        after_owners: Snapshot<ActiveReferences>,
        region: Snapshot<PhysicalRegion>,
    ) {
        proof_assert!(forall<index: Int>
            (*after_pool).contains(index) ==
                (0 <= index && index < (*after_pool).capacity() &&
                    !active_mutable_contains(*after_owners, index)));
    }

    /// A final-owner ledger has no remaining Mutable spans, so the actual
    /// physical pool is the entire allocation range. Separate this finite-map
    /// fact from the lifecycle protocol conjunction used by the caller.
    #[check(ghost)]
    #[requires(pool.invariant())]
    #[requires((*owners).len() == 0)]
    #[requires(forall<index: Int>
        (*pool).contains(index) ==
            (0 <= index && index < (*pool).capacity() &&
                !active_mutable_contains(*owners, index)))]
    #[ensures(forall<index: Int>
        0 <= index && index < (*pool).capacity() ==>
            (*pool).contains(index))]
    fn prove_full_pool_after_final_owner_release(
        pool: Snapshot<PhysicalPool>,
        owners: Snapshot<ActiveReferences>,
    ) {
        prove_no_active_mutable_on_empty(owners);
        proof_assert!(forall<index: Int>
            0 <= index && index < (*pool).capacity() ==>
                (*pool).contains(index));
    }

    /// Derive the event-counter agreement from the exact ledger decrement and
    /// the final-owner permit branch. The capsule callback can then reuse this
    /// small arithmetic proof without unfolding the whole payload protocol.
    #[check(ghost)]
    #[requires(self.ledger != None)]
    #[requires(self.ledger.unwrap_logic().invariant())]
    #[requires(self.ledger.unwrap_logic().count() + 1 == old@)]
    #[requires(new@ + 1 == old@)]
    #[requires(match self.retire {
        Some(retire) => old@ == 1 &&
            !self.ledger.unwrap_logic().is_live() &&
            self.ledger.unwrap_logic().count() == 0 &&
            retire.invariant() && retire.ledger_id() == self.ledger_id &&
            retire.active_ids().is_empty(),
        None => old@ > 1 && self.ledger.unwrap_logic().is_live(),
    })]
    #[ensures(self.agrees_with_value(*new))]
    fn prove_counter_agreement_after_owner_release(
        &self,
        old: Snapshot<usize>,
        new: Snapshot<usize>,
    ) {
        proof_assert!(self.agrees_with_value(*new));
    }

    /// Apply the ledger and physical-pool half of one actual native owner
    /// decrement. The caller supplies the ticket and region moved out of that
    /// owner's packet; neither a Snapshot of the ledger nor a reconstructed
    /// interval can stand in for these affine resources.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().counter_model() == *owner_model)]
    #[requires(payload.inner_logic().agrees_with_value(*old))]
    #[requires(old@ > 0 && new@ + 1 == old@)]
    #[requires(payload.inner_logic().public().3 == *expected_phase)]
    #[requires(payload.inner_logic().ledger != None &&
        payload.inner_logic().ledger.unwrap_logic().count() == old@)]
    #[requires(ticket.inner_logic().invariant())]
    #[requires(ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id)]
    #[requires(region.inner_logic().invariant())]
    #[requires(ticket.inner_logic().class() == RefKind::Mutable)]
    #[requires(ticket.inner_logic().owned_lo() == region.inner_logic().lo())]
    #[requires(ticket.inner_logic().owned_hi() == region.inner_logic().hi())]
    #[requires(payload.inner_logic().pool != None)]
    #[requires(region.inner_logic().namespace() ==
        payload.inner_logic().ledger.unwrap_logic().allocation_binding().0)]
    #[requires(region.inner_logic().capacity() ==
        payload.inner_logic().ledger.unwrap_logic().allocation_binding().2)]
    #[ensures((^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol())]
    #[ensures((^payload.inner_logic()).public() == payload.inner_logic().public())]
    #[ensures((^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model())]
    #[ensures((^payload.inner_logic()).agrees_with_value(*new))]
    #[ensures((^payload.inner_logic()).lease_key == payload.inner_logic().lease_key)]
    #[ensures((^payload.inner_logic()).lease.resource_id() ==
        payload.inner_logic().lease.resource_id())]
    #[ensures((^payload.inner_logic()).ledger.unwrap_logic().count() + 1 ==
        payload.inner_logic().ledger.unwrap_logic().count())]
    #[ensures((^payload.inner_logic()).ledger.unwrap_logic().active_ids() ==
        payload.inner_logic().ledger.unwrap_logic().active_ids().remove(
            ticket.inner_logic().owner_id()))]
    #[ensures(match (^payload.inner_logic()).retire {
        Some(retire) => old@ == 1 && retire.invariant() &&
            !(^payload.inner_logic()).ledger.unwrap_logic().is_live() &&
            (^payload.inner_logic()).ledger.unwrap_logic().count() == 0 &&
            (^payload.inner_logic()).terminal_action == None &&
            (forall<index: Int> 0 <= index && index <
                (^payload.inner_logic()).pool.unwrap_logic().capacity() ==>
                (^payload.inner_logic()).pool.unwrap_logic().contains(index)),
        None => old@ > 1 && (^payload.inner_logic()).retire == None &&
            (^payload.inner_logic()).ledger.unwrap_logic().is_live() &&
            (^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action,
    })]
    #[ensures(match result.inner_logic() {
        Some(action) => old@ == 1 && action.invariant() &&
            action.resource_id() == payload.inner_logic().phase_id &&
            action.resource_id() == *expected_phase &&
            (^payload.inner_logic()).terminal_action == None,
        None => old@ > 1 &&
            (^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action,
    })]
    fn drop_packet_owner(
        payload: Ghost<&mut SharedPayload>,
        owner_model: Snapshot<NativeCounter>,
        old: Snapshot<usize>,
        new: Snapshot<usize>,
        ticket: Ghost<OwnerTicket>,
        region: Ghost<PhysicalRegion>,
        expected_phase: Snapshot<Id>,
    ) -> Ghost<Option<TerminalAction>> {
        ghost! {
            let payload = payload.into_inner();
            let ticket = ticket.into_inner();
            let region = region.into_inner();
            payload.prove_public_phase_id(expected_phase);
            payload.prove_owner_region_excluded_from_pool(&ticket, &region);
            let before_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let before_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            let region_snapshot: Snapshot<PhysicalRegion> = snapshot!(region);
            let previous_action: Snapshot<Option<TerminalAction>> =
                snapshot!(payload.terminal_action);
            let ledger = payload.ledger.as_mut().unwrap();
            let retire = OwnerLedger::drop_owner(
                Ghost::new(ledger), Ghost::new(ticket),
            ).into_inner();
            let pool = payload.pool.take().unwrap();
            payload.pool = Some(pool.retire(region));
            let after_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let after_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            SharedPayload::prove_pool_complement_after_mutable_release(
                before_pool,
                after_pool,
                before_owners,
                after_owners,
                region_snapshot,
            );
            let terminal_action = match retire {
                Some(retire) => {
                    SharedPayload::prove_full_pool_after_final_owner_release(
                        after_pool,
                        after_owners,
                    );
                    payload.retire = Some(retire);
                    payload.terminal_action.take()
                }
                None => {
                    payload.retire = None;
                    None
                }
            };
            proof_assert!(payload.invariant());
            proof_assert!(payload.counter_model() == *owner_model);
            proof_assert!(payload.ledger.unwrap_logic().count() + 1 == old@);
            proof_assert!(match payload.retire {
                Some(retire) => old@ == 1 && retire.invariant() &&
                    !payload.ledger.unwrap_logic().is_live() &&
                    payload.ledger.unwrap_logic().count() == 0 &&
                    (forall<index: Int> 0 <= index && index <
                        payload.pool.unwrap_logic().capacity() ==>
                        payload.pool.unwrap_logic().contains(index)),
                None => old@ > 1 && payload.ledger.unwrap_logic().is_live(),
            });
            payload.prove_counter_agreement_after_owner_release(old, new);
            proof_assert!(payload.protocol());
            proof_assert!(match terminal_action {
                Some(action) => old@ == 1 && action.invariant() &&
                    action.resource_id() == payload.phase_id &&
                    action.resource_id() == *expected_phase &&
                    payload.terminal_action == None,
                None => old@ > 1 && payload.terminal_action == *previous_action,
            });
            terminal_action
        }
    }

    /// A checked idle cap and this packet's actual owner ticket exclude both
    /// active and terminal states: active leases conflict with the exclusive
    /// cap, while terminal retirement conflicts with the live ticket.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(cap.invariant() && cap.key() == self.lease_key)]
    #[requires(cap.fragment_id() == self.lease.resource_id())]
    #[requires(cap.fragment_value() == Some(Excl((self.lease.resource_id(), None))))]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[ensures(self.root != None && self.lease.is_idle())]
    fn prove_idle_cap_implies_root_some(
        &self,
        cap: &ReadyCap<HeaderLeaseState>,
        ticket: &OwnerTicket,
    ) {
        self.lease.require_idle_cap(cap);
        self.prove_live_owner_ticket(ticket);
        match (&self.root, &self.ledger, &self.storage) {
            (None, Some(_), Some(storage)) => {
                proof_assert!(self.lease.is_idle() && self.lease.is_active(
                    header_lease_stamp(
                        self.lease_generation,
                        self.header_id,
                        *self.counter_model,
                        self.ledger_id,
                        self.phase_id,
                        *storage,
                        *self.header_vec,
                        *self.original_capacity_repr,
                        self.lease_key,
                    )
                ));
                proof_assert!(false);
            }
            (None, _, _) => proof_assert!(false),
            (Some(_), _, _) => (),
        }
        proof_assert!(self.root != None);
    }

    /// Reconcile the current invariant state with the caller's affine lease
    /// fragment. The fragment forces the current generation and complete
    /// header/storage frame; a previously returned Snapshot alone would not.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(loan.inner_logic().invariant())]
    #[requires(payload.inner_logic().lease.resource_id() == loan.inner_logic().lease_id)]
    #[ensures(payload.inner_logic().root == None &&
        payload.inner_logic().lease_generation == loan.inner_logic().generation &&
        payload.inner_logic().header_id == loan.inner_logic().header_id &&
        payload.inner_logic().counter_model == loan.inner_logic().counter_model &&
        *payload.inner_logic().allocation_binding == (
            (*loan.inner_logic().storage_model).binding(),
            (*loan.inner_logic().storage_model).raw_pointer(),
            (*loan.inner_logic().storage_model).capacity()) &&
        payload.inner_logic().header_vec == loan.inner_logic().header_vec &&
        payload.inner_logic().original_capacity_repr == loan.inner_logic().original_capacity_repr &&
        payload.inner_logic().ledger_id == loan.inner_logic().ledger_id &&
        payload.inner_logic().phase_id == loan.inner_logic().phase_id &&
        loan.inner_logic().lease_cap.fragment_id() ==
            payload.inner_logic().lease.resource_id() &&
        loan.inner_logic().lease_key == payload.inner_logic().lease_key &&
        payload.inner_logic().storage != None &&
        same_stored_authority(payload.inner_logic().storage.unwrap_logic(),
            *loan.inner_logic().storage_model) &&
            payload.inner_logic().lease.is_active(header_lease_stamp(
            loan.inner_logic().generation,
            loan.inner_logic().header_id,
            *loan.inner_logic().counter_model,
            loan.inner_logic().ledger_id,
            loan.inner_logic().phase_id,
            *loan.inner_logic().storage_model,
            *loan.inner_logic().header_vec,
            *loan.inner_logic().original_capacity_repr,
            loan.inner_logic().lease_key,
        )))]
    fn prove_current_header_lease(
        payload: Ghost<&SharedPayload>,
        loan: Ghost<&SharedHeaderReadLoan>,
    ) -> Ghost<()> {
        ghost! {
            let payload = payload.into_inner();
            let loan = loan.into_inner();
            let loan_stamp = snapshot!(header_lease_stamp(
                loan.generation,
                loan.header_id,
                *loan.counter_model,
                loan.ledger_id,
                loan.phase_id,
                *loan.storage_model,
                *loan.header_vec,
                *loan.original_capacity_repr,
                loan.lease_key,
            ));
            payload.lease.require_active_fragment(loan_stamp, &loan.lease_cap);
            proof_assert!(payload.root == None);
            proof_assert!(payload.storage != None);
            let storage: Snapshot<StoredAuthority> =
                snapshot!(payload.storage.unwrap_logic());
            let current_stamp = snapshot!(header_lease_stamp(
                payload.lease_generation,
                payload.header_id,
                *payload.counter_model,
                payload.ledger_id,
                payload.phase_id,
                *storage,
                *payload.header_vec,
                *payload.original_capacity_repr,
                payload.lease_key,
            ));
            proof_assert!(payload.lease.is_active(*current_stamp));
            proof_assert!(*current_stamp == *loan_stamp);
            proof_assert!(payload.lease_generation == loan.generation);
            proof_assert!(payload.header_id == loan.header_id);
            proof_assert!(payload.counter_model == loan.counter_model);
            proof_assert!(payload.header_vec == loan.header_vec);
            proof_assert!(payload.original_capacity_repr == loan.original_capacity_repr);
            proof_assert!(payload.ledger_id == loan.ledger_id);
            proof_assert!(payload.phase_id == loan.phase_id);
            proof_assert!(same_stored_authority(*storage, *loan.storage_model));
        }
    }

    /// The actual final action and active read loan exclude every live-owner
    /// protocol state. A live state would still contain another affine close
    /// action for the same phase; retired states agree with only owner value
    /// zero. The active loan rules out the closed, idle-header protocol state.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().agrees_with_value(*value))]
    #[requires(payload.inner_logic().public().4 == loan.inner_logic().lease_id)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(action.inner_logic().invariant())]
    #[requires(action.inner_logic().resource_id() == payload.inner_logic().phase_id)]
    #[ensures(value@ == 0)]
    #[ensures((^action.inner_logic()).invariant())]
    #[ensures((^action.inner_logic()).resource_id() == action.inner_logic().resource_id())]
    fn prove_final_action_excludes_live_state(
        payload: Ghost<&SharedPayload>,
        loan: Ghost<&SharedHeaderReadLoan>,
        action: Ghost<&mut TerminalAction>,
        value: Snapshot<usize>,
    ) {
        ghost! {
            let payload = payload.into_inner();
            let loan = loan.into_inner();
            let action = action.into_inner();
            SharedPayload::prove_public_lease_resource_id(
                payload, snapshot!(loan.lease_id),
            );
            SharedPayload::prove_current_header_lease(
                Ghost::new(payload), Ghost::new(loan),
            );
            match (&payload.retire, &payload.terminal_action) {
                (None, Some(other_action)) => {
                    action.contradicts_other(other_action);
                }
                (Some(_retire), None) => {
                    proof_assert!(payload.ledger.unwrap_logic().count() == 0);
                    proof_assert!(value@ == 0);
                }
                _ => proof_assert!(false),
            }
        };
    }

    /// Preserve the payload's exact open phase authority relation separately
    /// from the larger callback protocol, so the terminal action remains
    /// bound to the authority after the payload's other fields are consumed.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(action.invariant() && action.resource_id() == self.phase_id)]
    #[ensures(self.phase.is_open())]
    #[ensures(self.phase.resource_id() == self.phase_id)]
    #[ensures(action.resource_id() == self.phase.resource_id())]
    fn prove_open_phase_matches_action(&self, action: &TerminalAction) {
        proof_assert!(self.phase.resource_id() == self.phase_id);
        proof_assert!(action.resource_id() == self.phase.resource_id());
        self.phase.require_action(action);
        proof_assert!(self.phase.is_open());
    }

    /// Move the unique header root into a sealed read loan. The payload keeps
    /// its storage and reference ledgers while recording exactly one active
    /// lease generation, so counter events can proceed without exposing a
    /// mutable header borrow.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().root != None && payload.inner_logic().lease.is_idle())]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key() == payload.inner_logic().lease_key)]
    #[requires(cap.inner_logic().fragment_id() == payload.inner_logic().lease.resource_id())]
    #[requires(cap.inner_logic().fragment_value() ==
        Some(Excl((payload.inner_logic().lease.resource_id(), None))))]
    #[ensures((^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).root == None &&
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation + 1 &&
        (^payload.inner_logic()).header_id == payload.inner_logic().header_id &&
        (^payload.inner_logic()).counter_model == payload.inner_logic().counter_model &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).ledger_id == payload.inner_logic().ledger_id &&
        (^payload.inner_logic()).phase_id == payload.inner_logic().phase_id &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).pool == payload.inner_logic().pool &&
        (^payload.inner_logic()).ledger == payload.inner_logic().ledger &&
        (^payload.inner_logic()).retire == payload.inner_logic().retire &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed &&
        result.inner_logic().invariant() &&
        result.inner_logic().generation == (^payload.inner_logic()).lease_generation &&
        result.inner_logic().header_id == payload.inner_logic().header_id &&
        result.inner_logic().counter_model == payload.inner_logic().counter_model &&
        (*result.inner_logic().storage_model).binding() ==
            (*payload.inner_logic().allocation_binding).0 &&
        (*result.inner_logic().storage_model).raw_pointer() ==
            (*payload.inner_logic().allocation_binding).1 &&
        (*result.inner_logic().storage_model).capacity() ==
            (*payload.inner_logic().allocation_binding).2 &&
        result.inner_logic().header_vec == payload.inner_logic().header_vec &&
        result.inner_logic().original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        result.inner_logic().ledger_id == payload.inner_logic().ledger_id &&
        result.inner_logic().phase_id == payload.inner_logic().phase_id &&
        result.inner_logic().lease_id == payload.inner_logic().lease.resource_id() &&
        result.inner_logic().lease_key == payload.inner_logic().lease_key &&
        same_stored_authority(*result.inner_logic().storage_model,
            payload.inner_logic().storage.unwrap_logic()))]
    pub(crate) fn take_header_read_loan(
        payload: Ghost<&mut SharedPayload>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
    ) -> Ghost<SharedHeaderReadLoan> {
        ghost! {
            let payload = payload.into_inner();
            let cap = cap.into_inner();
            payload.lease.require_idle_cap(&cap);
            let root = payload.root.take().unwrap();
            let storage_model: Snapshot<StoredAuthority> =
                snapshot!(payload.storage.unwrap_logic());
            let lease_id = payload.lease.resource_id;
            let generation = payload.lease_generation + 1int;
            let lease_key = cap.key_ghost().into_inner();
            let stamp = snapshot!(header_lease_stamp(
                generation,
                payload.header_id,
                *payload.counter_model,
                payload.ledger_id,
                payload.phase_id,
                *storage_model,
                *payload.header_vec,
                *payload.original_capacity_repr,
                lease_key,
            ));
            let lease_cap = HeaderLeaseAuthority::activate(
                Ghost::new(&mut payload.lease), stamp, Ghost::new(cap),
            ).into_inner();
            payload.lease_generation = generation;
            SharedHeaderReadLoan {
                root,
                header_id: payload.header_id,
                counter_model: payload.counter_model,
                header_vec: payload.header_vec,
                original_capacity_repr: payload.original_capacity_repr,
                storage_model,
                ledger_id: payload.ledger_id,
                phase_id: payload.phase_id,
                generation,
                lease_id,
                lease_key,
                lease_cap,
            }
        }
    }

    /// Consume the matching affine lease fragment and restore the exact root.
    /// The resource-algebra agreement makes the active stamp stable across
    /// intervening opens; counter events may update only the other payload
    /// resources while this lease is held.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(loan.inner_logic().invariant())]
    #[requires(payload.inner_logic().lease.resource_id() == loan.inner_logic().lease_id)]
    #[requires(payload.inner_logic().lease_key == loan.inner_logic().lease_key)]
    #[ensures((^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).root != None &&
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).header_id == payload.inner_logic().header_id &&
        (^payload.inner_logic()).counter_model == payload.inner_logic().counter_model &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).ledger_id == payload.inner_logic().ledger_id &&
        (^payload.inner_logic()).phase_id == payload.inner_logic().phase_id &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).pool == payload.inner_logic().pool &&
        (^payload.inner_logic()).ledger == payload.inner_logic().ledger &&
        (^payload.inner_logic()).retire == payload.inner_logic().retire &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.is_idle() &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed)]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == loan.inner_logic().lease_key)]
    #[ensures(result.inner_logic().fragment_id() == payload.inner_logic().lease.resource_id())]
    #[ensures(result.inner_logic().fragment_value() ==
        Some(Excl((payload.inner_logic().lease.resource_id(), None))))]
    pub(crate) fn restore_header_read_loan(
        payload: Ghost<&mut SharedPayload>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let payload = payload.into_inner();
            let loan = loan.into_inner();
            let stamp = snapshot!(header_lease_stamp(
                loan.generation,
                loan.header_id,
                *loan.counter_model,
                loan.ledger_id,
                loan.phase_id,
                *loan.storage_model,
                *loan.header_vec,
                *loan.original_capacity_repr,
                loan.lease_key,
            ));
            SharedPayload::prove_current_header_lease(Ghost::new(&*payload), Ghost::new(&loan));
            let SharedHeaderReadLoan { root, lease_cap, .. } = loan;
            let cap = HeaderLeaseAuthority::deactivate(
                Ghost::new(&mut payload.lease), stamp, Ghost::new(lease_cap),
            ).into_inner();
            payload.root = Some(root);
            cap
        }
    }
}

/// Read the exact NativeCounter through a live affine header loan. The
/// returned borrow is tied to the loan, and the only exposed header field is
/// the immutable counter cell.
#[requires(loan.inner_logic().invariant())]
#[requires(header_id == loan.inner_logic().header_id as *const Shared)]
#[ensures(result ==
    loan.inner_logic().root.val().ref_count.counter_reference())]
#[ensures(*result == *loan.inner_logic().counter_model)]
pub(crate) fn loaned_native_counter<'a>(
    header_id: *const Shared,
    loan: Ghost<&'a SharedHeaderReadLoan>,
) -> &'a NativeCounter {
    let root = ghost!(loan.root.as_ref());
    let header = unsafe { Perm::as_ref(header_id, root) };
    header.ref_count.native_counter()
}

/// A per-handle affine registration and physical view paired with the shared
/// capsule. The program pointers are B1-bound metadata, never reconstructed
/// from ghost raw-allocation descriptors.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedOwnerPacket {
    pub(crate) capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
    pub(crate) ticket: Ghost<OwnerTicket>,
    pub(crate) region: Ghost<PhysicalRegion>,
    pub(crate) base: BoundPtr,
    pub(crate) cursor: BoundPtr,
    lease_key: Ghost<ReadyKey>,
}

/// Proof-only input consumed by the source-level shared release body. The
/// packet carries this handle's actual owner ticket and region, while the
/// borrowed context remains in a Ghost parameter so none of its rights enter
/// the native representation.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedReleaseProof<'context, 'tokens> {
    pub(crate) packet: Ghost<SharedOwnerPacket>,
    pub(crate) context: Ghost<
        &'context mut BorrowedSharedContext<'tokens, HeaderLeaseState>,
    >,
}

/// Actual capabilities returned only after this packet wins the final
/// reference decrement and the capsule has been terminalized.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedRetirementResources {
    pub(crate) storage: StoredAuthority,
    pub(crate) region: PhysicalRegion,
    pub(crate) root: Box<Perm<*const Shared>>,
}

impl SharedOwnerPacket {
    /// Move the proof packet into Creusot's ghost heap. The native handle
    /// keeps only its ordinary pointer/length fields; the ticket, region,
    /// capsule, and registry key remain Ghost-owned proof state.
    #[ensures(result.inner_logic() == self)]
    #[ensures(result.inner_logic().invariant() == self.invariant())]
    pub(crate) fn into_ghost(self) -> Ghost<Self> {
        let SharedOwnerPacket {
            capsule,
            ticket,
            region,
            base,
            cursor,
            lease_key,
        } = self;
        ghost! {
            SharedOwnerPacket { capsule, ticket, region, base, cursor, lease_key }
        }
    }

    #[logic(open(self))]
    pub(crate) fn capsule_public(self) -> (*mut (), NativeCounter, Id, Id) {
        pearlite! {
            let payload = self.capsule.inner_logic().val().public().1;
            (payload.0, payload.1, payload.2, payload.3)
        }
    }

    /// Expose the packet's static event namespace through its invariant.
    /// This lets sibling modules derive the namespace from an actual packet
    /// instead of strengthening their callers with a redundant assumption.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(self.capsule.inner_logic().val().namespace() == event_namespace())]
    pub(crate) fn prove_event_namespace(&self) {
        proof_assert!(self.capsule.inner_logic().val().namespace() == event_namespace());
    }

    /// Export the packet invariant's cursor interval facts to callers outside
    /// this module. The packet keeps the actual ticket and region; this
    /// checked observer exposes only their existing geometry relation.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(self.region.inner_logic().invariant())]
    #[ensures(self.base.invariant() && self.cursor.invariant())]
    #[ensures(self.base@ == Some((
        self.region.inner_logic().namespace(),
        self.region.inner_logic().capacity(), 0int
    )))]
    #[ensures(self.cursor@ != None)]
    #[ensures(self.cursor@.unwrap_logic().0 ==
        self.region.inner_logic().namespace())]
    #[ensures(self.cursor@.unwrap_logic().1 ==
        self.region.inner_logic().capacity())]
    #[ensures(self.cursor.current_address() == self.base.current_address() +
        self.cursor@.unwrap_logic().2)]
    #[ensures(0 <= self.region.inner_logic().lo() &&
        self.region.inner_logic().lo() <= self.region.inner_logic().hi() &&
        self.region.inner_logic().hi() <= self.region.inner_logic().capacity() &&
        self.region.inner_logic().capacity() <= isize::MAX@)]
    #[ensures(self.region.inner_logic().lo() <=
        self.cursor@.unwrap_logic().2)]
    #[ensures(self.cursor@.unwrap_logic().2 <=
        self.region.inner_logic().hi())]
    pub(crate) fn prove_cursor_within_region(&self) {
        proof_assert!(self.region.inner_logic().lo() <=
            self.cursor@.unwrap_logic().2);
        proof_assert!(self.cursor@.unwrap_logic().2 <=
            self.region.inner_logic().hi());
    }

    #[logic(open(self))]
    pub(crate) fn capsule_allocation_binding(self) -> AllocationBinding {
        pearlite! { self.capsule.inner_logic().val().public().1.5 }
    }

    #[logic(open(self))]
    pub(crate) fn lease_resource_id(&self) -> Id {
        let payload = self.capsule.inner_logic().val().public().1;
        payload.4
    }

    #[logic(open(self))]
    pub(crate) fn lease_key(&self) -> ReadyKey {
        self.lease_key.inner_logic()
    }

    #[check(ghost)]
    #[ensures(result.inner_logic() == self.lease_key())]
    pub(crate) fn lease_key_ghost(&self) -> Ghost<ReadyKey> {
        self.lease_key
    }

    #[logic(open(self))]
    #[ensures(result == self.region.inner_logic().slot(index))]
    pub(crate) fn allocation_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! { self.region.inner_logic().slot(index) }
    }

    #[logic(open(self), prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.capsule.invariant() &&
            self.capsule.inner_logic().val().namespace() == event_namespace() &&
            self.capsule.inner_logic().val().public().0 ==
                self.capsule.inner_logic().val().public().1.1 &&
            self.ticket.inner_logic().invariant() &&
            self.ticket.inner_logic().ledger_id() == self.capsule_public().2 &&
            self.ticket.inner_logic().class() == RefKind::Mutable &&
            self.region.inner_logic().invariant() &&
            self.base.invariant() && self.cursor.invariant() &&
            self.ticket.inner_logic().allocation_binding() == (
                self.region.inner_logic().namespace(), self.base.raw_pointer(),
                self.region.inner_logic().capacity()) &&
            self.ticket.inner_logic().allocation_binding() ==
                self.capsule_allocation_binding() &&
            self.lease_key.inner_logic() == self.capsule.inner_logic().val().public().1.6 &&
            self.lease_key.inner_logic().resource_id() == self.lease_resource_id() &&
            self.lease_key.inner_logic().resource_id() == self.capsule.inner_logic().val().public().1.6.resource_id() &&
            self.lease_key.inner_logic().slot() == self.capsule.inner_logic().val().public().1.6.slot() &&
            self.ticket.inner_logic().owned_lo() == self.region.inner_logic().lo() &&
            self.ticket.inner_logic().owned_hi() == self.region.inner_logic().hi() &&
            self.base@ == Some((self.region.inner_logic().namespace(),
                self.region.inner_logic().capacity(), 0int)) &&
            self.cursor@ != None &&
            self.cursor.current_address() == self.base.current_address() +
                self.cursor@.unwrap_logic().2 &&
            self.cursor@.unwrap_logic().0 == self.region.inner_logic().namespace() &&
            self.cursor@.unwrap_logic().1 == self.region.inner_logic().capacity() &&
            0 <= self.region.inner_logic().lo() &&
            self.region.inner_logic().lo() <= self.region.inner_logic().hi() &&
            self.region.inner_logic().hi() <= self.region.inner_logic().capacity() &&
            self.region.inner_logic().capacity() <= isize::MAX@ &&
            self.cursor@.unwrap_logic().2 >= self.region.inner_logic().lo() &&
            self.cursor@.unwrap_logic().2 <= self.region.inner_logic().hi()
        }
    }
}

impl Invariant for SharedOwnerPacket {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

impl SharedOwnerPacket {
    /// Advance this handle's bound cursor while preserving the capsule, ticket,
    /// region, and base allocation descriptor. This moves only the packet's
    /// existing cursor; it does not split, clone, or rebind any authority.
    #[requires(self.invariant())]
    #[requires(count@ <= self.region.inner_logic().hi() - self.cursor@.unwrap_logic().2)]
    #[ensures(result.invariant())]
    #[ensures(result.capsule.inner_logic() == self.capsule.inner_logic())]
    #[ensures(result.capsule_public() == self.capsule_public())]
    #[ensures(result.ticket.inner_logic() == self.ticket.inner_logic())]
    #[ensures(result.region.inner_logic() == self.region.inner_logic())]
    #[ensures(result.lease_key() == self.lease_key())]
    #[ensures(forall<i:Int> 0 <= i && i < self.region.inner_logic().capacity() ==>
        result.allocation_slot(i) == self.allocation_slot(i))]
    #[ensures(result.base == self.base)]
    #[ensures(result.cursor@ == Some((
        self.cursor@.unwrap_logic().0,
        self.cursor@.unwrap_logic().1,
        self.cursor@.unwrap_logic().2 + count@
    )))]
    #[ensures(result.cursor.current_address() == self.cursor.current_address() + count@)]
    #[ensures(result.cursor.raw_pointer() ==
        self.cursor.raw_pointer().offset_logic(count@))]
    #[ensures(result.cursor.raw_pointer().addr_logic()@ ==
        self.cursor.raw_pointer().addr_logic()@ + count@)]
    pub(crate) fn advance_cursor(self, count: usize) -> Self {
        let mut result = self;
        result.cursor = result.cursor.advance_within(count);
        result
    }

    /// Build the two packet witnesses after the event callback has already
    /// split the actual ticket and region. Keeping packet construction out of
    /// the counter callback gives the callback and the packet invariant their
    /// own small proof boundaries; all affine values are still moved here.
    #[requires((*before_packet).invariant())]
    #[requires(capsule.inner_logic() == (*before_packet).capsule.inner_logic())]
    #[requires(key.inner_logic() == (*before_packet).lease_key())]
    #[requires(*before_ticket == (*before_packet).ticket.inner_logic())]
    #[requires(*before_region == (*before_packet).region.inner_logic())]
    #[requires(base == (*before_packet).base && cursor == (*before_packet).cursor)]
    #[requires(*cut == cursor@.unwrap_logic().2 + count@)]
    #[requires(cursor@.unwrap_logic().2 <= *cut && *cut <= (*before_region).hi())]
    #[requires(parent_ticket.inner_logic().invariant() &&
        child_ticket.inner_logic().invariant())]
    #[requires(left_region.inner_logic().invariant() &&
        right_region.inner_logic().invariant())]
    #[requires(mutable_owner_split_result(
        before_ticket,
        before_region,
        cut,
        (parent_ticket.inner_logic(), child_ticket.inner_logic(),
            left_region.inner_logic(), right_region.inner_logic())
    ))]
    #[requires(right_cursor.invariant() && right_cursor@ != None)]
    #[requires(right_cursor@.unwrap_logic().0 == cursor@.unwrap_logic().0 &&
        right_cursor@.unwrap_logic().1 == cursor@.unwrap_logic().1 &&
        right_cursor@.unwrap_logic().2 == *cut)]
    #[requires(right_cursor.current_address() == cursor.current_address() + count@)]
    #[requires(right_cursor.raw_pointer().addr_logic()@ ==
        cursor.raw_pointer().addr_logic()@ + count@)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(0 <= key.inner_logic().slot() &&
        key.inner_logic().slot() < context.inner_logic().ready_len())]
    #[requires(context.inner_logic().ready_entry_state(key.inner_logic().slot()) == None)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(loan.inner_logic().lease_key == key.inner_logic())]
    #[requires(loan.inner_logic().lease_id == key.inner_logic().resource_id())]
    #[ensures(result.0.invariant() && result.1.invariant())]
    #[ensures(result.0.capsule.inner_logic() == capsule.inner_logic() &&
        result.1.capsule.inner_logic() == capsule.inner_logic())]
    #[ensures(result.0.ticket.inner_logic() == parent_ticket.inner_logic() &&
        result.1.ticket.inner_logic() == child_ticket.inner_logic())]
    #[ensures(result.0.region.inner_logic() == left_region.inner_logic() &&
        result.1.region.inner_logic() == right_region.inner_logic())]
    #[ensures(result.0.lease_key() == key.inner_logic() &&
        result.1.lease_key() == key.inner_logic())]
    #[ensures(result.0.base == base && result.1.base == base)]
    #[ensures(result.0.cursor == cursor)]
    #[ensures(result.1.cursor@ == Some((
        cursor@.unwrap_logic().0,
        cursor@.unwrap_logic().1,
        *cut
    )))]
    #[ensures(result.1.cursor.current_address() ==
        cursor.current_address() + count@)]
    #[ensures(result.1.cursor.raw_pointer().addr_logic()@ ==
        right_cursor.raw_pointer().addr_logic()@)]
    #[ensures(result.1.cursor == right_cursor)]
    #[ensures(mutable_owner_split_result(
        before_ticket,
        before_region,
        cut,
        (result.0.ticket.inner_logic(),
            result.1.ticket.inner_logic(),
            result.0.region.inner_logic(),
            result.1.region.inner_logic())
    ))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_entry_state(key.inner_logic().slot()) == Some((
        key.inner_logic().resource_id(), key.inner_logic().slot(),
        key.inner_logic().resource_id(), Some(Excl((key.inner_logic().resource_id(), None)))
    )))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != key.inner_logic().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    fn build_split_packets(
        capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
        key: Ghost<ReadyKey>,
        parent_ticket: Ghost<OwnerTicket>,
        child_ticket: Ghost<OwnerTicket>,
        left_region: Ghost<PhysicalRegion>,
        right_region: Ghost<PhysicalRegion>,
        before_ticket: Snapshot<OwnerTicket>,
        before_region: Snapshot<PhysicalRegion>,
        before_packet: Snapshot<Self>,
        base: BoundPtr,
        cursor: BoundPtr,
        right_cursor: BoundPtr,
        count: usize,
        cut: Snapshot<Int>,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> (Self, Self) {
        // Return the actual lease fragment before forming the regular packet
        // pair. Every affine packet field stays Ghost-wrapped; no packet is
        // extracted from a Ghost aggregate or reconstructed from a snapshot.
        Self::restore_lease_after_owner_release(capsule, key, context, loan);
        (
            SharedOwnerPacket {
                capsule,
                ticket: parent_ticket,
                region: left_region,
                base,
                cursor,
                lease_key: key,
            },
            SharedOwnerPacket {
                capsule,
                ticket: child_ticket,
                region: right_region,
                base,
                cursor: right_cursor,
                lease_key: key,
            },
        )
    }

    /// Increment the real shared counter and divide this packet's existing
    /// Mutable ticket and physical region at the requested absolute offset.
    /// The same idle lease capability is temporarily moved from the caller's
    /// registry into a read loan and then returned to its original slot.
    #[requires(packet.invariant())]
    #[requires(header_id as *mut () == packet.capsule_public().0)]
    #[requires(*before_ticket == packet.ticket.inner_logic())]
    #[requires(*before_region == packet.region.inner_logic())]
    #[requires(*cut == packet.cursor@.unwrap_logic().2 + count@)]
    #[requires(count@ <= packet.region.inner_logic().hi() -
        packet.cursor@.unwrap_logic().2)]
    #[requires(packet.region.inner_logic().lo() <= *cut &&
        *cut <= packet.region.inner_logic().hi())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(0 <= packet.lease_key().slot() &&
        packet.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(context.inner_logic().ready_entry_state(
        packet.lease_key().slot()) == Some((
            packet.lease_key().resource_id(),
            packet.lease_key().slot(),
            packet.lease_key().resource_id(),
            Some(Excl((packet.lease_key().resource_id(), None)))
        )))]
    #[ensures(result.0.invariant() && result.1.invariant())]
    #[ensures(result.0.capsule.inner_logic() == packet.capsule.inner_logic() &&
        result.1.capsule.inner_logic() == packet.capsule.inner_logic())]
    #[ensures(result.0.capsule_public() == packet.capsule_public() &&
        result.1.capsule_public() == packet.capsule_public())]
    #[ensures(result.0.lease_key() == packet.lease_key() &&
        result.1.lease_key() == packet.lease_key())]
    #[ensures(result.0.base == packet.base && result.1.base == packet.base)]
    #[ensures(result.0.cursor == packet.cursor)]
    #[ensures(result.1.cursor@ == Some((
        packet.cursor@.unwrap_logic().0,
        packet.cursor@.unwrap_logic().1,
        *cut
    )))]
    #[ensures(result.1.cursor.current_address() ==
        packet.cursor.current_address() + count@)]
    #[ensures(result.1.cursor.raw_pointer() ==
        packet.cursor.raw_pointer().offset_logic(count@))]
    #[ensures(result.1.cursor.raw_pointer().addr_logic()@ ==
        packet.cursor.raw_pointer().addr_logic()@ + count@)]
    #[ensures(mutable_owner_split_result(
        before_ticket,
        before_region,
        cut,
        (
            result.0.ticket.inner_logic(),
            result.1.ticket.inner_logic(),
            result.0.region.inner_logic(),
            result.1.region.inner_logic(),
        )
    ))]
    #[ensures(context.inner_logic().unchanged(^context.inner_logic()))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).ready_entry_state(
        packet.lease_key().slot()) == context.inner_logic().ready_entry_state(
            packet.lease_key().slot()))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != packet.lease_key().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    pub(super) fn split_after_increment(
        packet: Self,
        header_id: *mut Shared,
        count: usize,
        cut: Snapshot<Int>,
        before_ticket: Snapshot<OwnerTicket>,
        before_region: Snapshot<PhysicalRegion>,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> (Self, Self) {
        let packet_value = packet;
        let base = packet_value.base;
        let cursor = packet_value.cursor;
        let right_cursor = cursor.advance_within(count);
        let before_packet: Snapshot<Self> = snapshot!(packet_value);
        let packet = packet_value.into_ghost();
        let mut context_ref = ghost!(context.into_inner());
        let packet_parts = ghost! {
            let mut packet = packet.into_inner();
            let key = packet.lease_key_ghost().into_inner();
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&**context_ref), Ghost::new(key),
            );
            let loan_result = packet.try_take_header_read_loan(
                Ghost::new(&mut **context_ref),
            );
            (
                packet.capsule.into_inner(),
                key,
                packet.ticket.into_inner(),
                packet.region.into_inner(),
                loan_result.into_inner(),
            )
        };
        let (capsule, key, ticket, region, loan_result) =
            packet_parts.split();
        let loan = ghost! { loan_result.into_inner().unwrap() };
        let expected_counter: Snapshot<NativeCounter> =
            snapshot!(*loan.inner_logic().counter_model);
        let expected_key: Snapshot<ReadyKey> = snapshot!(key.inner_logic());
        let expected_lease_id: Snapshot<Id> = snapshot!(key.inner_logic().resource_id());
        let expected_ledger_id: Snapshot<Id> =
            snapshot!(ticket.inner_logic().ledger_id());
        let expected_phase: Snapshot<Id> = snapshot!(loan.inner_logic().phase_id);
        let expected_before_ticket: Snapshot<OwnerTicket> =
            snapshot!(ticket.inner_logic());
        let expected_before_region: Snapshot<PhysicalRegion> =
            snapshot!(region.inner_logic());
        let expected_cut: Snapshot<Int> = snapshot!(*cut);
        let callback_frame = (
            expected_before_ticket,
            expected_before_region,
            expected_cut,
        );
        let counter = loaned_native_counter(
            header_id as *const Shared,
            ghost! { loan.borrow().into_inner() },
        );
        let (incremented, callback) = counter.try_increment_with(
            ghost! { capsule.into_inner().to_ref() },
            ghost!(&callback_frame),
            ghost!(
                #[requires(
                    (*payload).invariant() && payload.protocol() &&
                    payload.counter_model() == *expected_counter &&
                    payload.agrees_with_value(old) &&
                    old@ <= crate::ref_count_limit::MAX_REF_COUNT@ &&
                    new@ == old@ + 1 &&
                    payload.public().2 == *expected_ledger_id &&
                    payload.public().4 == *expected_lease_id &&
                    payload.public().6 == *expected_key &&
                    payload.public().3 == *expected_phase &&
                    ticket.inner_logic().invariant() &&
                    ticket.inner_logic().ledger_id() == payload.public().2 &&
                    ticket.inner_logic().class() == RefKind::Mutable &&
                    region.inner_logic().invariant() &&
                    ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
                    ticket.inner_logic().owned_hi() == region.inner_logic().hi() &&
                    ticket.inner_logic().allocation_binding().0 ==
                        region.inner_logic().namespace() &&
                    ticket.inner_logic().allocation_binding().2 ==
                        region.inner_logic().capacity() &&
                    ticket.inner_logic().owned_lo() <= *cut &&
                    *cut <= ticket.inner_logic().owned_hi() &&
                    *frame.0 == ticket.inner_logic() &&
                    *frame.1 == region.inner_logic() &&
                    frame.2 == cut
                )]
                #[ensures(
                    (^payload).invariant() && (^payload).protocol() &&
                    (^payload).public() == payload.public() &&
                    (^payload).counter_model() == payload.counter_model() &&
                    (^payload).agrees_with_value(new) &&
                    result.inner_logic().0.invariant() &&
                    result.inner_logic().1.invariant() &&
                    result.inner_logic().2.invariant() &&
                    result.inner_logic().3.invariant() &&
                    (^payload).ledger.unwrap_logic().has_ticket(result.inner_logic().0) &&
                    (^payload).ledger.unwrap_logic().has_ticket(result.inner_logic().1) &&
                    result.inner_logic().4 == frame.0 &&
                    result.inner_logic().5 == frame.1 &&
                    result.inner_logic().6 == frame.2 &&
                    mutable_owner_split_result(
                        result.inner_logic().4, result.inner_logic().5,
                        result.inner_logic().6,
                        (result.inner_logic().0, result.inner_logic().1,
                            result.inner_logic().2, result.inner_logic().3)
                    )
                )]
                move |payload: &mut SharedPayload,
                      old: usize,
                      new: usize,
                      frame: &(Snapshot<OwnerTicket>, Snapshot<PhysicalRegion>, Snapshot<Int>)| {
                    payload.prove_live_owner_matches_counter(&ticket, old);
                    SharedPayload::split_mutable_owner_after_increment(
                        Ghost::new(payload), ticket, region, cut,
                        old, new,
                    )
                }
            ),
        );
        if incremented.is_err() {
            crate::abort();
        }
        let split = ghost! { callback.into_inner().ok().unwrap().into_inner() };
        let (
            parent_ticket,
            child_ticket,
            left_region,
            right_region,
            _callback_before_ticket,
            _callback_before_region,
            _callback_cut,
        ) = split.split();
        Self::build_split_packets(
            capsule,
            key,
            parent_ticket,
            child_ticket,
            left_region,
            right_region,
            expected_before_ticket,
            expected_before_region,
            before_packet,
            base,
            cursor,
            right_cursor,
            count,
            cut,
            context_ref,
            loan,
        )
    }

    /// Try to transfer the capsule's affine header root into a read-only
    /// counter loan. A missing context cap yields `None`; a present cap is
    /// checked against the actual payload authority before the root moves.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(
        self.capsule.inner_logic().val().namespace()
    ))]
    #[requires(!context.inner_logic().ready_has_key(self.lease_key()) ||
        context.inner_logic().ready_entry_state(self.lease_key().slot()) == Some((
            self.lease_key().resource_id(), self.lease_key().slot(),
            self.lease_key().resource_id(),
            Some(Excl((self.lease_key().resource_id(), None))))))]
    #[ensures((^self).invariant() &&
        (^self).capsule.inner_logic() == self.capsule.inner_logic() &&
        (^self).ticket.inner_logic() == self.ticket.inner_logic() &&
        (^self).region.inner_logic() == self.region.inner_logic() &&
        (^self).base == self.base && (^self).cursor == self.cursor &&
        (^context.inner_logic()).invariant() &&
        (^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces() &&
        (^context.inner_logic()).ready_len() == context.inner_logic().ready_len() &&
        ((result.inner_logic() != None) ==
            context.inner_logic().ready_has_key(self.lease_key())) &&
        (forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
            i != self.lease_key().slot() ==>
            (^context.inner_logic()).ready_entry_state(i) ==
                context.inner_logic().ready_entry_state(i)) &&
        (result.inner_logic() == None ==>
            (^context.inner_logic()).ready_entry_state(self.lease_key().slot()) ==
                context.inner_logic().ready_entry_state(self.lease_key().slot())) &&
        (result.inner_logic() != None ==>
            (^context.inner_logic()).ready_entry_state(self.lease_key().slot()) == None) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().lease_key == self.lease_key()) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().invariant()) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().header_id ==
                self.capsule_public().0 as *mut Shared) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().lease_id == self.lease_resource_id()) &&
        (result.inner_logic() != None ==>
            *result.inner_logic().unwrap_logic().counter_model == self.capsule_public().1) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().phase_id == self.capsule_public().3) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().storage_model).binding() ==
                self.ticket.inner_logic().allocation_binding().0) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().storage_model).raw_pointer() ==
                self.ticket.inner_logic().allocation_binding().1) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().storage_model).capacity() ==
                self.ticket.inner_logic().allocation_binding().2) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().storage_model).capacity() ==
                self.region.inner_logic().capacity()))]
    pub(crate) fn try_take_header_read_loan(
        &mut self,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> Ghost<Option<SharedHeaderReadLoan>> {
        ghost! {
            let context = context.into_inner();
            let key = self.lease_key_ghost().into_inner();
            // Borrow the packet's actual affine owner ticket. It is neither
            // reconstructed from its model nor moved into the callback.
            let ticket_body: Ghost<&OwnerTicket> = Ghost::new(&*self.ticket);
            if !context.ready_has_key_ghost(key).into_inner() {
                None
            } else {
                let capsule = *self.capsule;
                let cap = BorrowedSharedContext::take(Ghost::new(&mut *context), key);
                let tokens = BorrowedSharedContext::reborrow_tokens(Ghost::new(&mut *context));
                let expected_key: Snapshot<ReadyKey> = snapshot!(key);
                let expected_lease_id: Snapshot<Id> = snapshot!(key.resource_id());
                let expected_phase: Snapshot<Id> =
                    snapshot!(self.capsule_public().3);
                capsule.to_ref().open_with_tokens(
                    tokens,
                    ghost!(
                        #[requires(
                            payload.invariant() &&
                            payload.protocol() &&
                            payload.public().3 == *expected_phase &&
                            payload.counter_model() == *owner_model &&
                            payload.agrees_with_value(*owner_value) &&
                            payload.public().4 == *expected_lease_id &&
                            payload.public().6 == *expected_key &&
                            ticket_body.inner_logic().invariant() &&
                            ticket_body.inner_logic().ledger_id() == payload.public().2 &&
                            cap.inner_logic().invariant() &&
                            cap.inner_logic().key() == *expected_key &&
                            cap.inner_logic().fragment_id() == *expected_lease_id &&
                            cap.inner_logic().fragment_value() ==
                                Some(Excl((*expected_lease_id, None)))
                        )]
                        #[ensures(
                            (^payload).public() == payload.public() &&
                            (^payload).counter_model() == payload.counter_model() &&
                            (^payload).protocol() &&
                            (^payload).agrees_with_value(*owner_value) &&
                            match result {
                                Some(loan) =>
                                    loan.invariant() &&
                                    loan.header_id == payload.header_id &&
                                    loan.lease_id == payload.lease.resource_id() &&
                                    loan.lease_key == payload.lease_key &&
                                    loan.phase_id == payload.public().3 &&
                                    *loan.counter_model == *payload.counter_model &&
                                    (*loan.storage_model).binding() == (*payload.allocation_binding).0 &&
                                    (*loan.storage_model).raw_pointer() == (*payload.allocation_binding).1 &&
                                    (*loan.storage_model).capacity() == (*payload.allocation_binding).2,
                                None => false,
                            }
                        )]
                        move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>| {
                            let cap = cap.into_inner();
                            let ticket = ticket_body.into_inner();
                            payload.prove_public_lease_resource_id(expected_lease_id);
                            payload.prove_public_lease_key(expected_key);
                            payload.prove_idle_cap_implies_root_some(&cap, ticket);
                            Some(SharedPayload::take_header_read_loan(
                                Ghost::new(payload), Ghost::new(cap),
                            ).into_inner())
                        }
                    ),
                ).into_inner()
            }
        }
    }

    /// Restore the exact root by consuming the matching affine lease fragment.
    /// Resource-algebra agreement rules out a stale generation or a different
    /// capsule state; the current payload cannot replace the active lease
    /// while the fragment remains in the caller's loan.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(loan.inner_logic().invariant())]
    #[requires(loan.inner_logic().lease_id == self.lease_resource_id())]
    #[requires(loan.inner_logic().lease_key == self.lease_key())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().ready_entry_state(self.lease_key().slot()) == None)]
    #[requires(self.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(context.inner_logic().token_namespaces().contains(
        self.capsule.inner_logic().val().namespace()
    ))]
    #[ensures((^self).invariant() && (^context.inner_logic()).invariant())]
    #[ensures((^self).capsule.inner_logic() == self.capsule.inner_logic())]
    #[ensures((^self).ticket.inner_logic() == self.ticket.inner_logic())]
    #[ensures((^self).region.inner_logic() == self.region.inner_logic())]
    #[ensures((^self).base == self.base && (^self).cursor == self.cursor)]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_has_key(self.lease_key()))]
    #[ensures((^context.inner_logic()).ready_entry_state(self.lease_key().slot()) == Some((
        self.lease_key().resource_id(), self.lease_key().slot(),
        self.lease_resource_id(),
        Some(Excl((self.lease_resource_id(), None))))))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != self.lease_key().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    pub(crate) fn try_restore_header_read_loan(
        &mut self,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<()> {
        ghost! {
            let context = context.into_inner();
            let capsule = *self.capsule;
            let expected_lease_id: Snapshot<Id> =
                snapshot!(self.lease_resource_id());
            let expected_key: Snapshot<ReadyKey> = snapshot!(self.lease_key());
            let ticket_body: Ghost<&OwnerTicket> = Ghost::new(&*self.ticket);
            let tokens = BorrowedSharedContext::reborrow_tokens(Ghost::new(&mut *context));
            let cap = capsule.to_ref().open_with_tokens(
                tokens,
                ghost!(
                    #[requires(
                        payload.invariant() &&
                        payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().4 == *expected_lease_id &&
                        payload.public().6 == *expected_key &&
                        ticket_body.inner_logic().invariant() &&
                        ticket_body.inner_logic().ledger_id() == payload.public().2 &&
                        loan.inner_logic().invariant() &&
                        loan.inner_logic().lease_id == *expected_lease_id &&
                        loan.inner_logic().lease_key == *expected_key
                    )]
                    #[ensures(
                        (^payload).public() == payload.public() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).protocol() &&
                        (^payload).agrees_with_value(*owner_value) &&
                        (^payload).ledger_id == payload.ledger_id &&
                        (^payload).lease_key == payload.lease_key &&
                        (^payload).lease.resource_id() == payload.lease.resource_id() &&
                        (^payload).ledger == payload.ledger &&
                        (^payload).retire == payload.retire &&
                        (^payload).phase == payload.phase &&
                        (^payload).terminal_action == payload.terminal_action &&
                        (^payload).closed == payload.closed &&
                        result.invariant() &&
                        result.key() == (^payload).public().6 &&
                        result.fragment_id() == (^payload).public().4 &&
                        result.fragment_value() ==
                            Some(Excl(((^payload).public().4, None)))
                    )]
                    move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>| {
                    let ticket = ticket_body.into_inner();
                    payload.prove_public_lease_resource_id(expected_lease_id);
                    payload.prove_public_lease_key(expected_key);
                    payload.prove_live_owner_ticket(ticket);
                    SharedPayload::restore_header_read_loan(Ghost::new(payload), loan).into_inner()
                }),
            ).into_inner();
            BorrowedSharedContext::return_cap(Ghost::new(&mut *context), Ghost::new(cap));
            ()
        }
    }

    /// Return the loan's actual capability after its packet ticket has been
    /// consumed by the native reference-decrement callback. This variant is
    /// deliberately keyed by the producer-issued lease key and capsule, so
    /// it does not require a now-retired packet as a proxy for the owner.
    #[check(ghost)]
    #[requires(capsule.inner_logic().invariant())]
    #[requires(capsule.inner_logic().val().namespace() == event_namespace())]
    // Keep the read loan bound to the actual capsule whose event state is
    // opened below; the active lease cap binds the remaining header stamp.
    #[requires(capsule.inner_logic().val().public().1.4 ==
        loan.inner_logic().lease_id)]
    #[requires(capsule.inner_logic().val().public().1.6 ==
        loan.inner_logic().lease_key)]
    #[requires(key.inner_logic() == loan.inner_logic().lease_key)]
    #[requires(key.inner_logic().resource_id() == loan.inner_logic().lease_id)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().ready_entry_state(key.inner_logic().slot()) == None)]
    #[requires(key.inner_logic().slot() < context.inner_logic().ready_len())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[ensures(capsule.inner_logic().invariant())]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_has_key(key.inner_logic()))]
    #[ensures((^context.inner_logic()).ready_entry_state(key.inner_logic().slot()) == Some((
        key.inner_logic().resource_id(), key.inner_logic().slot(),
        key.inner_logic().resource_id(),
        Some(Excl((key.inner_logic().resource_id(), None))))))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != key.inner_logic().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    fn restore_lease_after_owner_release(
        capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
        key: Ghost<ReadyKey>,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) {
        ghost! {
            let capsule = capsule.into_inner();
            let key = key.into_inner();
            let context = context.into_inner();
            let expected_key: Snapshot<ReadyKey> = snapshot!(key);
            let expected_lease_id: Snapshot<Id> = snapshot!(key.resource_id());
            let cap = capsule.to_ref().open_with_tokens(
                BorrowedSharedContext::reborrow_tokens(Ghost::new(&mut *context)),
                ghost!(
                    #[requires(
                    (*payload).invariant() && payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().4 == *expected_lease_id &&
                        payload.public().6 == *expected_key &&
                        loan.inner_logic().invariant() &&
                        loan.inner_logic().lease_id == *expected_lease_id &&
                        loan.inner_logic().lease_key == *expected_key
                    )]
                    #[ensures(
                        (^payload).public() == payload.public() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).protocol() &&
                        (^payload).agrees_with_value(*owner_value) &&
                        (^payload).ledger_id == payload.ledger_id &&
                        (^payload).lease_key == payload.lease_key &&
                        (^payload).lease.resource_id() == payload.lease.resource_id() &&
                        (^payload).ledger == payload.ledger &&
                        (^payload).retire == payload.retire &&
                        (^payload).phase == payload.phase &&
                        (^payload).terminal_action == payload.terminal_action &&
                        (^payload).closed == payload.closed &&
                        result.invariant() &&
                        result.key() == (^payload).public().6 &&
                        result.fragment_id() == (^payload).public().4 &&
                        result.fragment_value() ==
                            Some(Excl(((^payload).public().4, None)))
                    )]
                    move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>| {
                        SharedPayload::restore_header_read_loan(
                            Ghost::new(payload), loan,
                        ).into_inner()
                    }
                ),
            ).into_inner();
            BorrowedSharedContext::return_cap(Ghost::new(&mut *context), Ghost::new(cap));
            ()
        };
    }

    /// Finish the final owner's terminal transition after the native Release
    /// decrement has removed its ticket. The active read loan supplies the
    /// exact root and lease fragment; the payload supplies its actual stored
    /// Vec authority, retirement pool, and terminal phase action.
    #[check(ghost)]
    #[requires(capsule.inner_logic().invariant())]
    #[requires(capsule.inner_logic().val().namespace() == event_namespace())]
    #[requires(capsule.inner_logic().val().public().1.0 ==
        loan.inner_logic().header_id as *mut ())]
    #[requires(capsule.inner_logic().val().public().1.3 ==
        loan.inner_logic().phase_id)]
    #[requires(capsule.inner_logic().val().public().1.4 ==
        loan.inner_logic().lease_id)]
    #[requires(capsule.inner_logic().val().public().1.6 ==
        loan.inner_logic().lease_key)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(key.inner_logic() == loan.inner_logic().lease_key)]
    #[requires(key.inner_logic().resource_id() == loan.inner_logic().lease_id)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(action.inner_logic().invariant())]
    #[requires(action.inner_logic().resource_id() == loan.inner_logic().phase_id)]
    #[requires(context.inner_logic().ready_entry_state(key.inner_logic().slot()) == None)]
    #[requires(key.inner_logic().slot() < context.inner_logic().ready_len())]
    #[ensures(capsule.inner_logic().invariant())]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_entry_state(key.inner_logic().slot()) == None)]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    #[ensures(result.inner_logic().storage.invariant())]
    #[ensures(result.inner_logic().region.invariant())]
    #[ensures(result.inner_logic().region.namespace() ==
        result.inner_logic().storage.binding())]
    #[ensures(result.inner_logic().region.resource_id() ==
        result.inner_logic().storage.binding())]
    #[ensures(result.inner_logic().region.capacity() ==
        result.inner_logic().storage.capacity())]
    #[ensures(result.inner_logic().region.lo() == 0)]
    #[ensures(result.inner_logic().region.hi() ==
        result.inner_logic().storage.capacity())]
    #[ensures(result.inner_logic().root.invariant())]
    #[ensures(*result.inner_logic().root.ward() ==
        loan.inner_logic().header_id as *const Shared)]
    #[ensures(result.inner_logic().root.val().vec@ != None)]
    #[ensures(result.inner_logic().root.val().vec_model().matches_authority(
        result.inner_logic().storage))]
    fn finish_terminal_owner_release(
        capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
        key: Ghost<ReadyKey>,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        loan: Ghost<SharedHeaderReadLoan>,
        action: Ghost<TerminalAction>,
    ) -> Ghost<SharedRetirementResources> {
        ghost! {
            let capsule = capsule.into_inner();
            let key = key.into_inner();
            let context = context.into_inner();
            let expected_key: Snapshot<ReadyKey> = snapshot!(key);
            let expected_header: Snapshot<*mut ()> =
                snapshot!(loan.inner_logic().header_id as *mut ());
            let expected_lease_id: Snapshot<Id> =
                snapshot!(loan.inner_logic().lease_id);
            let expected_phase: Snapshot<Id> = snapshot!(loan.inner_logic().phase_id);
            let tokens = BorrowedSharedContext::reborrow_tokens(
                Ghost::new(&mut *context),
            );
            capsule.open_with_tokens(
                tokens,
                ghost!(
                    #[requires(
                        (*payload).invariant() && payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().0 == *expected_header &&
                        payload.public().4 == *expected_lease_id &&
                        payload.public().6 == *expected_key &&
                        payload.public().3 == *expected_phase &&
                        loan.invariant() &&
                        loan.lease_key == *expected_key &&
                        loan.phase_id == *expected_phase &&
                        action.invariant() &&
                        action.resource_id() == *expected_phase
                    )]
                    #[ensures(
                        (^payload).public() == payload.public() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).protocol() &&
                        (^payload).agrees_with_value(*owner_value) &&
                        result.storage.invariant() &&
                        result.region.invariant() &&
                        result.region.namespace() == result.storage.binding() &&
                        result.region.resource_id() == result.storage.binding() &&
                        result.region.capacity() == result.storage.capacity() &&
                        result.region.lo() == 0 &&
                        result.region.hi() == result.storage.capacity() &&
                        result.root.invariant() &&
                        *result.root.ward() == payload.header_id as *const Shared &&
                        result.root.val().vec@ != None &&
                        result.root.val().vec_model().matches_authority(result.storage)
                    )]
                    move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>| {
                        let loan = loan.into_inner();
                        let mut action = action.into_inner();
                        payload.prove_public_phase_id(expected_phase);
                        SharedPayload::prove_open_phase_matches_action(
                            &*payload, &action,
                        );
                        SharedPayload::prove_current_header_lease(
                            Ghost::new(&*payload), Ghost::new(&loan),
                        );
                        SharedPayload::prove_final_action_excludes_live_state(
                            Ghost::new(&*payload), Ghost::new(&loan),
                            Ghost::new(&mut action), owner_value,
                        );
                        payload.phase.require_action(&action);
                        payload.phase.close(action);
                        let stamp: Snapshot<HeaderLeaseStamp> = snapshot!(
                            header_lease_stamp(
                                payload.lease_generation,
                                payload.header_id,
                                *payload.counter_model,
                                payload.ledger_id,
                                payload.phase_id,
                                payload.storage.unwrap_logic(),
                                *payload.header_vec,
                                *payload.original_capacity_repr,
                                payload.lease_key,
                            )
                        );
                        let SharedHeaderReadLoan { root, lease_cap, .. } = loan;
                        let idle_cap = HeaderLeaseAuthority::deactivate(
                            Ghost::new(&mut payload.lease),
                            stamp,
                            Ghost::new(lease_cap),
                        ).into_inner();
                        drop(idle_cap);

                        let storage = payload.storage.take().unwrap();
                        let pool = payload.pool.take().unwrap();
                        let region = pool.finish(0int);
                        let ledger = payload.ledger.take().unwrap();
                        drop(ledger);
                        proof_assert!(*root.ward() == payload.header_id as *const Shared);
                        proof_assert!(root.invariant());
                        payload.closed = true;
                        proof_assert!(region.resource_id() == storage.binding());
                        proof_assert!(root.val().vec@ != None);
                        proof_assert!(root.val().vec_model().matches_authority(storage));
                        let result = SharedRetirementResources {
                            storage,
                            region,
                            root,
                        };
                        proof_assert!(payload.invariant() && payload.protocol());
                        result
                    }
                ),
            ).into_inner()
        }
    }

    /// Execute the native Release decrement and update the ledger from this
    /// packet's actual ticket and physical region in the atomic callback.
    /// Non-final owners return their lease to the ready context. The final
    /// owner returns its actual active loan and terminal action to the caller,
    /// keeping the root protected for the acquire/retirement half.
    #[requires(packet.inner_logic().invariant())]
    #[requires(header_id as *mut () == packet.inner_logic().capsule_public().0)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(
        packet.inner_logic().capsule.inner_logic().val().namespace()))]
    #[requires(context.inner_logic().ready_entry_state(packet.inner_logic().lease_key().slot()) == Some((
        packet.inner_logic().lease_key().resource_id(),
        packet.inner_logic().lease_key().slot(),
        packet.inner_logic().lease_key().resource_id(),
        Some(Excl((packet.inner_logic().lease_key().resource_id(), None))))))]
    #[requires(packet.inner_logic().lease_key().slot() < context.inner_logic().ready_len())]
    #[ensures(result.0@ > 0)]
    #[ensures((result.1.inner_logic() != None) == (result.0@ == 1))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(result.0@ > 1 ==> (^context.inner_logic()).ready_has_key(packet.inner_logic().lease_key()))]
    #[ensures(result.0@ > 1 ==> (^context.inner_logic()).ready_entry_state(packet.inner_logic().lease_key().slot()) == Some((
        packet.inner_logic().lease_key().resource_id(), packet.inner_logic().lease_key().slot(),
        packet.inner_logic().lease_key().resource_id(),
        Some(Excl((packet.inner_logic().lease_key().resource_id(), None))))))]
    #[ensures(result.0@ == 1 ==> !(^context.inner_logic()).ready_has_key(packet.inner_logic().lease_key()))]
    #[ensures(result.0@ == 1 ==> (^context.inner_logic()).ready_entry_state(packet.inner_logic().lease_key().slot()) == None)]
    #[ensures(result.0@ == 1 ==>
        result.1.inner_logic().unwrap_logic().storage.invariant() &&
        result.1.inner_logic().unwrap_logic().region.invariant() &&
        result.1.inner_logic().unwrap_logic().region.namespace() ==
            result.1.inner_logic().unwrap_logic().storage.binding() &&
        result.1.inner_logic().unwrap_logic().region.resource_id() ==
            result.1.inner_logic().unwrap_logic().storage.binding() &&
        result.1.inner_logic().unwrap_logic().region.capacity() ==
            result.1.inner_logic().unwrap_logic().storage.capacity() &&
        result.1.inner_logic().unwrap_logic().region.lo() == 0 &&
        result.1.inner_logic().unwrap_logic().region.hi() ==
            result.1.inner_logic().unwrap_logic().storage.capacity() &&
        result.1.inner_logic().unwrap_logic().root.invariant() &&
        *result.1.inner_logic().unwrap_logic().root.ward() == header_id as *const Shared &&
        result.1.inner_logic().unwrap_logic().root.val().vec@ != None &&
        result.1.inner_logic().unwrap_logic().root.val().vec_model().matches_authority(
            result.1.inner_logic().unwrap_logic().storage
        ))]
    #[ensures(result.0@ > 1 ==> result.1.inner_logic() == None)]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != packet.inner_logic().lease_key().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    pub(crate) fn release_native_owner(
        packet: Ghost<Self>,
        header_id: *mut Shared,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> (usize, Ghost<Option<SharedRetirementResources>>) {
        let mut context_ref = ghost!(context.into_inner());
        let packet_parts = ghost! {
            let mut packet = packet.into_inner();
            let key = packet.lease_key_ghost().into_inner();
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&**context_ref),
                Ghost::new(key),
            );
            let loan_result = packet.try_take_header_read_loan(
                Ghost::new(&mut **context_ref),
            );
            (
                packet.capsule.into_inner(),
                key,
                packet.ticket.into_inner(),
                packet.region.into_inner(),
                loan_result.into_inner(),
            )
        };
        let (capsule, key, ticket, region, loan_result) = packet_parts.split();
        let loan = ghost! { loan_result.into_inner().unwrap() };
        let expected_counter: Snapshot<NativeCounter> =
            snapshot!(*loan.inner_logic().counter_model);
        let expected_key: Snapshot<ReadyKey> = snapshot!(key.inner_logic());
        let expected_lease_id: Snapshot<Id> = snapshot!(key.inner_logic().resource_id());
        let expected_ledger_id: Snapshot<Id> = snapshot!(ticket.inner_logic().ledger_id());
        let expected_phase: Snapshot<Id> = snapshot!(loan.inner_logic().phase_id);
        let counter = loaned_native_counter(
            header_id as *const Shared,
            ghost! { loan.borrow().into_inner() },
        );
        let (old, callback) = counter.fetch_sub_release_with(
            ghost! { capsule.into_inner().to_ref() },
            ghost!(
                // Observe the current payload value here; the callback's
                // mutable-reference invariant already tracks its endpoints.
                #[requires(
                    (*payload).invariant() && payload.protocol() &&
                        payload.counter_model() == *expected_counter &&
                        payload.agrees_with_value(old) &&
                        new == if old == 0usize { usize::MAX } else { old - 1usize } &&
                    payload.public().2 == *expected_ledger_id &&
                    payload.public().4 == *expected_lease_id &&
                    payload.public().6 == *expected_key &&
                    payload.public().3 == *expected_phase &&
                    ticket.inner_logic().invariant() &&
                    ticket.inner_logic().ledger_id() == payload.public().2 &&
                    ticket.inner_logic().class() == RefKind::Mutable &&
                    region.inner_logic().invariant() &&
                    ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
                    ticket.inner_logic().owned_hi() == region.inner_logic().hi() &&
                    ticket.inner_logic().allocation_binding().0 ==
                        region.inner_logic().namespace() &&
                    ticket.inner_logic().allocation_binding().2 ==
                        region.inner_logic().capacity()
                )]
                #[ensures(
                    (^payload).public() == payload.public() &&
                    (^payload).counter_model() == payload.counter_model() &&
                    (^payload).protocol() &&
                    (^payload).agrees_with_value(new)
                )]
                #[ensures(match result {
                    Some(action) => old@ == 1 && action.invariant() &&
                        action.resource_id() == payload.public().3 &&
                        (^payload).terminal_action == None &&
                        (^payload).retire != None &&
                        (^payload).public().3 == payload.public().3 &&
                        (^payload).lease_key == payload.lease_key &&
                        (^payload).lease.resource_id() == payload.lease.resource_id(),
                    None => old@ > 1 &&
                        (^payload).terminal_action == payload.terminal_action &&
                        (^payload).retire == None &&
                        (^payload).public().3 == payload.public().3 &&
                        (^payload).lease_key == payload.lease_key &&
                        (^payload).lease.resource_id() == payload.lease.resource_id(),
                })]
                move |payload: &mut SharedPayload,
                     old: usize,
                     new: usize| {
                    payload.prove_live_owner_matches_counter(&ticket, old);
                    proof_assert!(new@ + 1 == old@);
                    SharedPayload::drop_packet_owner(
                        Ghost::new(payload), expected_counter,
                        snapshot!(old), snapshot!(new), ticket, region,
                        expected_phase,
                    ).into_inner()
                }
            ),
        );
        if old == 1 {
            let _ = counter.load_acquire_with(
                ghost! { capsule.into_inner().to_ref() },
                ghost!(
                    // Use the current payload value, not its future state.
                    #[requires(
                        (*payload).invariant() && payload.protocol() &&
                        payload.agrees_with_value(value)
                    )]
                    #[ensures(
                        (^payload).public() == payload.public() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).protocol() &&
                        (^payload).agrees_with_value(value)
                    )]
                    |payload: &mut SharedPayload, value: usize| ()
                ),
            );
        }
        let retirement = ghost! {
            match callback.into_inner() {
                None => {
                    SharedOwnerPacket::restore_lease_after_owner_release(
                        capsule, key, Ghost::new(&mut **context_ref), loan,
                    );
                    None
                }
                Some(action) => {
                    let result = SharedOwnerPacket::finish_terminal_owner_release(
                        capsule, key, Ghost::new(&mut **context_ref),
                        loan, Ghost::new(action),
                    ).into_inner();
                    BorrowedSharedContext::prove_tombstone_has_no_key(
                        Ghost::new(&**context_ref), key,
                    );
                    Some(result)
                }
            }
        };
        (old, retirement)
    }
}

/// Construct the sole Mutable registration for a newly promoted Vec-backed
/// allocation. The registration map is also the native strong-reference map;
/// the physical pool excludes the full region held by this packet.
#[requires(
    root.inner_logic().invariant() &&
    *root.inner_logic().ward() == header_id as *const Shared &&
    owner.inner_logic().0.invariant() &&
    (*root.inner_logic().val()).ref_count.counter_model() == *owner.inner_logic().0.ward() &&
    owner.inner_logic().0.val() == value && owner.inner_logic().1 == value &&
    !(*root.inner_logic().val()).ref_count.has_event_invariant() &&
    storage.inner_logic().0.invariant() &&
    (*root.inner_logic().val()).vec@ != None &&
    (*root.inner_logic().val()).vec_model().matches_authority(storage.inner_logic().0) &&
    ledger.inner_logic().0.invariant() && ledger.inner_logic().0.is_live() &&
    ledger.inner_logic().0.agrees_with_value(value@) &&
    ledger.inner_logic().0.count() == 1 && value@ == 1 &&
    ledger.inner_logic().0.allocation_binding() == (
        storage.inner_logic().0.binding(), storage.inner_logic().0.raw_pointer(),
        storage.inner_logic().0.capacity()) &&
    ledger.inner_logic().0.ledger_id() == ledger.inner_logic().1.ledger_id() &&
    ledger.inner_logic().1.invariant() &&
    ledger.inner_logic().1.class() == RefKind::Mutable &&
    ledger.inner_logic().0.has_ticket(ledger.inner_logic().1) &&
    ledger.inner_logic().1.owned_lo() == 0 &&
    ledger.inner_logic().1.owned_hi() == storage.inner_logic().0.capacity() &&
    0 <= ledger.inner_logic().1.owner_id() &&
    ledger.inner_logic().1.owner_id() < ledger.inner_logic().0.next_id() &&
    storage.inner_logic().1.invariant() &&
    storage.inner_logic().1.namespace() == storage.inner_logic().0.binding() &&
    storage.inner_logic().1.capacity() == storage.inner_logic().0.capacity() &&
    storage.inner_logic().1.lo() == 0 &&
    storage.inner_logic().1.hi() == storage.inner_logic().1.capacity() &&
    base.invariant() &&
    base@ == Some((storage.inner_logic().1.namespace(), storage.inner_logic().1.capacity(), 0int)) &&
    base.raw_pointer() == storage.inner_logic().0.raw_pointer() &&
    cursor.invariant() && cursor@ != None &&
    cursor@.unwrap_logic().0 == storage.inner_logic().1.namespace() &&
    cursor@.unwrap_logic().1 == storage.inner_logic().1.capacity() &&
    cursor.current_address() == base.current_address() + cursor@.unwrap_logic().2 &&
    cursor@.unwrap_logic().2 <= storage.inner_logic().1.hi() &&
    phase.inner_logic().0.invariant() && phase.inner_logic().0.is_open() &&
    phase.inner_logic().1.invariant() &&
    phase.inner_logic().1.resource_id() == phase.inner_logic().0.resource_id()
)]
#[requires(context.inner_logic().invariant())]
#[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
#[ensures((^context.inner_logic()).invariant())]
#[ensures(result.capsule.inner_logic().val().namespace() == event_namespace())]
#[ensures(result.capsule_public() ==
    (header_id as *mut (), (*root.inner_logic().val()).ref_count.counter_model(),
        ledger.inner_logic().0.ledger_id(), phase.inner_logic().0.resource_id()))]
#[ensures(result.base@ == base@ && result.cursor@ == cursor@)]
#[ensures(result.base.raw_pointer() == base.raw_pointer())]
#[ensures(result.cursor.raw_pointer() == cursor.raw_pointer())]
#[ensures(result.lease_key().resource_id() == result.lease_resource_id())]
#[ensures(result.lease_key().slot() == context.inner_logic().ready_len())]
#[ensures(context.inner_logic().registered_cap(
    (^context.inner_logic()), result.lease_key(),
    Some(Excl((result.lease_key().resource_id(), None)))
))]
#[ensures(result.ticket.inner_logic() == ledger.inner_logic().1)]
#[ensures(result.ticket.inner_logic().allocation_binding() == (
    storage.inner_logic().0.binding(), storage.inner_logic().0.raw_pointer(),
    storage.inner_logic().0.capacity()))]
#[ensures(result.ticket.inner_logic().owned_lo() == result.region.inner_logic().lo())]
#[ensures(result.ticket.inner_logic().owned_hi() == result.region.inner_logic().hi())]
#[ensures(result.region.inner_logic() == storage.inner_logic().1)]
#[ensures(ledger.inner_logic().0.ledger_id() == result.capsule_public().2)]
#[ensures(ledger.inner_logic().0.has_ticket(result.ticket.inner_logic()))]
#[ensures(result.ticket.inner_logic().owner_id() >= 0)]
#[ensures(result.ticket.inner_logic().owner_id() < ledger.inner_logic().0.next_id())]
#[ensures(result.region.inner_logic().namespace() == result.base@.unwrap_logic().0)]
#[ensures(result.region.inner_logic().capacity() == result.base@.unwrap_logic().1)]
#[ensures(result.region.inner_logic().capacity() == storage.inner_logic().0.capacity())]
#[ensures(result.base.raw_pointer() == storage.inner_logic().0.raw_pointer())]
#[ensures(result.cursor.current_address() == result.base.current_address() +
    result.cursor@.unwrap_logic().2)]
#[ensures(result.region.inner_logic().lo() == 0)]
#[ensures(result.region.inner_logic().hi() == result.region.inner_logic().capacity())]
#[ensures(result.invariant())]
#[check(terminates)]
pub(crate) fn make_count_one_packet(
    header_id: *mut Shared,
    root: Ghost<Box<Perm<*const Shared>>>,
    owner: Ghost<(Perm<NativeCounter>, usize)>,
    storage: Ghost<(StoredAuthority, PhysicalRegion)>,
    ledger: Ghost<(OwnerLedger, OwnerTicket)>,
    phase: Ghost<(PhaseAuthority, TerminalAction)>,
    base: BoundPtr,
    cursor: BoundPtr,
    value: usize,
    context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
) -> SharedOwnerPacket {
    let packet_parts = ghost! {
        let root = root.into_inner();
        let (owner, owner_value) = owner.into_inner();
        let (storage, region) = storage.into_inner();
        let (ledger, ticket) = ledger.into_inner();
        let (phase, action) = phase.into_inner();
        proof_assert!(owner_value == value);
        proof_assert!(owner.val() == value);
        proof_assert!(ledger.has_ticket(ticket));
        ledger.prove_ticket_is_active(&ticket);
        sole_active_id(snapshot!(ledger), snapshot!(ticket));
        let pool = PhysicalPool::empty_from(&region);
        let ledger_id = snapshot!(ledger.ledger_id()).into_ghost().into_inner();
        let phase_id = snapshot!(phase.resource_id()).into_ghost().into_inner();
        let (lease, lease_key) = HeaderLeaseAuthority::new(
            context,
            header_id,
        ).into_inner();
        let header = snapshot!(root.val());
        let allocation_binding: Snapshot<AllocationBinding> = snapshot!((
            storage.binding(), storage.raw_pointer(), storage.capacity(),
        ));
        let state = SharedPayload {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            allocation_binding,
            header_vec: snapshot!(header.vec_model()),
            original_capacity_repr: snapshot!(header.original_capacity_repr),
            ledger_id,
            phase_id,
            lease_generation: 0int,
            lease_key,
            lease,
            root: Some(root),
            storage: Some(storage),
            pool: Some(pool),
            ledger: Some(ledger),
            retire: None,
            phase,
            terminal_action: Some(action),
            closed: false,
        };
        proof_assert!(state.invariant() && state.protocol());
        proof_assert!(state.counter_model() == *owner.ward());
        proof_assert!(state.agrees_with_value(value));
        let invariant = EventBoundInvariant::new(Ghost::new(owner), Ghost::new(state));
        let capsule = GhostShared::new(invariant).into_inner();
        (capsule, ticket, region, lease_key)
    };
    let (capsule, ticket, region, lease_key) = packet_parts.split();
    SharedOwnerPacket { capsule, ticket, region, base, cursor, lease_key }
}

/// Build the two Mutable packets created when a unique Vec-backed handle is
/// first split. Their registrations partition the full physical allocation.
#[requires(
    root.inner_logic().invariant() &&
    *root.inner_logic().ward() == header_id as *const Shared &&
    owner.inner_logic().0.invariant() &&
    (*root.inner_logic().val()).ref_count.counter_model() == *owner.inner_logic().0.ward() &&
    owner.inner_logic().0.val() == 2usize && owner.inner_logic().1 == 2usize &&
    !(*root.inner_logic().val()).ref_count.has_event_invariant() &&
    storage.inner_logic().0.invariant() &&
    (*root.inner_logic().val()).vec@ != None &&
    (*root.inner_logic().val()).vec_model().matches_authority(storage.inner_logic().0) &&
    ledger.inner_logic().0.invariant() && ledger.inner_logic().0.is_live() &&
    ledger.inner_logic().0.agrees_with_value(2) && ledger.inner_logic().0.count() == 2 &&
    ledger.inner_logic().0.allocation_binding() == (
        storage.inner_logic().0.binding(), storage.inner_logic().0.raw_pointer(),
        storage.inner_logic().0.capacity()) &&
    ledger.inner_logic().0.ledger_id() == ledger.inner_logic().1.ledger_id() &&
    ledger.inner_logic().0.ledger_id() == ledger.inner_logic().2.ledger_id() &&
    ledger.inner_logic().1.invariant() && ledger.inner_logic().2.invariant() &&
    ledger.inner_logic().0.has_ticket(ledger.inner_logic().1) &&
    ledger.inner_logic().0.has_ticket(ledger.inner_logic().2) &&
    ledger.inner_logic().1.owner_id() != ledger.inner_logic().2.owner_id() &&
    ledger.inner_logic().1.owner_id() < ledger.inner_logic().0.next_id() &&
    ledger.inner_logic().2.owner_id() < ledger.inner_logic().0.next_id() &&
    ledger.inner_logic().1.class() == RefKind::Mutable &&
    ledger.inner_logic().2.class() == RefKind::Mutable &&
    ledger.inner_logic().1.allocation_binding() == ledger.inner_logic().0.allocation_binding() &&
    ledger.inner_logic().2.allocation_binding() == ledger.inner_logic().0.allocation_binding() &&
    ledger.inner_logic().1.owned_lo() == 0 && ledger.inner_logic().1.owned_hi() == *cut &&
    ledger.inner_logic().2.owned_lo() == *cut &&
    ledger.inner_logic().2.owned_hi() == storage.inner_logic().0.capacity() &&
    (forall<index: Int>
        active_mutable_contains(ledger.inner_logic().0.active_ids(), index) ==
            (0 <= index && index < storage.inner_logic().0.capacity())) &&
    storage.inner_logic().1.invariant() &&
    storage.inner_logic().1.namespace() == storage.inner_logic().0.binding() &&
    storage.inner_logic().1.capacity() == storage.inner_logic().0.capacity() &&
    storage.inner_logic().1.lo() == 0 &&
    storage.inner_logic().1.hi() == storage.inner_logic().1.capacity() &&
    0 <= *cut && *cut <= storage.inner_logic().1.capacity() &&
    base.invariant() &&
    base@ == Some((storage.inner_logic().1.namespace(), storage.inner_logic().1.capacity(), 0int)) &&
    base.raw_pointer() == storage.inner_logic().0.raw_pointer() &&
    left_cursor.invariant() && left_cursor@ != None &&
    left_cursor@.unwrap_logic().0 == storage.inner_logic().1.namespace() &&
    left_cursor@.unwrap_logic().1 == storage.inner_logic().1.capacity() &&
    left_cursor@.unwrap_logic().2 <= *cut &&
    left_cursor.current_address() == base.current_address() + left_cursor@.unwrap_logic().2 &&
    right_cursor.invariant() && right_cursor@ != None &&
    right_cursor@.unwrap_logic().0 == storage.inner_logic().1.namespace() &&
    right_cursor@.unwrap_logic().1 == storage.inner_logic().1.capacity() &&
    *cut <= right_cursor@.unwrap_logic().2 &&
    right_cursor@.unwrap_logic().2 <= storage.inner_logic().1.capacity() &&
    right_cursor.current_address() == base.current_address() + right_cursor@.unwrap_logic().2 &&
    phase.inner_logic().0.invariant() && phase.inner_logic().0.is_open() &&
    phase.inner_logic().1.invariant() &&
    phase.inner_logic().1.resource_id() == phase.inner_logic().0.resource_id()
)]
#[requires(context.inner_logic().invariant())]
#[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
#[ensures((^context.inner_logic()).invariant())]
#[ensures(result.0.capsule.inner_logic().val().namespace() == event_namespace())]
#[ensures(result.0.invariant() && result.1.invariant())]
#[ensures(result.0.capsule_public() == result.1.capsule_public())]
#[ensures(result.0.capsule_public() == (
    header_id as *mut (), (*root.inner_logic().val()).ref_count.counter_model(),
    ledger.inner_logic().0.ledger_id(), phase.inner_logic().0.resource_id()))]
#[ensures(result.0.capsule.inner_logic() == result.1.capsule.inner_logic())]
#[ensures(result.0.base == base && result.1.base == base)]
#[ensures(result.0.cursor == left_cursor && result.1.cursor == right_cursor)]
#[ensures(result.0.ticket.inner_logic() == ledger.inner_logic().1)]
#[ensures(result.1.ticket.inner_logic() == ledger.inner_logic().2)]
#[ensures(result.0.region.inner_logic().invariant() && result.1.region.inner_logic().invariant())]
#[ensures(result.0.region.inner_logic().namespace() == storage.inner_logic().1.namespace())]
#[ensures(result.1.region.inner_logic().namespace() == storage.inner_logic().1.namespace())]
#[ensures(result.0.region.inner_logic().capacity() == storage.inner_logic().1.capacity())]
#[ensures(result.1.region.inner_logic().capacity() == storage.inner_logic().1.capacity())]
#[ensures(result.0.region.inner_logic().resource_id() == storage.inner_logic().1.resource_id())]
#[ensures(result.1.region.inner_logic().resource_id() == storage.inner_logic().1.resource_id())]
#[ensures(result.0.region.inner_logic().lo() == 0 && result.0.region.inner_logic().hi() == *cut)]
#[ensures(result.1.region.inner_logic().lo() == *cut &&
    result.1.region.inner_logic().hi() == storage.inner_logic().1.capacity())]
#[ensures(forall<i: Int> 0 <= i && i < *cut ==>
    result.0.region.inner_logic().slot(i) == storage.inner_logic().1.slot(i))]
#[ensures(forall<i: Int> *cut <= i && i < storage.inner_logic().1.capacity() ==>
    result.1.region.inner_logic().slot(i) == storage.inner_logic().1.slot(i))]
#[ensures(result.0.ticket.inner_logic().owner_id() !=
    result.1.ticket.inner_logic().owner_id())]
#[ensures(result.0.ticket.inner_logic().owned_lo() == result.0.region.inner_logic().lo() &&
    result.0.ticket.inner_logic().owned_hi() == result.0.region.inner_logic().hi())]
#[ensures(result.1.ticket.inner_logic().owned_lo() == result.1.region.inner_logic().lo() &&
    result.1.ticket.inner_logic().owned_hi() == result.1.region.inner_logic().hi())]
#[ensures(result.0.lease_key() == result.1.lease_key())]
#[ensures(result.0.lease_key().resource_id() == result.0.lease_resource_id())]
#[ensures(result.0.lease_key().slot() == context.inner_logic().ready_len())]
#[ensures(context.inner_logic().registered_cap(
    (^context.inner_logic()), result.0.lease_key(),
    Some(Excl((result.0.lease_key().resource_id(), None)))
))]
#[check(terminates)]
pub(crate) fn make_count_two_packets(
    header_id: *mut Shared,
    root: Ghost<Box<Perm<*const Shared>>>,
    owner: Ghost<(Perm<NativeCounter>, usize)>,
    storage: Ghost<(StoredAuthority, PhysicalRegion)>,
    ledger: Ghost<(OwnerLedger, OwnerTicket, OwnerTicket)>,
    phase: Ghost<(PhaseAuthority, TerminalAction)>,
    base: BoundPtr,
    left_cursor: BoundPtr,
    right_cursor: BoundPtr,
    cut: Snapshot<Int>,
    context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
) -> (SharedOwnerPacket, SharedOwnerPacket) {
    let packet_parts = ghost! {
        let root = root.into_inner();
        let (owner, owner_value) = owner.into_inner();
        let (storage, region) = storage.into_inner();
        let (ledger, first_ticket, second_ticket) = ledger.into_inner();
        let (phase, action) = phase.into_inner();
        let cut = cut.into_ghost().into_inner();
        proof_assert!(owner_value == 2usize && owner.val() == 2usize);
        proof_assert!(ledger.count() == 2 && ledger.agrees_with_value(2));
        proof_assert!(first_ticket.owner_id() != second_ticket.owner_id());
        let pool = PhysicalPool::empty_from(&region);
        let (first_region, second_region) = region.split_at(cut);
        let ledger_id = snapshot!(ledger.ledger_id()).into_ghost().into_inner();
        let phase_id = snapshot!(phase.resource_id()).into_ghost().into_inner();
        let (lease, lease_key) = HeaderLeaseAuthority::new(
            context,
            header_id,
        ).into_inner();
        let header = snapshot!(root.val());
        let allocation_binding: Snapshot<AllocationBinding> = snapshot!((
            storage.binding(), storage.raw_pointer(), storage.capacity(),
        ));
        let state = SharedPayload {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            allocation_binding,
            header_vec: snapshot!(header.vec_model()),
            original_capacity_repr: snapshot!(header.original_capacity_repr),
            ledger_id,
            phase_id,
            lease_generation: 0int,
            lease_key,
            lease,
            root: Some(root),
            storage: Some(storage),
            pool: Some(pool),
            ledger: Some(ledger),
            retire: None,
            phase,
            terminal_action: Some(action),
            closed: false,
        };
        proof_assert!(state.invariant() && state.protocol());
        proof_assert!(state.counter_model() == *owner.ward());
        proof_assert!(state.agrees_with_value(2usize));
        let invariant = EventBoundInvariant::new(Ghost::new(owner), Ghost::new(state));
        let capsule = GhostShared::new(invariant).into_inner();
        let first_capsule = capsule.clone();
        (first_capsule, first_ticket, first_region, lease_key,
            capsule, second_ticket, second_region, lease_key)
    };
    let (first_capsule, first_ticket, first_region,
        first_lease_key, second_capsule, second_ticket, second_region,
        second_lease_key) = packet_parts.split();
    (
        SharedOwnerPacket {
            capsule: first_capsule,
            ticket: first_ticket,
            region: first_region,
            base,
            cursor: left_cursor,
            lease_key: first_lease_key,
        },
        SharedOwnerPacket {
            capsule: second_capsule,
            ticket: second_ticket,
            region: second_region,
            base,
            cursor: right_cursor,
            lease_key: second_lease_key,
        },
    )
}
