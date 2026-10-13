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
        resource::{Authority, Fragment},
        Ghost, GhostShared,
    },
    logic::{ra::{excl::Excl, RA}, Id},
    prelude::*,
};

use crate::{
    ownership_proof::{
        raw_vec::{BoundPtr, PhysicalPool, PhysicalRegion, Recovery},
        ref_count_ledger::{
            active_mutable_contains, sole_active_id, AllocationBinding, OwnerLedger,
            OwnerTicket, RefKind,
            RetirePermit,
        },
        terminal_phase::{PhaseAuthority, TerminalAction},
    },
    ref_count_ops::{CounterProtocol, EventBoundInvariant, NativeCounter},
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

type HeaderLeaseStamp = (
    Int,
    *mut Shared,
    NativeCounter,
    Id,
    Id,
    StoredAuthority,
    VecStorage,
    usize,
);
type HeaderLeaseState = Option<Excl<HeaderLeaseStamp>>;

/// The lease authority distinguishes an idle header, whose unit fragment stays
/// in the payload, from a leased header, whose active fragment travels with
/// the exact root permission. The fragment gates re-opening across arbitrary
/// atomic-invariant states; the resource does not count as a native reference.
#[allow(missing_debug_implementations)]
struct HeaderLeaseAuthority {
    resource_id: Id,
    authority: Authority<HeaderLeaseState>,
    idle_fragment: Option<Fragment<HeaderLeaseState>>,
}

#[logic(open)]
fn header_lease_stamp(
    generation: Int,
    header_id: *mut Shared,
    counter_model: NativeCounter,
    ledger_id: Id,
    phase_id: Id,
    storage: StoredAuthority,
    header_vec: VecStorage,
    original_capacity_repr: usize,
) -> HeaderLeaseStamp {
    (
        generation,
        header_id,
        counter_model,
        ledger_id,
        phase_id,
        storage,
        header_vec,
        original_capacity_repr,
    )
}

impl HeaderLeaseAuthority {
    #[logic(open(self))]
    fn resource_id(self) -> Id {
        self.resource_id
    }

    #[logic(open(self), prophetic)]
    fn is_idle(self) -> bool {
        pearlite! {
            self.authority@ == None && match self.idle_fragment {
                Some(fragment) => fragment@ == None,
                None => false,
            }
        }
    }

    #[logic(open(self), prophetic)]
    fn is_active(self, stamp: HeaderLeaseStamp) -> bool {
        pearlite! {
            self.authority@ == Some(Excl(stamp)) && self.idle_fragment == None
        }
    }

    #[check(ghost)]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().is_idle())]
    pub(crate) fn new() -> Ghost<Self> {
        ghost! {
            let mut authority: Authority<HeaderLeaseState> = Authority::alloc().into_inner();
            let unit: Snapshot<HeaderLeaseState> = snapshot!(None);
            let idle_fragment = authority.add_fragment(unit);
            let resource_id = authority.id_ghost();
            let lease = Self {
                resource_id,
                authority,
                idle_fragment: Some(idle_fragment),
            };
            proof_assert!(lease.invariant() && lease.is_idle());
            lease
        }
    }

    #[check(ghost)]
    #[requires(authority.inner_logic().invariant() && authority.inner_logic().is_idle())]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_active(*stamp))]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures(result.inner_logic().id() == authority.inner_logic().resource_id())]
    #[ensures(result.inner_logic()@ == Some(Excl(*stamp)))]
    fn activate(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
    ) -> Ghost<Fragment<HeaderLeaseState>> {
        ghost! {
            let authority = authority.into_inner();
            let mut fragment = authority.idle_fragment.take().unwrap();
            authority.authority.frag_lemma(&fragment);
            authority.authority.update(
                &mut fragment,
                snapshot!((Some(Excl(*stamp)), Some(Excl(*stamp)))),
            );
            proof_assert!(fragment@ == Some(Excl(*stamp)));
            proof_assert!(authority.authority@ == Some(Excl(*stamp)));
            fragment
        }
    }

    #[check(ghost)]
    #[requires(authority.inner_logic().invariant() &&
        authority.inner_logic().is_active(*stamp))]
    #[requires(fragment.inner_logic().id() == authority.inner_logic().resource_id())]
    #[requires(fragment.inner_logic()@ == Some(Excl(*stamp)))]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_idle())]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    fn deactivate(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
        fragment: Ghost<Fragment<HeaderLeaseState>>,
    ) -> Ghost<()> {
        ghost! {
            let authority = authority.into_inner();
            let mut fragment = fragment.into_inner();
            authority.authority.frag_lemma(&fragment);
            proof_assert!(fragment@.incl_eq(authority.authority@));
            proof_assert!(fragment@ == Some(Excl(*stamp)));
            proof_assert!(authority.authority@ == Some(Excl(*stamp)));
            authority.authority.update(&mut fragment, snapshot!((None, None)));
            proof_assert!(authority.authority@ == None && fragment@ == None);
            authority.idle_fragment = Some(fragment);
        }
    }

    /// The current authoritative lease must agree with the caller's affine
    /// fragment. In the idle case an active fragment is incompatible with the
    /// unit authority; in the leased case exclusivity forces the exact stamp.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(self.is_idle() || self.idle_fragment == None)]
    #[requires(fragment.id() == self.resource_id())]
    #[requires(fragment@ == Some(Excl(*stamp)))]
    #[ensures(self.is_active(*stamp))]
    fn require_active_fragment(
        &self,
        stamp: Snapshot<HeaderLeaseStamp>,
        fragment: &Fragment<HeaderLeaseState>,
    ) {
        self.authority.frag_lemma(fragment);
        match self.idle_fragment {
            Some(_) => {
                proof_assert!(self.is_idle());
                proof_assert!(fragment@.incl(self.authority@));
                proof_assert!(false);
            }
            None => {
                proof_assert!(fragment@.incl_eq(self.authority@));
                proof_assert!(self.authority@ == Some(Excl(*stamp)));
                proof_assert!(self.is_active(*stamp));
            }
        }
    }
}

impl Invariant for HeaderLeaseAuthority {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.authority.invariant() && self.authority.id() == self.resource_id &&
            match self.idle_fragment {
                Some(fragment) => fragment.id() == self.resource_id,
                None => true,
            }
        }
    }
}

#[logic(open)]
fn same_stored_authority(left: StoredAuthority, right: StoredAuthority) -> bool {
    pearlite! { left == right }
}

impl Protocol for SharedPayload {
    type Public = (*mut (), NativeCounter, Id, Id, Id, AllocationBinding);

    #[logic(open(self))]
    fn public(self) -> Self::Public {
        (
            self.header_id as *mut (),
            *self.counter_model,
            self.ledger_id,
            self.phase_id,
            self.lease.resource_id(),
            *self.allocation_binding,
        )
    }

    #[logic(open(self), prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
        self.phase.invariant() && self.phase.resource_id() == self.phase_id &&
        self.lease.invariant() && self.lease_generation >= 0 &&
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
                header.vec == *self.header_vec &&
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
                header.vec.matches_authority(storage)
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
                header.vec == *self.header_vec &&
                header.original_capacity_repr == *self.original_capacity_repr &&
                !header.ref_count.has_event_invariant() &&
                pool.capacity() == storage.capacity() &&
                pool.namespace() == storage.binding() &&
                pool.resource_id() == storage.binding() &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < storage.capacity()
                )) &&
                header.vec.matches_authority(storage)
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
/// fragment; it cannot expose a mutable header borrow or physical authority.
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
    lease_fragment: Fragment<HeaderLeaseState>,
}

impl SharedHeaderReadLoan {
    #[logic(open(self), prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.root.invariant() &&
            *self.root.ward() == self.header_id as *const Shared &&
            self.root.val().ref_count.counter_model() == *self.counter_model &&
            !self.root.val().ref_count.has_event_invariant() &&
            self.root.val().vec == *self.header_vec &&
            self.root.val().original_capacity_repr == *self.original_capacity_repr &&
            self.root.val().vec.matches_authority(*self.storage_model) &&
            self.generation > 0 &&
            self.lease_fragment.id() == self.lease_id &&
            self.lease_fragment@ == Some(Excl(header_lease_stamp(
                self.generation,
                self.header_id,
                *self.counter_model,
                self.ledger_id,
                self.phase_id,
                *self.storage_model,
                *self.header_vec,
                *self.original_capacity_repr,
            )))
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

    /// A present root is available only in an idle-lease branch of the
    /// payload protocol. Keep the implication explicit for quantified event
    /// callbacks.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol() && self.root != None)]
    #[ensures(self.lease.is_idle())]
    fn prove_root_some_implies_lease_idle(&self) {
        proof_assert!(self.lease.is_idle());
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
        loan.inner_logic().lease_fragment.id() ==
            payload.inner_logic().lease.resource_id() &&
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
            ));
            payload.lease.require_active_fragment(loan_stamp, &loan.lease_fragment);
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

    /// Move the unique header root into a sealed read loan. The payload keeps
    /// its storage and reference ledgers while recording exactly one active
    /// lease generation, so counter events can proceed without exposing a
    /// mutable header borrow.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().root != None && payload.inner_logic().lease.is_idle())]
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
        same_stored_authority(*result.inner_logic().storage_model,
            payload.inner_logic().storage.unwrap_logic()))]
    pub(crate) fn take_header_read_loan(
        payload: Ghost<&mut SharedPayload>,
    ) -> Ghost<SharedHeaderReadLoan> {
        ghost! {
            let payload = payload.into_inner();
            let root = payload.root.take().unwrap();
            let storage_model: Snapshot<StoredAuthority> =
                snapshot!(payload.storage.unwrap_logic());
            let lease_id = payload.lease.resource_id;
            let generation = payload.lease_generation + 1int;
            let stamp = snapshot!(header_lease_stamp(
                generation,
                payload.header_id,
                *payload.counter_model,
                payload.ledger_id,
                payload.phase_id,
                *storage_model,
                *payload.header_vec,
                *payload.original_capacity_repr,
            ));
            let lease_fragment = HeaderLeaseAuthority::activate(
                Ghost::new(&mut payload.lease), stamp,
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
                lease_fragment,
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
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.is_idle() &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed)]
    pub(crate) fn restore_header_read_loan(
        payload: Ghost<&mut SharedPayload>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<()> {
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
            ));
            SharedPayload::prove_current_header_lease(Ghost::new(&*payload), Ghost::new(&loan));
            let SharedHeaderReadLoan { root, lease_fragment, .. } = loan;
            HeaderLeaseAuthority::deactivate(
                Ghost::new(&mut payload.lease), stamp, Ghost::new(lease_fragment),
            );
            payload.root = Some(root);
            ()
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
}

impl SharedOwnerPacket {
    #[logic(open(self))]
    pub(crate) fn capsule_public(self) -> (*mut (), NativeCounter, Id, Id) {
        pearlite! {
            let payload = self.capsule.inner_logic().val().public().1;
            (payload.0, payload.1, payload.2, payload.3)
        }
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
    #[ensures(result == self.region.inner_logic().slot(index))]
    pub(crate) fn allocation_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! { self.region.inner_logic().slot(index) }
    }

    #[logic(open(self), prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.capsule.invariant() &&
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
    #[ensures(forall<i:Int> 0 <= i && i < self.region.inner_logic().capacity() ==>
        result.allocation_slot(i) == self.allocation_slot(i))]
    #[ensures(result.base == self.base)]
    #[ensures(result.cursor@ == Some((
        self.cursor@.unwrap_logic().0,
        self.cursor@.unwrap_logic().1,
        self.cursor@.unwrap_logic().2 + count@
    )))]
    #[ensures(result.cursor.current_address() == self.cursor.current_address() + count@)]
    #[ensures(result.cursor.raw_pointer().addr_logic()@ ==
        self.cursor.raw_pointer().addr_logic()@ + count@)]
    pub(crate) fn advance_cursor(self, count: usize) -> Self {
        let mut result = self;
        result.cursor = result.cursor.advance_within(count);
        result
    }

    /// Try to transfer the capsule's affine header root into a read-only
    /// counter loan. An already active lease yields `None`; it cannot be
    /// duplicated or reopened while the root is out.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(tokens.contains(
        self.capsule.inner_logic().val().namespace()
    ))]
    #[ensures((^self).invariant() &&
        (^self).capsule.inner_logic() == self.capsule.inner_logic() &&
        (^self).ticket.inner_logic() == self.ticket.inner_logic() &&
        (^self).region.inner_logic() == self.region.inner_logic() &&
        (^self).base == self.base && (^self).cursor == self.cursor &&
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
        tokens: Ghost<creusot_std::ghost::invariant::Tokens<'_>>,
    ) -> Ghost<Option<SharedHeaderReadLoan>> {
        ghost! {
            let capsule = *self.capsule;
            capsule.to_ref().open_with_tokens(
                tokens,
                ghost!(
                    #[requires(
                        payload.invariant() &&
                        payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value)
                    )]
                    #[ensures(
                        (^payload).public() == payload.public() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).protocol() &&
                        (^payload).agrees_with_value(*owner_value) &&
                        match result {
                            Some(loan) =>
                                loan.header_id == payload.header_id &&
                                loan.lease_id == payload.lease.resource_id() &&
                                *loan.counter_model == *payload.counter_model &&
                                (*loan.storage_model).binding() == (*payload.allocation_binding).0 &&
                                (*loan.storage_model).raw_pointer() == (*payload.allocation_binding).1 &&
                                (*loan.storage_model).capacity() == (*payload.allocation_binding).2,
                            None => true,
                        }
                    )]
                    |payload: &mut SharedPayload,
                     owner_model: Snapshot<NativeCounter>,
                     owner_value: Snapshot<usize>| {
                    if payload.root.is_some() {
                        payload.prove_root_some_implies_lease_idle();
                        Some(SharedPayload::take_header_read_loan(Ghost::new(payload)).into_inner())
                    } else {
                        None
                    }
                }),
            ).into_inner()
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
    #[requires(tokens.contains(
        self.capsule.inner_logic().val().namespace()
    ))]
    #[ensures((^self).invariant())]
    #[ensures((^self).capsule.inner_logic() == self.capsule.inner_logic())]
    #[ensures((^self).ticket.inner_logic() == self.ticket.inner_logic())]
    #[ensures((^self).region.inner_logic() == self.region.inner_logic())]
    #[ensures((^self).base == self.base && (^self).cursor == self.cursor)]
    pub(crate) fn try_restore_header_read_loan(
        &mut self,
        tokens: Ghost<creusot_std::ghost::invariant::Tokens<'_>>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<()> {
        ghost! {
            let capsule = *self.capsule;
            let expected_lease_id: Snapshot<Id> =
                snapshot!(self.lease_resource_id());
            capsule.to_ref().open_with_tokens(
                tokens,
                ghost!(
                    #[requires(
                        payload.invariant() &&
                        payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().4 == *expected_lease_id &&
                        loan.inner_logic().invariant() &&
                        loan.inner_logic().lease_id == *expected_lease_id
                    )]
                    #[ensures(
                        (^payload).public() == payload.public() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).protocol() &&
                        (^payload).agrees_with_value(*owner_value)
                    )]
                    move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>| {
                    payload.prove_public_lease_resource_id(expected_lease_id);
                    SharedPayload::restore_header_read_loan(Ghost::new(payload), loan);
                    ()
                }),
            ).into_inner()
        }
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
    (*root.inner_logic().val()).vec.matches_authority(storage.inner_logic().0) &&
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
#[ensures(result.capsule_public() ==
    (header_id as *mut (), (*root.inner_logic().val()).ref_count.counter_model(),
        ledger.inner_logic().0.ledger_id(), phase.inner_logic().0.resource_id()))]
#[ensures(result.base@ == base@ && result.cursor@ == cursor@)]
#[ensures(result.base.raw_pointer() == base.raw_pointer())]
#[ensures(result.cursor.raw_pointer() == cursor.raw_pointer())]
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
        let lease = HeaderLeaseAuthority::new().into_inner();
        let header = snapshot!(root.val());
        let allocation_binding: Snapshot<AllocationBinding> = snapshot!((
            storage.binding(), storage.raw_pointer(), storage.capacity(),
        ));
        let state = SharedPayload {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            allocation_binding,
            header_vec: snapshot!(header.vec),
            original_capacity_repr: snapshot!(header.original_capacity_repr),
            ledger_id,
            phase_id,
            lease_generation: 0int,
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
        (capsule, ticket, region)
    };
    let (capsule, ticket, region) = packet_parts.split();
    SharedOwnerPacket { capsule, ticket, region, base, cursor }
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
    (*root.inner_logic().val()).vec.matches_authority(storage.inner_logic().0) &&
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
        let lease = HeaderLeaseAuthority::new().into_inner();
        let header = snapshot!(root.val());
        let allocation_binding: Snapshot<AllocationBinding> = snapshot!((
            storage.binding(), storage.raw_pointer(), storage.capacity(),
        ));
        let state = SharedPayload {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            allocation_binding,
            header_vec: snapshot!(header.vec),
            original_capacity_repr: snapshot!(header.original_capacity_repr),
            ledger_id,
            phase_id,
            lease_generation: 0int,
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
        (first_capsule, first_ticket, first_region,
            capsule, second_ticket, second_region)
    };
    let (first_capsule, first_ticket, first_region,
        second_capsule, second_ticket, second_region) = packet_parts.split();
    (
        SharedOwnerPacket {
            capsule: first_capsule,
            ticket: first_ticket,
            region: first_region,
            base,
            cursor: left_cursor,
        },
        SharedOwnerPacket {
            capsule: second_capsule,
            ticket: second_ticket,
            region: second_region,
            base,
            cursor: right_cursor,
        },
    )
}
