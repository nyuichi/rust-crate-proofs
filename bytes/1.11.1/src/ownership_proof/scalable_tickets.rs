//! Sequential affine ticket inventory for repeated region subdivision.
//!
//! Each issued ticket is an authoritative-map fragment at a monotonically
//! fresh logical ID. The ID names an existing fragment; it never constructs
//! one. This module does not model native reference counts or handle access.

use creusot_std::{
    ghost::resource::{Authority, Fragment},
    logic::{FMap, Int, ra::{RA, excl::Excl}},
    prelude::*,
};

#[cfg(not(ticket_laws_only))]
use creusot_std::{
    ghost::{
        NotObjective,
        invariant::{NonAtomicInvariant, Protocol, declare_namespace},
    },
    logic::Id,
};

#[cfg(not(ticket_laws_only))]
use crate::ownership_proof::raw_vec::{PhysicalPool, PhysicalRegion, Recovery};

#[cfg(not(ticket_laws_only))]
declare_namespace! { SCALABLE_TICKETS }

type RegistrationMap = FMap<Int, Excl<()>>;

/// The authority contains exactly the monotonically issued ID prefix.
#[logic]
fn issued_prefix(registrations: RegistrationMap, next_id: Int) -> bool {
    pearlite! {
        0 <= next_id &&
        (forall<id: Int> registrations.contains(id) ==
            (0 <= id && id < next_id))
    }
}

/// One affine registration, paired with its owned physical interval.
#[cfg(not(ticket_laws_only))]
pub(crate) struct Ticket {
    id: Int,
    fragment: Fragment<RegistrationMap>,
    _not_objective: NotObjective,
}

#[cfg(not(ticket_laws_only))]
impl Ticket {
    /// Public logical projection used by callers without exposing the ticket
    /// representation.
    #[logic]
    pub(crate) fn logical_id(self) -> Int {
        self.id
    }
}

/// Pure pending geometry. An entry exists even when `lo == hi`, so an empty
/// physical interval still has an outstanding registration.
#[cfg(not(ticket_laws_only))]
#[derive(creusot_std::prelude::Clone, Copy)]
pub(crate) struct Status {
    pub(crate) capacity: Int,
    pub(crate) allocation: Id,
    pub(crate) registration: Id,
    pub(crate) next_id: Int,
    pub(crate) pending: Snapshot<FMap<Int, (Int, Int)>>,
}

#[cfg(not(ticket_laws_only))]
pub(crate) struct State {
    status: Status,
    recovery: Option<Recovery>,
    pool: Option<PhysicalPool>,
    authority: Option<Authority<RegistrationMap>>,
    returned: Option<Fragment<RegistrationMap>>,
    _not_objective: NotObjective,
}

#[cfg(not(ticket_laws_only))]
pub(crate) type Coordinator = NonAtomicInvariant<State>;
#[cfg(not(ticket_laws_only))]
pub(crate) type Packet = (Ticket, PhysicalRegion);

#[logic]
fn active_registrations(pending: FMap<Int, (Int, Int)>) -> RegistrationMap {
    pearlite! { pending.map(|(_id, _bounds)| Excl(())) }
}

#[logic]
fn pending_interval_bounds_valid(pending: FMap<Int, (Int, Int)>, capacity: Int) -> bool {
    pearlite! {
        forall<id: Int> match pending.get(id) {
            None => true,
            Some((lo, hi)) => 0 <= lo && lo <= hi && hi <= capacity,
        }
    }
}

#[logic]
fn pending_outside(pending: FMap<Int, (Int, Int)>, index: Int) -> bool {
    pearlite! {
        forall<id: Int> match pending.get(id) {
            Some((lo, hi)) => !(lo <= index && index < hi),
            None => true,
        }
    }
}

/// Replacing one half-open interval by its contiguous children preserves the
/// valid bounds of every entry and the exact set of covered indices.
#[check(ghost)]
#[requires((*pending).get(*parent) == Some((*lo, *hi)))]
#[requires(*left != *right)]
#[requires(!(*pending).contains(*left) && !(*pending).contains(*right))]
#[requires(*lo <= *at && *at <= *hi)]
#[requires(pending_interval_bounds_valid(*pending, *capacity))]
#[ensures(pending_interval_bounds_valid(
    (*pending).remove(*parent)
        .insert(*left, (*lo, *at))
        .insert(*right, (*at, *hi)),
    *capacity
))]
#[ensures(forall<index: Int>
    pending_outside(
        (*pending).remove(*parent)
            .insert(*left, (*lo, *at))
            .insert(*right, (*at, *hi)),
        index,
    ) == pending_outside(*pending, index))]
fn pending_split_geometry(
    pending: Snapshot<FMap<Int, (Int, Int)>>,
    parent: Snapshot<Int>,
    left: Snapshot<Int>,
    right: Snapshot<Int>,
    lo: Snapshot<Int>,
    at: Snapshot<Int>,
    hi: Snapshot<Int>,
    capacity: Snapshot<Int>,
) {
}

/// The registration projection of one interval remains a singleton ticket,
/// including when the interval bounds are equal.
#[check(ghost)]
#[ensures(active_registrations(FMap::singleton(*id, *bounds)) ==
    FMap::singleton(*id, Excl(())))]
fn active_singleton(id: Snapshot<Int>, bounds: Snapshot<(Int, Int)>) {
    let left: Snapshot<RegistrationMap> =
        snapshot!(active_registrations(FMap::singleton(*id, *bounds)));
    let right: Snapshot<RegistrationMap> = snapshot!(FMap::singleton(*id, Excl(())));
    proof_assert!((*left).ext_eq(*right));
}

/// Mapping the pending ledger commutes with removing a registration key.
#[check(ghost)]
#[ensures(active_registrations((*pending).remove(*id)) ==
    active_registrations(*pending).remove(*id))]
fn active_remove(pending: Snapshot<FMap<Int, (Int, Int)>>, id: Snapshot<Int>) {
    let left: Snapshot<RegistrationMap> = snapshot!(active_registrations((*pending).remove(*id)));
    let right: Snapshot<RegistrationMap> = snapshot!(active_registrations(*pending).remove(*id));
    proof_assert!((*left).ext_eq(*right));
}

/// Mapping the pending ledger commutes with inserting a registration key.
#[check(ghost)]
#[ensures(active_registrations((*pending).insert(*id, *bounds)) ==
    active_registrations(*pending).insert(*id, Excl(())))]
fn active_insert(
    pending: Snapshot<FMap<Int, (Int, Int)>>,
    id: Snapshot<Int>,
    bounds: Snapshot<(Int, Int)>,
) {
    let left: Snapshot<RegistrationMap> = snapshot!(active_registrations((*pending).insert(*id, *bounds)));
    let right: Snapshot<RegistrationMap> = snapshot!(active_registrations(*pending).insert(*id, Excl(())));
    proof_assert!((*left).ext_eq(*right));
}

/// Returning the singleton fragment for a pending key and retaining the
/// registration projection of the remaining ledger reconstructs the original
/// active map.
#[check(ghost)]
#[requires((*pending).contains(*id))]
#[ensures(FMap::singleton(*id, Excl(())).op(active_registrations((*pending).remove(*id))) ==
    Some(active_registrations(*pending)))]
fn active_partition_at(pending: Snapshot<FMap<Int, (Int, Int)>>, id: Snapshot<Int>) {
    active_remove(pending, id);
    let ticket: Snapshot<RegistrationMap> = snapshot!(FMap::singleton(*id, Excl(())));
    let rest: Snapshot<RegistrationMap> = snapshot!(active_registrations((*pending).remove(*id)));
    let whole: Snapshot<RegistrationMap> = snapshot!(active_registrations(*pending));
    proof_assert!(forall<key: Int>
        (*ticket).get(key).op((*rest).get(key)) == Some((*whole).get(key)));
    let merged: Snapshot<RegistrationMap> = snapshot!((*ticket).total_op(*rest));
    proof_assert!((*merged).ext_eq(*whole));
    proof_assert!((*ticket).op(*rest) == Some(*whole));
}

/// Move one existing affine ticket from the active ledger into the returned
/// inventory while preserving the authority/fragment composition.
#[check(ghost)]
#[requires((*returned).op(*ticket) == Some(*new_returned))]
#[requires((*ticket).op(*remaining) == Some(*active))]
#[requires((*returned).op(*active) == Some(*authority))]
#[ensures((*new_returned).op(*remaining) == Some(*authority))]
fn ledger_transfer(
    returned: Snapshot<RegistrationMap>,
    ticket: Snapshot<RegistrationMap>,
    remaining: Snapshot<RegistrationMap>,
    active: Snapshot<RegistrationMap>,
    new_returned: Snapshot<RegistrationMap>,
    authority: Snapshot<RegistrationMap>,
) {
    proof_assert!(RegistrationMap::associative_some(
        *returned,
        *ticket,
        *remaining,
        *new_returned,
        *active,
    ) == ());
}

/// Adding one fresh singleton registration preserves the ability to add a
/// different fresh singleton registration afterward.
#[check(ghost)]
#[requires((*map).get(*first) == None)]
#[requires((*map).get(*second) == None)]
#[requires(*first != *second)]
#[ensures((*map).op(FMap::singleton(*first, Excl(())))
    .unwrap_logic()
    .op(FMap::singleton(*second, Excl(()))) != None)]
fn fresh_ticket_extension(
    map: Snapshot<RegistrationMap>,
    first: Snapshot<Int>,
    second: Snapshot<Int>,
) {
    let first_ticket: Snapshot<RegistrationMap> =
        snapshot!(FMap::singleton(*first, Excl(())));
    let second_ticket: Snapshot<RegistrationMap> =
        snapshot!(FMap::singleton(*second, Excl(())));
    proof_assert!(forall<key: Int>
        (*map).get(key).op((*first_ticket).get(key)) != None);
    proof_assert!((*map).op(*first_ticket) != None);
    let after_first: Snapshot<RegistrationMap> =
        snapshot!((*map).op(*first_ticket).unwrap_logic());

    proof_assert!(forall<key: Int>
        Some((*after_first).get(key)) == (*map).get(key).op((*first_ticket).get(key))
    );
    proof_assert!(forall<key: Int>
        (*after_first).get(key).op((*second_ticket).get(key)) != None);
    proof_assert!((*after_first).op(*second_ticket) != None);
}

/// Extending the active inventory at one fresh key preserves its exact
/// composition with the returned inventory.
#[check(ghost)]
#[requires((*returned).op(*active) == Some(*authority))]
#[requires((*authority).get(*fresh) == None)]
#[ensures((*returned).op((*active).insert(*fresh, Excl(()))) ==
    Some((*authority).insert(*fresh, Excl(()))))]
fn ledger_extend_fresh(
    returned: Snapshot<RegistrationMap>,
    active: Snapshot<RegistrationMap>,
    authority: Snapshot<RegistrationMap>,
    fresh: Snapshot<Int>,
) {
    let next_active: Snapshot<RegistrationMap> = snapshot!((*active).insert(*fresh, Excl(())));
    let next_authority: Snapshot<RegistrationMap> = snapshot!((*authority).insert(*fresh, Excl(())));
    proof_assert!(forall<key: Int>
        (*returned).get(key).op((*next_active).get(key)) == Some((*next_authority).get(key)));
    let merged: Snapshot<RegistrationMap> = snapshot!((*returned).total_op(*next_active));
    proof_assert!((*merged).ext_eq(*next_authority));
    proof_assert!((*returned).op(*next_active) == Some(*next_authority));
}

/// Allocate the next two monotonic ticket IDs, preserving the issued prefix.
#[check(ghost)]
#[requires(issued_prefix(authority@, *next))]
#[ensures(issued_prefix((^authority)@, *next + 2))]
#[ensures((^authority)@ ==
    (authority@.op(FMap::singleton(*next, Excl(()))).unwrap_logic())
        .op(FMap::singleton(*next + 1, Excl(()))).unwrap_logic())]
#[ensures((^authority)@ == authority@.insert(*next, Excl(())).insert(*next + 1, Excl(())))]
#[ensures(result.0@ == FMap::singleton(*next, Excl(())))]
#[ensures(result.1@ == FMap::singleton(*next + 1, Excl(())))]
#[ensures(result.0.id() == authority.id() &&
    result.1.id() == authority.id() &&
    (^authority).id() == authority.id())]
fn issue_two_at_frontier(
    authority: &mut Authority<RegistrationMap>,
    next: Snapshot<Int>,
) -> (Fragment<RegistrationMap>, Fragment<RegistrationMap>) {
    let before: Snapshot<RegistrationMap> = snapshot!(authority@);
    let second: Snapshot<Int> = snapshot!(*next + 1);
    fresh_ticket_extension(before, next, second);
    let left = authority.add_fragment(snapshot!(FMap::singleton(*next, Excl(()))));
    proof_assert!(authority@.ext_eq((*before).insert(*next, Excl(()))));
    let right = authority.add_fragment(snapshot!(FMap::singleton(*second, Excl(()))));
    proof_assert!(authority@.ext_eq((*before).insert(*next, Excl(())).insert(*second, Excl(()))));
    proof_assert!(forall<id: Int> authority@.contains(id) == (0 <= id && id < *next + 2));
    (left, right)
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
pub(crate) fn packet_matches(packet: Packet, status: Status) -> bool {
    pearlite! {
        packet.0.fragment.id() == status.registration &&
        packet.0.fragment@ == FMap::singleton(packet.0.logical_id(), Excl(())) &&
        (*status.pending).get(packet.0.logical_id()) == Some((packet.1.lo(), packet.1.hi())) &&
        packet.1.invariant() &&
        packet.1.capacity() == status.capacity &&
        packet.1.namespace() == status.allocation &&
        packet.1.resource_id() == status.allocation
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_resources_valid(state: State) -> bool {
    pearlite! {
        match (state.recovery, state.pool) {
            (Some(recovery), Some(pool)) =>
                recovery.invariant() && pool.invariant() &&
                recovery.capacity() == state.status.capacity &&
                pool.capacity() == state.status.capacity &&
                recovery.namespace() == state.status.allocation &&
                pool.namespace() == state.status.allocation &&
                pool.resource_id() == state.status.allocation,
            _ => false,
        }
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_ledger_valid(state: State) -> bool {
    pearlite! {
        match (state.authority, state.returned) {
            (Some(authority), Some(returned)) =>
                authority.invariant() &&
                authority.id() == returned.id() &&
                authority.id() == state.status.registration &&
                returned@.op(active_registrations(*state.status.pending)) == Some(authority@) &&
                issued_prefix(authority@, state.status.next_id),
            _ => false,
        }
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_interval_bounds_valid(state: State) -> bool {
    pearlite! {
        pending_interval_bounds_valid(*state.status.pending, state.status.capacity)
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_pending_pool_disjoint(state: State) -> bool {
    pearlite! {
        match state.pool {
            Some(pool) => (forall<id: Int> match (*state.status.pending).get(id) {
                None => true,
                Some((lo, hi)) =>
                    (forall<index: Int> lo <= index && index < hi ==> !pool.contains(index)),
            }),
            _ => false,
        }
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_pending_regions_disjoint(state: State) -> bool {
    pearlite! {
        (forall<left: Int, right: Int> match
            ((*state.status.pending).get(left), (*state.status.pending).get(right)) {
            (Some((left_lo, left_hi)), Some((right_lo, right_hi))) =>
                left == right || left_hi <= right_lo || right_hi <= left_lo,
            _ => true,
        })
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_pool_complement_coverage(state: State) -> bool {
    pearlite! {
        match state.pool {
            Some(pool) => (forall<index: Int> pool.contains(index) == (
                0 <= index && index < state.status.capacity &&
                pending_outside(*state.status.pending, index)
            )),
            _ => false,
        }
    }
}

#[cfg(not(ticket_laws_only))]
#[logic(prophetic)]
fn state_geometry_valid(state: State) -> bool {
    pearlite! {
        state_interval_bounds_valid(state) &&
        state_pending_pool_disjoint(state) &&
        state_pending_regions_disjoint(state) &&
        state_pool_complement_coverage(state)
    }
}

#[cfg(not(ticket_laws_only))]
impl Protocol for State {
    type Public = Status;

    #[logic]
    fn public(self) -> Status { self.status }

    #[logic(prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
            0 <= self.status.capacity &&
            state_resources_valid(self) &&
            state_ledger_valid(self) &&
            state_geometry_valid(self)
        }
    }
}

/// Bind a full B1 allocation to the first affine ticket.
#[cfg(not(ticket_laws_only))]
#[ensures(result.0.invariant())]
#[ensures(result.0@ == Some((result.2.inner_logic().0.public().allocation, result.1@, 0int)))]
#[ensures(result.2.inner_logic().0.public().capacity == result.1@)]
#[ensures(result.2.inner_logic().0.public().next_id == 1)]
#[ensures((*result.2.inner_logic().0.public().pending).len() == 1)]
#[ensures(*result.2.inner_logic().0.public().pending ==
    FMap::singleton(0int, (0int, result.1@)))]
#[ensures((*result.2.inner_logic().0.public().pending).get(0int) == Some((0int, result.1@)))]
#[ensures(result.2.inner_logic().1.0.logical_id() == 0int)]
#[ensures(result.2.inner_logic().1.1.lo() == 0int)]
#[ensures(result.2.inner_logic().1.1.hi() == result.1@)]
#[ensures(packet_matches(result.2.inner_logic().1, result.2.inner_logic().0.public()))]
pub(crate) fn initialize(input: alloc::vec::Vec<u8>) -> (crate::ownership_proof::raw_vec::BoundPtr, usize, Ghost<(Coordinator, Packet)>) {
    let (raw, _len, caps) = crate::ownership_proof::raw_vec::detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let created = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let capacity_int = *Int::new(capacity as i128);
        let allocation_snapshot: Snapshot<Id> = snapshot!(recovery.namespace());
        let allocation = allocation_snapshot.into_ghost().into_inner();
        let mut authority = Authority::<RegistrationMap>::alloc().into_inner();
        let root_value: Snapshot<RegistrationMap> = snapshot!(FMap::singleton(0, Excl(())));
        let root_ticket = authority.add_fragment(root_value);
        let registration = authority.id_ghost();
        let returned = Fragment::new_unit(authority.id_ghost());
        let pending: Snapshot<FMap<Int, (Int, Int)>> = snapshot!(FMap::singleton(0, (0, capacity@)));
        let root_bounds: Snapshot<(Int, Int)> = snapshot!((0, capacity@));
        let root_id: Snapshot<Int> = snapshot!(0);
        active_singleton(root_id, root_bounds);
        let state = State {
            status: Status { capacity: capacity_int, allocation, registration, next_id: 1int,
                pending },
            recovery: Some(recovery), pool: Some(pool), authority: Some(authority),
            returned: Some(returned), _not_objective: NotObjective {},
        };
        let coordinator = NonAtomicInvariant::new(
            Ghost::new(state), snapshot!(SCALABLE_TICKETS()),
        ).into_inner();
        (coordinator, (Ticket { id: 0int, fragment: root_ticket, _not_objective: NotObjective {} }, region))
    };
    (bound, capacity, created)
}

/// Close one complete registry transition before returning to the invariant.
/// This helper moves only affine ticket resources; physical regions are split
/// independently by `replace` and never minted or recovered here.
#[cfg(not(ticket_laws_only))]
#[check(ghost)]
#[requires(state.inner_logic().protocol())]
#[requires(parent.inner_logic().fragment.id() == state.inner_logic().status.registration)]
#[requires(parent.inner_logic().fragment@ == FMap::singleton(parent.inner_logic().logical_id(), Excl(())))]
#[requires((*state.inner_logic().status.pending).contains(parent.inner_logic().logical_id()))]
#[requires((*state.inner_logic().status.pending).get(parent.inner_logic().logical_id()).unwrap_logic().0 <= *at_snapshot &&
    *at_snapshot <= (*state.inner_logic().status.pending).get(parent.inner_logic().logical_id()).unwrap_logic().1)]
#[ensures((^state.inner_logic()).protocol())]
#[ensures((^state.inner_logic()).recovery == state.inner_logic().recovery &&
    (^state.inner_logic()).pool == state.inner_logic().pool)]
#[ensures((^state.inner_logic()).status.capacity == state.inner_logic().status.capacity &&
    (^state.inner_logic()).status.allocation == state.inner_logic().status.allocation &&
    (^state.inner_logic()).status.registration == state.inner_logic().status.registration)]
#[ensures((^state.inner_logic()).status.next_id == state.inner_logic().status.next_id + 2)]
#[ensures(*(^state.inner_logic()).status.pending ==
    (*state.inner_logic().status.pending).remove(parent.inner_logic().logical_id())
        .insert(state.inner_logic().status.next_id,
            ((*state.inner_logic().status.pending).get(parent.inner_logic().logical_id()).unwrap_logic().0, *at_snapshot))
        .insert(state.inner_logic().status.next_id + 1,
            (*at_snapshot, (*state.inner_logic().status.pending).get(parent.inner_logic().logical_id()).unwrap_logic().1)))]
#[ensures(result.inner_logic().0.logical_id() == state.inner_logic().status.next_id &&
    result.inner_logic().1.logical_id() == state.inner_logic().status.next_id + 1)]
#[ensures(result.inner_logic().0.fragment.id() == state.inner_logic().status.registration &&
    result.inner_logic().1.fragment.id() == state.inner_logic().status.registration)]
#[ensures(result.inner_logic().0.fragment@ == FMap::singleton(result.inner_logic().0.logical_id(), Excl(())) &&
    result.inner_logic().1.fragment@ == FMap::singleton(result.inner_logic().1.logical_id(), Excl(())))]
fn split_registration(
    state: Ghost<&mut State>,
    parent: Ghost<Ticket>,
    at_snapshot: Snapshot<Int>,
) -> Ghost<(Ticket, Ticket)> {
    ghost! {
        let state = state.into_inner();
        let parent = parent.into_inner();
        let parent_id = parent.id;
        let parent_lo: Snapshot<Int> = snapshot!(state.status.pending.get(parent_id).unwrap_logic().0);
        let parent_hi: Snapshot<Int> = snapshot!(state.status.pending.get(parent_id).unwrap_logic().1);
        let left_bounds: Snapshot<(Int, Int)> = snapshot!((*parent_lo, *at_snapshot));
        let right_bounds: Snapshot<(Int, Int)> = snapshot!((*at_snapshot, *parent_hi));
        let parent_fragment = parent.fragment;
        let old_pending = state.status.pending;
        let parent_id_snapshot: Snapshot<Int> = snapshot!(parent_id);
        active_partition_at(old_pending, parent_id_snapshot);
        let parent_fragment_value: Snapshot<RegistrationMap> = snapshot!(parent_fragment@);
        let old_active: Snapshot<RegistrationMap> =
            snapshot!(active_registrations(*old_pending));
        let remaining_active: Snapshot<RegistrationMap> = snapshot! {
            active_registrations((*old_pending).remove(parent_id))
        };
        let ids: Snapshot<(Int, Int, Int)> = snapshot!((
            state.status.next_id,
            state.status.next_id + 1,
            state.status.next_id + 2,
        ));
        let (left_id, right_id, next_id) = ids.into_ghost().into_inner();
        let mut authority = state.authority.take().unwrap();
        let old_authority: Snapshot<RegistrationMap> = snapshot!(authority@);
        let returned = state.returned.take().unwrap();
        let old_returned: Snapshot<RegistrationMap> = snapshot!(returned@);
        let returned = returned.join(parent_fragment);
        let new_returned: Snapshot<RegistrationMap> = snapshot!(returned@);
        ledger_transfer(
            old_returned,
            parent_fragment_value,
            remaining_active,
            old_active,
            new_returned,
            old_authority,
        );
        let left_id_snapshot: Snapshot<Int> = snapshot!(left_id);
        let right_id_snapshot: Snapshot<Int> = snapshot!(right_id);
        ledger_extend_fresh(new_returned, remaining_active, old_authority, left_id_snapshot);
        let active_with_left: Snapshot<RegistrationMap> =
            snapshot!((*remaining_active).insert(left_id, Excl(())));
        let authority_with_left: Snapshot<RegistrationMap> =
            snapshot!((*old_authority).insert(left_id, Excl(())));
        ledger_extend_fresh(new_returned, active_with_left, authority_with_left, right_id_snapshot);
        let (left_ticket, right_ticket) =
            issue_two_at_frontier(&mut authority, left_id_snapshot);
        state.authority = Some(authority);
        state.returned = Some(returned);
        let without_parent: Snapshot<FMap<Int, (Int, Int)>> =
            snapshot!((*old_pending).remove(parent_id));
        active_remove(old_pending, parent_id_snapshot);
        let capacity: Snapshot<Int> = snapshot!(state.status.capacity);
        pending_split_geometry(
            old_pending,
            parent_id_snapshot,
            left_id_snapshot,
            right_id_snapshot,
            parent_lo,
            at_snapshot,
            parent_hi,
            capacity,
        );
        active_insert(without_parent, left_id_snapshot, left_bounds);
        let with_left: Snapshot<FMap<Int, (Int, Int)>> =
            snapshot!((*without_parent).insert(left_id, *left_bounds));
        active_insert(with_left, right_id_snapshot, right_bounds);
        let pending: Snapshot<FMap<Int, (Int, Int)>> = snapshot! {
            old_pending.remove(parent_id)
                .insert(left_id, *left_bounds)
                .insert(right_id, *right_bounds)
        };
        let next_id: Snapshot<Int> = snapshot!(next_id);
        state.status.pending = pending;
        state.status.next_id = next_id.into_ghost().into_inner();
        proof_assert!(state_ledger_valid(*state));
        proof_assert!(state_interval_bounds_valid(*state));
        proof_assert!(state_pending_pool_disjoint(*state));
        proof_assert!(state_pending_regions_disjoint(*state));
        proof_assert!(state_pool_complement_coverage(*state));
        proof_assert!(state_geometry_valid(*state));
        (Ticket { id: left_id, fragment: left_ticket, _not_objective: NotObjective {} },
         Ticket { id: right_id, fragment: right_ticket, _not_objective: NotObjective {} })
    }
}

/// Replace one pending ticket with two fresh tickets over a contiguous split.
/// The old ID is returned before the new IDs are issued; the authoritative map
/// prevents reuse even after arbitrary out-of-order returns.
#[cfg(not(ticket_laws_only))]
#[requires(packet_matches(packet.inner_logic(), coordinator.inner_logic().public()))]
#[requires(packet.inner_logic().1.lo() <= at@ && at@ <= packet.inner_logic().1.hi())]
#[ensures((^coordinator.inner_logic()).public().capacity == coordinator.inner_logic().public().capacity)]
#[ensures((^coordinator.inner_logic()).public().allocation == coordinator.inner_logic().public().allocation)]
#[ensures((^coordinator.inner_logic()).public().registration == coordinator.inner_logic().public().registration)]
#[ensures((^coordinator.inner_logic()).public().next_id == coordinator.inner_logic().public().next_id + 2)]
#[ensures(*(^coordinator.inner_logic()).public().pending ==
    (*coordinator.inner_logic().public().pending)
        .remove(packet.inner_logic().0.logical_id())
        .insert(coordinator.inner_logic().public().next_id,
            (packet.inner_logic().1.lo(), at@))
        .insert(coordinator.inner_logic().public().next_id + 1,
            (at@, packet.inner_logic().1.hi())))]
#[ensures((* (^coordinator.inner_logic()).public().pending).len() ==
    (*coordinator.inner_logic().public().pending).len() + 1)]
#[ensures(forall<other: Packet>
    packet_matches(other, coordinator.inner_logic().public()) &&
    other.0.logical_id() != packet.inner_logic().0.logical_id() ==>
        packet_matches(other, (^coordinator.inner_logic()).public()))]
#[ensures(result.inner_logic().0.0.logical_id() == coordinator.inner_logic().public().next_id)]
#[ensures(result.inner_logic().1.0.logical_id() == coordinator.inner_logic().public().next_id + 1)]
#[ensures(result.inner_logic().0.1.lo() == packet.inner_logic().1.lo())]
#[ensures(result.inner_logic().0.1.hi() == at@)]
#[ensures(result.inner_logic().1.1.lo() == at@)]
#[ensures(result.inner_logic().1.1.hi() == packet.inner_logic().1.hi())]
#[ensures(packet_matches(result.inner_logic().0, (^coordinator.inner_logic()).public()))]
#[ensures(packet_matches(result.inner_logic().1, (^coordinator.inner_logic()).public()))]
pub(crate) fn replace(
    coordinator: Ghost<&mut Coordinator>,
    packet: Ghost<Packet>,
    at: usize,
) -> Ghost<(Packet, Packet)> {
    ghost! {
        let (parent_ticket, parent_region) = packet.split();
        let region = parent_region.into_inner();
        let at_int = *Int::new(at as i128);
        let at_snapshot: Snapshot<Int> = snapshot!(at_int);
        let (left_region, right_region) = region.split_at(at_int);
        let tickets = NonAtomicInvariant::open_mut(coordinator, move |state: Ghost<&mut State>| {
            split_registration(state, parent_ticket, at_snapshot)
        });
        let (left_ticket, right_ticket) = tickets.into_inner();
        ((left_ticket, left_region), (right_ticket, right_region))
    }
}

/// Return a ticket and its region to their independent inventories.
#[cfg(not(ticket_laws_only))]
#[requires(packet_matches(packet.inner_logic(), coordinator.inner_logic().public()))]
#[ensures((^coordinator.inner_logic()).public().capacity == coordinator.inner_logic().public().capacity)]
#[ensures((^coordinator.inner_logic()).public().allocation == coordinator.inner_logic().public().allocation)]
#[ensures((^coordinator.inner_logic()).public().registration == coordinator.inner_logic().public().registration)]
#[ensures((^coordinator.inner_logic()).public().next_id == coordinator.inner_logic().public().next_id)]
#[ensures(*(^coordinator.inner_logic()).public().pending ==
    (*coordinator.inner_logic().public().pending).remove(packet.inner_logic().0.logical_id()))]
#[ensures((* (^coordinator.inner_logic()).public().pending).len() + 1 ==
    (*coordinator.inner_logic().public().pending).len())]
#[ensures(forall<other: Packet>
    packet_matches(other, coordinator.inner_logic().public()) &&
    other.0.logical_id() != packet.inner_logic().0.logical_id() ==>
        packet_matches(other, (^coordinator.inner_logic()).public()))]
pub(crate) fn retire(coordinator: Ghost<&mut Coordinator>, packet: Ghost<Packet>) {
    ghost! {
        NonAtomicInvariant::open_mut(coordinator, move |state: Ghost<&mut State>| {
            ghost! {
                let state = state.into_inner();
                let (ticket, region) = packet.into_inner();
                let old_pending = state.status.pending;
                let ticket_id: Snapshot<Int> = snapshot!(ticket.id);
                active_partition_at(old_pending, ticket_id);
                let ticket_value: Snapshot<RegistrationMap> = snapshot!(ticket.fragment@);
                let active: Snapshot<RegistrationMap> =
                    snapshot!(active_registrations(*old_pending));
                let remaining: Snapshot<RegistrationMap> = snapshot! {
                    active_registrations((*old_pending).remove(ticket.id))
                };
                let returned = state.returned.take().unwrap();
                let old_returned: Snapshot<RegistrationMap> = snapshot!(returned@);
                let authority_resource = state.authority.take().unwrap();
                let authority: Snapshot<RegistrationMap> = snapshot!(authority_resource@);
                state.authority = Some(authority_resource);
                let returned = returned.join(ticket.fragment);
                let new_returned: Snapshot<RegistrationMap> = snapshot!(returned@);
                ledger_transfer(old_returned, ticket_value, remaining, active, new_returned, authority);
                state.returned = Some(returned);
                let pool = state.pool.take().unwrap().retire(region);
                state.pool = Some(pool);
                let pending: Snapshot<FMap<Int, (Int, Int)>> = snapshot!((*old_pending).remove(ticket.id));
                state.status.pending = pending;
            }
        });
    };
}

/// Recover full allocation ownership only after every (including empty)
/// registration has returned.
#[cfg(not(ticket_laws_only))]
#[requires((*coordinator.inner_logic().public().pending).is_empty())]
#[ensures(result.0.invariant() && result.1.invariant())]
#[ensures(result.0.capacity() == coordinator.inner_logic().public().capacity)]
#[ensures(result.1.capacity() == coordinator.inner_logic().public().capacity)]
#[ensures(result.0.namespace() == coordinator.inner_logic().public().allocation)]
#[ensures(result.1.namespace() == coordinator.inner_logic().public().allocation)]
#[ensures(result.1.resource_id() == coordinator.inner_logic().public().allocation)]
#[ensures(result.1.lo() == 0 && result.1.hi() == coordinator.inner_logic().public().capacity)]
pub(crate) fn finish(coordinator: Ghost<Coordinator>) -> Ghost<(Recovery, PhysicalRegion)> {
    ghost! {
        let state = coordinator.into_inner().into_inner();
        let recovery = state.recovery.unwrap();
        let pool = state.pool.unwrap();
        let _authority = state.authority.unwrap();
        let _returned = state.returned.unwrap();
        (recovery, pool.finish(0int))
    }
}
