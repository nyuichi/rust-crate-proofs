//! Proof-only ownership capsule for a `BytesMut` shared control block.
//!
//! This module does not change the native `Shared` layout. Its generic event
//! invariant owns the one counter permission; the payload owns header/storage
//! authority, the all-reference ledger, pool, and phase state.
//! Per-handle tickets and physical regions remain outside the control block.

use alloc::boxed::Box;
use creusot_std::{
    ghost::{
        invariant::{Namespace, Protocol},
        perm::Perm,
        resource::Authority,
        Ghost, GhostShared,
    },
    logic::{ra::{excl::Excl, RA}, FMap, Id},
    prelude::*,
};

use crate::{
    ownership_proof::{
        raw_vec::{slot_known, BoundPtr, PhysicalPool, PhysicalRegion, Recovery},
        ref_count_ledger::{
            active_mutable_contains, sole_active_id, ActiveReferences,
            AllocationBinding, OwnerLedger, prove_no_active_mutable_on_empty,
            prove_owner_readonly_to_mutable_effect,
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
    /// An active physical-pool lend is marked independently from the header
    /// lease. Ordinary operations keep this empty; the later region-loan
    /// transition must carry the matching actual `PhysicalRegion`.
    lent_span: Option<LentSpan>,
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

pub(crate) type LentSpan = (Int, Int, Int);

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
    Option<LentSpan>,
);
/// One exclusive value distinguishes idle from every active generation. In
/// particular, an idle capability cannot compose with an active authority by
/// treating an absent second tuple component as the unit of a product RA.
pub(crate) type HeaderLeaseState = Option<Excl<(Id, Option<HeaderLeaseStamp>, Int)>>;

/// The exclusive header capability carries the live ledger cardinality.
/// These predicates let callers establish an idle/active state without
/// guessing that count; opening the capsule couples the value to its actual
/// OwnerLedger before it can be used.
#[logic(open)]
pub(crate) fn header_lease_idle(value: HeaderLeaseState, resource_id: Id) -> bool {
    pearlite! {
        exists<count: Int> 0 <= count &&
            value == Some(Excl((resource_id, None, count)))
    }
}

#[logic(open)]
pub(crate) fn header_lease_ready_entry_idle(
    entry: Option<(Id, Int, Id, HeaderLeaseState)>,
    resource_id: Id,
    slot: Int,
    fragment_id: Id,
) -> bool {
    pearlite! {
        match entry {
            Some((actual_resource_id, actual_slot, actual_fragment_id, value)) =>
                actual_resource_id == resource_id && actual_slot == slot &&
                actual_fragment_id == fragment_id &&
                exists<count: Int> 0 <= count &&
                    value == Some(Excl((fragment_id, None, count))),
            None => false,
        }
    }
}

/// Value-only count observation for the real capability stored in a Context
/// entry. The `ReadyCap` remains the authority; this projection exposes no
/// permission or way to update the entry.
#[logic(open)]
pub(crate) fn header_lease_ready_entry_count(
    entry: Option<(Id, Int, Id, HeaderLeaseState)>,
    resource_id: Id,
    slot: Int,
) -> Int {
    pearlite! {
        match entry {
            Some((actual_resource_id, actual_slot, fragment_id, value)) =>
                if actual_resource_id == resource_id && actual_slot == slot &&
                    fragment_id == resource_id {
                    match value {
                        Some(Excl((_, _, count))) => count,
                        None => -1,
                    }
                } else { -1 },
            None => -1,
        }
    }
}

#[logic(open)]
fn header_lease_active(
    value: HeaderLeaseState,
    resource_id: Id,
    stamp: HeaderLeaseStamp,
) -> bool {
    pearlite! {
        exists<count: Int> value == Some(Excl((resource_id, Some(stamp), count)))
    }
}

#[logic(open)]
fn header_lease_count(value: HeaderLeaseState) -> Int {
    pearlite! {
        match value {
            Some(Excl((_, _, count))) => count,
            None => -1,
        }
    }
}

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
    lent_span: Option<LentSpan>,
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
        lent_span,
    )
}

#[logic(open(self), prophetic)]
fn no_active_lend(payload: SharedPayload) -> bool {
    pearlite! { payload.lent_span == None }
}

#[logic(open(self), prophetic)]
fn no_active_lend_loan(loan: SharedHeaderReadLoan) -> bool {
    pearlite! { loan.state.lent_span == None && loan.state.lent_region == None }
}

#[logic(open(self), prophetic)]
fn no_active_lend_state(state: &SharedHeaderReadLoanState) -> bool {
    pearlite! { state.lent_span == None && state.lent_region == None }
}

/// Exact pool state while the header lease is active. In the ordinary case,
/// every Readonly span remains in the pool. A physical lend removes exactly
/// one complete active Readonly ticket span; the affine loan carries the
/// corresponding `PhysicalRegion`.
#[logic(opaque, prophetic)]
#[ensures(result == (
    pool.capacity() == capacity && pool.namespace() == binding &&
    pool.resource_id() == binding &&
    match lent_span {
        None =>
            (forall<index: Int> pool.contains(index) == (
                0 <= index && index < capacity &&
                !active_mutable_contains(ledger.active_ids(), index)
            )) &&
            (forall<owner_id: Int> match ledger.active_ids().get(owner_id) {
                Some(Excl((_, RefKind::Readonly, lo, hi))) =>
                    forall<index: Int> lo <= index && index < hi ==>
                        pool.contains(index) && slot_known(pool.slot(index)),
                _ => true,
            }),
        Some((owner_id, lo, hi)) =>
            0 <= lo && lo <= hi && hi <= capacity &&
            ledger.active_ids().get(owner_id) ==
                Some(Excl((ledger.allocation_binding(), RefKind::Readonly, lo, hi))) &&
            (forall<index: Int> pool.contains(index) == (
                0 <= index && index < capacity &&
                !active_mutable_contains(ledger.active_ids(), index) &&
                !(lo <= index && index < hi)
            )) &&
            (forall<other_owner: Int> match ledger.active_ids().get(other_owner) {
                Some(Excl((_, RefKind::Readonly, other_lo, other_hi))) =>
                    forall<index: Int> other_lo <= index && index < other_hi &&
                        !(lo <= index && index < hi) ==>
                            pool.contains(index) && slot_known(pool.slot(index)),
                _ => true,
            }),
    }
))]
pub(crate) fn active_lease_pool_protocol(
    pool: PhysicalPool,
    ledger: OwnerLedger,
    capacity: Int,
    binding: Id,
    lent_span: Option<LentSpan>,
) -> bool {
    pearlite! {
        pool.capacity() == capacity && pool.namespace() == binding &&
        pool.resource_id() == binding &&
        match lent_span {
            None =>
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < capacity &&
                    !active_mutable_contains(ledger.active_ids(), index)
                )) &&
                (forall<owner_id: Int> match ledger.active_ids().get(owner_id) {
                    Some(Excl((_, RefKind::Readonly, lo, hi))) =>
                        forall<index: Int> lo <= index && index < hi ==>
                            pool.contains(index) && slot_known(pool.slot(index)),
                    _ => true,
                }),
            Some((owner_id, lo, hi)) =>
                0 <= lo && lo <= hi && hi <= capacity &&
                ledger.active_ids().get(owner_id) ==
                    Some(Excl((ledger.allocation_binding(), RefKind::Readonly, lo, hi))) &&
                (forall<index: Int> pool.contains(index) == (
                    0 <= index && index < capacity &&
                    !active_mutable_contains(ledger.active_ids(), index) &&
                    !(lo <= index && index < hi)
                )) &&
                (forall<other_owner: Int> match ledger.active_ids().get(other_owner) {
                    Some(Excl((_, RefKind::Readonly, other_lo, other_hi))) =>
                        forall<index: Int> other_lo <= index && index < other_hi &&
                            !(lo <= index && index < hi) ==>
                                pool.contains(index) && slot_known(pool.slot(index)),
                    _ => true,
                }),
        }
    }
}

/// Move one active Readonly ticket's known span out of a stable pool and
/// establish the corresponding active-lease pool relation. The pool and
/// region are the results of the actual affine interval split; the protocol
/// transition is derived from the old pool frame and the same active ledger.
#[check(ghost)]
#[requires(pool.invariant())]
#[requires(active_lease_pool_protocol(
    pool, *ledger, (*storage).capacity(), (*storage).binding(), None,
))]
#[requires(ledger.active_ids().get(owner_id) == Some(Excl((
    ledger.allocation_binding(), RefKind::Readonly, lo, hi,
))))]
#[requires(0 <= lo && lo <= hi && hi <= (*storage).capacity())]
#[requires(forall<index: Int> lo <= index && index < hi ==>
    pool.contains(index) && slot_known(pool.slot(index)))]
#[ensures(active_lease_pool_protocol(
    result.0, *ledger, (*storage).capacity(), (*storage).binding(),
    Some((owner_id, lo, hi)),
))]
#[ensures(exact_pool_region_interval_split(
    pool, result.0, result.1, lo, hi,
))]
#[ensures(forall<index: Int> lo <= index && index < hi ==>
    slot_known(result.1.slot(index)))]
pub(crate) fn lend_active_readonly_pool_interval(
    pool: PhysicalPool,
    ledger: Snapshot<OwnerLedger>,
    storage: Snapshot<StoredAuthority>,
    owner_id: Int,
    lo: Int,
    hi: Int,
) -> (PhysicalPool, PhysicalRegion) {
    let old_pool: Snapshot<PhysicalPool> = snapshot!(pool);
    let (new_pool, region) = pool.lend_interval(lo, hi);
    proof_assert!(active_lease_pool_protocol(
        new_pool, *ledger, (*storage).capacity(), (*storage).binding(),
        Some((owner_id, lo, hi)),
    ));
    proof_assert!(exact_pool_region_interval_split(*old_pool, new_pool, region, lo, hi));
    (new_pool, region)
}

/// Return the actual lent Readonly region to its complementary pool and
/// re-establish the stable pool relation. This is the inverse affine
/// interval transfer; the marker and authoritative ticket entry must match
/// the same span, and Known slots come from the carried region itself.
#[check(ghost)]
#[requires(pool.invariant() && region.invariant())]
#[requires(ledger.invariant() && ledger.is_live())]
#[requires(active_lease_pool_protocol(
    pool, *ledger, (*storage).capacity(), (*storage).binding(),
    Some((owner_id, lo, hi)),
))]
#[requires(ledger.active_ids().get(owner_id) == Some(Excl((
    ledger.allocation_binding(), RefKind::Readonly, lo, hi,
))))]
#[requires(region.lo() == lo && region.hi() == hi)]
#[requires(region.capacity() == (*storage).capacity() &&
    region.namespace() == (*storage).binding() &&
    region.resource_id() == (*storage).binding())]
#[requires(forall<index: Int> lo <= index && index < hi ==>
    slot_known(region.slot(index)))]
#[ensures(active_lease_pool_protocol(
    result, *ledger, (*storage).capacity(), (*storage).binding(), None,
))]
#[ensures(result.capacity() == pool.capacity())]
#[ensures(result.namespace() == pool.namespace())]
#[ensures(result.resource_id() == pool.resource_id())]
#[ensures(forall<index: Int> result.contains(index) ==
    (pool.contains(index) || lo <= index && index < hi))]
#[ensures(forall<index: Int> result.slot(index) ==
    if pool.contains(index) { pool.slot(index) }
    else if lo <= index && index < hi { region.slot(index) }
    else { None })]
pub(crate) fn retire_active_readonly_pool_interval(
    pool: PhysicalPool,
    region: PhysicalRegion,
    ledger: &OwnerLedger,
    storage: Snapshot<StoredAuthority>,
    owner_id: Int,
    lo: Int,
    hi: Int,
) -> PhysicalPool {
    let old_pool: Snapshot<PhysicalPool> = snapshot!(pool);
    let old_region: Snapshot<PhysicalRegion> = snapshot!(region);
    ledger.prove_readonly_owner_interval_excluded_from_mutable_coverage(
        owner_id, lo, hi,
    );
    let result = pool.retire(region);
    proof_assert!(active_lease_pool_protocol(
        result, *ledger, (*storage).capacity(), (*storage).binding(), None,
    ));
    proof_assert!(result.capacity() == (*old_pool).capacity());
    proof_assert!(result.namespace() == (*old_pool).namespace());
    proof_assert!(result.resource_id() == (*old_pool).resource_id());
    proof_assert!(forall<index: Int> result.contains(index) ==
        ((*old_pool).contains(index) || lo <= index && index < hi));
    proof_assert!(forall<index: Int> result.slot(index) ==
        if (*old_pool).contains(index) { (*old_pool).slot(index) }
        else if lo <= index && index < hi { (*old_region).slot(index) }
        else { None });
    result
}

#[logic(opaque, prophetic)]
#[ensures(result == (match (lent_span, region) {
    (None, None) => true,
    (Some((_, lo, hi)), Some(region)) =>
        region.invariant() && region.lo() == lo && region.hi() == hi &&
        region.namespace() == storage.binding() &&
        region.resource_id() == storage.binding() &&
        region.capacity() == storage.capacity() &&
        forall<index: Int> lo <= index && index < hi ==>
            slot_known(region.slot(index)),
    _ => false,
}))]
pub(crate) fn lent_region_matches(
    lent_span: Option<LentSpan>,
    region: Option<PhysicalRegion>,
    storage: StoredAuthority,
) -> bool {
    pearlite! {
        match (lent_span, region) {
            (None, None) => true,
            (Some((_, lo, hi)), Some(region)) =>
                region.invariant() && region.lo() == lo && region.hi() == hi &&
                region.namespace() == storage.binding() &&
                region.resource_id() == storage.binding() &&
                region.capacity() == storage.capacity() &&
                forall<index: Int> lo <= index && index < hi ==>
                    slot_known(region.slot(index)),
            _ => false,
        }
    }
}

/// Payload facts which are unchanged when the header lease is activated.
#[logic(opaque, prophetic)]
#[ensures(result == (
    after.invariant() && after.protocol() && after.root == None &&
    after.public() == before.public() &&
    after.counter_model == before.counter_model &&
    after.allocation_binding == before.allocation_binding &&
    after.header_vec == before.header_vec &&
    after.original_capacity_repr == before.original_capacity_repr &&
    after.ledger_id == before.ledger_id && after.phase_id == before.phase_id &&
    after.storage == before.storage && after.ledger == before.ledger &&
    after.retire == before.retire && after.phase == before.phase &&
    after.terminal_action == before.terminal_action &&
    after.lease_key == before.lease_key &&
    after.lease.resource_id() == before.lease.resource_id() &&
    after.closed == before.closed &&
    after.lease_generation == before.lease_generation + 1 &&
    after.lent_span == Some((ticket.owner_id(), ticket.owned_lo(), ticket.owned_hi()))
))]
fn header_region_payload_frame(
    before: SharedPayload,
    after: SharedPayload,
    ticket: OwnerTicket,
) -> bool {
    pearlite! {
        after.invariant() && after.protocol() && after.root == None &&
        after.public() == before.public() &&
        after.counter_model == before.counter_model &&
        after.allocation_binding == before.allocation_binding &&
        after.header_vec == before.header_vec &&
        after.original_capacity_repr == before.original_capacity_repr &&
        after.ledger_id == before.ledger_id && after.phase_id == before.phase_id &&
        after.storage == before.storage && after.ledger == before.ledger &&
        after.retire == before.retire && after.phase == before.phase &&
        after.terminal_action == before.terminal_action &&
        after.lease_key == before.lease_key &&
        after.lease.resource_id() == before.lease.resource_id() &&
        after.closed == before.closed &&
        after.lease_generation == before.lease_generation + 1 &&
        after.lent_span == Some((ticket.owner_id(), ticket.owned_lo(), ticket.owned_hi()))
    }
}

/// The returned loan carries the exact root and active-capability identity,
/// while the ticket is the live readonly owner whose span was lent.
#[logic(opaque, prophetic)]
#[ensures(result == (
    before.root != None && before.ledger != None &&
    ticket.class() == RefKind::Readonly && ticket.ledger_id() == before.ledger_id &&
    ticket.allocation_binding() == *before.allocation_binding &&
    before.ledger.unwrap_logic().active_ids().get(ticket.owner_id()) ==
        Some(Excl((ticket.allocation_binding(), ticket.class(),
            ticket.owned_lo(), ticket.owned_hi()))) &&
    loan.invariant() && Some(loan.root) == before.root &&
    loan.state.generation == after.lease_generation &&
    loan.state.header_id == before.header_id &&
    loan.state.counter_model == before.counter_model &&
    loan.state.header_vec == before.header_vec &&
    loan.state.original_capacity_repr == before.original_capacity_repr &&
    same_stored_authority(*loan.state.storage_model, before.storage.unwrap_logic()) &&
    loan.state.ledger_id == before.ledger_id && loan.state.phase_id == before.phase_id &&
    loan.state.lease_id == before.lease.resource_id() &&
    loan.state.lease_key == before.lease_key &&
    loan.state.lease_cap.key() == before.lease_key &&
    loan.state.lease_cap.fragment_id() == before.lease.resource_id() &&
    loan.state.lent_span == after.lent_span &&
    lent_region_matches(loan.state.lent_span, loan.state.lent_region, *loan.state.storage_model)
))]
fn header_region_ticket_loan_frame(
    before: SharedPayload,
    after: SharedPayload,
    ticket: OwnerTicket,
    loan: SharedHeaderReadLoan,
) -> bool {
    pearlite! {
        before.root != None && before.ledger != None &&
        ticket.class() == RefKind::Readonly && ticket.ledger_id() == before.ledger_id &&
        ticket.allocation_binding() == *before.allocation_binding &&
        before.ledger.unwrap_logic().active_ids().get(ticket.owner_id()) ==
            Some(Excl((ticket.allocation_binding(), ticket.class(),
                ticket.owned_lo(), ticket.owned_hi()))) &&
        loan.invariant() && Some(loan.root) == before.root &&
        loan.state.generation == after.lease_generation &&
        loan.state.header_id == before.header_id &&
        loan.state.counter_model == before.counter_model &&
        loan.state.header_vec == before.header_vec &&
        loan.state.original_capacity_repr == before.original_capacity_repr &&
        same_stored_authority(*loan.state.storage_model, before.storage.unwrap_logic()) &&
        loan.state.ledger_id == before.ledger_id && loan.state.phase_id == before.phase_id &&
        loan.state.lease_id == before.lease.resource_id() &&
        loan.state.lease_key == before.lease_key &&
        loan.state.lease_cap.key() == before.lease_key &&
        loan.state.lease_cap.fragment_id() == before.lease.resource_id() &&
        loan.state.lent_span == after.lent_span &&
        lent_region_matches(loan.state.lent_span, loan.state.lent_region, *loan.state.storage_model)
    }
}

/// The pool keeps every other slot unchanged, including Unknown-valued slots
/// outside the lent interval, while the actual region owns the ticket span.
#[logic(opaque, prophetic)]
#[ensures(result == (
    new_pool.capacity() == old_pool.capacity() &&
    new_pool.namespace() == old_pool.namespace() &&
    new_pool.resource_id() == old_pool.resource_id() &&
    region.invariant() && region.capacity() == old_pool.capacity() &&
    region.namespace() == old_pool.namespace() &&
    region.resource_id() == old_pool.resource_id() &&
    region.lo() == lo && region.hi() == hi &&
    (forall<index: Int> new_pool.contains(index) ==
        (old_pool.contains(index) && !(lo <= index && index < hi))) &&
    (forall<index: Int> new_pool.slot(index) ==
        if old_pool.contains(index) && !(lo <= index && index < hi) {
            old_pool.slot(index)
        } else { None }) &&
    (forall<index: Int> region.slot(index) ==
        if lo <= index && index < hi { old_pool.slot(index) } else { None })
))]
fn exact_pool_region_interval_split(
    old_pool: PhysicalPool,
    new_pool: PhysicalPool,
    region: PhysicalRegion,
    lo: Int,
    hi: Int,
) -> bool {
    pearlite! {
        new_pool.capacity() == old_pool.capacity() &&
        new_pool.namespace() == old_pool.namespace() &&
        new_pool.resource_id() == old_pool.resource_id() &&
        region.invariant() && region.capacity() == old_pool.capacity() &&
        region.namespace() == old_pool.namespace() &&
        region.resource_id() == old_pool.resource_id() &&
        region.lo() == lo && region.hi() == hi &&
        (forall<index: Int> new_pool.contains(index) ==
            (old_pool.contains(index) && !(lo <= index && index < hi))) &&
        (forall<index: Int> new_pool.slot(index) ==
            if old_pool.contains(index) && !(lo <= index && index < hi) {
                old_pool.slot(index)
            } else { None }) &&
        (forall<index: Int> region.slot(index) ==
            if lo <= index && index < hi { old_pool.slot(index) } else { None })
    }
}

/// Exact state transition exported by the active region-lend operation.
#[logic(opaque, prophetic)]
#[ensures(result == (
    header_region_payload_frame(before, after, ticket) &&
    header_region_ticket_loan_frame(before, after, ticket, loan) &&
    before.pool != None && after.pool != None &&
    exact_pool_region_interval_split(
        before.pool.unwrap_logic(), after.pool.unwrap_logic(),
        loan.state.lent_region.unwrap_logic(), ticket.owned_lo(), ticket.owned_hi())
))]
pub(crate) fn exact_header_region_lend_transition(
    before: SharedPayload,
    after: SharedPayload,
    ticket: OwnerTicket,
    loan: SharedHeaderReadLoan,
) -> bool {
    pearlite! {
        header_region_payload_frame(before, after, ticket) &&
        header_region_ticket_loan_frame(before, after, ticket, loan) &&
        before.pool != None && after.pool != None &&
        exact_pool_region_interval_split(
            before.pool.unwrap_logic(), after.pool.unwrap_logic(),
            loan.state.lent_region.unwrap_logic(), ticket.owned_lo(), ticket.owned_hi())
    }
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
        pearlite! { header_lease_idle(self.authority@, self.resource_id) }
    }

    #[logic(open(self), prophetic)]
    fn is_active(self, stamp: HeaderLeaseStamp) -> bool {
        pearlite! { header_lease_active(self.authority@, self.resource_id, stamp) }
    }

    #[logic(open(self))]
    fn count(&self) -> Int {
        pearlite! { header_lease_count(self.authority@) }
    }

    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[requires(*initial_count >= 0)]
    #[ensures(result.inner_logic().0.invariant())]
    #[ensures(result.inner_logic().0.is_idle())]
    #[ensures(result.inner_logic().0.resource_id() == result.inner_logic().1.resource_id())]
    #[ensures(result.inner_logic().0.header_id() == header_id)]
    #[ensures(result.inner_logic().0.count() == *initial_count)]
    #[ensures(result.inner_logic().1.slot() >= 0)]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures(context.inner_logic().registered_cap(
        (^context.inner_logic()),
        result.inner_logic().1,
        Some(Excl((result.inner_logic().1.resource_id(), None, *initial_count)))
    ))]
    pub(crate) fn new(
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        header_id: *mut Shared,
        initial_count: Snapshot<Int>,
    ) -> Ghost<(Self, ReadyKey)> {
        ghost! {
            let initial_count = initial_count.into_ghost().into_inner();
            let mut authority: Authority<HeaderLeaseState> = Authority::alloc().into_inner();
            let resource_id = authority.id_ghost();
            let idle: HeaderLeaseState = Some(Excl((resource_id, None, initial_count)));
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
    #[requires(header_lease_idle(
        cap.inner_logic().fragment_value(), authority.inner_logic().resource_id()))]
    #[requires(cap.inner_logic().fragment_value() != None)]
    #[requires(header_lease_count(cap.inner_logic().fragment_value()) == *count)]
    #[requires(authority.inner_logic().count() == *count)]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_active(*stamp))]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures((^authority.inner_logic()).header_id() ==
        authority.inner_logic().header_id())]
    #[ensures((^authority.inner_logic()).count() == *count)]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == cap.inner_logic().key())]
    #[ensures(result.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[ensures(header_lease_active(result.inner_logic().fragment_value(),
        authority.inner_logic().resource_id(), *stamp))]
    #[ensures(header_lease_count(result.inner_logic().fragment_value()) == *count)]
    fn activate(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
        count: Snapshot<Int>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let authority = authority.into_inner();
            let mut cap = cap.into_inner();
            let count = count.into_ghost().into_inner();
            let idle: HeaderLeaseState = Some(Excl((authority.resource_id, None, count)));
            let resource_id = authority.resource_id;
            let fragment = cap.fragment_ref().into_inner();
            authority.authority.frag_lemma(fragment);
            proof_assert!(fragment@ == idle);
            proof_assert!(authority.authority@ == idle);
            cap.update(&mut authority.authority, snapshot!((
                Some(Excl((resource_id, Some(*stamp), count))),
                Some(Excl((resource_id, Some(*stamp), count))),
            )));
            proof_assert!(header_lease_active(cap.fragment_value(), resource_id, *stamp));
            proof_assert!(header_lease_count(cap.fragment_value()) == count);
            proof_assert!(header_lease_active(authority.authority@, resource_id, *stamp));
            proof_assert!(header_lease_count(authority.authority@) == count);
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
    #[requires(header_lease_active(cap.inner_logic().fragment_value(),
        authority.inner_logic().resource_id(), *stamp))]
    #[requires(cap.inner_logic().fragment_value() != None)]
    #[requires(header_lease_count(cap.inner_logic().fragment_value()) ==
        authority.inner_logic().count())]
    #[requires(*new_count >= 0)]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_idle())]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures((^authority.inner_logic()).header_id() ==
        authority.inner_logic().header_id())]
    #[ensures((^authority.inner_logic()).count() == *new_count)]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == cap.inner_logic().key())]
    #[ensures(result.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[ensures(header_lease_idle(result.inner_logic().fragment_value(),
        authority.inner_logic().resource_id()))]
    #[ensures(header_lease_count(result.inner_logic().fragment_value()) == *new_count)]
    fn deactivate(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
        new_count: Snapshot<Int>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let authority = authority.into_inner();
            let mut cap = cap.into_inner();
            let resource_id = authority.resource_id;
            let fragment = cap.fragment_ref().into_inner();
            authority.authority.frag_lemma(fragment);
            proof_assert!(fragment@.incl_eq(authority.authority@));
            let old_count = snapshot!(header_lease_count(authority.authority@))
                .into_ghost()
                .into_inner();
            proof_assert!(header_lease_active(fragment@, authority.resource_id, *stamp));
            proof_assert!(header_lease_active(authority.authority@, authority.resource_id, *stamp));
            proof_assert!(header_lease_count(authority.authority@) == old_count);
            cap.update(&mut authority.authority, snapshot!((
                Some(Excl((resource_id, None, *new_count))),
                Some(Excl((resource_id, None, *new_count))),
            )));
            proof_assert!(header_lease_idle(authority.authority@, resource_id));
            proof_assert!(header_lease_count(authority.authority@) == *new_count);
            proof_assert!(header_lease_idle(cap.fragment_value(), resource_id));
            proof_assert!(header_lease_count(cap.fragment_value()) == *new_count);
            cap
        }
    }

    /// Refresh the count carried by the actual active lease fragment after a
    /// checked ledger mutation. The same ReadyCap updates the authoritative
    /// state and the caller's fragment together.
    #[check(ghost)]
    #[requires(authority.inner_logic().invariant() &&
        authority.inner_logic().is_active(*stamp))]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key().resource_id() == authority.inner_logic().resource_id())]
    #[requires(cap.inner_logic().fragment_id() == authority.inner_logic().resource_id())]
    #[requires(header_lease_active(cap.inner_logic().fragment_value(),
        authority.inner_logic().resource_id(), *stamp))]
    #[requires(*new_count >= 0)]
    #[requires(header_lease_count(cap.inner_logic().fragment_value()) ==
        authority.inner_logic().count())]
    #[ensures((^authority.inner_logic()).invariant() &&
        (^authority.inner_logic()).is_active(*stamp))]
    #[ensures((^authority.inner_logic()).resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures((^authority.inner_logic()).header_id() ==
        authority.inner_logic().header_id())]
    #[ensures((^authority.inner_logic()).count() == *new_count)]
    #[ensures((^cap.inner_logic()).invariant())]
    #[ensures((^cap.inner_logic()).key() == cap.inner_logic().key())]
    #[ensures((^cap.inner_logic()).fragment_id() == cap.inner_logic().fragment_id())]
    #[ensures((^cap.inner_logic()).key().resource_id() ==
        authority.inner_logic().resource_id())]
    #[ensures(header_lease_active((^cap.inner_logic()).fragment_value(),
        authority.inner_logic().resource_id(), *stamp))]
    #[ensures(header_lease_count((^cap.inner_logic()).fragment_value()) == *new_count)]
    fn refresh_active_count(
        authority: Ghost<&mut Self>,
        stamp: Snapshot<HeaderLeaseStamp>,
        new_count: Snapshot<Int>,
        cap: Ghost<&mut ReadyCap<HeaderLeaseState>>,
    ) {
        ghost! {
            let authority = authority.into_inner();
            let cap = cap.into_inner();
            let resource_id = authority.resource_id;
            let old_count = snapshot!(header_lease_count(authority.authority@))
                .into_ghost()
                .into_inner();
            let fragment = cap.fragment_ref().into_inner();
            authority.authority.frag_lemma(fragment);
            proof_assert!(fragment@ == Some(Excl((
                resource_id,
                Some(*stamp),
                old_count,
            ))));
            proof_assert!(authority.authority@ == Some(Excl((
                resource_id,
                Some(*stamp),
                old_count,
            ))));
            cap.update(&mut authority.authority, snapshot!((
                Some(Excl((resource_id, Some(*stamp), *new_count))),
                Some(Excl((resource_id, Some(*stamp), *new_count))),
            )));
            proof_assert!(authority.authority@ == Some(Excl((
                resource_id,
                Some(*stamp),
                *new_count,
            ))));
            proof_assert!(cap.fragment_value() == Some(Excl((
                resource_id,
                Some(*stamp),
                *new_count,
            ))));
        };
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
    #[requires(header_lease_active(cap.fragment_value(), self.resource_id, *stamp))]
    #[requires(cap.fragment_value() != None)]
    #[ensures(self.is_active(*stamp))]
    #[ensures(header_lease_count(cap.fragment_value()) == self.count())]
    fn require_active_fragment(
        &self,
        stamp: Snapshot<HeaderLeaseStamp>,
        cap: &ReadyCap<HeaderLeaseState>,
    ) {
        let _ = ghost! {
            let fragment = cap.fragment_ref().into_inner();
            self.authority.frag_lemma(fragment);
            proof_assert!(fragment@.incl_eq(self.authority@));
            proof_assert!(header_lease_active(self.authority@, self.resource_id, *stamp));
            proof_assert!(header_lease_count(self.authority@) ==
                header_lease_count(fragment@));
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
    #[requires(header_lease_idle(cap.fragment_value(), self.resource_id()))]
    #[requires(cap.fragment_value() != None)]
    #[ensures(self.is_idle())]
    #[ensures(header_lease_count(cap.fragment_value()) == self.count())]
    fn require_idle_cap(&self, cap: &ReadyCap<HeaderLeaseState>) {
        let _ = ghost! {
            let fragment = cap.fragment_ref().into_inner();
            self.authority.frag_lemma(fragment);
            proof_assert!(fragment@.incl_eq(self.authority@));
            proof_assert!(header_lease_idle(self.authority@, self.resource_id));
            proof_assert!(header_lease_count(fragment@) ==
                header_lease_count(self.authority@));
            proof_assert!(self.is_idle());
        };
    }
}

impl Invariant for HeaderLeaseAuthority {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.authority.invariant() && self.authority.id() == self.resource_id &&
            self.count() >= 0
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
                no_active_lend(self) && !self.closed && self.phase.is_open() &&
                self.lease.is_idle() && self.lease.count() == ledger.count() &&
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
                active_lease_pool_protocol(
                    pool, ledger, storage.capacity(), storage.binding(), None,
                ) &&
                header.vec@ != None && header.vec_model().matches_authority(storage)
            },
            (Some(root), Some(storage), Some(pool), Some(ledger), Some(retire), None) => {
                let header = root.val();
                no_active_lend(self) && !self.closed && self.phase.is_open() &&
                self.lease.is_idle() && self.lease.count() == ledger.count() &&
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
                self.lease.count() == ledger.count() &&
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
                    self.lent_span,
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
                active_lease_pool_protocol(
                    pool, ledger, storage.capacity(), storage.binding(), self.lent_span,
                )
            },
            (None, Some(storage), Some(pool), Some(ledger), Some(retire), None) => {
                !self.closed && self.phase.is_open() &&
                self.lease.count() == ledger.count() &&
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
                    self.lent_span,
                )) &&
                no_active_lend(self) &&
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
                no_active_lend(self) && self.closed && self.phase.is_closed() &&
                self.lease.is_idle() && self.lease.count() == 0 && retire.invariant() &&
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
        self.lease.invariant() &&
        (self.header_id.addr_logic() & super::KIND_MASK) == super::KIND_ARC
    }
}

/// Affine, read-only access to a Shared header while its root permission is
/// temporarily outside the event payload. It carries the exact root and lease
/// capability. Optional physical authority is present only as the actual
/// affine `PhysicalRegion` paired with its exact lease span.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedHeaderReadLoan {
    root: Box<Perm<*const Shared>>,
    state: SharedHeaderReadLoanState,
}

/// The callback-mutable portion of a header loan. Keeping the root permission
/// in the enclosing loan lets the counter reference borrow that disjoint field
/// while callbacks update the actual lease capability here.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedHeaderReadLoanState {
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
    stamp: Snapshot<HeaderLeaseStamp>,
    lent_span: Option<LentSpan>,
    lent_region: Option<PhysicalRegion>,
    lease_cap: ReadyCap<HeaderLeaseState>,
}

impl SharedHeaderReadLoan {
    /// Project the actual storage authority before the affine loan is split.
    /// The snapshot records value metadata only; the root permission and
    /// borrowed Vec remain the source of allocation liveness and access.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(*result == *self.state.storage_model)]
    #[ensures((*result).invariant())]
    #[ensures(self.root.val().vec_model().matches_authority(*result))]
    pub(crate) fn storage_authority_snapshot(&self) -> Snapshot<StoredAuthority> {
        self.state.storage_model
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn lease_count_value(self) -> Int {
        pearlite! { header_lease_count(self.state.lease_cap.fragment_value()) }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn lease_key_value(self) -> ReadyKey {
        pearlite! { self.state.lease_key }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn lease_id_value(self) -> Id {
        pearlite! { self.state.lease_id }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn lent_span_value(self) -> Option<LentSpan> {
        pearlite! { self.state.lent_span }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn lent_region_value(self) -> Option<PhysicalRegion> {
        pearlite! { self.state.lent_region }
    }

    /// Record the logical descriptor of this loan's actual region while the
    /// affine loan is live. The returned snapshot carries values only; it is
    /// not a replacement for the region permission.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(self.lent_region_value() != None)]
    #[ensures(*result == self.lent_region_value().unwrap_logic())]
    #[ensures((*result).invariant())]
    #[ensures((*result).resource_id() == (*result).namespace())]
    pub(crate) fn lent_region_snapshot(&self) -> Snapshot<PhysicalRegion> {
        let region = self.state.lent_region.as_ref().unwrap();
        snapshot!(*region)
    }

    #[logic(open(self), prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            self.root.invariant() &&
            *self.root.ward() == self.state.header_id as *const Shared &&
            self.root.val().ref_count.counter_model() == *self.state.counter_model &&
            !self.root.val().ref_count.has_event_invariant() &&
            self.root.val().vec@ != None &&
            self.root.val().vec_model() == *self.state.header_vec &&
            self.root.val().original_capacity_repr == *self.state.original_capacity_repr &&
            self.root.val().vec_model().matches_authority(*self.state.storage_model) &&
            self.state.invariant()
        }
    }

    /// Build the shared proof-only loan carrier from the actual activated
    /// capability and exact header facts. Keeping this constructor common to
    /// ordinary and region-backed takes makes the stored stamp relation and
    /// active cap relation one checked boundary.
    #[check(ghost)]
    #[requires(root.invariant())]
    #[requires(*root.ward() == header_id as *const Shared)]
    #[requires(root.val().ref_count.counter_model() == *counter_model)]
    #[requires(!root.val().ref_count.has_event_invariant())]
    #[requires(root.val().vec@ != None)]
    #[requires(root.val().vec_model() == *header_vec)]
    #[requires(root.val().original_capacity_repr == *original_capacity_repr)]
    #[requires(root.val().vec_model().matches_authority(*storage_model))]
    #[requires((*storage_model).invariant())]
    #[requires(generation > 0)]
    #[requires(lent_region_matches(lent_span, lent_region, *storage_model))]
    #[requires(*stamp == header_lease_stamp(
        generation, header_id, *counter_model, ledger_id, phase_id,
        *storage_model, *header_vec, *original_capacity_repr, lease_key,
        lent_span,
    ))]
    #[requires(lease_cap.invariant())]
    #[requires(lease_cap.key() == lease_key)]
    #[requires(lease_key.resource_id() == lease_id)]
    #[requires(lease_cap.fragment_id() == lease_id)]
    #[requires(header_lease_active(lease_cap.fragment_value(), lease_id, *stamp))]
    #[requires(lease_cap.fragment_value() != None)]
    #[requires(header_lease_count(lease_cap.fragment_value()) >= 0)]
    #[ensures(result.invariant())]
    #[ensures(result.root == root)]
    #[ensures(result.state.header_id == header_id)]
    #[ensures(result.state.counter_model == counter_model)]
    #[ensures(result.state.header_vec == header_vec)]
    #[ensures(result.state.original_capacity_repr == original_capacity_repr)]
    #[ensures(result.state.storage_model == storage_model)]
    #[ensures((*result.state.storage_model).invariant())]
    #[ensures(result.state.ledger_id == ledger_id)]
    #[ensures(result.state.phase_id == phase_id)]
    #[ensures(result.state.generation == generation)]
    #[ensures(result.state.lease_id == lease_id)]
    #[ensures(result.state.lease_key == lease_key)]
    #[ensures(result.state.stamp == stamp)]
    #[ensures(result.state.lent_span == lent_span)]
    #[ensures(result.state.lent_region == lent_region)]
    #[ensures(result.state.lease_cap == lease_cap)]
    fn from_active_parts(
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
        stamp: Snapshot<HeaderLeaseStamp>,
        lent_span: Option<LentSpan>,
        lent_region: Option<PhysicalRegion>,
        lease_cap: ReadyCap<HeaderLeaseState>,
    ) -> SharedHeaderReadLoan {
        let loan = SharedHeaderReadLoan {
            root,
            state: SharedHeaderReadLoanState {
                header_id,
                counter_model,
                header_vec,
                original_capacity_repr,
                storage_model,
                ledger_id,
                phase_id,
                generation,
                lease_id,
                lease_key,
                stamp,
                lent_span,
                lent_region,
                lease_cap,
            },
        };
        proof_assert!(loan.invariant());
        proof_assert!((*loan.state.storage_model).invariant());
        loan
    }
}

impl Invariant for SharedHeaderReadLoanState {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.generation > 0 &&
            (*self.storage_model).invariant() &&
            lent_region_matches(self.lent_span, self.lent_region, *self.storage_model) &&
            self.lease_cap.invariant() &&
            self.lease_cap.key() == self.lease_key &&
            self.lease_key.resource_id() == self.lease_id &&
            self.lease_cap.fragment_id() == self.lease_id &&
            header_lease_active(
                self.lease_cap.fragment_value(), self.lease_id, *self.stamp,
            ) && header_lease_count(self.lease_cap.fragment_value()) >= 0 &&
            *self.stamp == header_lease_stamp(
                self.generation,
                self.header_id,
                *self.counter_model,
                self.ledger_id,
                self.phase_id,
                *self.storage_model,
                *self.header_vec,
                *self.original_capacity_repr,
                self.lease_key,
                self.lent_span,
            )
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
    #[ensures(ticket.allocation_binding() == *self.allocation_binding)]
    #[ensures(self.ledger.unwrap_logic().active_ids().get(ticket.owner_id()) ==
        Some(Excl((ticket.allocation_binding(), ticket.class(),
            ticket.owned_lo(), ticket.owned_hi()))))]
    #[ensures(0 <= ticket.owned_lo() &&
        ticket.owned_lo() <= ticket.owned_hi() &&
        ticket.owned_hi() <= self.ledger.unwrap_logic().allocation_binding().2)]
    #[ensures(self.retire == None)]
    #[ensures(self.terminal_action != None)]
    fn prove_live_owner_ticket(&self, ticket: &OwnerTicket) {
        match (&self.ledger, &self.retire) {
            (Some(ledger), None) => ledger.prove_ticket_is_active(ticket),
            (Some(_), Some(retire)) => retire.contradicts_live_ticket(ticket),
            (None, Some(retire)) => retire.contradicts_live_ticket(ticket),
            (None, None) => proof_assert!(false),
        }
    }

    /// Export the pool-slot consequence for one actual active Readonly ticket.
    /// Membership comes from its affine ticket and live ledger; the payload
    /// protocol supplies slot facts from the actual pool resource.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(self.lent_span == None)]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[requires(ticket.class() == RefKind::Readonly)]
    #[ensures(self.ledger != None && self.ledger.unwrap_logic().has_ticket(*ticket))]
    #[ensures(self.ledger.unwrap_logic().active_ids().get(ticket.owner_id()) ==
        Some(Excl((ticket.allocation_binding(), ticket.class(),
            ticket.owned_lo(), ticket.owned_hi()))))]
    #[ensures(ticket.allocation_binding() == *self.allocation_binding)]
    #[ensures(self.pool != None)]
    #[ensures(0 <= ticket.owned_lo() &&
        ticket.owned_lo() <= ticket.owned_hi() &&
        ticket.owned_hi() <= self.pool.unwrap_logic().capacity())]
    #[ensures(forall<index: Int>
        ticket.owned_lo() <= index && index < ticket.owned_hi() ==>
            self.pool.unwrap_logic().contains(index) &&
                slot_known(self.pool.unwrap_logic().slot(index)))]
    fn prove_readonly_ticket_pool_known(&self, ticket: &OwnerTicket) {
        self.prove_live_owner_ticket(ticket);
        let ledger = match &self.ledger {
            Some(ledger) => ledger,
            None => {
                proof_assert!(false);
                return;
            }
        };
        let pool = match &self.pool {
            Some(pool) => pool,
            None => {
                proof_assert!(false);
                return;
            }
        };
        ledger.prove_ticket_is_active(ticket);
        proof_assert!(ledger.active_ids().get(ticket.owner_id()) ==
            Some(Excl((
                ticket.allocation_binding(), ticket.class(),
                ticket.owned_lo(), ticket.owned_hi(),
            ))));
        proof_assert!(forall<index: Int>
            ticket.owned_lo() <= index && index < ticket.owned_hi() ==>
                pool.contains(index) && slot_known(pool.slot(index)));
    }

    /// A count-one Readonly ticket is the only ledger entry, so the active
    /// pool protocol says the actual pool contains the entire allocation.
    /// This is the physical source for promoting a cursor view into a
    /// Mutable suffix; the ticket and counter observation remain the proof
    /// of uniqueness, while this function only exposes the pool consequence.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(self.agrees_with_value(value) && value@ == 1)]
    #[requires(self.lent_span == None)]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[requires(ticket.class() == RefKind::Readonly)]
    #[ensures(self.ledger != None && self.ledger.unwrap_logic().is_live())]
    #[ensures(self.ledger.unwrap_logic().count() == 1)]
    #[ensures(self.ledger.unwrap_logic().has_ticket(*ticket))]
    #[ensures(self.ledger.unwrap_logic().active_ids() ==
        FMap::singleton(ticket.owner_id(), Excl(ticket.owner_binding())))]
    #[ensures(self.pool != None)]
    #[ensures(self.pool.unwrap_logic().capacity() ==
        self.ledger.unwrap_logic().allocation_binding().2)]
    #[ensures(forall<index: Int> 0 <= index &&
        index < self.pool.unwrap_logic().capacity() ==>
            self.pool.unwrap_logic().contains(index))]
    fn prove_unique_readonly_full_pool(
        &self,
        ticket: &OwnerTicket,
        value: usize,
    ) {
        self.prove_live_owner_matches_counter(ticket, value);
        let ledger = match &self.ledger {
            Some(ledger) => ledger,
            None => {
                proof_assert!(false);
                return;
            }
        };
        let pool = match &self.pool {
            Some(pool) => pool,
            None => {
                proof_assert!(false);
                return;
            }
        };
        ledger.prove_ticket_is_active(ticket);
        sole_active_id(snapshot!(*ledger), snapshot!(*ticket));
        proof_assert!(ledger.count() == 1);
        proof_assert!(forall<index: Int>
            active_mutable_contains(ledger.active_ids(), index) == false);
        proof_assert!(forall<index: Int>
            0 <= index && index < pool.capacity() ==>
                pool.contains(index));
    }

    /// Reclassify the actual unique Readonly ticket and transfer its complete
    /// allocation suffix from the actual pool into a Mutable region. The
    /// prefix before `lo` stays in the pool; Unknown spare slots in the
    /// suffix remain Unknown in the returned region.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().agrees_with_value(value) && value@ == 1)]
    #[requires(payload.inner_logic().lent_span == None)]
    #[requires(ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id &&
        ticket.inner_logic().class() == RefKind::Readonly)]
    #[requires(ticket.inner_logic().owned_lo() <= *lo)]
    #[requires(*lo <= ticket.inner_logic().owned_hi())]
    #[requires(ticket.inner_logic().owned_hi() <=
        payload.inner_logic().ledger.unwrap_logic().allocation_binding().2)]
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
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation
    )]
    #[ensures(
        (^payload.inner_logic()).ledger.unwrap_logic().count() == 1 &&
        (^payload.inner_logic()).ledger.unwrap_logic().ledger_id() == payload.inner_logic().ledger.unwrap_logic().ledger_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().allocation_binding() == payload.inner_logic().ledger.unwrap_logic().allocation_binding() &&
        (^payload.inner_logic()).ledger.unwrap_logic().next_id() == payload.inner_logic().ledger.unwrap_logic().next_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().has_ticket((^ticket.inner_logic()))
    )]
    #[ensures(
        (^ticket.inner_logic()).invariant() &&
        (^ticket.inner_logic()).ledger_id() == ticket.inner_logic().ledger_id() &&
        (^ticket.inner_logic()).owner_id() == ticket.inner_logic().owner_id() &&
        (^ticket.inner_logic()).fragment_resource_id() == ticket.inner_logic().fragment_resource_id() &&
        (^ticket.inner_logic()).allocation_binding() == ticket.inner_logic().allocation_binding() &&
        (^ticket.inner_logic()).class() == RefKind::Mutable &&
        (^ticket.inner_logic()).owned_lo() == *lo &&
        (^ticket.inner_logic()).owned_hi() == payload.inner_logic().ledger.unwrap_logic().allocation_binding().2
    )]
    #[ensures(
        result.inner_logic().invariant() &&
        result.inner_logic().namespace() == payload.inner_logic().ledger.unwrap_logic().allocation_binding().0 &&
        result.inner_logic().capacity() == payload.inner_logic().ledger.unwrap_logic().allocation_binding().2 &&
        result.inner_logic().lo() == *lo &&
        result.inner_logic().hi() == payload.inner_logic().ledger.unwrap_logic().allocation_binding().2 &&
        (forall<index: Int> result.inner_logic().slot(index) ==
            if *lo <= index && index < payload.inner_logic().ledger.unwrap_logic().allocation_binding().2 {
                payload.inner_logic().pool.unwrap_logic().slot(index)
            } else { None }) &&
        (forall<index: Int> *lo <= index && index < ticket.inner_logic().owned_hi() ==>
            slot_known(result.inner_logic().slot(index))) &&
        (forall<index: Int> 0 <= index && index < *lo ==>
            (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
                payload.inner_logic().pool.unwrap_logic().slot(index) &&
            (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
                payload.inner_logic().pool.unwrap_logic().contains(index))
    )]
    pub(crate) fn promote_unique_readonly_owner(
        payload: Ghost<&mut SharedPayload>,
        ticket: Ghost<&mut OwnerTicket>,
        value: usize,
        lo: Snapshot<Int>,
    ) -> Ghost<PhysicalRegion> {
        ghost! {
            let payload = payload.into_inner();
            let ticket = ticket.into_inner();
            let lo = lo.into_ghost().into_inner();
            payload.prove_unique_readonly_full_pool(ticket, value);
            payload.prove_readonly_ticket_pool_known(ticket);

            let before_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let before_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            let before_ticket: Snapshot<OwnerTicket> = snapshot!(*ticket);
            let binding = snapshot!((
                ticket.allocation_binding(), RefKind::Mutable, lo,
                payload.ledger.unwrap_logic().allocation_binding().2,
            ));
            let ledger = payload.ledger.as_mut().unwrap();
            ledger.prove_singleton_reclassification_compatible(
                &*ticket, binding,
            );
            OwnerLedger::reclassify(
                Ghost::new(ledger), Ghost::new(ticket), binding,
            );
            let after_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let capacity_snapshot: Snapshot<Int> = snapshot!(
                payload.ledger.unwrap_logic().allocation_binding().2
            );
            prove_owner_readonly_to_mutable_effect(
                before_owners, after_owners, before_ticket,
                snapshot!(lo), capacity_snapshot,
            );

            let capacity = capacity_snapshot.into_ghost().into_inner();
            let mut pool = payload.pool.take().unwrap();
            let (new_pool, region) = pool.lend_interval(lo, capacity);
            payload.pool = Some(new_pool);
            let after_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            SharedPayload::prove_pool_complement_after_mutable_acquire(
                before_pool, after_pool, before_owners, after_owners,
                snapshot!(region), lo, capacity,
            );
            proof_assert!(active_lease_pool_protocol(
                payload.pool.unwrap_logic(), payload.ledger.unwrap_logic(),
                payload.storage.unwrap_logic().capacity(),
                payload.storage.unwrap_logic().binding(), None,
            ));
            proof_assert!(payload.invariant() && payload.protocol());
            proof_assert!(ticket.class() == RefKind::Mutable);
            proof_assert!(ticket.owned_lo() == lo);
            proof_assert!(ticket.owned_hi() == capacity);
            region
        }
    }

    /// Apply the actual owner-ledger/pool transition for one Acquire
    /// observation. The callback result carries only the updated actual
    /// ticket and optional region; packet assembly remains outside the
    /// callback so the event proof does not duplicate packet invariants.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().agrees_with_value(value))]
    #[requires(payload.inner_logic().lent_span == None)]
    #[requires(ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id &&
        ticket.inner_logic().class() == RefKind::Readonly &&
        ticket.inner_logic().allocation_binding() ==
            *payload.inner_logic().allocation_binding)]
    #[requires(ticket.inner_logic().owned_lo() <= *lo &&
        *lo <= ticket.inner_logic().owned_hi() &&
        ticket.inner_logic().owned_hi() <=
            payload.inner_logic().ledger.unwrap_logic().allocation_binding().2)]
    #[requires(*visible_hi == ticket.inner_logic().owned_hi())]
    #[requires(loan.inner_logic().invariant() &&
        no_active_lend_state(loan.inner_logic()))]
    #[requires(payload.inner_logic().lease.resource_id() ==
        loan.inner_logic().lease_id)]
    #[ensures(header_lease_count(
        loan.inner_logic().lease_cap.fragment_value(),
    ) == value@)]
    #[ensures(value@ > 0)]
    #[ensures((^payload.inner_logic()).invariant() &&
        (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).public() == payload.inner_logic().public() &&
        (^payload.inner_logic()).counter_model() ==
            payload.inner_logic().counter_model() &&
        (^payload.inner_logic()).agrees_with_value(value))]
    #[ensures(result.inner_logic().invariant())]
    #[ensures((^payload.inner_logic()).allocation_binding ==
        payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr ==
            payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).root == payload.inner_logic().root &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).retire == payload.inner_logic().retire &&
        (^payload.inner_logic()).terminal_action ==
            payload.inner_logic().terminal_action &&
        (^payload.inner_logic()).lease == payload.inner_logic().lease &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease_generation ==
            payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed)]
    #[ensures(match result.inner_logic().region {
        Some(region) => value@ == 1 &&
            result.inner_logic().ticket.inner_logic().invariant() &&
            result.inner_logic().ticket.inner_logic().ledger_id() ==
                ticket.inner_logic().ledger_id() &&
            result.inner_logic().ticket.inner_logic().owner_id() ==
                ticket.inner_logic().owner_id() &&
            result.inner_logic().ticket.inner_logic().fragment_resource_id() ==
                ticket.inner_logic().fragment_resource_id() &&
            result.inner_logic().ticket.inner_logic().allocation_binding() ==
                ticket.inner_logic().allocation_binding() &&
            result.inner_logic().ticket.inner_logic().class() == RefKind::Mutable &&
            result.inner_logic().ticket.inner_logic().owned_lo() == *lo &&
            result.inner_logic().ticket.inner_logic().owned_hi() ==
                payload.inner_logic().ledger.unwrap_logic().allocation_binding().2 &&
            region.inner_logic().invariant() &&
            region.inner_logic().resource_id() == region.inner_logic().namespace() &&
            region.inner_logic().namespace() == ticket.inner_logic().allocation_binding().0 &&
            region.inner_logic().capacity() == ticket.inner_logic().allocation_binding().2 &&
            region.inner_logic().lo() == *lo &&
            region.inner_logic().hi() == ticket.inner_logic().allocation_binding().2 &&
            (forall<index: Int> region.inner_logic().slot(index) ==
                if *lo <= index && index < ticket.inner_logic().allocation_binding().2 {
                    payload.inner_logic().pool.unwrap_logic().slot(index)
                } else { None }) &&
            *lo <= ticket.inner_logic().owned_hi() &&
            (forall<index: Int> *lo <= index &&
                index < ticket.inner_logic().owned_hi() ==>
                slot_known(region.inner_logic().slot(index))),
        None => value@ != 1 && (^payload.inner_logic()) == *payload.inner_logic() &&
            result.inner_logic().ticket.inner_logic() == ticket.inner_logic(),
    })]
    #[ensures(*result.inner_logic().visible_hi == *visible_hi)]
    #[ensures(value@ > 0)]
    pub(crate) fn transition_unique_readonly_owner(
        payload: Ghost<&mut SharedPayload>,
        ticket: Ghost<OwnerTicket>,
        loan: Ghost<&SharedHeaderReadLoanState>,
        value: usize,
        lo: Snapshot<Int>,
        visible_hi: Snapshot<Int>,
    ) -> Ghost<SharedReadonlyOwnerTransition> {
        ghost! {
            let payload = payload.into_inner();
            let mut ticket = ticket.into_inner();
            let loan = loan.into_inner();
            payload.prove_live_owner_matches_counter(&ticket, value);
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(loan),
            );
            if value == 1 {
                let region = SharedPayload::promote_unique_readonly_owner(
                    Ghost::new(payload), Ghost::new(&mut ticket), value, lo,
                ).into_inner();
                SharedReadonlyOwnerTransition {
                    ticket: Ghost::new(ticket),
                    region: Some(Ghost::new(region)),
                    visible_hi,
                }
            } else {
                SharedReadonlyOwnerTransition {
                    ticket: Ghost::new(ticket),
                    region: None,
                    visible_hi,
                }
            }
        }
    }

    /// Export the stable pool relation before moving the pool into an
    /// interval-lend transition. This keeps the root-present protocol's
    /// relation available across the affine `Option::take` boundary.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(self.root != None && self.lent_span == None)]
    #[requires(self.storage != None && self.pool != None && self.ledger != None)]
    #[requires(self.ledger.unwrap_logic().is_live())]
    #[ensures(active_lease_pool_protocol(
        self.pool.unwrap_logic(),
        self.ledger.unwrap_logic(),
        self.storage.unwrap_logic().capacity(),
        self.storage.unwrap_logic().binding(),
        None,
    ))]
    fn prove_stable_active_pool_protocol(&self) {
        proof_assert!(active_lease_pool_protocol(
            self.pool.unwrap_logic(),
            self.ledger.unwrap_logic(),
            self.storage.unwrap_logic().capacity(),
            self.storage.unwrap_logic().binding(),
            None,
        ));
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

    /// A distinct live peer ticket proves that releasing this owner cannot
    /// empty the actual owner ledger. This keeps the nonterminal guarantee
    /// tied to two affine fragments rather than an earlier counter snapshot.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(self.agrees_with_value(value))]
    #[requires(ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == self.ledger_id)]
    #[requires(peer.inner_logic().invariant() &&
        peer.inner_logic().ledger_id() == self.ledger_id)]
    #[requires(ticket.inner_logic().owner_id() != peer.inner_logic().owner_id())]
    #[ensures(value@ > 1)]
    #[ensures((^ticket.inner_logic()) == *ticket.inner_logic())]
    #[ensures((^ticket.inner_logic()).invariant())]
    #[ensures((^ticket.inner_logic()).ledger_id() == ticket.inner_logic().ledger_id())]
    #[ensures((^ticket.inner_logic()).owner_id() == ticket.inner_logic().owner_id())]
    #[ensures((^ticket.inner_logic()).fragment_resource_id() ==
        ticket.inner_logic().fragment_resource_id())]
    #[ensures((^ticket.inner_logic()).owner_binding() == ticket.inner_logic().owner_binding())]
    #[ensures((^ticket.inner_logic()).class() == ticket.inner_logic().class())]
    #[ensures((^ticket.inner_logic()).owned_lo() == ticket.inner_logic().owned_lo())]
    #[ensures((^ticket.inner_logic()).owned_hi() == ticket.inner_logic().owned_hi())]
    fn prove_peer_ticket_keeps_count_above_one(
        &self,
        ticket: Ghost<&mut OwnerTicket>,
        peer: Ghost<&OwnerTicket>,
        value: usize,
    ) {
        ghost! {
            let ticket = ticket.into_inner();
            let peer = peer.into_inner();
            self.prove_live_owner_matches_counter(ticket, value);
            self.prove_live_owner_ticket(ticket);
            self.prove_live_owner_ticket(peer);
            let ledger_snapshot: Snapshot<OwnerLedger> =
                snapshot!(self.ledger.unwrap_logic());
            if value == 1usize {
                crate::ownership_proof::ref_count_ledger::contradict_two_tickets(
                    ledger_snapshot,
                    Ghost::new(ticket),
                    Ghost::new(peer),
                );
            }
            proof_assert!(value@ > 1);
        };
    }

    /// Split one live Mutable ticket and its exact physical region after the
    /// native relaxed-CAS increment has succeeded. The caller supplies the
    /// actual ticket and region held by its packet; the payload's ledger and
    /// pool remain the authorities for reference count and free-space
    /// coverage.
    #[check(ghost)]
    #[requires(
        payload.inner_logic().invariant() && payload.inner_logic().protocol() &&
        payload.inner_logic().lent_span == None &&
        payload.inner_logic().agrees_with_value(old) &&
        old@ <= crate::ref_count_limit::MAX_REF_COUNT@ && new@ == old@ + 1
    )]
    #[requires(
        ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id &&
        ticket.inner_logic().class() == RefKind::Mutable &&
        region.inner_logic().invariant() &&
        ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
        ticket.inner_logic().owned_hi() == region.inner_logic().hi() &&
        ticket.inner_logic().allocation_binding().0 == region.inner_logic().namespace() &&
        ticket.inner_logic().allocation_binding().2 == region.inner_logic().capacity() &&
        ticket.inner_logic().owned_lo() <= *cut &&
        *cut <= ticket.inner_logic().owned_hi()
    )]
    #[requires(
        loan.inner_logic().invariant() &&
        loan.inner_logic().lease_id == payload.inner_logic().lease.resource_id()
    )]
    #[ensures(
        (^loan.inner_logic()).invariant() &&
        (^loan.inner_logic()).header_id == loan.inner_logic().header_id &&
        (^loan.inner_logic()).counter_model == loan.inner_logic().counter_model &&
        (^loan.inner_logic()).header_vec == loan.inner_logic().header_vec &&
        (^loan.inner_logic()).original_capacity_repr ==
            loan.inner_logic().original_capacity_repr &&
        (^loan.inner_logic()).storage_model == loan.inner_logic().storage_model &&
        (^loan.inner_logic()).ledger_id == loan.inner_logic().ledger_id &&
        (^loan.inner_logic()).phase_id == loan.inner_logic().phase_id &&
        (^loan.inner_logic()).generation == loan.inner_logic().generation &&
        (^loan.inner_logic()).lease_id == loan.inner_logic().lease_id &&
        (^loan.inner_logic()).lease_key == loan.inner_logic().lease_key &&
        (^loan.inner_logic()).stamp == loan.inner_logic().stamp &&
        (^loan.inner_logic()).lent_span == loan.inner_logic().lent_span &&
        (^loan.inner_logic()).lent_region == loan.inner_logic().lent_region &&
        (^loan.inner_logic()).lease_cap.key() == loan.inner_logic().lease_cap.key() &&
        (^loan.inner_logic()).lease_cap.fragment_id() ==
            loan.inner_logic().lease_cap.fragment_id() &&
        header_lease_active(
            (^loan.inner_logic()).lease_cap.fragment_value(),
            (^loan.inner_logic()).lease_id, *(^loan.inner_logic()).stamp,
        ) &&
        header_lease_count((^loan.inner_logic()).lease_cap.fragment_value()) == new@
    )]
    #[ensures(
        (^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
        (^payload.inner_logic()).agrees_with_value(new) &&
        (^payload.inner_logic()).public() == payload.inner_logic().public() &&
        (^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model() &&
        (^payload.inner_logic()).ledger_id == payload.inner_logic().ledger_id &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).pool == payload.inner_logic().pool &&
        (^payload.inner_logic()).root == payload.inner_logic().root &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease.invariant() &&
        (^payload.inner_logic()).lease.resource_id() ==
            payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.header_id() ==
            payload.inner_logic().lease.header_id() &&
        (^payload.inner_logic()).lease.is_active(header_lease_stamp(
            loan.inner_logic().generation,
            loan.inner_logic().header_id,
            *loan.inner_logic().counter_model,
            loan.inner_logic().ledger_id,
            loan.inner_logic().phase_id,
            *loan.inner_logic().storage_model,
            *loan.inner_logic().header_vec,
            *loan.inner_logic().original_capacity_repr,
            loan.inner_logic().lease_key,
            loan.inner_logic().lent_span,
        )) &&
        (^payload.inner_logic()).lease.count() == new@ &&
        (^payload.inner_logic()).lease_generation ==
            payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).ledger.unwrap_logic().count() == new@
    )]
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
        loan: Ghost<&mut SharedHeaderReadLoanState>,
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
            let loan = loan.into_inner();
            let cut_value = cut.into_ghost().into_inner();
            let ticket_snapshot: Snapshot<OwnerTicket> = snapshot!(ticket);
            let region_snapshot: Snapshot<PhysicalRegion> = snapshot!(region);
            let cut_snapshot: Snapshot<Int> = snapshot!(cut_value);

            payload.prove_live_owner_matches_counter(&ticket, old);
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&*loan),
            );
            let before_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let mut ledger = payload.ledger.take().unwrap();
            let child = OwnerLedger::split_owner(
                Ghost::new(&mut ledger), Ghost::new(&mut ticket), snapshot!(cut_value),
            ).into_inner();
            payload.ledger = Some(ledger);
            let new_count: Snapshot<Int> = snapshot!(payload.ledger.unwrap_logic().count());
            HeaderLeaseAuthority::refresh_active_count(
                Ghost::new(&mut payload.lease),
                loan.stamp,
                new_count,
                Ghost::new(&mut loan.lease_cap),
            );

            let (left_region, right_region) = region.split_at(cut_value);
            proof_assert!(new@ == old@ + 1);
            proof_assert!(payload.invariant() && payload.protocol());
            proof_assert!(payload.agrees_with_value(new));
            proof_assert!(payload.ledger.unwrap_logic().count() == new@);
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&*loan),
            );
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
        payload.inner_logic().lent_span == None &&
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
        (forall<index: Int> *visible_lo <= index && index < *visible_hi ==>
            slot_known(region.inner_logic().slot(index))) &&
        payload.inner_logic().ledger != None &&
        payload.inner_logic().ledger.unwrap_logic().has_ticket(ticket.inner_logic()) &&
        payload.inner_logic().pool != None
    )]
    #[ensures(
        (^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
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
            region.inner_logic().lo() <= index &&
                index < region.inner_logic().hi() ==>
                (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
                    region.inner_logic().slot(index)) &&
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
            proof_assert!(forall<index: Int>
                (*region_before).lo() <= index &&
                    index < (*region_before).hi() ==>
                    (*after_pool).slot(index) == (*region_before).slot(index));
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

    /// Reindex caller facts stated relative to a visible cursor as facts over
    /// the actual allocation's absolute slot coordinates.
    #[check(ghost)]
    #[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
        slot_known((*region).slot(*lo + offset)))]
    #[ensures(forall<index: Int> *lo <= index && index < *lo + len@ ==>
        slot_known((*region).slot(index)))]
    fn prove_shifted_region_slots_known(
        region: Snapshot<PhysicalRegion>,
        lo: Snapshot<Int>,
        len: Snapshot<usize>,
    ) {
        proof_assert!(forall<index: Int> *lo <= index && index < *lo + len@ ==>
            slot_known((*region).slot(index)));
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

    /// Re-establish the pool/owner complement when a unique Readonly ticket
    /// becomes Mutable and receives the matching actual pool interval.
    #[check(ghost)]
    #[requires(before_pool.invariant() && after_pool.invariant())]
    #[requires((*before_pool).capacity() == (*after_pool).capacity())]
    #[requires(region.invariant())]
    #[requires(region.lo() == lo && region.hi() == hi)]
    #[requires(region.capacity() == (*before_pool).capacity())]
    #[requires(forall<index: Int>
        (*before_pool).contains(index) ==
            (0 <= index && index < (*before_pool).capacity() &&
                !active_mutable_contains(*before_owners, index)))]
    #[requires(forall<index: Int>
        (*after_pool).contains(index) ==
            ((*before_pool).contains(index) && !(lo <= index && index < hi)))]
    #[requires(forall<index: Int>
        active_mutable_contains(*after_owners, index) ==
            (active_mutable_contains(*before_owners, index) ||
                lo <= index && index < hi))]
    #[ensures(forall<index: Int>
        (*after_pool).contains(index) ==
            (0 <= index && index < (*after_pool).capacity() &&
                !active_mutable_contains(*after_owners, index)))]
    fn prove_pool_complement_after_mutable_acquire(
        before_pool: Snapshot<PhysicalPool>,
        after_pool: Snapshot<PhysicalPool>,
        before_owners: Snapshot<ActiveReferences>,
        after_owners: Snapshot<ActiveReferences>,
        region: Snapshot<PhysicalRegion>,
        lo: Int,
        hi: Int,
    ) {
        proof_assert!(forall<index: Int>
            (*after_pool).contains(index) ==
                (0 <= index && index < (*after_pool).capacity() &&
                    !active_mutable_contains(*after_owners, index)));
    }

    /// Removing a Readonly ticket leaves the physical pool and all Mutable
    /// coverage unchanged. The actual ticket is consumed by the ledger
    /// transition; this lemma only relates the resulting pool model to the
    /// active-owner map.
    #[check(ghost)]
    #[requires(before_pool.invariant() && after_pool.invariant())]
    #[requires((*before_pool).capacity() == (*after_pool).capacity())]
    #[requires((*before_pool).namespace() == (*after_pool).namespace())]
    #[requires((*before_pool).resource_id() == (*after_pool).resource_id())]
    #[requires(forall<index: Int>
        (*after_pool).contains(index) == (*before_pool).contains(index))]
    #[requires(forall<index: Int>
        (*after_pool).slot(index) == (*before_pool).slot(index))]
    #[requires(forall<index: Int>
        (*before_pool).contains(index) ==
            (0 <= index && index < (*before_pool).capacity() &&
                !active_mutable_contains(*before_owners, index)))]
    #[requires(forall<index: Int>
        active_mutable_contains(*after_owners, index) ==
            active_mutable_contains(*before_owners, index))]
    #[ensures(forall<index: Int>
        (*after_pool).contains(index) ==
            (0 <= index && index < (*after_pool).capacity() &&
                !active_mutable_contains(*after_owners, index)))]
    fn prove_pool_complement_after_readonly_release(
        before_pool: Snapshot<PhysicalPool>,
        after_pool: Snapshot<PhysicalPool>,
        before_owners: Snapshot<ActiveReferences>,
        after_owners: Snapshot<ActiveReferences>,
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
    #[requires(
        payload.inner_logic().invariant() && payload.inner_logic().protocol() &&
        payload.inner_logic().lent_span == None &&
        payload.inner_logic().counter_model() == *owner_model &&
        payload.inner_logic().agrees_with_value(*old) &&
        old@ > 0 && new@ + 1 == old@ &&
        payload.inner_logic().public().3 == *expected_phase &&
        payload.inner_logic().ledger != None &&
        payload.inner_logic().ledger.unwrap_logic().count() == old@ &&
        payload.inner_logic().pool != None &&
        region.inner_logic().namespace() ==
            payload.inner_logic().ledger.unwrap_logic().allocation_binding().0 &&
        region.inner_logic().capacity() ==
            payload.inner_logic().ledger.unwrap_logic().allocation_binding().2
    )]
    #[requires(
        ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id &&
        region.inner_logic().invariant() &&
        ticket.inner_logic().class() == RefKind::Mutable &&
        ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
        ticket.inner_logic().owned_hi() == region.inner_logic().hi()
    )]
    #[requires(
        loan.inner_logic().invariant() &&
        loan.inner_logic().lease_id == payload.inner_logic().lease.resource_id()
    )]
    #[ensures(
        (^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
        (^payload.inner_logic()).public() == payload.inner_logic().public() &&
        (^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model() &&
        (^payload.inner_logic()).agrees_with_value(*new) &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).root == payload.inner_logic().root &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.header_id() == payload.inner_logic().lease.header_id() &&
        (^payload.inner_logic()).lease.count() == new@ &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed &&
        true
    )]
    #[ensures(
        (^loan.inner_logic()).invariant() &&
        (^loan.inner_logic()).header_id == loan.inner_logic().header_id &&
        (^loan.inner_logic()).counter_model == loan.inner_logic().counter_model &&
        (^loan.inner_logic()).header_vec == loan.inner_logic().header_vec &&
        (^loan.inner_logic()).original_capacity_repr ==
            loan.inner_logic().original_capacity_repr &&
        (^loan.inner_logic()).storage_model == loan.inner_logic().storage_model &&
        (^loan.inner_logic()).ledger_id == loan.inner_logic().ledger_id &&
        (^loan.inner_logic()).phase_id == loan.inner_logic().phase_id &&
        (^loan.inner_logic()).generation == loan.inner_logic().generation &&
        (^loan.inner_logic()).lease_id == loan.inner_logic().lease_id &&
        (^loan.inner_logic()).lease_key == loan.inner_logic().lease_key &&
        (^loan.inner_logic()).stamp == loan.inner_logic().stamp &&
        (^loan.inner_logic()).lent_span == loan.inner_logic().lent_span &&
        (^loan.inner_logic()).lent_region == loan.inner_logic().lent_region &&
        (^loan.inner_logic()).lease_cap.key() == loan.inner_logic().lease_cap.key() &&
        (^loan.inner_logic()).lease_cap.fragment_id() ==
            loan.inner_logic().lease_cap.fragment_id() &&
        header_lease_active((^loan.inner_logic()).lease_cap.fragment_value(),
            (^loan.inner_logic()).lease_id, *(^loan.inner_logic()).stamp) &&
        header_lease_count((^loan.inner_logic()).lease_cap.fragment_value()) == new@
    )]
    #[ensures(
        (^payload.inner_logic()).ledger.unwrap_logic().count() + 1 ==
            payload.inner_logic().ledger.unwrap_logic().count() &&
        (^payload.inner_logic()).ledger.unwrap_logic().ledger_id() ==
            payload.inner_logic().ledger.unwrap_logic().ledger_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().allocation_binding() ==
            payload.inner_logic().ledger.unwrap_logic().allocation_binding() &&
        (^payload.inner_logic()).ledger.unwrap_logic().next_id() ==
            payload.inner_logic().ledger.unwrap_logic().next_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().active_ids() ==
            payload.inner_logic().ledger.unwrap_logic().active_ids().remove(
                ticket.inner_logic().owner_id())
    )]
    #[ensures(
        (^payload.inner_logic()).pool.unwrap_logic().capacity() ==
            payload.inner_logic().pool.unwrap_logic().capacity() &&
        (^payload.inner_logic()).pool.unwrap_logic().namespace() ==
            payload.inner_logic().pool.unwrap_logic().namespace() &&
        (^payload.inner_logic()).pool.unwrap_logic().resource_id() ==
            payload.inner_logic().pool.unwrap_logic().resource_id()
    )]
    #[ensures(forall<index: Int>
        (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
            (payload.inner_logic().pool.unwrap_logic().contains(index) ||
                region.inner_logic().lo() <= index && index < region.inner_logic().hi()))]
    #[ensures(forall<index: Int>
        (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
            (if payload.inner_logic().pool.unwrap_logic().contains(index) {
                payload.inner_logic().pool.unwrap_logic().slot(index)
            } else if region.inner_logic().lo() <= index &&
                index < region.inner_logic().hi() {
                region.inner_logic().slot(index)
            } else { None }))]
    #[ensures(forall<index: Int>
        (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
            (0 <= index && index <
                (^payload.inner_logic()).pool.unwrap_logic().capacity() &&
                !active_mutable_contains(
                    (^payload.inner_logic()).ledger.unwrap_logic().active_ids(), index)))]
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
    fn drop_mutable_packet_owner(
        payload: Ghost<&mut SharedPayload>,
        owner_model: Snapshot<NativeCounter>,
        old: Snapshot<usize>,
        new: Snapshot<usize>,
        ticket: Ghost<OwnerTicket>,
        region: Ghost<PhysicalRegion>,
        loan: Ghost<&mut SharedHeaderReadLoanState>,
        expected_phase: Snapshot<Id>,
    ) -> Ghost<Option<TerminalAction>> {
        ghost! {
            let payload = payload.into_inner();
            let ticket = ticket.into_inner();
            let region = region.into_inner();
            let loan = loan.into_inner();
            payload.prove_public_phase_id(expected_phase);
            payload.prove_owner_region_excluded_from_pool(&ticket, &region);
            let before_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let before_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            let region_snapshot: Snapshot<PhysicalRegion> = snapshot!(region);
            payload.prove_live_owner_ticket(&ticket);
            let previous_action: Snapshot<Option<TerminalAction>> =
                snapshot!(payload.terminal_action);
            proof_assert!(*previous_action != None);
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&*loan),
            );
            let ledger = payload.ledger.as_mut().unwrap();
            let retire = OwnerLedger::drop_owner(
                Ghost::new(ledger), Ghost::new(ticket),
            ).into_inner();
            let pool = payload.pool.take().unwrap();
            payload.pool = Some(pool.retire(region));
            let remaining_count: Snapshot<Int> =
                snapshot!(payload.ledger.unwrap_logic().count());
            HeaderLeaseAuthority::refresh_active_count(
                Ghost::new(&mut payload.lease),
                loan.stamp,
                remaining_count,
                Ghost::new(&mut loan.lease_cap),
            );
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
            proof_assert!(payload.terminal_action == *previous_action);
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
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&*loan),
            );
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

    /// Remove one actual Readonly reference ticket after its native Release
    /// event. The span is already represented by the physical pool, so this
    /// path consumes no fabricated or reconstructed region capability.
    #[check(ghost)]
    #[requires(
        payload.inner_logic().invariant() && payload.inner_logic().protocol() &&
        payload.inner_logic().lent_span == None &&
        payload.inner_logic().counter_model() == *owner_model &&
        payload.inner_logic().agrees_with_value(*old) &&
        old@ > 0 && new@ + 1 == old@ &&
        payload.inner_logic().public().3 == *expected_phase &&
        payload.inner_logic().ledger != None &&
        payload.inner_logic().ledger.unwrap_logic().count() == old@
    )]
    #[requires(
        ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id &&
        ticket.inner_logic().class() == RefKind::Readonly
    )]
    #[requires(
        loan.inner_logic().invariant() &&
        loan.inner_logic().lease_id == payload.inner_logic().lease.resource_id()
    )]
    #[ensures(
        (^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
        (^payload.inner_logic()).public() == payload.inner_logic().public() &&
        (^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model() &&
        (^payload.inner_logic()).agrees_with_value(*new) &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).root == payload.inner_logic().root &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.header_id() == payload.inner_logic().lease.header_id() &&
        (^payload.inner_logic()).lease.count() == new@ &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed
    )]
    #[ensures(
        (^loan.inner_logic()).invariant() &&
        (^loan.inner_logic()).lease_id == loan.inner_logic().lease_id &&
        (^loan.inner_logic()).lease_key == loan.inner_logic().lease_key &&
        (^loan.inner_logic()).stamp == loan.inner_logic().stamp &&
        (^loan.inner_logic()).header_id == loan.inner_logic().header_id &&
        (^loan.inner_logic()).counter_model == loan.inner_logic().counter_model &&
        (^loan.inner_logic()).header_vec == loan.inner_logic().header_vec &&
        (^loan.inner_logic()).original_capacity_repr ==
            loan.inner_logic().original_capacity_repr &&
        (^loan.inner_logic()).storage_model == loan.inner_logic().storage_model &&
        (^loan.inner_logic()).ledger_id == loan.inner_logic().ledger_id &&
        (^loan.inner_logic()).phase_id == loan.inner_logic().phase_id &&
        (^loan.inner_logic()).generation == loan.inner_logic().generation &&
        (^loan.inner_logic()).lent_span == loan.inner_logic().lent_span &&
        (^loan.inner_logic()).lent_region == loan.inner_logic().lent_region &&
        (^loan.inner_logic()).lease_cap.key() == loan.inner_logic().lease_cap.key() &&
        (^loan.inner_logic()).lease_cap.fragment_id() ==
            loan.inner_logic().lease_cap.fragment_id() &&
        header_lease_active((^loan.inner_logic()).lease_cap.fragment_value(),
            (^loan.inner_logic()).lease_id, *(^loan.inner_logic()).stamp) &&
        header_lease_count((^loan.inner_logic()).lease_cap.fragment_value()) == new@
    )]
    #[ensures(
        (^payload.inner_logic()).ledger.unwrap_logic().count() + 1 ==
            payload.inner_logic().ledger.unwrap_logic().count() &&
        (^payload.inner_logic()).ledger.unwrap_logic().ledger_id() ==
            payload.inner_logic().ledger.unwrap_logic().ledger_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().allocation_binding() ==
            payload.inner_logic().ledger.unwrap_logic().allocation_binding() &&
        (^payload.inner_logic()).ledger.unwrap_logic().next_id() ==
            payload.inner_logic().ledger.unwrap_logic().next_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().active_ids() ==
            payload.inner_logic().ledger.unwrap_logic().active_ids().remove(
                ticket.inner_logic().owner_id())
    )]
    #[ensures(
        (^payload.inner_logic()).pool.unwrap_logic().capacity() ==
            payload.inner_logic().pool.unwrap_logic().capacity() &&
        (^payload.inner_logic()).pool.unwrap_logic().namespace() ==
            payload.inner_logic().pool.unwrap_logic().namespace() &&
        (^payload.inner_logic()).pool.unwrap_logic().resource_id() ==
            payload.inner_logic().pool.unwrap_logic().resource_id() &&
        (forall<index: Int> (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
            payload.inner_logic().pool.unwrap_logic().contains(index)) &&
        (forall<index: Int> (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
            payload.inner_logic().pool.unwrap_logic().slot(index)) &&
        (forall<index: Int>
            (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
                (0 <= index && index < (^payload.inner_logic()).pool.unwrap_logic().capacity() &&
                    !active_mutable_contains(
                        (^payload.inner_logic()).ledger.unwrap_logic().active_ids(), index)))
    )]
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
    fn drop_readonly_packet_owner(
        payload: Ghost<&mut SharedPayload>,
        owner_model: Snapshot<NativeCounter>,
        old: Snapshot<usize>,
        new: Snapshot<usize>,
        ticket: Ghost<OwnerTicket>,
        loan: Ghost<&mut SharedHeaderReadLoanState>,
        expected_phase: Snapshot<Id>,
    ) -> Ghost<Option<TerminalAction>> {
        ghost! {
            let payload = payload.into_inner();
            let ticket = ticket.into_inner();
            let loan = loan.into_inner();
            payload.prove_public_phase_id(expected_phase);
            payload.prove_live_owner_ticket(&ticket);
            proof_assert!(ticket.class() == RefKind::Readonly);

            let before_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let before_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            let previous_action: Snapshot<Option<TerminalAction>> =
                snapshot!(payload.terminal_action);
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&*loan),
            );
            let ledger = payload.ledger.as_mut().unwrap();
            let retire = OwnerLedger::drop_owner(
                Ghost::new(ledger), Ghost::new(ticket),
            ).into_inner();
            let remaining_count: Snapshot<Int> =
                snapshot!(payload.ledger.unwrap_logic().count());
            HeaderLeaseAuthority::refresh_active_count(
                Ghost::new(&mut payload.lease),
                loan.stamp,
                remaining_count,
                Ghost::new(&mut loan.lease_cap),
            );
            let after_owners: Snapshot<ActiveReferences> =
                snapshot!(payload.ledger.unwrap_logic().active_ids());
            let after_pool: Snapshot<PhysicalPool> =
                snapshot!(payload.pool.unwrap_logic());
            SharedPayload::prove_pool_complement_after_readonly_release(
                before_pool, after_pool, before_owners, after_owners,
            );
            let terminal_action = match retire {
                Some(retire) => {
                    SharedPayload::prove_full_pool_after_final_owner_release(
                        after_pool, after_owners,
                    );
                    payload.retire = Some(retire);
                    payload.terminal_action.take()
                }
                None => {
                    payload.retire = None;
                    None
                }
            };
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&*loan),
            );
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

    /// Consume one owner's actual rights after the common native Release
    /// event. Mutable regions are retired; Readonly spans are already in the
    /// pool and are removed from the owner ledger without altering that pool.
    #[check(ghost)]
    #[requires(
        payload.inner_logic().invariant() && payload.inner_logic().protocol() &&
        payload.inner_logic().lent_span == None &&
        payload.inner_logic().counter_model() == *owner_model &&
        payload.inner_logic().agrees_with_value(*old) &&
        old@ > 0 && new@ + 1 == old@ &&
        payload.inner_logic().public().3 == *expected_phase &&
        payload.inner_logic().ledger != None &&
        payload.inner_logic().ledger.unwrap_logic().count() == old@
    )]
    #[requires(match owner.inner_logic() {
        SharedReleaseOwner::Mutable { ticket, region } =>
            ticket.invariant() && ticket.ledger_id() == payload.inner_logic().ledger_id &&
            ticket.class() == RefKind::Mutable && region.invariant() &&
            ticket.owned_lo() == region.lo() && ticket.owned_hi() == region.hi() &&
            ticket.allocation_binding().0 == region.namespace() &&
            ticket.allocation_binding().2 == region.capacity(),
        SharedReleaseOwner::Readonly { ticket } =>
            ticket.invariant() && ticket.ledger_id() == payload.inner_logic().ledger_id &&
            ticket.class() == RefKind::Readonly,
    })]
    #[requires(
        loan.inner_logic().invariant() &&
        loan.inner_logic().lease_id == payload.inner_logic().lease.resource_id()
    )]
    #[ensures(
        (^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol() &&
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
        (^payload.inner_logic()).public() == payload.inner_logic().public() &&
        (^payload.inner_logic()).counter_model() == payload.inner_logic().counter_model() &&
        (^payload.inner_logic()).agrees_with_value(*new) &&
        (^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding &&
        (^payload.inner_logic()).header_vec == payload.inner_logic().header_vec &&
        (^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        (^payload.inner_logic()).root == payload.inner_logic().root &&
        (^payload.inner_logic()).storage == payload.inner_logic().storage &&
        (^payload.inner_logic()).phase == payload.inner_logic().phase &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.header_id() == payload.inner_logic().lease.header_id() &&
        (^payload.inner_logic()).lease.count() == new@ &&
        (^payload.inner_logic()).lease_key == payload.inner_logic().lease_key &&
        (^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed
    )]
    #[ensures(
        (^loan.inner_logic()).invariant() &&
        (^loan.inner_logic()).header_id == loan.inner_logic().header_id &&
        (^loan.inner_logic()).counter_model == loan.inner_logic().counter_model &&
        (^loan.inner_logic()).header_vec == loan.inner_logic().header_vec &&
        (^loan.inner_logic()).original_capacity_repr ==
            loan.inner_logic().original_capacity_repr &&
        (^loan.inner_logic()).storage_model == loan.inner_logic().storage_model &&
        (^loan.inner_logic()).ledger_id == loan.inner_logic().ledger_id &&
        (^loan.inner_logic()).phase_id == loan.inner_logic().phase_id &&
        (^loan.inner_logic()).generation == loan.inner_logic().generation &&
        (^loan.inner_logic()).lease_id == loan.inner_logic().lease_id &&
        (^loan.inner_logic()).lease_key == loan.inner_logic().lease_key &&
        (^loan.inner_logic()).stamp == loan.inner_logic().stamp &&
        (^loan.inner_logic()).lent_span == loan.inner_logic().lent_span &&
        (^loan.inner_logic()).lent_region == loan.inner_logic().lent_region &&
        (^loan.inner_logic()).lease_cap.key() == loan.inner_logic().lease_cap.key() &&
        (^loan.inner_logic()).lease_cap.fragment_id() ==
            loan.inner_logic().lease_cap.fragment_id() &&
        header_lease_active((^loan.inner_logic()).lease_cap.fragment_value(),
            (^loan.inner_logic()).lease_id, *(^loan.inner_logic()).stamp) &&
        header_lease_count((^loan.inner_logic()).lease_cap.fragment_value()) == new@
    )]
    #[ensures(
        (^payload.inner_logic()).ledger.unwrap_logic().count() + 1 ==
            payload.inner_logic().ledger.unwrap_logic().count() &&
        (^payload.inner_logic()).ledger.unwrap_logic().ledger_id() ==
            payload.inner_logic().ledger.unwrap_logic().ledger_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().allocation_binding() ==
            payload.inner_logic().ledger.unwrap_logic().allocation_binding() &&
        (^payload.inner_logic()).ledger.unwrap_logic().next_id() ==
            payload.inner_logic().ledger.unwrap_logic().next_id() &&
        (^payload.inner_logic()).ledger.unwrap_logic().active_ids() ==
            payload.inner_logic().ledger.unwrap_logic().active_ids().remove(
                owner.inner_logic().owner_id())
    )]
    #[ensures(
        (^payload.inner_logic()).pool.unwrap_logic().capacity() ==
            payload.inner_logic().pool.unwrap_logic().capacity() &&
        (^payload.inner_logic()).pool.unwrap_logic().namespace() ==
            payload.inner_logic().pool.unwrap_logic().namespace() &&
        (^payload.inner_logic()).pool.unwrap_logic().resource_id() ==
            payload.inner_logic().pool.unwrap_logic().resource_id() &&
        (forall<index: Int> (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
            match owner.inner_logic() {
                SharedReleaseOwner::Mutable { region, .. } =>
                    payload.inner_logic().pool.unwrap_logic().contains(index) ||
                        region.lo() <= index && index < region.hi(),
                SharedReleaseOwner::Readonly { .. } =>
                    payload.inner_logic().pool.unwrap_logic().contains(index),
            }) &&
        (forall<index: Int> (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
            match owner.inner_logic() {
                SharedReleaseOwner::Mutable { region, .. } =>
                    if payload.inner_logic().pool.unwrap_logic().contains(index) {
                        payload.inner_logic().pool.unwrap_logic().slot(index)
                    } else if region.lo() <= index && index < region.hi() {
                        region.slot(index)
                    } else { None },
                SharedReleaseOwner::Readonly { .. } =>
                    payload.inner_logic().pool.unwrap_logic().slot(index),
            }) &&
        (forall<index: Int>
            (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
                (0 <= index && index < (^payload.inner_logic()).pool.unwrap_logic().capacity() &&
                    !active_mutable_contains(
                        (^payload.inner_logic()).ledger.unwrap_logic().active_ids(), index)))
    )]
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
        owner: Ghost<SharedReleaseOwner>,
        loan: Ghost<&mut SharedHeaderReadLoanState>,
        expected_phase: Snapshot<Id>,
    ) -> Ghost<Option<TerminalAction>> {
        ghost! {
            let payload = payload.into_inner();
            let loan = loan.into_inner();
            match owner.into_inner() {
                SharedReleaseOwner::Mutable { ticket, region } => {
                    payload.prove_live_owner_ticket(&ticket);
                    Self::drop_mutable_packet_owner(
                        Ghost::new(payload), owner_model, old, new,
                        Ghost::new(ticket), Ghost::new(region),
                        Ghost::new(loan), expected_phase,
                    ).into_inner()
                }
                SharedReleaseOwner::Readonly { ticket } =>
                    Self::drop_readonly_packet_owner(
                        Ghost::new(payload), owner_model, old, new,
                        Ghost::new(ticket), Ghost::new(loan), expected_phase,
                    ).into_inner(),
            }
        }
    }

    /// A checked idle cap and this packet's actual owner ticket exclude both
    /// active and terminal states: active leases conflict with the exclusive
    /// cap, while terminal retirement conflicts with the live ticket.
    #[check(ghost)]
    #[requires(self.invariant() && self.protocol())]
    #[requires(cap.invariant() && cap.key() == self.lease_key)]
    #[requires(cap.fragment_id() == self.lease.resource_id())]
    #[requires(header_lease_idle(cap.fragment_value(), self.lease.resource_id()))]
    #[requires(ticket.invariant() && ticket.ledger_id() == self.ledger_id)]
    #[ensures(self.root != None && self.lease.is_idle())]
    #[ensures(self.lent_span == None)]
    #[ensures(self.ledger != None && self.ledger.unwrap_logic().is_live())]
    #[ensures(header_lease_count(cap.fragment_value()) == self.lease.count())]
    #[ensures(header_lease_count(cap.fragment_value()) ==
        self.ledger.unwrap_logic().count())]
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
                        self.lent_span,
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
        payload.inner_logic().lent_span == loan.inner_logic().lent_span &&
        loan.inner_logic().lease_cap.fragment_id() ==
            payload.inner_logic().lease.resource_id() &&
        header_lease_count(loan.inner_logic().lease_cap.fragment_value()) ==
            payload.inner_logic().ledger.unwrap_logic().count() &&
        payload.inner_logic().lease.count() ==
            payload.inner_logic().ledger.unwrap_logic().count() &&
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
            loan.inner_logic().lent_span,
            )))]
    fn prove_current_header_lease(
        payload: Ghost<&SharedPayload>,
        loan: Ghost<&SharedHeaderReadLoanState>,
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
                loan.lent_span,
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
                payload.lent_span,
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
    #[requires(payload.inner_logic().lent_span == None)]
    #[requires(payload.inner_logic().agrees_with_value(*value))]
    #[requires(payload.inner_logic().public().4 == loan.inner_logic().state.lease_id)]
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
                payload, snapshot!(loan.state.lease_id),
            );
            SharedPayload::prove_current_header_lease(
                Ghost::new(payload), Ghost::new(&loan.state),
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
    #[requires(payload.inner_logic().lent_span == None)]
    #[requires(payload.inner_logic().root != None && payload.inner_logic().lease.is_idle())]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key() == payload.inner_logic().lease_key)]
    #[requires(cap.inner_logic().fragment_id() == payload.inner_logic().lease.resource_id())]
    #[requires(header_lease_idle(cap.inner_logic().fragment_value(),
        payload.inner_logic().lease.resource_id()))]
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
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed &&
        result.inner_logic().invariant() &&
        result.inner_logic().state.generation == (^payload.inner_logic()).lease_generation &&
        result.inner_logic().state.header_id == payload.inner_logic().header_id &&
        result.inner_logic().state.counter_model == payload.inner_logic().counter_model &&
        (*result.inner_logic().state.storage_model).invariant() &&
        (*result.inner_logic().state.storage_model).binding() ==
            (*payload.inner_logic().allocation_binding).0 &&
        (*result.inner_logic().state.storage_model).raw_pointer() ==
            (*payload.inner_logic().allocation_binding).1 &&
        (*result.inner_logic().state.storage_model).capacity() ==
            (*payload.inner_logic().allocation_binding).2 &&
        result.inner_logic().state.header_vec == payload.inner_logic().header_vec &&
        result.inner_logic().state.original_capacity_repr == payload.inner_logic().original_capacity_repr &&
        result.inner_logic().state.lent_span == payload.inner_logic().lent_span &&
        result.inner_logic().state.ledger_id == payload.inner_logic().ledger_id &&
        result.inner_logic().state.phase_id == payload.inner_logic().phase_id &&
        result.inner_logic().state.lease_id == payload.inner_logic().lease.resource_id() &&
        result.inner_logic().state.lease_key == payload.inner_logic().lease_key &&
        header_lease_count(result.inner_logic().state.lease_cap.fragment_value()) ==
            header_lease_count(cap.inner_logic().fragment_value()) &&
        same_stored_authority(*result.inner_logic().state.storage_model,
            payload.inner_logic().storage.unwrap_logic()))]
    #[ensures(no_active_lend_loan(result.inner_logic()))]
    pub(crate) fn take_header_read_loan(
        payload: Ghost<&mut SharedPayload>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
    ) -> Ghost<SharedHeaderReadLoan> {
        ghost! {
            let payload = payload.into_inner();
            let cap = cap.into_inner();
            payload.lease.require_idle_cap(&cap);
            let root = payload.root.take().unwrap();
            let count: Snapshot<Int> = snapshot!(payload.ledger.unwrap_logic().count());
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
                payload.lent_span,
            ));
            let lease_cap = HeaderLeaseAuthority::activate(
                Ghost::new(&mut payload.lease), stamp, count, Ghost::new(cap),
            ).into_inner();
            payload.lease_generation = generation;
            SharedHeaderReadLoan::from_active_parts(
                root,
                payload.header_id,
                payload.counter_model,
                payload.header_vec,
                payload.original_capacity_repr,
                storage_model,
                payload.ledger_id,
                payload.phase_id,
                generation,
                lease_id,
                lease_key,
                stamp,
                payload.lent_span,
                None,
                lease_cap,
            )
        }
    }

    /// Move one Readonly ticket's exact, fully known physical span out of the
    /// stable pool and pair it with the active header lease. The returned
    /// loan owns the actual region; the marker alone never grants access.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().lent_span == None)]
    #[requires(payload.inner_logic().root != None && payload.inner_logic().lease.is_idle())]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key() == payload.inner_logic().lease_key)]
    #[requires(cap.inner_logic().fragment_id() == payload.inner_logic().lease.resource_id())]
    #[requires(header_lease_idle(cap.inner_logic().fragment_value(),
        payload.inner_logic().lease.resource_id()))]
    #[requires(ticket.inner_logic().invariant() &&
        ticket.inner_logic().ledger_id() == payload.inner_logic().ledger_id)]
    #[requires(ticket.inner_logic().class() == RefKind::Readonly)]
    #[ensures(exact_header_region_lend_transition(
        *payload.inner_logic(), ^payload.inner_logic(),
        *ticket.inner_logic(), result.inner_logic(),
    ))]
    #[ensures(result.inner_logic().state.lent_region != None)]
    #[ensures(result.inner_logic().state.lent_region.unwrap_logic().namespace() ==
        ticket.inner_logic().allocation_binding().0)]
    #[ensures(result.inner_logic().state.lent_region.unwrap_logic().capacity() ==
        ticket.inner_logic().allocation_binding().2)]
    #[ensures(header_lease_count(
        result.inner_logic().state.lease_cap.fragment_value()) ==
        header_lease_count(cap.inner_logic().fragment_value()))]
    pub(crate) fn take_header_region_read_loan(
        payload: Ghost<&mut SharedPayload>,
        cap: Ghost<ReadyCap<HeaderLeaseState>>,
        ticket: Ghost<&OwnerTicket>,
    ) -> Ghost<SharedHeaderReadLoan> {
        ghost! {
            let payload = payload.into_inner();
            let cap = cap.into_inner();
            let ticket = ticket.into_inner();
            payload.lease.require_idle_cap(&cap);
            payload.prove_idle_cap_implies_root_some(&cap, ticket);
            payload.prove_readonly_ticket_pool_known(ticket);
            payload.prove_stable_active_pool_protocol();

            let owner_id = snapshot!(ticket.owner_id()).into_ghost().into_inner();
            let lo = snapshot!(ticket.owned_lo()).into_ghost().into_inner();
            let hi = snapshot!(ticket.owned_hi()).into_ghost().into_inner();
            let lent_span = Some((owner_id, lo, hi));
            let ledger_model: Snapshot<OwnerLedger> =
                snapshot!(payload.ledger.unwrap_logic());
            let storage_model: Snapshot<StoredAuthority> =
                snapshot!(payload.storage.unwrap_logic());
            let old_pool = payload.pool.take().unwrap();
            let (pool, region) = lend_active_readonly_pool_interval(
                old_pool,
                ledger_model,
                storage_model,
                owner_id,
                lo,
                hi,
            );
            payload.pool = Some(pool);
            payload.lent_span = lent_span;

            let root = payload.root.take().unwrap();
            let count: Snapshot<Int> = snapshot!(payload.ledger.unwrap_logic().count());
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
                lent_span,
            ));
            let lease_cap = HeaderLeaseAuthority::activate(
                Ghost::new(&mut payload.lease), stamp, count, Ghost::new(cap),
            ).into_inner();
            payload.lease_generation = generation;

            proof_assert!(payload.lease.resource_id() == lease_id);
            proof_assert!(header_lease_active(lease_cap.fragment_value(), lease_id,
                header_lease_stamp(
                    generation,
                    payload.header_id,
                    *payload.counter_model,
                    payload.ledger_id,
                    payload.phase_id,
                    *storage_model,
                    *payload.header_vec,
                    *payload.original_capacity_repr,
                    lease_key,
                    lent_span,
                )));
            proof_assert!(header_lease_count(lease_cap.fragment_value()) == *count);

            proof_assert!(lent_region_matches(
                lent_span, Some(region), *storage_model,
            ));
            let loan = SharedHeaderReadLoan::from_active_parts(
                root,
                payload.header_id,
                payload.counter_model,
                payload.header_vec,
                payload.original_capacity_repr,
                storage_model,
                payload.ledger_id,
                payload.phase_id,
                generation,
                lease_id,
                lease_key,
                stamp,
                lent_span,
                Some(region),
                lease_cap,
            );
            proof_assert!(payload.invariant() && payload.protocol());
            loan
        }
    }

    /// Consume an active Readonly region loan, retire its actual affine
    /// region into the complementary pool, deactivate the exact lease stamp,
    /// and restore the saved header root. The pool and region join is the
    /// body-checked inverse of `take_header_region_read_loan`.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().root == None)]
    #[requires(payload.inner_logic().lent_span == loan.inner_logic().state.lent_span)]
    #[requires(match (loan.inner_logic().state.lent_span, loan.inner_logic().state.lent_region) {
        (Some((_, lo, hi)), Some(region)) => region.invariant() &&
            region.lo() == lo && region.hi() == hi,
        _ => false,
    })]
    #[requires(loan.inner_logic().invariant())]
    #[requires(payload.inner_logic().lease.resource_id() == loan.inner_logic().state.lease_id)]
    #[requires(payload.inner_logic().lease_key == loan.inner_logic().state.lease_key)]
    #[ensures((^payload.inner_logic()).invariant() && (^payload.inner_logic()).protocol())]
    #[ensures((^payload.inner_logic()).root == Some(loan.inner_logic().root))]
    #[ensures((^payload.inner_logic()).lent_span == None)]
    #[ensures((^payload.inner_logic()).lease.is_idle())]
    #[ensures((^payload.inner_logic()).lease_generation == payload.inner_logic().lease_generation)]
    #[ensures((^payload.inner_logic()).header_id == payload.inner_logic().header_id)]
    #[ensures((^payload.inner_logic()).counter_model == payload.inner_logic().counter_model)]
    #[ensures((^payload.inner_logic()).allocation_binding == payload.inner_logic().allocation_binding)]
    #[ensures((^payload.inner_logic()).header_vec == payload.inner_logic().header_vec)]
    #[ensures((^payload.inner_logic()).original_capacity_repr == payload.inner_logic().original_capacity_repr)]
    #[ensures((^payload.inner_logic()).ledger_id == payload.inner_logic().ledger_id)]
    #[ensures((^payload.inner_logic()).phase_id == payload.inner_logic().phase_id)]
    #[ensures((^payload.inner_logic()).storage == payload.inner_logic().storage)]
    #[ensures((^payload.inner_logic()).ledger == payload.inner_logic().ledger)]
    #[ensures((^payload.inner_logic()).retire == payload.inner_logic().retire)]
    #[ensures((^payload.inner_logic()).phase == payload.inner_logic().phase)]
    #[ensures((^payload.inner_logic()).terminal_action == payload.inner_logic().terminal_action)]
    #[ensures((^payload.inner_logic()).lease_key == payload.inner_logic().lease_key)]
    #[ensures((^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id())]
    #[ensures((^payload.inner_logic()).closed == payload.inner_logic().closed)]
    #[ensures(active_lease_pool_protocol(
        (^payload.inner_logic()).pool.unwrap_logic(),
        (^payload.inner_logic()).ledger.unwrap_logic(),
        (^payload.inner_logic()).storage.unwrap_logic().capacity(),
        (^payload.inner_logic()).storage.unwrap_logic().binding(),
        None,
    ))]
    #[ensures(forall<index: Int>
        (^payload.inner_logic()).pool.unwrap_logic().contains(index) ==
            (payload.inner_logic().pool.unwrap_logic().contains(index) ||
                match loan.inner_logic().state.lent_span {
                    Some((_, lo, hi)) => lo <= index && index < hi,
                    None => false,
                }))]
    #[ensures(forall<index: Int>
        (^payload.inner_logic()).pool.unwrap_logic().slot(index) ==
            if payload.inner_logic().pool.unwrap_logic().contains(index) {
                payload.inner_logic().pool.unwrap_logic().slot(index)
            } else if match loan.inner_logic().state.lent_span {
                Some((_, lo, hi)) => lo <= index && index < hi,
                None => false,
            } {
                loan.inner_logic().state.lent_region.unwrap_logic().slot(index)
            } else { None })]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == loan.inner_logic().state.lease_key)]
    #[ensures(result.inner_logic().fragment_id() == payload.inner_logic().lease.resource_id())]
    #[ensures(header_lease_idle(
        result.inner_logic().fragment_value(), payload.inner_logic().lease.resource_id()))]
    #[ensures(header_lease_count(result.inner_logic().fragment_value()) ==
        (^payload.inner_logic()).ledger.unwrap_logic().count())]
    pub(crate) fn restore_header_region_read_loan(
        payload: Ghost<&mut SharedPayload>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let payload = payload.into_inner();
            let loan = loan.into_inner();
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&loan.state),
            );

            let SharedHeaderReadLoan { root, state } = loan;
            let SharedHeaderReadLoanState {
                lent_span, lent_region, stamp, lease_cap, ..
            } = state;
            let (owner_id, lo, hi) = lent_span.unwrap();
            let region = lent_region.unwrap();
            let storage_model: Snapshot<StoredAuthority> =
                snapshot!(payload.storage.unwrap_logic());
            let pool = payload.pool.take().unwrap();
            let ledger = payload.ledger.as_ref().unwrap();
            let pool = retire_active_readonly_pool_interval(
                pool,
                region,
                ledger,
                storage_model,
                owner_id,
                lo,
                hi,
            );
            payload.pool = Some(pool);

            let count: Snapshot<Int> = snapshot!(payload.ledger.unwrap_logic().count());
            let cap = HeaderLeaseAuthority::deactivate(
                Ghost::new(&mut payload.lease), stamp, count, Ghost::new(lease_cap),
            ).into_inner();
            payload.lent_span = None;
            payload.root = Some(root);
            proof_assert!(payload.invariant() && payload.protocol());
            cap
        }
    }

    /// Consume the matching affine lease fragment and restore the exact root.
    /// The resource-algebra agreement makes the active stamp stable across
    /// intervening opens; counter events may update only the other payload
    /// resources while this lease is held.
    #[check(ghost)]
    #[requires(payload.inner_logic().invariant() && payload.inner_logic().protocol())]
    #[requires(payload.inner_logic().lent_span == None)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(no_active_lend_loan(loan.inner_logic()))]
    #[requires(payload.inner_logic().lease.resource_id() == loan.inner_logic().state.lease_id)]
    #[requires(payload.inner_logic().lease_key == loan.inner_logic().state.lease_key)]
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
        (^payload.inner_logic()).lent_span == payload.inner_logic().lent_span &&
        (^payload.inner_logic()).lease.resource_id() == payload.inner_logic().lease.resource_id() &&
        (^payload.inner_logic()).lease.is_idle() &&
        (^payload.inner_logic()).closed == payload.inner_logic().closed)]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == loan.inner_logic().state.lease_key)]
    #[ensures(result.inner_logic().fragment_id() == payload.inner_logic().lease.resource_id())]
    #[ensures(header_lease_idle(
        result.inner_logic().fragment_value(), payload.inner_logic().lease.resource_id()))]
    #[ensures(header_lease_count(result.inner_logic().fragment_value()) ==
        (^payload.inner_logic()).ledger.unwrap_logic().count())]
    #[ensures(header_lease_count(result.inner_logic().fragment_value()) ==
        header_lease_count(
            loan.inner_logic().state.lease_cap.fragment_value()))]
    pub(crate) fn restore_header_read_loan(
        payload: Ghost<&mut SharedPayload>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<ReadyCap<HeaderLeaseState>> {
        ghost! {
            let payload = payload.into_inner();
            let loan = loan.into_inner();
            let stamp = snapshot!(header_lease_stamp(
                loan.state.generation,
                loan.state.header_id,
                *loan.state.counter_model,
                loan.state.ledger_id,
                loan.state.phase_id,
                *loan.state.storage_model,
                *loan.state.header_vec,
                *loan.state.original_capacity_repr,
                loan.state.lease_key,
                loan.state.lent_span,
            ));
            SharedPayload::prove_current_header_lease(
                Ghost::new(&*payload), Ghost::new(&loan.state),
            );
            let SharedHeaderReadLoan { root, state } = loan;
            let SharedHeaderReadLoanState { lease_cap, .. } = state;
            let count: Snapshot<Int> = snapshot!(payload.ledger.unwrap_logic().count());
            let cap = HeaderLeaseAuthority::deactivate(
                Ghost::new(&mut payload.lease), stamp, count, Ghost::new(lease_cap),
            ).into_inner();
            payload.root = Some(root);
            cap
        }
    }
}

/// Read the exact NativeCounter through a live affine header loan. The
/// returned borrow is tied to the loan, and the only exposed header field is
/// the immutable counter cell.
#[requires(root.inner_logic().invariant())]
#[requires(*root.inner_logic().ward() == header_id)]
#[ensures(result == root.inner_logic().val().ref_count.counter_reference())]
#[ensures(*result == root.inner_logic().val().ref_count.counter_model())]
#[check(terminates)]
pub(crate) fn loaned_native_counter<'a>(
    header_id: *const Shared,
    root: Ghost<&'a Perm<*const Shared>>,
) -> &'a NativeCounter {
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

/// Per-handle immutable view after its Mutable physical region has been
/// returned to the capsule's pool. The ticket, persistent capsule, and
/// producer-issued lease key are the original affine/persistent values;
/// pointer and length fields are ordinary metadata. The pool observer below
/// exposes slot facts only and does not grant byte-read authority.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedReadonlyPacket {
    pub(crate) capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
    pub(crate) ticket: Ghost<OwnerTicket>,
    pub(crate) base: BoundPtr,
    pub(crate) cursor: BoundPtr,
    pub(crate) len: usize,
    pub(crate) lease_key: Ghost<ReadyKey>,
}

/// Result of the one real Acquire used by the original unique shared-view
/// conversion. The same readonly handle remains available when the observed
/// count is not one; the unique arm owns the reclassified ticket and actual
/// physical suffix returned by the capsule transition.
#[allow(missing_debug_implementations)]
pub(crate) enum SharedReadonlyPromotion {
    Unique(SharedOwnerPacket),
    NotUnique(SharedReadonlyPacket),
}

/// The actual ownership transition produced inside the one-Acquire callback.
/// It deliberately carries only the owner ticket and optional physical region;
/// packet metadata and Context restoration stay outside the atomic callback.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedReadonlyOwnerTransition {
    pub(crate) ticket: Ghost<OwnerTicket>,
    pub(crate) region: Option<Ghost<PhysicalRegion>>,
    pub(crate) visible_hi: Snapshot<Int>,
}

impl SharedReadonlyOwnerTransition {
    #[logic(open(self), prophetic)]
    pub(crate) fn valid(self) -> bool {
        pearlite! {
            self.ticket.inner_logic().invariant() &&
            match self.region {
                Some(region) =>
                    self.ticket.inner_logic().class() == RefKind::Mutable &&
                    region.inner_logic().invariant() &&
                    region.inner_logic().resource_id() == region.inner_logic().namespace() &&
                    region.inner_logic().namespace() ==
                        self.ticket.inner_logic().allocation_binding().0 &&
                    region.inner_logic().capacity() ==
                        self.ticket.inner_logic().allocation_binding().2 &&
                    self.ticket.inner_logic().owned_hi() ==
                        self.ticket.inner_logic().allocation_binding().2 &&
                    region.inner_logic().lo() == self.ticket.inner_logic().owned_lo() &&
                    region.inner_logic().hi() == self.ticket.inner_logic().owned_hi() &&
                    self.ticket.inner_logic().owned_lo() <= *self.visible_hi &&
                    *self.visible_hi <= self.ticket.inner_logic().owned_hi() &&
                    (forall<index: Int>
                        self.ticket.inner_logic().owned_lo() <= index &&
                        index < *self.visible_hi ==>
                            slot_known(region.inner_logic().slot(index))),
                None => self.ticket.inner_logic().class() == RefKind::Readonly,
            }
        }
    }
}

impl Invariant for SharedReadonlyOwnerTransition {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

/// Common proof carrier for the one Release event shared by Mutable and
/// Readonly handles. The variant preserves the packet's actual capabilities;
/// the Readonly variant carries no PhysicalRegion because its slots are
/// already in the payload pool.
#[allow(missing_debug_implementations)]
pub(crate) enum SharedReleasePacket {
    Mutable(SharedOwnerPacket),
    Readonly(SharedReadonlyPacket),
}

/// Proof-only input consumed by the source-level shared release body. The
/// packet carries this handle's actual owner ticket and region, while the
/// borrowed context remains in a Ghost parameter so none of its rights enter
/// the native representation.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedReleaseProof<'context, 'tokens, 'peer> {
    pub(crate) packet: Ghost<SharedReleasePacket>,
    pub(crate) context: Ghost<
        &'context mut BorrowedSharedContext<'tokens, HeaderLeaseState>,
    >,
    pub(crate) peer_ticket: Ghost<Option<&'peer OwnerTicket>>,
}

/// Actual capabilities returned only after this packet wins the final
/// reference decrement and the capsule has been terminalized.
#[allow(missing_debug_implementations)]
pub(crate) struct SharedRetirementResources {
    pub(crate) storage: StoredAuthority,
    pub(crate) region: PhysicalRegion,
    pub(crate) root: Box<Perm<*const Shared>>,
}

/// Actual per-reference rights consumed by the common Release event. Mutable
/// owners return their region to the pool; readonly owners already have their
/// visible span represented there and therefore carry no region to retire.
#[allow(missing_debug_implementations)]
pub(crate) enum SharedReleaseOwner {
    Mutable {
        ticket: OwnerTicket,
        region: PhysicalRegion,
    },
    Readonly {
        ticket: OwnerTicket,
    },
}

impl SharedReleaseOwner {
    #[logic(open(self))]
    fn owner_id(&self) -> Int {
        pearlite! {
            match self {
                SharedReleaseOwner::Mutable { ticket, .. } => ticket.owner_id(),
                SharedReleaseOwner::Readonly { ticket } => ticket.owner_id(),
            }
        }
    }

    #[logic(open(self))]
    fn is_mutable(&self) -> bool {
        pearlite! {
            match self {
                SharedReleaseOwner::Mutable { .. } => true,
                SharedReleaseOwner::Readonly { .. } => false,
            }
        }
    }
}

impl SharedReleasePacket {
    /// Common context acquisition for both packet classes. It borrows the
    /// packet's actual ticket and preserves the same ready cap/loan protocol;
    /// packet geometry never supplies authority here.
    #[check(ghost)]
    #[requires(capsule.inner_logic().invariant())]
    #[requires(capsule.inner_logic().val().namespace() == event_namespace())]
    #[requires(capsule.inner_logic().val().public().1.6 == key.inner_logic())]
    #[requires(ticket.inner_logic().invariant())]
    #[requires(ticket.inner_logic().ledger_id() ==
        capsule.inner_logic().val().public().1.2)]
    #[requires(ticket.inner_logic().allocation_binding() ==
        capsule.inner_logic().val().public().1.5)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(!context.inner_logic().ready_has_key(key.inner_logic()) ||
        header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(key.inner_logic().slot()), key.inner_logic().resource_id(), key.inner_logic().slot(), key.inner_logic().resource_id()))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((result.inner_logic() != None) ==
        context.inner_logic().ready_has_key(key.inner_logic()))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != key.inner_logic().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    #[ensures(result.inner_logic() == None ==>
        (^context.inner_logic()).ready_entry_state(key.inner_logic().slot()) ==
            context.inner_logic().ready_entry_state(key.inner_logic().slot()))]
    #[ensures(result.inner_logic() != None ==>
        (^context.inner_logic()).ready_entry_state(key.inner_logic().slot()) == None)]
    #[ensures(result.inner_logic() != None ==>
        no_active_lend_loan(result.inner_logic().unwrap_logic()))]
    #[ensures(result.inner_logic() != None ==>
        result.inner_logic().unwrap_logic().invariant() &&
        result.inner_logic().unwrap_logic().state.lease_key == key.inner_logic() &&
        result.inner_logic().unwrap_logic().state.lease_id == key.inner_logic().resource_id() &&
        result.inner_logic().unwrap_logic().state.header_id ==
            capsule.inner_logic().val().public().1.0 as *mut Shared &&
        *result.inner_logic().unwrap_logic().state.counter_model ==
            capsule.inner_logic().val().public().1.1 &&
        result.inner_logic().unwrap_logic().state.phase_id ==
            capsule.inner_logic().val().public().1.3 &&
        (*result.inner_logic().unwrap_logic().state.storage_model).invariant() &&
        (*result.inner_logic().unwrap_logic().state.storage_model).binding() ==
            ticket.inner_logic().allocation_binding().0 &&
        (*result.inner_logic().unwrap_logic().state.storage_model).raw_pointer() ==
            ticket.inner_logic().allocation_binding().1 &&
        (*result.inner_logic().unwrap_logic().state.storage_model).capacity() ==
            ticket.inner_logic().allocation_binding().2)]
    #[ensures(result.inner_logic() != None ==>
        header_lease_count(
            result.inner_logic().unwrap_logic().state.lease_cap.fragment_value()
        ) == header_lease_ready_entry_count(
            context.inner_logic().ready_entry_state(key.inner_logic().slot()),
            key.inner_logic().resource_id(), key.inner_logic().slot()))]
    fn try_take_header_read_loan_for(
        capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
        key: Ghost<ReadyKey>,
        ticket: Ghost<&OwnerTicket>,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> Ghost<Option<SharedHeaderReadLoan>> {
        ghost! {
            let context = context.into_inner();
            let key = key.into_inner();
            if !context.ready_has_key_ghost(key).into_inner() {
                None
            } else {
                let cap = BorrowedSharedContext::take(Ghost::new(&mut *context), key);
                let expected_count: Snapshot<Int> = snapshot!(
                    header_lease_count(cap.inner_logic().fragment_value())
                );
                let tokens = BorrowedSharedContext::reborrow_tokens(
                    Ghost::new(&mut *context),
                );
                let expected_key: Snapshot<ReadyKey> = snapshot!(key);
                let expected_lease_id: Snapshot<Id> = snapshot!(key.resource_id());
                let expected_phase: Snapshot<Id> = snapshot!(
                    capsule.inner_logic().val().public().1.3
                );
                capsule.to_ref().open_with_tokens_with_frame(
                    tokens,
                    expected_count,
                    ghost!(
                        #[requires(
                            payload.invariant() && payload.protocol() &&
                            payload.public().3 == *expected_phase &&
                            payload.counter_model() == *owner_model &&
                            payload.agrees_with_value(*owner_value) &&
                            payload.public().4 == *expected_lease_id &&
                            payload.public().6 == *expected_key &&
                            ticket.inner_logic().invariant() &&
                            ticket.inner_logic().ledger_id() == payload.public().2 &&
                            cap.inner_logic().invariant() &&
                            cap.inner_logic().key() == *expected_key &&
                            cap.inner_logic().fragment_id() == *expected_lease_id &&
                            header_lease_idle(
                                cap.inner_logic().fragment_value(), *expected_lease_id) &&
                            header_lease_count(cap.inner_logic().fragment_value()) ==
                                *expected_count
                        )]
                        #[ensures(
                            (^payload).public() == payload.public() &&
                            (^payload).counter_model() == payload.counter_model() &&
                            (^payload).protocol() &&
                            (^payload).agrees_with_value(*owner_value) &&
                            match result {
                                Some(loan) => loan.invariant() &&
                                    no_active_lend_loan(loan) &&
                                    loan.state.header_id == payload.header_id &&
                                    loan.state.lease_id == payload.lease.resource_id() &&
                                    loan.state.lease_key == payload.lease_key &&
                                    loan.state.phase_id == payload.public().3 &&
                                    *loan.state.counter_model == *payload.counter_model &&
                                    (*loan.state.storage_model).binding() ==
                                        (*payload.allocation_binding).0 &&
                                    (*loan.state.storage_model).raw_pointer() ==
                                        (*payload.allocation_binding).1 &&
                                    (*loan.state.storage_model).capacity() ==
                                        (*payload.allocation_binding).2 &&
                                    (*loan.state.storage_model).invariant() &&
                                    header_lease_count(
                                        loan.state.lease_cap.fragment_value()) == *expected_count,
                                None => false,
                            }
                        )]
                        move |payload: &mut SharedPayload,
                              owner_model: Snapshot<NativeCounter>,
                              owner_value: Snapshot<usize>,
                              expected_count: Snapshot<Int>| {
                            let cap = cap.into_inner();
                            let ticket = ticket.into_inner();
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

    #[logic(open(self), prophetic)]
    fn valid(self) -> bool {
        pearlite! {
            match self {
                SharedReleasePacket::Mutable(packet) => packet.invariant(),
                SharedReleasePacket::Readonly(packet) => packet.invariant(),
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn capsule_public(self) -> (*mut (), NativeCounter, Id, Id) {
        pearlite! {
            match self {
                SharedReleasePacket::Mutable(packet) => packet.capsule_public(),
                SharedReleasePacket::Readonly(packet) => packet.capsule_public(),
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn capsule_namespace(self) -> Namespace {
        pearlite! {
            match self {
                SharedReleasePacket::Mutable(packet) =>
                    packet.capsule.inner_logic().val().namespace(),
                SharedReleasePacket::Readonly(packet) =>
                    packet.capsule.inner_logic().val().namespace(),
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn lease_key(&self) -> ReadyKey {
        pearlite! {
            match self {
                SharedReleasePacket::Mutable(packet) => packet.lease_key(),
                SharedReleasePacket::Readonly(packet) => packet.lease_key(),
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn owner_allocation_binding(self) -> AllocationBinding {
        pearlite! {
            match self {
                SharedReleasePacket::Mutable(packet) =>
                    packet.ticket.inner_logic().allocation_binding(),
                SharedReleasePacket::Readonly(packet) =>
                    packet.ticket.inner_logic().allocation_binding(),
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn owner_id(self) -> Int {
        pearlite! {
            match self {
                SharedReleasePacket::Mutable(packet) =>
                    packet.ticket.inner_logic().owner_id(),
                SharedReleasePacket::Readonly(packet) =>
                    packet.ticket.inner_logic().owner_id(),
            }
        }
    }

    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(!context.inner_logic().ready_has_key(self.lease_key()) ||
        header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(self.lease_key().slot()), self.lease_key().resource_id(), self.lease_key().slot(), self.lease_key().resource_id()))]
    #[ensures((^self).invariant() && (^self).lease_key() == self.lease_key())]
    #[ensures((^self) == *self)]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((result.inner_logic() != None) ==
        context.inner_logic().ready_has_key(self.lease_key()))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != self.lease_key().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) ==
            context.inner_logic().ready_entry_state(i))]
    #[ensures(result.inner_logic() == None ==>
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()) ==
            context.inner_logic().ready_entry_state(self.lease_key().slot()))]
    #[ensures(result.inner_logic() != None ==>
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()) == None)]
    #[ensures(result.inner_logic() != None ==>
        result.inner_logic().unwrap_logic().invariant() &&
        result.inner_logic().unwrap_logic().state.lease_key == self.lease_key() &&
        no_active_lend_loan(result.inner_logic().unwrap_logic()) &&
        result.inner_logic().unwrap_logic().state.lease_id == self.lease_key().resource_id() &&
        result.inner_logic().unwrap_logic().state.header_id ==
            self.capsule_public().0 as *mut Shared &&
        *result.inner_logic().unwrap_logic().state.counter_model == self.capsule_public().1 &&
        result.inner_logic().unwrap_logic().state.phase_id == self.capsule_public().3)]
    #[ensures(result.inner_logic() != None ==>
        header_lease_count(
            result.inner_logic().unwrap_logic().state.lease_cap.fragment_value()
        ) == header_lease_ready_entry_count(
            context.inner_logic().ready_entry_state(self.lease_key().slot()),
            self.lease_key().resource_id(), self.lease_key().slot()))]
    pub(crate) fn try_take_header_read_loan(
        &mut self,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> Ghost<Option<SharedHeaderReadLoan>> {
        match self {
            SharedReleasePacket::Mutable(packet) =>
                SharedReleasePacket::try_take_header_read_loan_for(
                    packet.capsule,
                    packet.lease_key_ghost(),
                    ghost!(&*packet.ticket),
                    context,
                ),
            SharedReleasePacket::Readonly(packet) =>
                SharedReleasePacket::try_take_header_read_loan_for(
                    packet.capsule,
                    packet.lease_key,
                    ghost!(&*packet.ticket),
                    context,
                ),
        }
    }
}

impl Invariant for SharedReleasePacket {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

impl SharedOwnerPacket {
    /// Read the retained Vec's metadata under the packet's actual header root.
    /// The lease is temporarily removed from the caller's registry, the
    /// original Shared header is borrowed through its root permission, and
    /// the same idle cap is restored before this returns.
    #[check(terminates)]
    #[requires(self.invariant())]
    #[requires(header_id as *mut () == self.capsule_public().0)]
    #[requires(pointer == self.cursor.raw_pointer() as *const u8)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(self.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(header_lease_ready_entry_idle(
        context.inner_logic().ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot(),
        self.lease_key().resource_id()))]
    #[ensures(self.invariant())]
    #[ensures((^self) == *self)]
    #[ensures(result.0 == self.base.raw_pointer())]
    #[ensures(result.1@ <= result.2@)]
    #[ensures(result.2@ == self.ticket.inner_logic().allocation_binding().2)]
    #[ensures(result.3@ == self.cursor@.unwrap_logic().2)]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_has_key(self.lease_key()))]
    #[ensures(header_lease_ready_entry_idle(
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot(),
        self.lease_key().resource_id()))]
    #[ensures(header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot()) ==
        header_lease_ready_entry_count(
            context.inner_logic().ready_entry_state(self.lease_key().slot()),
            self.lease_key().resource_id(), self.lease_key().slot()))]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() &&
        index != self.lease_key().slot() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    pub(crate) fn read_vec_metadata_offset(
        &mut self,
        header_id: *mut Shared,
        pointer: *const u8,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> (*mut u8, usize, usize, isize) {
        let mut context_ref = ghost!(context.into_inner());
        let key = self.lease_key_ghost();
        let capsule = self.capsule;
        let loan = ghost! {
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&**context_ref), key,
            );
            SharedReleasePacket::try_take_header_read_loan_for(
                capsule,
                key,
                Ghost::new(&*self.ticket),
                Ghost::new(&mut **context_ref),
            ).into_inner().unwrap()
        };
        let storage_authority = ghost! {
            loan.borrow().into_inner().storage_authority_snapshot()
        };
        let (mut root, state) = ghost! {
            let SharedHeaderReadLoan { root, state } = loan.into_inner();
            (root, state)
        }.split();
        let region = self.region.borrow();
        let (metadata, offset) = {
            let header = unsafe {
                Perm::as_mut(header_id, ghost! { root.as_mut() })
            };
            header.vec_mut().prove_authority_matches(storage_authority);
            let (pointer, len, capacity, offset) = unsafe {
                crate::vec_storage::shared_vec_metadata_offset(
                    header.vec_mut(),
                    pointer,
                    &self.base,
                    &self.cursor,
                    region,
                    storage_authority,
                )
            };
            ((pointer, len, capacity), offset)
        };
        let loan = ghost! {
            SharedHeaderReadLoan {
                root: root.into_inner(),
                state: state.into_inner(),
            }
        };
        SharedOwnerPacket::restore_lease_after_owner_release(
            capsule,
            key,
            ghost! { &mut **context_ref },
            loan,
        );
        (metadata.0, metadata.1, metadata.2, offset)
    }

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

    /// Move this Mutable packet's actual ticket and region into the common
    /// Release carrier without changing either capability.
    #[requires(self.invariant())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic() == SharedReleasePacket::Mutable(self))]
    #[ensures(result.inner_logic().capsule_public() == self.capsule_public())]
    #[ensures(result.inner_logic().capsule_namespace() ==
        self.capsule.inner_logic().val().namespace())]
    #[ensures(result.inner_logic().lease_key() == self.lease_key())]
    #[ensures(result.inner_logic().owner_allocation_binding() ==
        self.ticket.inner_logic().allocation_binding())]
    #[ensures(result.inner_logic().owner_id() ==
        self.ticket.inner_logic().owner_id())]
    pub(crate) fn into_release_packet(self) -> Ghost<SharedReleasePacket> {
        let SharedOwnerPacket { capsule, ticket, region, base, cursor, lease_key } = self;
        ghost! {
            SharedReleasePacket::Mutable(SharedOwnerPacket {
                capsule, ticket, region, base, cursor, lease_key,
            })
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
    #[ensures((self.capsule_public().0.addr_logic() & super::KIND_MASK) ==
        super::KIND_ARC)]
    #[ensures(self.ticket.inner_logic().allocation_binding() ==
        (self.region.inner_logic().namespace(), self.base.raw_pointer(),
            self.region.inner_logic().capacity()))]
    #[ensures(self.ticket.inner_logic().allocation_binding() ==
        self.capsule_allocation_binding())]
    #[ensures(self.ticket.inner_logic().owned_lo() == self.region.inner_logic().lo())]
    #[ensures(self.ticket.inner_logic().owned_hi() == self.region.inner_logic().hi())]
    pub(crate) fn prove_cursor_within_region(&self) {
        proof_assert!(self.region.inner_logic().lo() <=
            self.cursor@.unwrap_logic().2);
        proof_assert!((self.capsule_public().0.addr_logic() & super::KIND_MASK) ==
            super::KIND_ARC);
        proof_assert!(self.cursor@.unwrap_logic().2 <=
            self.region.inner_logic().hi());
        proof_assert!(self.ticket.inner_logic().allocation_binding() ==
            (self.region.inner_logic().namespace(), self.base.raw_pointer(),
                self.region.inner_logic().capacity()));
        proof_assert!(self.ticket.inner_logic().allocation_binding() ==
            self.capsule_allocation_binding());
        proof_assert!(self.ticket.inner_logic().owned_lo() == self.region.inner_logic().lo());
        proof_assert!(self.ticket.inner_logic().owned_hi() == self.region.inner_logic().hi());
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
            (self.capsule_public().0.addr_logic() & super::KIND_MASK) ==
                super::KIND_ARC &&
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

impl Invariant for SharedReadonlyPacket {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.valid()
    }
}

impl SharedReadonlyOwnerTransition {
    /// Reassemble a packet after the atomic callback has returned only the
    /// changed affine owner rights. The source snapshot supplies value facts;
    /// the capsule, pointer descriptors, ticket, and region below are the
    /// actual values moved through the checked transitions.
    #[check(ghost)]
    // Validate the historical packet value itself, not Snapshot's trivial
    // wrapper invariant. Actual permissions are still supplied separately.
    #[requires((*source).invariant())]
    #[requires(transition.inner_logic().invariant())]
    #[requires(capsule.inner_logic() == source.capsule.inner_logic())]
    #[requires(base == source.base)]
    #[requires(cursor == source.cursor)]
    #[requires(len@ == source.len@)]
    #[requires(lease_key.inner_logic() == source.lease_key.inner_logic())]
    #[requires(transition.inner_logic().ticket.inner_logic().ledger_id() ==
        source.ticket.inner_logic().ledger_id())]
    #[requires(transition.inner_logic().ticket.inner_logic().owner_id() ==
        source.ticket.inner_logic().owner_id())]
    #[requires(transition.inner_logic().ticket.inner_logic().fragment_resource_id() ==
        source.ticket.inner_logic().fragment_resource_id())]
    #[requires(transition.inner_logic().ticket.inner_logic().allocation_binding() ==
        source.ticket.inner_logic().allocation_binding())]
    #[requires(match transition.inner_logic().region {
        Some(_) => value@ == 1 &&
            transition.inner_logic().ticket.inner_logic().class() == RefKind::Mutable &&
            transition.inner_logic().ticket.inner_logic().owned_lo() ==
                cursor@.unwrap_logic().2 &&
            transition.inner_logic().ticket.inner_logic().owned_hi() ==
                source.ticket.inner_logic().allocation_binding().2,
        None => value@ != 1,
    })]
    #[requires(*transition.inner_logic().visible_hi ==
        cursor@.unwrap_logic().2 + len@)]
    #[requires(match transition.inner_logic().region {
        Some(_) => true,
        None => transition.inner_logic().ticket.inner_logic() ==
            source.ticket.inner_logic(),
    })]
    #[ensures(match result {
        SharedReadonlyPromotion::Unique(packet) => value@ == 1 &&
            packet.invariant() &&
            packet.capsule.inner_logic() == capsule.inner_logic() &&
            packet.capsule_public() == (*source).capsule_public() &&
            packet.ticket.inner_logic().ledger_id() ==
                source.ticket.inner_logic().ledger_id() &&
            packet.ticket.inner_logic().owner_id() ==
                source.ticket.inner_logic().owner_id() &&
            packet.ticket.inner_logic().fragment_resource_id() ==
                source.ticket.inner_logic().fragment_resource_id() &&
            packet.ticket.inner_logic().allocation_binding() ==
                source.ticket.inner_logic().allocation_binding() &&
            packet.ticket.inner_logic().class() == RefKind::Mutable &&
            packet.ticket.inner_logic().owned_lo() == cursor@.unwrap_logic().2 &&
            packet.ticket.inner_logic().owned_hi() ==
                source.ticket.inner_logic().allocation_binding().2 &&
            packet.region.inner_logic().lo() == cursor@.unwrap_logic().2 &&
            packet.region.inner_logic().hi() ==
                source.ticket.inner_logic().allocation_binding().2 &&
            packet.region.inner_logic().capacity() ==
                source.ticket.inner_logic().allocation_binding().2 &&
            packet.ticket.inner_logic().owned_hi() ==
                source.ticket.inner_logic().allocation_binding().2 &&
            packet.cursor@ != None &&
            cursor@.unwrap_logic().2 + len@ <=
                packet.region.inner_logic().hi() &&
            (forall<index: Int> cursor@.unwrap_logic().2 <= index &&
                index < cursor@.unwrap_logic().2 + len@ ==>
                slot_known(packet.region.inner_logic().slot(index))) &&
            packet.base == base && packet.cursor == cursor &&
            packet.lease_key() == lease_key.inner_logic(),
        SharedReadonlyPromotion::NotUnique(packet) => value@ != 1 &&
            packet.invariant() &&
            packet.capsule.inner_logic() == capsule.inner_logic() &&
            packet.ticket.inner_logic() == source.ticket.inner_logic() &&
            packet.base == base && packet.cursor == cursor &&
            packet.len@ == len@ && packet.lease_key() == lease_key.inner_logic(),
    })]
    pub(crate) fn finish_acquire_promotion(
        source: Snapshot<SharedReadonlyPacket>,
        capsule: Ghost<GhostShared<EventBoundInvariant<SharedPayload>>>,
        base: BoundPtr,
        cursor: BoundPtr,
        len: usize,
        lease_key: Ghost<ReadyKey>,
        value: usize,
        transition: Ghost<SharedReadonlyOwnerTransition>,
    ) -> SharedReadonlyPromotion {
        let (ticket, region) = ghost! {
            let transition = transition.into_inner();
            (
                transition.ticket.into_inner(),
                match transition.region {
                    Some(region) => Some(region.into_inner()),
                    None => None,
                },
            )
        }.split();
        if value == 1 {
            proof_assert!(region.inner_logic() != None);
            let region = ghost! { region.into_inner().unwrap() };
            SharedReadonlyPromotion::Unique(SharedOwnerPacket {
                capsule,
                ticket,
                region,
                base,
                cursor,
                lease_key,
            })
        } else {
            proof_assert!(region.inner_logic() == None);
            SharedReadonlyPromotion::NotUnique(SharedReadonlyPacket {
                capsule,
                ticket,
                base,
                cursor,
                len,
                lease_key,
            })
        }
    }
}

impl SharedReadonlyPacket {
    /// Move this Readonly packet's actual ticket into the common Release
    /// carrier. Its `None` region is a type-state fact, not a reconstructed
    /// capability; the payload pool already contains this view.
    #[requires(self.invariant())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic() == SharedReleasePacket::Readonly(self))]
    #[ensures(result.inner_logic().capsule_public() == self.capsule_public())]
    #[ensures(result.inner_logic().capsule_namespace() ==
        self.capsule.inner_logic().val().namespace())]
    #[ensures(result.inner_logic().lease_key() == self.lease_key())]
    #[ensures(result.inner_logic().owner_allocation_binding() ==
        self.ticket.inner_logic().allocation_binding())]
    #[ensures(result.inner_logic().owner_id() ==
        self.ticket.inner_logic().owner_id())]
    pub(crate) fn into_release_packet(self) -> Ghost<SharedReleasePacket> {
        let SharedReadonlyPacket { capsule, ticket, base, cursor, len, lease_key } = self;
        ghost! {
            SharedReleasePacket::Readonly(SharedReadonlyPacket {
                capsule, ticket, base, cursor, len, lease_key,
            })
        }
    }

    #[logic(open(self))]
    pub(crate) fn capsule_public(self) -> (*mut (), NativeCounter, Id, Id) {
        pearlite! {
            let payload = self.capsule.inner_logic().val().public().1;
            (payload.0, payload.1, payload.2, payload.3)
        }
    }

    /// Project the packet invariant's exact relationship between the actual
    /// Readonly ticket and the capsule's public ledger identity. The ticket
    /// remains the same affine value; this helper only exposes an invariant
    /// fact at the packet interface.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(self.ticket.inner_logic().ledger_id() == self.capsule_public().2)]
    pub(crate) fn prove_ticket_ledger_matches_capsule(&self) {
        proof_assert!(self.ticket.inner_logic().ledger_id() == self.capsule_public().2);
    }

    /// Export the Readonly packet invariant's static event namespace to the
    /// shared native release adapter. This observes the actual capsule held
    /// by the packet and creates no capability.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(self.capsule.inner_logic().val().namespace() == event_namespace())]
    pub(crate) fn prove_event_namespace(&self) {
        proof_assert!(self.capsule.inner_logic().val().namespace() == event_namespace());
    }

    #[logic(open(self))]
    pub(crate) fn capsule_allocation_binding(self) -> AllocationBinding {
        pearlite! { self.capsule.inner_logic().val().public().1.5 }
    }

    #[logic(open(self))]
    pub(crate) fn lease_resource_id(self) -> Id {
        pearlite! { self.capsule.inner_logic().val().public().1.4 }
    }

    #[logic(open(self))]
    pub(crate) fn lease_key(self) -> ReadyKey {
        self.lease_key.inner_logic()
    }

    #[check(ghost)]
    #[ensures(result.inner_logic() == self.lease_key())]
    pub(crate) fn lease_key_ghost(&self) -> Ghost<ReadyKey> {
        self.lease_key
    }

    /// Observe this packet's real native counter once with Acquire ordering.
    /// If that same observation is one, reclassify this actual Readonly owner
    /// and take its actual physical suffix; otherwise return the original
    /// Readonly packet unchanged. The active header lease is restored through
    /// the same Context slot on both paths.
    #[check(terminates)]
    #[requires(self.invariant())]
    #[requires(self.capsule_public().0 == header_id as *mut ())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(self.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(header_lease_ready_entry_idle(
        context.inner_logic().ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot(),
        self.lease_key().resource_id()))]
    #[ensures(result.0@ > 0)]
    #[ensures(result.0@ == header_lease_ready_entry_count(
        context.inner_logic().ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot()))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(header_lease_ready_entry_idle(
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot(),
        self.lease_key().resource_id()))]
    #[ensures(header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot()) == result.0@)]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() &&
        index != self.lease_key().slot() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    #[ensures(match result.1 {
        SharedReadonlyPromotion::Unique(packet) =>
            result.0@ == 1 && packet.invariant() &&
            packet.capsule.inner_logic() == self.capsule.inner_logic() &&
            packet.capsule_public() == self.capsule_public() &&
                    packet.ticket.inner_logic().allocation_binding() ==
                        self.ticket.inner_logic().allocation_binding() &&
            packet.ticket.inner_logic().owner_id() == self.ticket.inner_logic().owner_id() &&
            packet.ticket.inner_logic().class() == RefKind::Mutable &&
            packet.ticket.inner_logic().owned_lo() == self.cursor@.unwrap_logic().2 &&
            packet.ticket.inner_logic().owned_hi() ==
                self.ticket.inner_logic().allocation_binding().2 &&
            packet.region.inner_logic().lo() == self.cursor@.unwrap_logic().2 &&
            packet.region.inner_logic().hi() ==
                self.ticket.inner_logic().allocation_binding().2 &&
            packet.region.inner_logic().capacity() ==
                self.ticket.inner_logic().allocation_binding().2 &&
            packet.ticket.inner_logic().owned_hi() ==
                self.ticket.inner_logic().allocation_binding().2 &&
            packet.cursor@ != None &&
            self.cursor@.unwrap_logic().2 + self.len@ <=
                packet.region.inner_logic().hi() &&
            (forall<index: Int> 0 <= index && index < self.len@ ==>
                slot_known(packet.region.inner_logic().slot(
                    self.cursor@.unwrap_logic().2 + index))) &&
            packet.base == self.base && packet.cursor == self.cursor &&
            packet.lease_key() == self.lease_key(),
        SharedReadonlyPromotion::NotUnique(packet) =>
            result.0@ != 1 && packet.invariant() &&
            packet.capsule.inner_logic() == self.capsule.inner_logic() &&
            packet.ticket.inner_logic() == self.ticket.inner_logic() &&
            packet.base == self.base && packet.cursor == self.cursor &&
            packet.len@ == self.len@ && packet.lease_key() == self.lease_key(),
    })]
    pub(crate) fn acquire_and_promote_unique(
        self,
        header_id: *const (),
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> (usize, SharedReadonlyPromotion) {
        let source = snapshot!(self);
        let SharedReadonlyPacket { capsule, ticket, base, cursor, len, lease_key } = self;
        let mut context_ref = ghost!(context.into_inner());
        let lo: Snapshot<Int> = snapshot!(cursor@.unwrap_logic().2);
        let visible_hi_for_transition: Snapshot<Int> =
            snapshot!(ticket.inner_logic().owned_hi());
        let visible_hi_for_frame: Snapshot<Int> =
            snapshot!(ticket.inner_logic().owned_hi());
        proof_assert!(*visible_hi_for_transition == ticket.inner_logic().owned_hi());
        proof_assert!(*visible_hi_for_frame == ticket.inner_logic().owned_hi());

        let loan = ghost! {
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&**context_ref), lease_key,
            );
            SharedReleasePacket::try_take_header_read_loan_for(
                capsule,
                lease_key,
                Ghost::new(&*ticket),
                Ghost::new(&mut **context_ref),
            ).into_inner().unwrap()
        };
        let loan_ref = loan.borrow();
        let root_ref = ghost!(loan_ref.into_inner().root.as_ref());
        let counter = loaned_native_counter(header_id.cast(), root_ref);
        let (value, output) = counter.load_acquire_with_frame(
            ghost! { capsule.into_inner().to_ref() },
            loan_ref,
            ghost!(
                #[requires(
                    payload.invariant() && payload.protocol() &&
                    payload.counter_model() == *loan.state.counter_model &&
                    payload.agrees_with_value(value) &&
                    ticket.inner_logic().invariant() &&
                    ticket.inner_logic().ledger_id() == payload.ledger_id &&
                    ticket.inner_logic().class() == RefKind::Readonly &&
                    ticket.inner_logic().allocation_binding() == *payload.allocation_binding &&
                    *lo == cursor@.unwrap_logic().2 &&
                    ticket.inner_logic().owned_lo() <= *lo &&
                    *lo <= ticket.inner_logic().owned_hi() &&
                    *visible_hi_for_transition ==
                        ticket.inner_logic().owned_hi() &&
                    *visible_hi_for_frame == *visible_hi_for_transition &&
                    loan.invariant() &&
                    no_active_lend_state(&loan.state) &&
                    payload.lease.resource_id() ==
                        loan.state.lease_id
                )]
                #[ensures(
                    (^payload).invariant() && (^payload).protocol() &&
                    (^payload).public() == payload.public() &&
                    (^payload).counter_model() == payload.counter_model() &&
                    (^payload).agrees_with_value(value) &&
                    value@ > 0 &&
                    result.invariant() &&
                    (^payload).allocation_binding == payload.allocation_binding &&
                    (^payload).header_vec == payload.header_vec &&
                    (^payload).original_capacity_repr == payload.original_capacity_repr &&
                    (^payload).root == payload.root &&
                    (^payload).storage == payload.storage &&
                    (^payload).phase == payload.phase &&
                    (^payload).retire == payload.retire &&
                    (^payload).terminal_action == payload.terminal_action &&
                    (^payload).lease == payload.lease &&
                    (^payload).lease_key == payload.lease_key &&
                    (^payload).lease_generation == payload.lease_generation &&
                    (^payload).closed == payload.closed &&
                    header_lease_count(
                        loan.state.lease_cap.fragment_value(),
                    ) == value@ &&
                    *result.visible_hi ==
                        old(*visible_hi_for_frame) &&
                    match result.region {
                        Some(region) => value@ == 1 &&
                            result.ticket.inner_logic().class() == RefKind::Mutable &&
                            result.ticket.inner_logic().owner_id() ==
                                old(ticket.inner_logic().owner_id()) &&
                            result.ticket.inner_logic().ledger_id() ==
                                old(ticket.inner_logic().ledger_id()) &&
                            result.ticket.inner_logic().fragment_resource_id() ==
                                old(ticket.inner_logic().fragment_resource_id()) &&
                            result.ticket.inner_logic().allocation_binding() ==
                                old(ticket.inner_logic().allocation_binding()) &&
                            result.ticket.inner_logic().owned_lo() == old(*lo) &&
                            result.ticket.inner_logic().owned_hi() ==
                                old(ticket.inner_logic().allocation_binding().2) &&
                            region.inner_logic().namespace() ==
                                old(ticket.inner_logic().allocation_binding().0) &&
                            region.inner_logic().capacity() ==
                                old(ticket.inner_logic().allocation_binding().2) &&
                            region.inner_logic().lo() == old(*lo) &&
                            region.inner_logic().hi() ==
                                old(ticket.inner_logic().allocation_binding().2),
                        None => value@ != 1 &&
                            result.ticket.inner_logic() ==
                                old(ticket.inner_logic()),
                    }
                )]
                move |payload: &mut SharedPayload, value: usize,
                      loan: &SharedHeaderReadLoan| {
                    let ticket = ticket.into_inner();
                    payload.prove_live_owner_ticket(&ticket);
                    SharedPayload::prove_current_header_lease(
                        Ghost::new(&*payload), Ghost::new(&loan.state),
                    );
                    let transition = SharedPayload::transition_unique_readonly_owner(
                        Ghost::new(payload), Ghost::new(ticket),
                        Ghost::new(&loan.state), value, lo,
                        visible_hi_for_transition,
                    );
                    transition.into_inner()
                }
            ),
        );

        let restore_context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>> =
            ghost! { &mut **context_ref };
        SharedOwnerPacket::restore_lease_after_owner_release(
            capsule, lease_key, restore_context, loan,
        );
        let promoted = SharedReadonlyOwnerTransition::finish_acquire_promotion(
            source, capsule, base, cursor, len, lease_key, value, output,
        );
        (value, promoted)
    }

    /// Take this packet's actual Readonly ticket span out of the payload pool
    /// under the matching idle header cap. The returned active lease carries
    /// the actual affine PhysicalRegion used by B4.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(self.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(self.lease_key().slot()), self.lease_key().resource_id(), self.lease_key().slot(), self.lease_key().resource_id()))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_entry_state(self.lease_key().slot()) == None)]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() &&
        index != self.lease_key().slot() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().lease_key_value() == self.lease_key())]
    #[ensures(result.inner_logic().lease_id_value() == self.lease_resource_id())]
    #[ensures(result.inner_logic().lent_span_value() == Some((
        self.ticket.inner_logic().owner_id(),
        self.ticket.inner_logic().owned_lo(),
        self.ticket.inner_logic().owned_hi(),
    )))]
    #[ensures(result.inner_logic().lent_region_value() != None)]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().lo() ==
        self.ticket.inner_logic().owned_lo())]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().hi() ==
        self.ticket.inner_logic().owned_hi())]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().lo() ==
        self.cursor@.unwrap_logic().2)]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().hi() ==
        self.cursor@.unwrap_logic().2 + self.len@)]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().namespace() ==
        self.ticket.inner_logic().allocation_binding().0)]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().capacity() ==
        self.ticket.inner_logic().allocation_binding().2)]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().namespace() ==
        self.cursor@.unwrap_logic().0)]
    #[ensures(result.inner_logic().lent_region_value().unwrap_logic().capacity() ==
        self.cursor@.unwrap_logic().1)]
    #[ensures(forall<index: Int>
        self.ticket.inner_logic().owned_lo() <= index &&
        index < self.ticket.inner_logic().owned_hi() ==>
            slot_known(result.inner_logic().lent_region_value().unwrap_logic().slot(index)))]
    #[ensures(forall<index: Int>
        0 <= index && index < self.len@ ==>
            slot_known(result.inner_logic().lent_region_value().unwrap_logic().slot(
                self.cursor@.unwrap_logic().2 + index)))]
    #[ensures(result.inner_logic().lease_count_value() ==
        header_lease_ready_entry_count(
            context.inner_logic().ready_entry_state(self.lease_key().slot()),
            self.lease_key().resource_id(), self.lease_key().slot()))]
    pub(crate) fn take_header_region_read_loan(
        &self,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> Ghost<SharedHeaderReadLoan> {
        ghost! {
            let context = context.into_inner();
            let key = self.lease_key_ghost().into_inner();
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&*context), Ghost::new(key),
            );
            let cap = BorrowedSharedContext::take(
                Ghost::new(&mut *context), key,
            );
            let expected_count: Snapshot<Int> = snapshot!(
                header_lease_count(cap.inner_logic().fragment_value())
            );
            let tokens = BorrowedSharedContext::reborrow_tokens(
                Ghost::new(&mut *context),
            );
            let expected_key: Snapshot<ReadyKey> = snapshot!(key);
            let expected_lease_id: Snapshot<Id> = snapshot!(self.lease_resource_id());
            let expected_phase: Snapshot<Id> = snapshot!(self.capsule_public().3);
            let ticket_body: Ghost<&OwnerTicket> = Ghost::new(&*self.ticket);
            let capsule = *self.capsule;
            capsule.to_ref().open_with_tokens_with_frame(
                tokens,
                expected_count,
                ghost!(
                    #[requires(
                        payload.invariant() && payload.protocol() &&
                        payload.public().3 == *expected_phase &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().4 == *expected_lease_id &&
                        payload.public().6 == *expected_key &&
                        cap.inner_logic().invariant() &&
                        cap.inner_logic().key() == *expected_key &&
                        cap.inner_logic().fragment_id() == *expected_lease_id &&
                        header_lease_idle(
                            cap.inner_logic().fragment_value(), *expected_lease_id) &&
                        header_lease_count(cap.inner_logic().fragment_value()) ==
                            *expected_count &&
                        ticket_body.inner_logic().invariant() &&
                        ticket_body.inner_logic().ledger_id() == payload.ledger_id &&
                        ticket_body.inner_logic().class() == RefKind::Readonly &&
                        ticket_body.inner_logic().allocation_binding() ==
                            *payload.allocation_binding
                    )]
                    #[ensures(
                        (^payload).invariant() && (^payload).protocol() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).agrees_with_value(*owner_value) &&
                        exact_header_region_lend_transition(
                            *payload, ^payload,
                            old(*ticket_body.inner_logic()), result
                        ) &&
                        header_lease_count(result.state.lease_cap.fragment_value()) ==
                            *expected_count
                    )]
                    move |payload: &mut SharedPayload,
                          owner_model: Snapshot<NativeCounter>,
                          owner_value: Snapshot<usize>,
                          expected_count: Snapshot<Int>| {
                        let cap = cap.into_inner();
                        let ticket = ticket_body.into_inner();
                        payload.prove_public_lease_resource_id(expected_lease_id);
                        payload.prove_public_lease_key(expected_key);
                        payload.prove_idle_cap_implies_root_some(&cap, ticket);
                        payload.prove_live_owner_ticket(ticket);
                        SharedPayload::take_header_region_read_loan(
                            Ghost::new(payload), Ghost::new(cap), Ghost::new(ticket),
                        ).into_inner()
                    }
                ),
            ).into_inner()
        }
    }

    /// Borrow this packet's view through the PhysicalRegion carried by its
    /// actual active read loan. The slice lifetime is tied to that loan; this
    /// method neither constructs nor transfers physical authority.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(loan.inner_logic().invariant())]
    #[requires(loan.inner_logic().lease_key_value() == self.lease_key())]
    #[requires(loan.inner_logic().lent_region_value() != None)]
    #[requires(loan.inner_logic().lent_region_value().unwrap_logic().namespace() ==
        self.cursor@.unwrap_logic().0)]
    #[requires(loan.inner_logic().lent_region_value().unwrap_logic().capacity() ==
        self.cursor@.unwrap_logic().1)]
    #[requires(loan.inner_logic().lent_region_value().unwrap_logic().lo() <=
        self.cursor@.unwrap_logic().2)]
    #[requires(self.cursor@.unwrap_logic().2 + self.len@ <=
        loan.inner_logic().lent_region_value().unwrap_logic().hi())]
    #[requires(forall<index: Int>
        0 <= index && index < self.len@ ==>
            slot_known(loan.inner_logic().lent_region_value().unwrap_logic().slot(
                self.cursor@.unwrap_logic().2 + index)))]
    #[ensures(result@.len() == self.len@)]
    #[ensures(forall<index: Int>
        0 <= index && index < self.len@ ==>
            loan.inner_logic().lent_region_value().unwrap_logic().slot(
                self.cursor@.unwrap_logic().2 + index
            ) == Some(Some(result@[index])))]
    pub(crate) fn borrow_slice_from_header_read_loan<'a>(
        &'a self,
        loan: Ghost<&'a SharedHeaderReadLoan>,
    ) -> &'a [u8] {
        let region: Ghost<&'a PhysicalRegion> = ghost! {
            loan.into_inner().state.lent_region.as_ref().unwrap()
        };
        unsafe {
            crate::ownership_proof::raw_vec::borrow_bound(
                &self.cursor, self.len, region,
            )
        }
    }

    /// Return the actual region and active cap to the payload, then restore
    /// that same idle cap to the packet's original registry slot.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(loan.inner_logic().invariant())]
    #[requires(loan.inner_logic().lease_key_value() == self.lease_key())]
    #[requires(loan.inner_logic().lease_id_value() == self.lease_resource_id())]
    #[requires(loan.inner_logic().lent_span_value() == Some((
        self.ticket.inner_logic().owner_id(),
        self.ticket.inner_logic().owned_lo(),
        self.ticket.inner_logic().owned_hi(),
    )))]
    #[requires(loan.inner_logic().lent_region_value() != None)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(self.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(context.inner_logic().ready_entry_state(self.lease_key().slot()) == None)]
    #[ensures(self.invariant())]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_has_key(self.lease_key()))]
    #[ensures(header_lease_ready_entry_idle((^context.inner_logic()).ready_entry_state(self.lease_key().slot()), self.lease_key().resource_id(), self.lease_key().slot(), self.lease_key().resource_id()))]
    #[ensures(header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(self.lease_key().slot()),
        self.lease_key().resource_id(), self.lease_key().slot()) ==
        loan.inner_logic().lease_count_value())]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() &&
        index != self.lease_key().slot() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    pub(crate) fn restore_header_region_read_loan(
        &self,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
        loan: Ghost<SharedHeaderReadLoan>,
    ) -> Ghost<()> {
        ghost! {
            let context = context.into_inner();
            let expected_key: Snapshot<ReadyKey> = snapshot!(self.lease_key());
            let expected_lease_id: Snapshot<Id> = snapshot!(self.lease_resource_id());
            let expected_phase: Snapshot<Id> = snapshot!(self.capsule_public().3);
            let loan_ref = loan.borrow().into_inner();
            let expected_count: Snapshot<Int> = snapshot!(header_lease_count(
                loan_ref.state.lease_cap.fragment_value(),
            ));
            let ticket_body: Ghost<&OwnerTicket> = Ghost::new(&*self.ticket);
            let capsule = *self.capsule;
            let callback = ghost!(
                    #[requires(
                        payload.invariant() && payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().3 == *expected_phase &&
                        payload.public().4 == *expected_lease_id &&
                        payload.public().6 == *expected_key &&
                        header_lease_count(
                            loan.inner_logic().state.lease_cap.fragment_value(),
                        ) == *expected_count &&
                        ticket_body.inner_logic().invariant() &&
                        ticket_body.inner_logic().ledger_id() == payload.ledger_id &&
                        ticket_body.inner_logic().class() == RefKind::Readonly &&
                        loan.inner_logic().lease_key_value() == *expected_key &&
                        loan.inner_logic().lease_id_value() == *expected_lease_id &&
                        loan.inner_logic().lent_span_value() == Some((
                            ticket_body.inner_logic().owner_id(),
                            ticket_body.inner_logic().owned_lo(),
                            ticket_body.inner_logic().owned_hi(),
                        )) && loan.inner_logic().lent_region_value() != None
                    )]
                    #[ensures(
                        (^payload).invariant() && (^payload).protocol() &&
                        (^payload).counter_model() == payload.counter_model() &&
                        (^payload).public() == payload.public() &&
                        (^payload).agrees_with_value(*owner_value) &&
                        (^payload).root != None &&
                        (^payload).lent_span == None &&
                        (^payload).lease.is_idle() &&
                        active_lease_pool_protocol(
                            (^payload).pool.unwrap_logic(),
                            (^payload).ledger.unwrap_logic(),
                            (^payload).storage.unwrap_logic().capacity(),
                            (^payload).storage.unwrap_logic().binding(),
                            None,
                        ) && result.invariant() &&
                        result.key() == old(*expected_key) &&
                        result.fragment_id() == old(*expected_lease_id) &&
                        header_lease_idle(result.fragment_value(), old(*expected_lease_id)) &&
                        header_lease_count(result.fragment_value()) ==
                            (^payload).ledger.unwrap_logic().count() &&
                        header_lease_count(result.fragment_value()) == *expected_count
                    )]
                    move |payload: &mut SharedPayload,
                          owner_model: Snapshot<NativeCounter>,
                          owner_value: Snapshot<usize>,
                          expected_count: Snapshot<Int>| {
                        let ticket = ticket_body.into_inner();
                        payload.prove_live_owner_ticket(ticket);
                        SharedPayload::prove_current_header_lease(
                            Ghost::new(&*payload),
                            Ghost::new(&loan.borrow().into_inner().state),
                        );
                        proof_assert!(header_lease_count(
                            loan.inner_logic().state.lease_cap.fragment_value(),
                        ) == *expected_count);
                        SharedPayload::restore_header_region_read_loan(
                            Ghost::new(payload), loan,
                        ).into_inner()
                    }
                );
            let tokens = BorrowedSharedContext::reborrow_tokens(
                Ghost::new(&mut *context),
            );
            let cap = capsule.to_ref().open_with_tokens_with_frame(
                tokens, expected_count, callback,
            ).into_inner();
            BorrowedSharedContext::return_cap(
                Ghost::new(&mut *context), Ghost::new(cap),
            );
            ()
        }
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
            self.ticket.inner_logic().class() == RefKind::Readonly &&
            (self.capsule_public().0.addr_logic() & super::KIND_MASK) ==
                super::KIND_ARC &&
            self.ticket.inner_logic().allocation_binding() ==
                self.capsule_allocation_binding() &&
            self.base.invariant() && self.cursor.invariant() &&
            self.base@ == Some((
                self.capsule_allocation_binding().0,
                self.capsule_allocation_binding().2, 0int
            )) &&
            self.cursor@ != None &&
            self.cursor@.unwrap_logic().0 == self.capsule_allocation_binding().0 &&
            self.cursor@.unwrap_logic().1 == self.capsule_allocation_binding().2 &&
            self.base.raw_pointer() == self.capsule_allocation_binding().1 &&
            self.cursor.current_address() == self.base.current_address() +
                self.cursor@.unwrap_logic().2 &&
            self.ticket.inner_logic().owned_lo() ==
                self.cursor@.unwrap_logic().2 &&
            self.ticket.inner_logic().owned_hi() ==
                self.cursor@.unwrap_logic().2 + self.len@ &&
            0 <= self.ticket.inner_logic().owned_lo() &&
            self.ticket.inner_logic().owned_hi() <=
                self.capsule_allocation_binding().2 &&
            self.lease_key.inner_logic() == self.capsule.inner_logic().val().public().1.6 &&
            self.lease_key.inner_logic().resource_id() == self.lease_resource_id() &&
            self.lease_key.inner_logic().slot() ==
                self.capsule.inner_logic().val().public().1.6.slot() &&
            0 <= self.len@ &&
            self.cursor@.unwrap_logic().2 + self.len@ <=
                self.capsule_allocation_binding().2
        }
    }
}

impl SharedOwnerPacket {
    /// Reclassify this packet's actual Mutable ticket to its visible immutable
    /// interval and retire the actual physical region into the shared pool.
    /// The counter and header event are only observed through the invariant;
    /// no native reference-count operation occurs in this source transition.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(len@ <= self.region.inner_logic().hi() -
        self.cursor@.unwrap_logic().2)]
    #[requires(forall<index: Int> 0 <= index && index < len@ ==>
        crate::ownership_proof::raw_vec::slot_known(
            self.region.inner_logic().slot(
                self.cursor@.unwrap_logic().2 + index)))]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(self.lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(self.lease_key().slot()), self.lease_key().resource_id(), self.lease_key().slot(), self.lease_key().resource_id()))]
    #[ensures(result.invariant())]
    #[ensures(result.capsule.inner_logic() == self.capsule.inner_logic())]
    #[ensures(result.capsule_public() == self.capsule_public())]
    #[ensures(result.capsule_allocation_binding() == self.capsule_allocation_binding())]
    #[ensures(result.lease_key() == self.lease_key())]
    #[ensures(result.lease_resource_id() == self.lease_resource_id())]
    #[ensures(result.base == self.base && result.cursor == self.cursor)]
    #[ensures(result.cursor.raw_pointer() == self.cursor.raw_pointer())]
    #[ensures(result.len@ == len@)]
    #[ensures(result.ticket.inner_logic().invariant())]
    #[ensures(result.ticket.inner_logic().ledger_id() == self.ticket.inner_logic().ledger_id())]
    #[ensures(result.ticket.inner_logic().owner_id() == self.ticket.inner_logic().owner_id())]
    #[ensures(result.ticket.inner_logic().fragment_resource_id() ==
        self.ticket.inner_logic().fragment_resource_id())]
    #[ensures(result.ticket.inner_logic().allocation_binding() ==
        self.ticket.inner_logic().allocation_binding())]
    #[ensures(result.ticket.inner_logic().class() == RefKind::Readonly)]
    #[ensures(result.ticket.inner_logic().owned_lo() ==
        self.cursor@.unwrap_logic().2)]
    #[ensures(result.ticket.inner_logic().owned_hi() ==
        self.cursor@.unwrap_logic().2 + len@)]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    pub(crate) fn freeze_into_readonly(
        self,
        len: usize,
        context: Ghost<&mut BorrowedSharedContext<'_, HeaderLeaseState>>,
    ) -> SharedReadonlyPacket {
        let base = self.base;
        let cursor = self.cursor;
        let packet_parts = ghost! {
            let capsule = *self.capsule;
            let ticket = self.ticket.into_inner();
            let region = self.region.into_inner();
            let lease_key = self.lease_key.into_inner();
            (capsule, ticket, region, lease_key)
        };
        let (capsule, ticket, region, lease_key) = packet_parts.split();
        let visible_lo: Snapshot<Int> = snapshot!(cursor@.unwrap_logic().2);
        let visible_hi: Snapshot<Int> = snapshot!(cursor@.unwrap_logic().2 + len@);
        let region_before: Snapshot<PhysicalRegion> = snapshot!(region.inner_logic());
        let callback_frame = (
            snapshot!(ticket.inner_logic().ledger_id()),
            snapshot!(ticket.inner_logic().owner_id()),
            snapshot!(ticket.inner_logic().fragment_resource_id()),
            snapshot!(lease_key.inner_logic()),
            snapshot!(lease_key.inner_logic().resource_id()),
            visible_lo,
            visible_hi,
            region_before,
        );
        let callback_frame_ref = &callback_frame;
        let mut context_ref = ghost!(context.into_inner());

        let packet_parts = ghost! {
            let lease_key = lease_key.into_inner();
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&**context_ref), Ghost::new(lease_key),
            );
            // Keep the actual idle lease capability out of the callback's
            // registry slot while the invariant proves the matching stable
            // payload state. The same affine capability is returned below.
            let cap = BorrowedSharedContext::take(
                Ghost::new(&mut **context_ref), lease_key,
            );
            let cap_body: Ghost<&ReadyCap<HeaderLeaseState>> =
                ghost! { cap.borrow().into_inner() };
            let tokens = BorrowedSharedContext::reborrow_tokens(
                Ghost::new(&mut **context_ref),
            );
            let capsule = capsule.into_inner();
            let readonly_ticket = capsule.to_ref().open_with_tokens(
                tokens,
                ghost!(
                #[requires(
                    payload.invariant() && payload.protocol() &&
                    payload.counter_model() == *owner_model &&
                    payload.agrees_with_value(*owner_value) &&
                    payload.public().2 == *callback_frame_ref.0 &&
                    payload.public().4 == *callback_frame_ref.4 &&
                    payload.public().6 == *callback_frame_ref.3 &&
                    cap_body.inner_logic().invariant() &&
                    cap_body.inner_logic().key() == *callback_frame_ref.3 &&
                    cap_body.inner_logic().fragment_id() == *callback_frame_ref.4 &&
                    header_lease_idle(cap_body.inner_logic().fragment_value(),
                        *callback_frame_ref.4) &&
                    ticket.inner_logic().invariant() &&
                    ticket.inner_logic().ledger_id() == *callback_frame_ref.0 &&
                    ticket.inner_logic().owner_id() == *callback_frame_ref.1 &&
                    ticket.inner_logic().fragment_resource_id() == *callback_frame_ref.2 &&
                    ticket.inner_logic().ledger_id() == payload.ledger_id &&
                    ticket.inner_logic().class() == RefKind::Mutable &&
                    ticket.inner_logic().allocation_binding() ==
                        *payload.allocation_binding &&
                    region.inner_logic().invariant() &&
                    ticket.inner_logic().owned_lo() == region.inner_logic().lo() &&
                    ticket.inner_logic().owned_hi() == region.inner_logic().hi() &&
                    ticket.inner_logic().allocation_binding().0 ==
                        region.inner_logic().namespace() &&
                    ticket.inner_logic().allocation_binding().2 ==
                        region.inner_logic().capacity() &&
                    ticket.inner_logic().owned_lo() <= *callback_frame_ref.5 &&
                    *callback_frame_ref.6 == *callback_frame_ref.5 + len@ &&
                    *callback_frame_ref.5 <= *callback_frame_ref.6 &&
                    *callback_frame_ref.6 <= ticket.inner_logic().owned_hi() &&
                    *callback_frame_ref.7 == region.inner_logic() &&
                    region.inner_logic().lo() <= *callback_frame_ref.5 &&
                    *callback_frame_ref.6 <= region.inner_logic().hi() &&
                    (forall<index: Int> 0 <= index && index < len@ ==>
                        crate::ownership_proof::raw_vec::slot_known(
                            region.inner_logic().slot(*callback_frame_ref.5 + index)))
                )]
                #[ensures(
                    (^payload).invariant() && (^payload).protocol() &&
                    (^payload).public() == payload.public() &&
                    (^payload).counter_model() == payload.counter_model() &&
                    (^payload).agrees_with_value(*owner_value) &&
                    (^payload).ledger.unwrap_logic().count() ==
                        payload.ledger.unwrap_logic().count() &&
                    header_lease_count(cap_body.inner_logic().fragment_value()) ==
                        (^payload).ledger.unwrap_logic().count() &&
                    (^payload).ledger.unwrap_logic().next_id() ==
                        payload.ledger.unwrap_logic().next_id() &&
                    (^payload).ledger.unwrap_logic().has_ticket(result) &&
                    result.invariant() &&
                    result.ledger_id() == *callback_frame_ref.0 &&
                    result.owner_id() == *callback_frame_ref.1 &&
                    result.fragment_resource_id() == *callback_frame_ref.2 &&
                    result.class() == RefKind::Readonly &&
                    result.allocation_binding() == *payload.allocation_binding &&
                    result.owned_lo() == *callback_frame_ref.5 &&
                    result.owned_hi() == *callback_frame_ref.6 &&
                    (forall<index: Int>
                        (*callback_frame_ref.7).lo() <= index &&
                            index < (*callback_frame_ref.7).hi() ==>
                            (^payload).pool.unwrap_logic().slot(index) ==
                                (*callback_frame_ref.7).slot(index)) &&
                    (forall<index: Int>
                        *callback_frame_ref.5 <= index &&
                            index < *callback_frame_ref.6 ==>
                        (^payload).pool.unwrap_logic().contains(index)) &&
                    (forall<index: Int> 0 <= index && index < len@ ==>
                        crate::ownership_proof::raw_vec::slot_known(
                            (^payload).pool.unwrap_logic().slot(
                                *callback_frame_ref.5 + index))) &&
                    (forall<index: Int>
                        (^payload).pool.unwrap_logic().contains(index) ==
                            (0 <= index && index <
                                (^payload).pool.unwrap_logic().capacity() &&
                                !active_mutable_contains(
                                    (^payload).ledger.unwrap_logic().active_ids(),
                                    index)))
                )]
                |payload: &mut SharedPayload,
                 owner_model: Snapshot<NativeCounter>,
                 owner_value: Snapshot<usize>| {
                    let cap = cap_body.into_inner();
                    let ticket = ticket.into_inner();
                    let region = region.into_inner();
                    payload.prove_idle_cap_implies_root_some(cap, &ticket);
                    SharedPayload::prove_shifted_region_slots_known(
                        snapshot!(region),
                        callback_frame_ref.5,
                        snapshot!(len),
                    );
                    payload.prove_live_owner_ticket(&ticket);
                    let ticket = SharedPayload::freeze_mutable_owner(
                        Ghost::new(payload),
                        Ghost::new(ticket),
                        Ghost::new(region),
                        callback_frame_ref.5,
                        callback_frame_ref.6,
                    ).into_inner();
                    payload.prove_live_owner_ticket(&ticket);
                    match &payload.ledger {
                        Some(ledger) =>
                            ledger.prove_readonly_ticket_excluded_from_mutable_coverage(
                                &ticket,
                            ),
                        None => proof_assert!(false),
                    }
                    ticket
                }
                ),
            ).into_inner();
            BorrowedSharedContext::return_cap(
                Ghost::new(&mut **context_ref), cap,
            );
            (capsule, readonly_ticket, lease_key)
        };
        let (capsule, readonly_ticket, lease_key) = packet_parts.split();

        let packet = SharedReadonlyPacket {
            capsule,
            ticket: readonly_ticket,
            base,
            cursor,
            len,
            lease_key,
        };
        proof_assert!(packet.invariant());
        packet
    }

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
    #[requires(loan.inner_logic().state.lease_key == key.inner_logic())]
    #[requires(loan.inner_logic().state.lease_id == key.inner_logic().resource_id())]
    #[requires(no_active_lend_loan(loan.inner_logic()))]
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
    #[ensures(header_lease_ready_entry_idle((^context.inner_logic()).ready_entry_state(key.inner_logic().slot()), key.inner_logic().resource_id(), key.inner_logic().slot(), key.inner_logic().resource_id()))]
    #[ensures(header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(key.inner_logic().slot()),
        key.inner_logic().resource_id(), key.inner_logic().slot()) ==
        header_lease_count(loan.inner_logic().state.lease_cap.fragment_value()))]
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
    #[requires(header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(packet.lease_key().slot()), packet.lease_key().resource_id(), packet.lease_key().slot(), packet.lease_key().resource_id()))]
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
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(header_lease_ready_entry_idle(
        (^context.inner_logic()).ready_entry_state(packet.lease_key().slot()),
        packet.lease_key().resource_id(), packet.lease_key().slot(),
        packet.lease_key().resource_id()))]
    #[ensures(header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(packet.lease_key().slot()),
        packet.lease_key().resource_id(), packet.lease_key().slot()) ==
        header_lease_ready_entry_count(
            context.inner_logic().ready_entry_state(packet.lease_key().slot()),
            packet.lease_key().resource_id(), packet.lease_key().slot()) + 1)]
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
            snapshot!(*loan.inner_logic().state.counter_model);
        let expected_key: Snapshot<ReadyKey> = snapshot!(key.inner_logic());
        let expected_lease_id: Snapshot<Id> = snapshot!(key.inner_logic().resource_id());
        let expected_ledger_id: Snapshot<Id> =
            snapshot!(ticket.inner_logic().ledger_id());
        let expected_phase: Snapshot<Id> = snapshot!(loan.inner_logic().state.phase_id);
        let expected_before_ticket: Snapshot<OwnerTicket> =
            snapshot!(ticket.inner_logic());
        let expected_before_region: Snapshot<PhysicalRegion> =
            snapshot!(region.inner_logic());
        let expected_cut: Snapshot<Int> = snapshot!(*cut);
        let (root, state) = ghost! {
            let loan = loan.into_inner();
            (loan.root, loan.state)
        }.split();
        let callback_frame = ghost!((
            expected_before_ticket,
            expected_before_region,
            expected_cut,
            state.into_inner(),
        ));
        let root_ref = ghost!(root.borrow().into_inner().as_ref());
        let counter = loaned_native_counter(
            header_id as *const Shared,
            root_ref,
        );
        let (incremented, callback) = counter.try_increment_with(
            ghost! { capsule.into_inner().to_ref() },
            callback_frame,
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
                    frame.3.invariant() &&
                    no_active_lend_state(&frame.3) &&
                    frame.3.lease_id == *expected_lease_id &&
                    frame.3.lease_key == *expected_key &&
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
                    result.1.3.invariant() &&
                    result.1.3.header_id == frame.3.header_id &&
                    result.1.3.counter_model == frame.3.counter_model &&
                    result.1.3.header_vec == frame.3.header_vec &&
                    result.1.3.original_capacity_repr ==
                        frame.3.original_capacity_repr &&
                    result.1.3.storage_model == frame.3.storage_model &&
                    result.1.3.ledger_id == frame.3.ledger_id &&
                    result.1.3.phase_id == frame.3.phase_id &&
                    result.1.3.generation == frame.3.generation &&
                    result.1.3.lease_id == frame.3.lease_id &&
                    result.1.3.lease_key == frame.3.lease_key &&
                    result.1.3.stamp == frame.3.stamp &&
                    result.1.3.lent_span == frame.3.lent_span &&
                    result.1.3.lent_region == frame.3.lent_region &&
                    result.1.3.lease_cap.key() == frame.3.lease_cap.key() &&
                    result.1.3.lease_cap.fragment_id() ==
                        frame.3.lease_cap.fragment_id() &&
                    result.1.0 == frame.0 &&
                    result.1.1 == frame.1 &&
                    result.1.2 == frame.2 &&
                    header_lease_active(
                        result.1.3.lease_cap.fragment_value(),
                        result.1.3.lease_id, *result.1.3.stamp,
                    ) &&
                    header_lease_count(result.1.3.lease_cap.fragment_value()) == new@ &&
                    old@ == header_lease_count(frame.3.lease_cap.fragment_value()) &&
                    result.0.inner_logic().0.invariant() &&
                    result.0.inner_logic().1.invariant() &&
                    result.0.inner_logic().2.invariant() &&
                    result.0.inner_logic().3.invariant() &&
                    (^payload).ledger.unwrap_logic().has_ticket(result.0.inner_logic().0) &&
                    (^payload).ledger.unwrap_logic().has_ticket(result.0.inner_logic().1) &&
                    result.0.inner_logic().4 == frame.0 &&
                    result.0.inner_logic().5 == frame.1 &&
                    result.0.inner_logic().6 == frame.2 &&
                    mutable_owner_split_result(
                        result.0.inner_logic().4, result.0.inner_logic().5,
                        result.0.inner_logic().6,
                        (result.0.inner_logic().0, result.0.inner_logic().1,
                            result.0.inner_logic().2, result.0.inner_logic().3)
                    )
                )]
                move |payload: &mut SharedPayload,
                      old: usize,
                      new: usize,
                      mut frame: (
                          Snapshot<OwnerTicket>, Snapshot<PhysicalRegion>, Snapshot<Int>,
                          SharedHeaderReadLoanState,
                      )| {
                    payload.prove_live_owner_matches_counter(&ticket, old);
                    SharedPayload::prove_current_header_lease(
                        Ghost::new(&*payload),
                        Ghost::new(&frame.3),
                    );
                    let output = SharedPayload::split_mutable_owner_after_increment(
                        Ghost::new(payload), ticket, region,
                        Ghost::new(&mut frame.3), cut,
                        old, new,
                    );
                    (output, frame)
                }
            ),
        );
        if incremented.is_err() {
            crate::abort();
        }
        let split = ghost! {
            let (split_output, callback_frame) =
                callback.into_inner().ok().unwrap();
            let (_, _, _, state) = callback_frame;
            let state = state;
            let loan = SharedHeaderReadLoan::from_active_parts(
                root.into_inner(),
                state.header_id,
                state.counter_model,
                state.header_vec,
                state.original_capacity_repr,
                state.storage_model,
                state.ledger_id,
                state.phase_id,
                state.generation,
                state.lease_id,
                state.lease_key,
                state.stamp,
                state.lent_span,
                state.lent_region,
                state.lease_cap,
            );
            proof_assert!(loan.invariant());
            proof_assert!(loan.state.lease_key == key.inner_logic());
            proof_assert!(loan.state.lease_id == key.inner_logic().resource_id());
            proof_assert!(no_active_lend_state(&loan.state));
            (split_output.into_inner(), loan)
        };
        let (split, loan) = split.split();
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
        header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(self.lease_key().slot()), self.lease_key().resource_id(), self.lease_key().slot(), self.lease_key().resource_id()))]
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
            result.inner_logic().unwrap_logic().state.lease_key == self.lease_key()) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().invariant()) &&
        (result.inner_logic() != None ==>
            no_active_lend_loan(result.inner_logic().unwrap_logic())) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().state.header_id ==
                self.capsule_public().0 as *mut Shared) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().state.lease_id == self.lease_resource_id()) &&
        (result.inner_logic() != None ==>
            *result.inner_logic().unwrap_logic().state.counter_model == self.capsule_public().1) &&
        (result.inner_logic() != None ==>
            result.inner_logic().unwrap_logic().state.phase_id == self.capsule_public().3) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().state.storage_model).binding() ==
                self.ticket.inner_logic().allocation_binding().0) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().state.storage_model).raw_pointer() ==
                self.ticket.inner_logic().allocation_binding().1) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().state.storage_model).capacity() ==
                self.ticket.inner_logic().allocation_binding().2) &&
        (result.inner_logic() != None ==>
            (*result.inner_logic().unwrap_logic().state.storage_model).capacity() ==
                self.region.inner_logic().capacity()))]
    #[ensures(result.inner_logic() != None ==>
        header_lease_count(
            result.inner_logic().unwrap_logic().state.lease_cap.fragment_value()
        ) == header_lease_ready_entry_count(
            context.inner_logic().ready_entry_state(self.lease_key().slot()),
            self.lease_key().resource_id(), self.lease_key().slot()))]
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
                let expected_count: Snapshot<Int> = snapshot!(
                    header_lease_count(cap.inner_logic().fragment_value())
                );
                let tokens = BorrowedSharedContext::reborrow_tokens(Ghost::new(&mut *context));
                let expected_key: Snapshot<ReadyKey> = snapshot!(key);
                let expected_lease_id: Snapshot<Id> = snapshot!(key.resource_id());
                let expected_phase: Snapshot<Id> =
                    snapshot!(self.capsule_public().3);
                capsule.to_ref().open_with_tokens_with_frame(
                    tokens,
                    expected_count,
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
                            header_lease_idle(
                                cap.inner_logic().fragment_value(), *expected_lease_id) &&
                            header_lease_count(cap.inner_logic().fragment_value()) ==
                                *expected_count
                        )]
                        #[ensures(
                            (^payload).public() == payload.public() &&
                            (^payload).counter_model() == payload.counter_model() &&
                            (^payload).protocol() &&
                            (^payload).agrees_with_value(*owner_value) &&
                            match result {
                                Some(loan) =>
                                    loan.invariant() &&
                                    no_active_lend_loan(loan) &&
                                    loan.state.header_id == payload.header_id &&
                                    loan.state.lease_id == payload.lease.resource_id() &&
                                    loan.state.lease_key == payload.lease_key &&
                                    loan.state.phase_id == payload.public().3 &&
                                    *loan.state.counter_model == *payload.counter_model &&
                                    (*loan.state.storage_model).binding() == (*payload.allocation_binding).0 &&
                                    (*loan.state.storage_model).raw_pointer() == (*payload.allocation_binding).1 &&
                                    (*loan.state.storage_model).capacity() == (*payload.allocation_binding).2 &&
                                    header_lease_count(
                                        loan.state.lease_cap.fragment_value()) == *expected_count,
                                None => false,
                            }
                        )]
                        move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>,
                         expected_count: Snapshot<Int>| {
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
    #[requires(no_active_lend_loan(loan.inner_logic()))]
    #[requires(loan.inner_logic().state.lease_id == self.lease_resource_id())]
    #[requires(loan.inner_logic().state.lease_key == self.lease_key())]
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
    #[ensures(header_lease_ready_entry_idle((^context.inner_logic()).ready_entry_state(self.lease_key().slot()), self.lease_key().resource_id(), self.lease_key().slot(), self.lease_resource_id()))]
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
                        no_active_lend_loan(loan.inner_logic()) &&
                        loan.inner_logic().state.lease_id == *expected_lease_id &&
                        loan.inner_logic().state.lease_key == *expected_key
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
                        header_lease_idle(result.fragment_value(), (^payload).public().4) &&
                        header_lease_count(result.fragment_value()) ==
                            (^payload).ledger.unwrap_logic().count()
                    )]
                    move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>| {
                    let ticket = ticket_body.into_inner();
                    SharedPayload::prove_current_header_lease(
                        Ghost::new(&*payload),
                        Ghost::new(&loan.borrow().into_inner().state),
                    );
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
        loan.inner_logic().state.lease_id)]
    #[requires(capsule.inner_logic().val().public().1.6 ==
        loan.inner_logic().state.lease_key)]
    #[requires(key.inner_logic() == loan.inner_logic().state.lease_key)]
    #[requires(key.inner_logic().resource_id() == loan.inner_logic().state.lease_id)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(no_active_lend_loan(loan.inner_logic()))]
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
    #[ensures(header_lease_ready_entry_idle((^context.inner_logic()).ready_entry_state(key.inner_logic().slot()), key.inner_logic().resource_id(), key.inner_logic().slot(), key.inner_logic().resource_id()))]
    #[ensures(header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(key.inner_logic().slot()),
        key.inner_logic().resource_id(), key.inner_logic().slot()) ==
        header_lease_count(loan.inner_logic().state.lease_cap.fragment_value()))]
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
            let loan_ref = loan.borrow().into_inner();
            let expected_count: Snapshot<Int> = snapshot!(header_lease_count(
                loan_ref.state.lease_cap.fragment_value(),
            ));
            let cap = capsule.to_ref().open_with_tokens_with_frame(
                BorrowedSharedContext::reborrow_tokens(Ghost::new(&mut *context)),
                expected_count,
                ghost!(
                    #[requires(
                    (*payload).invariant() && payload.protocol() &&
                        payload.counter_model() == *owner_model &&
                        payload.agrees_with_value(*owner_value) &&
                        payload.public().4 == *expected_lease_id &&
                        payload.public().6 == *expected_key &&
                        loan.inner_logic().invariant() &&
                        header_lease_count(
                            loan.inner_logic().state.lease_cap.fragment_value(),
                        ) == *expected_count &&
                        no_active_lend_loan(loan.inner_logic()) &&
                        loan.inner_logic().state.lease_id == *expected_lease_id &&
                        loan.inner_logic().state.lease_key == *expected_key
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
                        header_lease_idle(result.fragment_value(), (^payload).public().4) &&
                        header_lease_count(result.fragment_value()) ==
                            (^payload).ledger.unwrap_logic().count() &&
                        header_lease_count(result.fragment_value()) == *expected_count
                    )]
                    move |payload: &mut SharedPayload,
                         owner_model: Snapshot<NativeCounter>,
                         owner_value: Snapshot<usize>,
                         expected_count: Snapshot<Int>| {
                        SharedPayload::prove_current_header_lease(
                            Ghost::new(&*payload),
                            Ghost::new(&loan.borrow().into_inner().state),
                        );
                        proof_assert!(header_lease_count(
                            loan.inner_logic().state.lease_cap.fragment_value(),
                        ) == *expected_count);
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
        loan.inner_logic().state.header_id as *mut ())]
    #[requires(capsule.inner_logic().val().public().1.3 ==
        loan.inner_logic().state.phase_id)]
    #[requires(capsule.inner_logic().val().public().1.4 ==
        loan.inner_logic().state.lease_id)]
    #[requires(capsule.inner_logic().val().public().1.6 ==
        loan.inner_logic().state.lease_key)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(event_namespace()))]
    #[requires(key.inner_logic() == loan.inner_logic().state.lease_key)]
    #[requires(key.inner_logic().resource_id() == loan.inner_logic().state.lease_id)]
    #[requires(loan.inner_logic().invariant())]
    #[requires(no_active_lend_loan(loan.inner_logic()))]
    #[requires(action.inner_logic().invariant())]
    #[requires(action.inner_logic().resource_id() == loan.inner_logic().state.phase_id)]
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
    #[ensures(result.inner_logic().storage.binding() ==
        capsule.inner_logic().val().public().1.5.0)]
    #[ensures(result.inner_logic().storage.raw_pointer() ==
        capsule.inner_logic().val().public().1.5.1)]
    #[ensures(result.inner_logic().storage.capacity() ==
        capsule.inner_logic().val().public().1.5.2)]
    #[ensures(result.inner_logic().region.namespace() ==
        capsule.inner_logic().val().public().1.5.0)]
    #[ensures(result.inner_logic().region.resource_id() ==
        capsule.inner_logic().val().public().1.5.0)]
    #[ensures(result.inner_logic().region.capacity() ==
        capsule.inner_logic().val().public().1.5.2)]
    #[ensures(result.inner_logic().region.lo() == 0)]
    #[ensures(result.inner_logic().region.hi() ==
        result.inner_logic().storage.capacity())]
    #[ensures(result.inner_logic().root.invariant())]
    #[ensures(*result.inner_logic().root.ward() ==
        loan.inner_logic().state.header_id as *const Shared)]
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
                snapshot!(loan.inner_logic().state.header_id as *mut ());
            let expected_lease_id: Snapshot<Id> =
                snapshot!(loan.inner_logic().state.lease_id);
            let expected_phase: Snapshot<Id> = snapshot!(loan.inner_logic().state.phase_id);
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
                        no_active_lend_loan(loan.inner_logic()) &&
                        loan.state.lease_key == *expected_key &&
                        loan.state.phase_id == *expected_phase &&
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
                        result.storage.binding() == payload.public().5.0 &&
                        result.storage.raw_pointer() == payload.public().5.1 &&
                        result.storage.capacity() == payload.public().5.2 &&
                        result.region.namespace() == payload.public().5.0 &&
                        result.region.resource_id() == payload.public().5.0 &&
                        result.region.capacity() == payload.public().5.2 &&
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
                            Ghost::new(&*payload), Ghost::new(&loan.state),
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
                                payload.lent_span,
                            )
                        );
                        let SharedHeaderReadLoan { root, state } = loan;
                        let SharedHeaderReadLoanState { lease_cap, .. } = state;
                        let current_count: Snapshot<Int> =
                            snapshot!(payload.ledger.unwrap_logic().count());
                        let idle_cap = HeaderLeaseAuthority::deactivate(
                            Ghost::new(&mut payload.lease),
                            stamp,
                            current_count,
                            Ghost::new(lease_cap),
                        ).into_inner();
                        drop(idle_cap);

                        let storage = payload.storage.take().unwrap();
                        proof_assert!(storage.binding() == (*payload.allocation_binding).0);
                        proof_assert!(storage.raw_pointer() == (*payload.allocation_binding).1);
                        proof_assert!(storage.capacity() == (*payload.allocation_binding).2);
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

}

impl SharedReleasePacket {
    /// Execute the native Release decrement and update the ledger from this
    /// packet's actual ticket and physical region in the atomic callback.
    /// Non-final owners return their lease to the ready context. The final
    /// owner returns its actual active loan and terminal action to the caller,
    /// keeping the root protected for the acquire/retirement half.
    #[requires(packet.inner_logic().invariant())]
    #[requires(header_id as *mut () == packet.inner_logic().capsule_public().0)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(
        packet.inner_logic().capsule_namespace()))]
    #[requires(header_lease_ready_entry_idle(context.inner_logic().ready_entry_state(packet.inner_logic().lease_key().slot()), packet.inner_logic().lease_key().resource_id(), packet.inner_logic().lease_key().slot(), packet.inner_logic().lease_key().resource_id()))]
    #[requires(packet.inner_logic().lease_key().slot() < context.inner_logic().ready_len())]
    #[requires(peer_ticket.inner_logic() == None || match peer_ticket.inner_logic() {
        Some(peer) => peer.invariant() &&
            peer.ledger_id() == packet.inner_logic().capsule_public().2 &&
            peer.fragment_resource_id() == packet.inner_logic().capsule_public().2 &&
            peer.allocation_binding() == packet.inner_logic().owner_allocation_binding() &&
            peer.owner_id() != packet.inner_logic().owner_id(),
        None => true,
    })]
    #[ensures(result.0@ > 0)]
    #[ensures(result.0@ == header_lease_ready_entry_count(
        context.inner_logic().ready_entry_state(packet.inner_logic().lease_key().slot()),
        packet.inner_logic().lease_key().resource_id(),
        packet.inner_logic().lease_key().slot()))]
    #[ensures(peer_ticket.inner_logic() != None ==> result.0@ > 1)]
    #[ensures((result.1.inner_logic() != None) == (result.0@ == 1))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(result.0@ > 1 ==> (^context.inner_logic()).ready_has_key(packet.inner_logic().lease_key()))]
    #[ensures(result.0@ > 1 ==> header_lease_ready_entry_idle((^context.inner_logic()).ready_entry_state(packet.inner_logic().lease_key().slot()), packet.inner_logic().lease_key().resource_id(), packet.inner_logic().lease_key().slot(), packet.inner_logic().lease_key().resource_id()))]
    #[ensures(result.0@ > 1 ==> header_lease_ready_entry_count(
        (^context.inner_logic()).ready_entry_state(packet.inner_logic().lease_key().slot()),
        packet.inner_logic().lease_key().resource_id(),
        packet.inner_logic().lease_key().slot()) == result.0@ - 1)]
    #[ensures(result.0@ == 1 ==> !(^context.inner_logic()).ready_has_key(packet.inner_logic().lease_key()))]
    #[ensures(result.0@ == 1 ==> (^context.inner_logic()).ready_entry_state(packet.inner_logic().lease_key().slot()) == None)]
    #[ensures(result.0@ == 1 ==>
        result.1.inner_logic().unwrap_logic().storage.invariant() &&
        result.1.inner_logic().unwrap_logic().storage.binding() ==
            packet.inner_logic().owner_allocation_binding().0 &&
        result.1.inner_logic().unwrap_logic().storage.raw_pointer() ==
            packet.inner_logic().owner_allocation_binding().1 &&
        result.1.inner_logic().unwrap_logic().storage.capacity() ==
            packet.inner_logic().owner_allocation_binding().2 &&
        result.1.inner_logic().unwrap_logic().region.invariant() &&
        result.1.inner_logic().unwrap_logic().region.namespace() ==
            packet.inner_logic().owner_allocation_binding().0 &&
        result.1.inner_logic().unwrap_logic().region.resource_id() ==
            packet.inner_logic().owner_allocation_binding().0 &&
        result.1.inner_logic().unwrap_logic().region.capacity() ==
            result.1.inner_logic().unwrap_logic().storage.capacity() &&
        result.1.inner_logic().unwrap_logic().region.namespace() ==
            result.1.inner_logic().unwrap_logic().storage.binding() &&
        result.1.inner_logic().unwrap_logic().region.resource_id() ==
            result.1.inner_logic().unwrap_logic().storage.binding() &&
        result.1.inner_logic().unwrap_logic().region.capacity() ==
            packet.inner_logic().owner_allocation_binding().2 &&
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
        peer_ticket: Ghost<Option<&OwnerTicket>>,
    ) -> (usize, Ghost<Option<SharedRetirementResources>>) {
        let mut context_ref = ghost!(context.into_inner());
        let packet_parts = ghost! {
            let mut packet = packet.into_inner();
            let key = match &packet {
                SharedReleasePacket::Mutable(packet) => packet.lease_key.into_inner(),
                SharedReleasePacket::Readonly(packet) => packet.lease_key.into_inner(),
            };
            BorrowedSharedContext::prove_ready_has_key_from_entry(
                Ghost::new(&**context_ref), Ghost::new(key),
            );
            let loan_result = packet.try_take_header_read_loan(
                Ghost::new(&mut **context_ref),
            );
            match packet {
                SharedReleasePacket::Mutable(packet) => (
                    packet.capsule.into_inner(),
                    key,
                    packet.ticket.into_inner(),
                    Some(packet.region.into_inner()),
                    loan_result.into_inner(),
                ),
                SharedReleasePacket::Readonly(packet) => (
                    packet.capsule.into_inner(),
                    key,
                    packet.ticket.into_inner(),
                    None,
                    loan_result.into_inner(),
                ),
            }
        };
        let (capsule, key, ticket, region, loan_result) = packet_parts.split();
        let loan = ghost! { loan_result.into_inner().unwrap() };
        let expected_counter: Snapshot<NativeCounter> =
            snapshot!(*loan.inner_logic().state.counter_model);
        let expected_key: Snapshot<ReadyKey> = snapshot!(key.inner_logic());
        let expected_lease_id: Snapshot<Id> = snapshot!(key.inner_logic().resource_id());
        let expected_ledger_id: Snapshot<Id> = snapshot!(ticket.inner_logic().ledger_id());
        let expected_phase: Snapshot<Id> = snapshot!(loan.inner_logic().state.phase_id);
        let (root, state) = ghost! {
            let loan = loan.into_inner();
            (loan.root, loan.state)
        }.split();
        let peer_present: Snapshot<bool> = snapshot!(peer_ticket.inner_logic() != None);
        let release_frame = ghost!((peer_present, state.into_inner()));
        let root_ref = ghost!(root.borrow().into_inner().as_ref());
        let counter = loaned_native_counter(
            header_id as *const Shared,
            root_ref,
        );
        let (old, callback) = counter.fetch_sub_release_with(
            ghost! { capsule.into_inner().to_ref() },
            release_frame,
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
                    *frame.0 == (peer_ticket.inner_logic() != None) &&
                    frame.1.invariant() &&
                    no_active_lend_state(&frame.1) &&
                    frame.1.lease_id == *expected_lease_id &&
                    frame.1.lease_key == *expected_key &&
                    ticket.inner_logic().invariant() &&
                    ticket.inner_logic().ledger_id() == payload.public().2 &&
                    (peer_ticket.inner_logic() == None ||
                        match peer_ticket.inner_logic() {
                            Some(peer) => peer.invariant() &&
                                peer.ledger_id() == payload.public().2 &&
                                peer.fragment_resource_id() == payload.public().2 &&
                                peer.allocation_binding() == *payload.allocation_binding &&
                                peer.owner_id() != ticket.inner_logic().owner_id(),
                            None => true,
                        }) &&
                    match region.inner_logic() {
                        Some(region) => ticket.inner_logic().class() == RefKind::Mutable &&
                            region.invariant() &&
                            ticket.inner_logic().owned_lo() == region.lo() &&
                            ticket.inner_logic().owned_hi() == region.hi() &&
                            ticket.inner_logic().allocation_binding().0 == region.namespace() &&
                            ticket.inner_logic().allocation_binding().2 == region.capacity(),
                        None => ticket.inner_logic().class() == RefKind::Readonly,
                    }
                )]
                #[ensures(
                    (^payload).public() == payload.public() &&
                    (^payload).counter_model() == payload.counter_model() &&
                    (^payload).protocol() &&
                    (^payload).agrees_with_value(new) &&
                    result.1.0 == frame.0 &&
                    result.1.1.invariant() &&
                    result.1.1.header_id == frame.1.header_id &&
                    result.1.1.counter_model == frame.1.counter_model &&
                    result.1.1.header_vec == frame.1.header_vec &&
                    result.1.1.original_capacity_repr == frame.1.original_capacity_repr &&
                    result.1.1.storage_model == frame.1.storage_model &&
                    result.1.1.ledger_id == frame.1.ledger_id &&
                    result.1.1.phase_id == frame.1.phase_id &&
                    result.1.1.generation == frame.1.generation &&
                    result.1.1.lease_id == frame.1.lease_id &&
                    result.1.1.lease_key == frame.1.lease_key &&
                    result.1.1.stamp == frame.1.stamp &&
                    result.1.1.lent_span == frame.1.lent_span &&
                    result.1.1.lent_region == frame.1.lent_region &&
                    result.1.1.lease_cap.key() == frame.1.lease_cap.key() &&
                    result.1.1.lease_cap.fragment_id() == frame.1.lease_cap.fragment_id() &&
                    header_lease_active(
                        result.1.1.lease_cap.fragment_value(),
                        result.1.1.lease_id, *result.1.1.stamp,
                    ) &&
                    header_lease_count(result.1.1.lease_cap.fragment_value()) == new@
                    && old@ == header_lease_count(frame.1.lease_cap.fragment_value())
                )]
                #[ensures(match result.0 {
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
                #[ensures(*frame.0 ==> old@ > 1)]
                move |payload: &mut SharedPayload,
                     old: usize,
                     new: usize,
                     frame: (Snapshot<bool>, SharedHeaderReadLoanState)| {
                    let (peer_present, mut loan_for_callback) = frame;
                    let mut ticket = ticket.into_inner();
                    match peer_ticket.into_inner() {
                        Some(peer) => {
                            payload.prove_peer_ticket_keeps_count_above_one(
                                Ghost::new(&mut ticket), Ghost::new(peer), old,
                            );
                        }
                        None => payload.prove_live_owner_matches_counter(&ticket, old),
                    }
                    proof_assert!(new@ + 1 == old@);
                    SharedPayload::prove_current_header_lease(
                        Ghost::new(&*payload),
                        Ghost::new(&loan_for_callback),
                    );
                    let owner = match region.into_inner() {
                        Some(region) => Ghost::new(SharedReleaseOwner::Mutable {
                            ticket, region,
                        }),
                        None => Ghost::new(SharedReleaseOwner::Readonly { ticket }),
                    };
                    let action = SharedPayload::drop_packet_owner(
                        Ghost::new(payload), expected_counter,
                        snapshot!(old), snapshot!(new), owner,
                        Ghost::new(&mut loan_for_callback),
                        expected_phase,
                    ).into_inner();
                    (action, (peer_present, loan_for_callback))
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
            let (action, (_, state)) = callback.into_inner();
            let loan = Ghost::new(SharedHeaderReadLoan::from_active_parts(
                root.into_inner(),
                state.header_id,
                state.counter_model,
                state.header_vec,
                state.original_capacity_repr,
                state.storage_model,
                state.ledger_id,
                state.phase_id,
                state.generation,
                state.lease_id,
                state.lease_key,
                state.stamp,
                state.lent_span,
                state.lent_region,
                state.lease_cap,
            ));
            proof_assert!(loan.inner_logic().invariant());
            proof_assert!(no_active_lend_loan(loan.inner_logic()));
            match action {
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
#[requires((header_id.addr_logic() & super::KIND_MASK) == super::KIND_ARC)]
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
    Some(Excl((result.lease_key().resource_id(), None, 1int)))
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
#[ensures((result.capsule_public().0.addr_logic() & super::KIND_MASK) ==
    super::KIND_ARC)]
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
        let initial_count: Snapshot<Int> = snapshot!(ledger.count());
        let (lease, lease_key) = HeaderLeaseAuthority::new(
            context,
            header_id,
            initial_count,
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
            lent_span: None,
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
    (forall<owner_id: Int>
        ledger.inner_logic().0.active_ids().get(owner_id) ==
            if owner_id == 0 {
                Some(Excl((ledger.inner_logic().0.allocation_binding(),
                    RefKind::Mutable, 0int, *cut)))
            } else if owner_id == 1 {
                Some(Excl((ledger.inner_logic().0.allocation_binding(),
                    RefKind::Mutable, *cut,
                    ledger.inner_logic().0.allocation_binding().2)))
            } else {
                None
            }) &&
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
#[requires((header_id.addr_logic() & super::KIND_MASK) == super::KIND_ARC)]
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
    Some(Excl((result.0.lease_key().resource_id(), None, 2int)))
))]
#[ensures(header_lease_ready_entry_idle(
    (^context.inner_logic()).ready_entry_state(result.0.lease_key().slot()),
    result.0.lease_key().resource_id(), result.0.lease_key().slot(),
    result.0.lease_key().resource_id()))]
#[ensures(header_lease_ready_entry_count(
    (^context.inner_logic()).ready_entry_state(result.0.lease_key().slot()),
    result.0.lease_key().resource_id(), result.0.lease_key().slot()) == 2)]
#[ensures(result.0.ticket.inner_logic().fragment_resource_id() ==
    result.0.capsule_public().2 &&
    result.1.ticket.inner_logic().fragment_resource_id() ==
        result.1.capsule_public().2)]
#[ensures((result.0.capsule_public().0.addr_logic() & super::KIND_MASK) ==
    super::KIND_ARC)]
#[ensures((^context.inner_logic()).token_namespaces() ==
    context.inner_logic().token_namespaces())]
#[ensures((^context.inner_logic()).ready_len() ==
    context.inner_logic().ready_len() + 1)]
#[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() ==>
    (^context.inner_logic()).ready_entry_state(i) ==
        context.inner_logic().ready_entry_state(i))]
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
            first_ticket.prove_fragment_resource_matches_ledger();
            second_ticket.prove_fragment_resource_matches_ledger();
        proof_assert!(forall<owner_id: Int>
            match ledger.active_ids().get(owner_id) {
                Some(Excl((_, RefKind::Readonly, _, _))) => false,
                _ => true,
            });
        proof_assert!(owner_value == 2usize && owner.val() == 2usize);
        proof_assert!(ledger.count() == 2 && ledger.agrees_with_value(2));
        proof_assert!(first_ticket.owner_id() != second_ticket.owner_id());
        let pool = PhysicalPool::empty_from(&region);
        let (first_region, second_region) = region.split_at(cut);
        let ledger_id = snapshot!(ledger.ledger_id()).into_ghost().into_inner();
        let phase_id = snapshot!(phase.resource_id()).into_ghost().into_inner();
        let initial_count: Snapshot<Int> = snapshot!(ledger.count());
        let (lease, lease_key) = HeaderLeaseAuthority::new(
            context,
            header_id,
            initial_count,
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
            lent_span: None,
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
