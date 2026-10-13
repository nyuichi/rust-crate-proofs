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
        Ghost, GhostShared,
    },
    logic::Id,
    prelude::*,
};

use crate::{
    ownership_proof::{
        raw_vec::{BoundPtr, PhysicalPool, PhysicalRegion},
        ref_count_ledger::{
            active_mutable_contains, sole_active_id, OwnerLedger, OwnerTicket, RefKind,
            RetirePermit,
        },
        terminal_phase::{PhaseAuthority, TerminalAction},
    },
    ref_count_ops::{CounterProtocol, EventBoundInvariant, NativeCounter},
    vec_storage::StoredAuthority,
};

use super::Shared;

/// Payload for the generic counter event invariant. The invariant owns the
/// sole native counter permission outside this value; this payload records
/// only its opaque model and the shared storage/reference ledgers.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedPayload {
    header_id: *mut Shared,
    counter_model: Snapshot<NativeCounter>,
    ledger_id: Id,
    phase_id: Id,
    root: Option<Box<Perm<*const Shared>>>,
    storage: Option<StoredAuthority>,
    pool: Option<PhysicalPool>,
    ledger: Option<OwnerLedger>,
    retire: Option<RetirePermit>,
    phase: PhaseAuthority,
    terminal_action: Option<TerminalAction>,
    closed: bool,
}

impl Protocol for SharedPayload {
    type Public = (*mut (), NativeCounter, Id, Id);

    #[logic(open(self))]
    fn public(self) -> Self::Public {
        (
            self.header_id as *mut (),
            *self.counter_model,
            self.ledger_id,
            self.phase_id,
        )
    }

    #[logic(open(self), prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
        self.phase.invariant() && self.phase.resource_id() == self.phase_id &&
        match (self.root, self.storage, self.pool, self.ledger,
            self.retire, self.terminal_action) {
            (Some(root), Some(storage), Some(pool), Some(ledger), None, Some(action)) => {
                let header = root.val();
                !self.closed && self.phase.is_open() &&
                root.invariant() && storage.invariant() && pool.invariant() &&
                ledger.invariant() && ledger.is_live() && ledger.count() > 0 &&
                ledger.ledger_id() == self.ledger_id &&
                action.invariant() &&
                action.resource_id() == self.phase_id &&
                *root.ward() == self.header_id as *const Shared &&
                header.ref_count.counter_model() == *self.counter_model &&
                !header.ref_count.has_event_invariant() &&
                ledger.allocation_binding() == (
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
                root.invariant() && storage.invariant() && pool.invariant() &&
                ledger.invariant() && !ledger.is_live() && ledger.count() == 0 &&
                retire.invariant() && retire.ledger_id() == self.ledger_id &&
                retire.active_ids().is_empty() &&
                ledger.ledger_id() == self.ledger_id &&
                ledger.allocation_binding() == (
                    storage.binding(), storage.raw_pointer(), storage.capacity()) &&
                *root.ward() == self.header_id as *const Shared &&
                header.ref_count.counter_model() == *self.counter_model &&
                !header.ref_count.has_event_invariant() &&
                pool.capacity() == storage.capacity() &&
                pool.namespace() == storage.binding() &&
                pool.resource_id() == storage.binding() &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < storage.capacity()
                )) &&
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
        self.phase.invariant() && inv(self.terminal_action)
    }
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
        let state = SharedPayload {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            ledger_id,
            phase_id,
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
        let state = SharedPayload {
            header_id,
            counter_model: snapshot!(*owner.ward()),
            ledger_id,
            phase_id,
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
