//! Proof-only ownership capsule for a `BytesMut` shared control block.
//!
//! This module does not change the native `Shared` layout. Its capsule owns
//! the exact final header root, counter permission, storage authority, active
//! owner ledger, and one-shot phase authority. Per-handle ticket and physical
//! region remain in `SharedOwnerPacket`, outside the shared control block.

use alloc::boxed::Box;
use creusot_std::{
    ghost::{
        invariant::{declare_namespace, AtomicInvariant, Protocol},
        perm::Perm,
        Ghost, GhostShared,
    },
    logic::Id,
    prelude::*,
};

use crate::{
    ownership_proof::{
        raw_vec::{BoundPtr, PhysicalRegion},
        ref_count_ledger::{OwnerLedger, OwnerTicket, RetirePermit},
        terminal_phase::{PhaseAuthority, TerminalAction},
    },
    ref_count_ops::NativeCounter,
    vec_storage::StoredAuthority,
};

use super::Shared;

declare_namespace! { BYTES_MUT_SHARED_CAPSULE }

/// One stable control-block protocol. Native header, counter, and allocation
/// metadata stay in their actual fields; this value only owns their ghost
/// permissions and the current protocol state.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedState {
    header_id: *mut Shared,
    counter_model: Snapshot<NativeCounter>,
    ledger_id: Id,
    phase_id: Id,
    root: Option<Box<Perm<*const Shared>>>,
    counter_owner: Option<Perm<NativeCounter>>,
    storage: Option<StoredAuthority>,
    ledger: Option<OwnerLedger>,
    retire: Option<RetirePermit>,
    phase: PhaseAuthority,
    terminal_action: Option<TerminalAction>,
    value: usize,
    closed: bool,
}

impl Protocol for SharedState {
    type Public = (*mut (), NativeCounter, Id, Id);

    #[logic(open(self))]
    fn public(self) -> Self::Public {
        (self.header_id as *mut (), *self.counter_model, self.ledger_id, self.phase_id)
    }

    #[logic(open(self), prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
        self.phase.invariant() && self.phase.resource_id() == self.phase_id &&
        match (self.root, self.counter_owner, self.storage, self.ledger,
            self.retire, self.terminal_action) {
            (Some(root), Some(owner), Some(storage), Some(ledger), None, Some(action)) => {
                let header = root.val();
                !self.closed && self.value@ > 0 && self.phase.is_open() &&
                root.invariant() && owner.invariant() && storage.invariant() &&
                ledger.invariant() && ledger.is_live() && ledger.ledger_id() == self.ledger_id &&
                action.invariant() &&
                action.resource_id() == self.phase_id &&
                *root.ward() == self.header_id as *const Shared &&
                header.ref_count.counter_model() == *owner.ward() &&
                header.ref_count.counter_model() == *self.counter_model &&
                !header.ref_count.has_event_invariant() &&
                owner.val() == self.value &&
                ledger.agrees_with_value(self.value@) && ledger.count() > 0 &&
                header.vec.matches_authority(storage)
            },
            (Some(root), Some(owner), Some(storage), None, Some(retire), None) => {
                let header = root.val();
                !self.closed && self.value@ == 0 && self.phase.is_open() &&
                root.invariant() && owner.invariant() && storage.invariant() &&
                retire.invariant() && retire.ledger_id() == self.ledger_id &&
                retire.active_ids().is_empty() &&
                *root.ward() == self.header_id as *const Shared &&
                header.ref_count.counter_model() == *owner.ward() &&
                header.ref_count.counter_model() == *self.counter_model &&
                !header.ref_count.has_event_invariant() && owner.val()@ == 0 &&
                header.vec.matches_authority(storage)
            },
            (None, None, None, None, Some(retire), None) =>
                self.closed && self.phase.is_closed() && retire.invariant() &&
                retire.ledger_id() == self.ledger_id && retire.active_ids().is_empty(),
            _ => false,
        }
        }
    }
}

impl Invariant for SharedState {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        inv(self.root) && inv(self.counter_owner) && inv(self.storage) &&
        inv(self.ledger) && inv(self.retire) && self.phase.invariant() && inv(self.terminal_action)
    }
}

/// A per-handle affine registration and physical view paired with the shared
/// capsule. The program pointers are B1-bound metadata, never reconstructed
/// from ghost raw-allocation descriptors.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedOwnerPacket {
    pub(crate) capsule: Ghost<GhostShared<AtomicInvariant<SharedState>>>,
    pub(crate) ticket: Ghost<OwnerTicket>,
    pub(crate) region: Ghost<PhysicalRegion>,
    pub(crate) base: BoundPtr,
    pub(crate) cursor: BoundPtr,
}

impl SharedOwnerPacket {
    #[logic(open(self))]
    pub(crate) fn capsule_public(self) -> (*mut (), NativeCounter, Id, Id) {
        pearlite! { self.capsule.inner_logic().val().public() }
    }

    #[logic(prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.capsule.invariant() &&
            self.ticket.inner_logic().invariant() &&
            self.ticket.inner_logic().ledger_id() == self.capsule_public().2 &&
            self.region.inner_logic().invariant() &&
            self.base.invariant() && self.cursor.invariant() &&
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

/// Build the first shared-owner capsule after the native header has reached
/// its final Box address. The constructor consumes every authority exactly
/// once and binds the returned packet to the exact counter field and B1
/// storage namespace supplied by the generic constructors.
#[requires(
    root.inner_logic().invariant() &&
    *root.inner_logic().ward() == header_id as *const Shared &&
    owner.inner_logic().invariant() &&
    (*root.inner_logic().val()).ref_count.counter_model() == *owner.inner_logic().ward() &&
    owner.inner_logic().val() == value &&
    !(*root.inner_logic().val()).ref_count.has_event_invariant() &&
    storage.inner_logic().invariant() &&
    (*root.inner_logic().val()).vec.matches_authority(storage.inner_logic()) &&
    ledger.inner_logic().invariant() && ledger.inner_logic().is_live() &&
    ledger.inner_logic().agrees_with_value(value@) && ledger.inner_logic().count() == 1 &&
    value@ == 1 &&
    ledger.inner_logic().ledger_id() == ticket.inner_logic().ledger_id() &&
    ticket.inner_logic().invariant() &&
    ledger.inner_logic().has_ticket(ticket.inner_logic()) &&
    0 <= ticket.inner_logic().owner_id() &&
    ticket.inner_logic().owner_id() < ledger.inner_logic().next_id() &&
    region.inner_logic().invariant() &&
    region.inner_logic().namespace() == storage.inner_logic().binding() &&
    region.inner_logic().capacity() == storage.inner_logic().capacity() &&
    region.inner_logic().lo() == 0 &&
    region.inner_logic().hi() == region.inner_logic().capacity() &&
    base.invariant() &&
    base@ == Some((region.inner_logic().namespace(), region.inner_logic().capacity(), 0int)) &&
    base.raw_pointer() == storage.inner_logic().raw_pointer() &&
    cursor.invariant() && cursor@ != None &&
    cursor@.unwrap_logic().0 == region.inner_logic().namespace() &&
    cursor@.unwrap_logic().1 == region.inner_logic().capacity() &&
    cursor.current_address() == base.current_address() + cursor@.unwrap_logic().2 &&
    cursor@.unwrap_logic().2 <= region.inner_logic().hi() &&
    phase.inner_logic().invariant() && phase.inner_logic().is_open() &&
    action.inner_logic().invariant() &&
    action.inner_logic().resource_id() == phase.inner_logic().resource_id()
)]
#[ensures(result.capsule_public() ==
    (header_id as *mut (), (*root.inner_logic().val()).ref_count.counter_model(), ledger.inner_logic().ledger_id(), phase.inner_logic().resource_id()))]
#[ensures(result.base@ == base@ && result.cursor@ == cursor@)]
#[ensures(result.base.raw_pointer() == base.raw_pointer())]
#[ensures(result.cursor.raw_pointer() == cursor.raw_pointer())]
#[ensures(result.ticket.inner_logic() == ticket.inner_logic())]
#[ensures(result.region.inner_logic() == region.inner_logic())]
#[ensures(ledger.inner_logic().ledger_id() == result.capsule_public().2)]
#[ensures(ledger.inner_logic().has_ticket(result.ticket.inner_logic()))]
#[ensures(result.ticket.inner_logic().owner_id() >= 0)]
#[ensures(result.ticket.inner_logic().owner_id() < ledger.inner_logic().next_id())]
#[ensures(result.region.inner_logic().namespace() == result.base@.unwrap_logic().0)]
#[ensures(result.region.inner_logic().capacity() == result.base@.unwrap_logic().1)]
#[ensures(result.region.inner_logic().capacity() == storage.inner_logic().capacity())]
#[ensures(result.base.raw_pointer() == storage.inner_logic().raw_pointer())]
#[ensures(result.cursor.current_address() == result.base.current_address() +
    result.cursor@.unwrap_logic().2)]
#[ensures(result.region.inner_logic().lo() == 0)]
#[ensures(result.region.inner_logic().hi() == result.region.inner_logic().capacity())]
#[ensures(result.invariant())]
#[check(terminates)]
pub(crate) fn make_count_one_packet(
    header_id: *mut Shared,
    root: Ghost<Box<Perm<*const Shared>>>,
    owner: Ghost<Perm<NativeCounter>>,
    storage: Ghost<StoredAuthority>,
    ledger: Ghost<OwnerLedger>,
    ticket: Ghost<OwnerTicket>,
    region: Ghost<PhysicalRegion>,
    base: BoundPtr,
    cursor: BoundPtr,
    phase: Ghost<PhaseAuthority>,
    action: Ghost<TerminalAction>,
    value: usize,
) -> SharedOwnerPacket {
    let capsule = ghost! {
        let root = root.into_inner();
        let owner = owner.into_inner();
        let storage = storage.into_inner();
        let ledger = ledger.into_inner();
        let phase = phase.into_inner();
        let action = action.into_inner();
        let ledger_id = snapshot!(ledger.ledger_id()).into_ghost().into_inner();
        let phase_id = snapshot!(phase.resource_id()).into_ghost().into_inner();
        let state = SharedState {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            ledger_id,
            phase_id,
            root: Some(root),
            counter_owner: Some(owner),
            storage: Some(storage),
            ledger: Some(ledger),
            retire: None,
            phase,
            terminal_action: Some(action),
            value,
            closed: false,
        };
        proof_assert!(state.invariant() && state.protocol());
        let invariant = AtomicInvariant::new(
            Ghost::new(state),
            snapshot!(BYTES_MUT_SHARED_CAPSULE()),
        );
        GhostShared::new(invariant).into_inner()
    };
    SharedOwnerPacket { capsule, ticket, region, base, cursor }
}
