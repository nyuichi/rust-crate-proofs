//! Sequential ghost coordinator over real physical regions; no Shared/atomic/Drop claim.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
use alloc::vec::Vec;
use creusot_std::{
    ghost::{NotObjective, invariant::{NonAtomicInvariant, Protocol, declare_namespace}, resource::Resource},
    logic::{Id, ra::excl::Excl},
    prelude::*,
};
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
use raw_vec::{BoundPtr, Recovery, PhysicalRegion, PhysicalPool, detach_vec, deallocate_bound_vec};

declare_namespace! { RETIREMENT }
type RegistrationRA = (Option<Excl<()>>, Option<Excl<()>>);

struct Ticket {
    resource: Resource<RegistrationRA>,
    _not_objective: NotObjective,
}

#[derive(creusot_std::prelude::Clone, Copy)]
struct Status {
    capacity: Int,
    allocation: Id,
    split: Int,
    registration: Id,
    left_pending: bool,
    right_pending: bool,
}

struct State {
    status: Status,
    recovery: Option<Recovery>,
    pool: Option<PhysicalPool>,
    returned: Option<Resource<RegistrationRA>>,
    _not_objective: NotObjective,
}
type Coordinator = NonAtomicInvariant<State>;
type Packet = (Ticket, PhysicalRegion);

#[logic]
fn ticket_value(left: bool) -> RegistrationRA {
    if left { (Some(Excl(())), None) } else { (None, Some(Excl(()))) }
}

#[logic(prophetic)]
fn packet_matches(packet: Packet, status: Status, left: bool) -> bool {
    pearlite! {
        packet.0.resource.id() == status.registration &&
        packet.0.resource@ == ticket_value(left) &&
        packet.1.invariant() &&
        packet.1.capacity() == status.capacity &&
        packet.1.namespace() == status.allocation &&
        packet.1.resource_id() == status.allocation &&
        packet.1.lo() == (if left { 0 } else { status.split }) &&
        packet.1.hi() == (if left { status.split } else { status.capacity })
    }
}

impl Protocol for State {
    type Public = Status;
    #[logic]
    fn public(self) -> Status { self.status }
    #[logic(prophetic)]
    fn protocol(self) -> bool {
        pearlite! {
            0 <= self.status.split && self.status.split <= self.status.capacity &&
            match (self.recovery, self.pool, self.returned) {
                (Some(recovery), Some(pool), Some(returned)) =>
                    recovery.invariant() && pool.invariant() &&
                    recovery.capacity() == self.status.capacity &&
                    pool.capacity() == self.status.capacity &&
                    recovery.namespace() == self.status.allocation &&
                    pool.namespace() == self.status.allocation &&
                    pool.resource_id() == self.status.allocation &&
                    returned.id() == self.status.registration &&
                    returned@ == (
                        if self.status.left_pending { None } else { Some(Excl(())) },
                        if self.status.right_pending { None } else { Some(Excl(())) }
                    ) &&
                    forall<index: Int> pool.contains(index) == (
                        (!self.status.left_pending && 0 <= index && index < self.status.split) ||
                        (!self.status.right_pending && self.status.split <= index && index < self.status.capacity)
                    ),
                _ => false,
            }
        }
    }
}

#[requires(split@ <= input@.len())]
#[ensures(result.0.invariant())]
#[ensures(result.0@ == Some((result.2.inner_logic().0.public().allocation, result.1@, 0int)))]
#[ensures(result.2.inner_logic().0.public().capacity == result.1@)]
#[ensures(result.2.inner_logic().0.public().split == split@)]
#[ensures(result.2.inner_logic().0.public().left_pending)]
#[ensures(result.2.inner_logic().0.public().right_pending)]
#[ensures(packet_matches(result.2.inner_logic().1, result.2.inner_logic().0.public(), true))]
#[ensures(packet_matches(result.2.inner_logic().2, result.2.inner_logic().0.public(), false))]
fn initialize(input: Vec<u8>, split: usize) -> (BoundPtr, usize, Ghost<(Coordinator, Packet, Packet)>) {
    let (raw, _len, caps) = detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let initialized = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let split_int = *Int::new(split as i128);
        let capacity_int = *Int::new(capacity as i128);
        let allocation_snapshot: Snapshot<Id> = snapshot!(recovery.namespace());
        let allocation = allocation_snapshot.into_ghost().into_inner();
        let (left, right) = region.split_at(split_int);
        let registrations = Resource::<RegistrationRA>::alloc(snapshot!((Some(Excl(())), Some(Excl(()))))).into_inner();
        let (left_ticket, right_ticket) = registrations.split(snapshot!((Some(Excl(())), None)), snapshot!((None, Some(Excl(())))));
        let registration = left_ticket.id_ghost();
        let (left_ticket, returned) = left_ticket.split(snapshot!((Some(Excl(())), None)), snapshot!((None, None)));
        let state = State {
            status: Status { capacity: capacity_int, allocation, split: split_int, registration, left_pending: true, right_pending: true },
            recovery: Some(recovery), pool: Some(pool), returned: Some(returned),
            _not_objective: NotObjective {},
        };
        let coordinator = NonAtomicInvariant::new(Ghost::new(state), snapshot!(RETIREMENT())).into_inner();
        (coordinator,
         (Ticket { resource: left_ticket, _not_objective: NotObjective {} }, left),
         (Ticket { resource: right_ticket, _not_objective: NotObjective {} }, right))
    };
    (bound, capacity, initialized)
}

#[requires(packet_matches(packet.inner_logic(), coordinator.inner_logic().public(), left))]
#[requires(if left { coordinator.inner_logic().public().left_pending } else { coordinator.inner_logic().public().right_pending })]
#[ensures((^coordinator.inner_logic()).public().capacity == coordinator.inner_logic().public().capacity)]
#[ensures((^coordinator.inner_logic()).public().allocation == coordinator.inner_logic().public().allocation)]
#[ensures((^coordinator.inner_logic()).public().split == coordinator.inner_logic().public().split)]
#[ensures((^coordinator.inner_logic()).public().registration == coordinator.inner_logic().public().registration)]
#[ensures((^coordinator.inner_logic()).public().left_pending == (if left { false } else { coordinator.inner_logic().public().left_pending }))]
#[ensures((^coordinator.inner_logic()).public().right_pending == (if left { coordinator.inner_logic().public().right_pending } else { false }))]
fn retire(coordinator: Ghost<&mut Coordinator>, packet: Ghost<Packet>, left: bool) {
    ghost! {
    NonAtomicInvariant::open_mut(coordinator, move |state: Ghost<&mut State>| {
        ghost! {
            let state = state.into_inner();
            let (ticket, region) = packet.into_inner();
            let returned = state.returned.take().unwrap();
            state.returned = Some(returned.join(ticket.resource));
            let pool = state.pool.take().unwrap();
            state.pool = Some(pool.retire(region));
            if left { state.status.left_pending = false; }
            else { state.status.right_pending = false; }
        };
    });
    };
}

#[requires(bound.invariant())]
#[requires(bound@ == Some((coordinator.inner_logic().public().allocation, capacity@, 0int)))]
#[requires(coordinator.inner_logic().public().capacity == capacity@)]
#[requires(!coordinator.inner_logic().public().left_pending)]
#[requires(!coordinator.inner_logic().public().right_pending)]
fn finalize(bound: BoundPtr, capacity: usize, coordinator: Ghost<Coordinator>) {
    let full = ghost! {
        let state = coordinator.into_inner().into_inner();
        let recovery = state.recovery.unwrap();
        let pool = state.pool.unwrap();
        let _returned = state.returned.unwrap();
        (recovery, pool.finish(0int))
    };
    // Always executed in ordinary code; ghost state never selects this native free.
    unsafe { deallocate_bound_vec(bound, capacity, full); }
}

#[requires(split@ <= input@.len())]
pub fn retire_left_then_right(input: Vec<u8>, split: usize) {
    let (bound, capacity, initialized) = initialize(input, split);
    let (mut coordinator, left, right) = initialized.split();
    retire(ghost!(&mut *coordinator), left, true);
    retire(ghost!(&mut *coordinator), right, false);
    finalize(bound, capacity, coordinator);
}

#[requires(split@ <= input@.len())]
pub fn retire_right_then_left(input: Vec<u8>, split: usize) {
    let (bound, capacity, initialized) = initialize(input, split);
    let (mut coordinator, left, right) = initialized.split();
    retire(ghost!(&mut *coordinator), right, false);
    retire(ghost!(&mut *coordinator), left, true);
    finalize(bound, capacity, coordinator);
}

/// The left physical region is empty, so the right retirement returns every byte.
/// Its missing left registration must nevertheless block finalization.
#[cfg(feature = "negative_missing_empty_ticket")]
pub fn reject_missing_empty_ticket(input: Vec<u8>) {
    let (bound, capacity, initialized) = initialize(input, 0);
    let (mut coordinator, left, right) = initialized.split();
    let _ = left;
    retire(ghost!(&mut *coordinator), right, false);
    proof_assert!(coordinator.public().split == 0);
    proof_assert!(coordinator.public().left_pending && !coordinator.public().right_pending);
    finalize(bound, capacity, coordinator);
}
