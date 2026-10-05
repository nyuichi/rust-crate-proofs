//! Reusable two-ticket registry from the proved sequential retirement probe.
//! This module moves real B1 capabilities; it contains no physical or atomic trust.
use creusot_std::{
    ghost::{NotObjective, invariant::{NonAtomicInvariant, Protocol, declare_namespace}, resource::Resource},
    logic::{Id, ra::excl::Excl}, prelude::*,
};
use super::raw_vec::{Recovery, PhysicalRegion, PhysicalPool};
declare_namespace! { RETIREMENT }
type RegistrationRA = (Option<Excl<()>>, Option<Excl<()>>);

pub(crate) struct Ticket {
    resource: Resource<RegistrationRA>,
    _not_objective: NotObjective,
}

#[derive(creusot_std::prelude::Clone, Copy)]
pub(crate) struct Status {
    pub(crate) capacity: Int,
    pub(crate) allocation: Id,
    pub(crate) split: Int,
    pub(crate) registration: Id,
    pub(crate) left_pending: bool,
    pub(crate) right_pending: bool,
}

pub(crate) struct State {
    status: Status,
    recovery: Option<Recovery>,
    pool: Option<PhysicalPool>,
    returned: Option<Resource<RegistrationRA>>,
    _not_objective: NotObjective,
}
pub(crate) type Coordinator = NonAtomicInvariant<State>;
pub(crate) type Packet = (Ticket, PhysicalRegion);

#[logic]
fn ticket_value(left: bool) -> RegistrationRA {
    if left { (Some(Excl(())), None) } else { (None, Some(Excl(()))) }
}

#[logic(prophetic)]
pub(crate) fn packet_matches(packet: Packet, allocation: Id, capacity: Int, split: Int, registration: Id, left: bool) -> bool {
    pearlite! {
        packet.0.resource.id() == registration &&
        packet.0.resource@ == ticket_value(left) &&
        packet.1.invariant() &&
        packet.1.capacity() == capacity &&
        packet.1.namespace() == allocation &&
        packet.1.resource_id() == allocation &&
        packet.1.lo() == (if left { 0 } else { split }) &&
        packet.1.hi() == (if left { split } else { capacity })
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


#[requires(split <= capacity)]
#[requires(caps.inner_logic().0.invariant() && caps.inner_logic().1.invariant())]
#[requires(caps.inner_logic().0.capacity() == capacity@ && caps.inner_logic().1.capacity() == capacity@)]
#[requires(caps.inner_logic().1.namespace() == caps.inner_logic().0.namespace())]
#[requires(caps.inner_logic().1.resource_id() == caps.inner_logic().0.namespace())]
#[requires(caps.inner_logic().1.lo() == 0 && caps.inner_logic().1.hi() == capacity@)]
#[ensures(result.inner_logic().0.public().capacity == capacity@)]
#[ensures(result.inner_logic().0.public().allocation == caps.inner_logic().0.namespace())]
#[ensures(result.inner_logic().0.public().split == split@)]
#[ensures(result.inner_logic().0.public().left_pending && result.inner_logic().0.public().right_pending)]
#[ensures(packet_matches(result.inner_logic().1, result.inner_logic().0.public().allocation, capacity@, split@, result.inner_logic().0.public().registration, true))]
#[ensures(packet_matches(result.inner_logic().2, result.inner_logic().0.public().allocation, capacity@, split@, result.inner_logic().0.public().registration, false))]
#[ensures(forall<index: Int> result.inner_logic().1.1.slot(index) ==
    if 0 <= index && index < split@ { caps.inner_logic().1.slot(index) } else { None })]
#[ensures(forall<index: Int> result.inner_logic().2.1.slot(index) ==
    if split@ <= index && index < capacity@ { caps.inner_logic().1.slot(index) } else { None })]
pub(crate) fn initialize(capacity: usize, split: usize, caps: Ghost<(Recovery, PhysicalRegion)>) -> Ghost<(Coordinator, Packet, Packet)> {
    ghost! {
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
    }
}
#[requires(packet_matches(packet.inner_logic(), coordinator.inner_logic().public().allocation, coordinator.inner_logic().public().capacity, coordinator.inner_logic().public().split, coordinator.inner_logic().public().registration, left))]
#[requires(if left { coordinator.inner_logic().public().left_pending } else { coordinator.inner_logic().public().right_pending })]
#[ensures((^coordinator.inner_logic()).public().capacity == coordinator.inner_logic().public().capacity)]
#[ensures((^coordinator.inner_logic()).public().allocation == coordinator.inner_logic().public().allocation)]
#[ensures((^coordinator.inner_logic()).public().split == coordinator.inner_logic().public().split)]
#[ensures((^coordinator.inner_logic()).public().registration == coordinator.inner_logic().public().registration)]
#[ensures((^coordinator.inner_logic()).public().left_pending == (if left { false } else { coordinator.inner_logic().public().left_pending }))]
#[ensures((^coordinator.inner_logic()).public().right_pending == (if left { coordinator.inner_logic().public().right_pending } else { false }))]
pub(crate) fn retire(coordinator: Ghost<&mut Coordinator>, packet: Ghost<Packet>, left: bool) {
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


#[requires(!coordinator.inner_logic().public().left_pending && !coordinator.inner_logic().public().right_pending)]
#[ensures(result.inner_logic().0.invariant() && result.inner_logic().1.invariant())]
#[ensures(result.inner_logic().0.capacity() == coordinator.inner_logic().public().capacity)]
#[ensures(result.inner_logic().1.capacity() == coordinator.inner_logic().public().capacity)]
#[ensures(result.inner_logic().0.namespace() == coordinator.inner_logic().public().allocation)]
#[ensures(result.inner_logic().1.namespace() == coordinator.inner_logic().public().allocation)]
#[ensures(result.inner_logic().1.resource_id() == coordinator.inner_logic().public().allocation)]
#[ensures(result.inner_logic().1.lo() == 0)]
#[ensures(result.inner_logic().1.hi() == coordinator.inner_logic().public().capacity)]
pub(crate) fn finish(coordinator: Ghost<Coordinator>) -> Ghost<(Recovery, PhysicalRegion)> {
    ghost! {
        let state = coordinator.into_inner().into_inner();
        let recovery = state.recovery.unwrap();
        let pool = state.pool.unwrap();
        let _returned = state.returned.unwrap();
        (recovery, pool.finish(0int))
    }
}

/// Open the private ticket/region relation at its defining module boundary.
/// This body composes B4-bound and preserves the affine ticket unchanged.
#[requires(packet_matches(*packet.inner_logic(), status.inner_logic().allocation,
    status.inner_logic().capacity, status.inner_logic().split,
    status.inner_logic().registration, left.inner_logic()))]
#[requires(bound.invariant())]
#[requires(bound@ != None && bound@.unwrap_logic().0 == status.inner_logic().allocation &&
    bound@.unwrap_logic().1 == status.inner_logic().capacity)]
#[requires((if left.inner_logic() { 0int } else { status.inner_logic().split }) <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <=
    (if left.inner_logic() { status.inner_logic().split } else { status.inner_logic().capacity }))]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    super::raw_vec::slot_known(packet.inner_logic().1.slot(
        bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@ && (^result)@.len() == len@)]
#[ensures(packet_matches(^packet.inner_logic(), status.inner_logic().allocation,
    status.inner_logic().capacity, status.inner_logic().split,
    status.inner_logic().registration, left.inner_logic()))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    (^packet.inner_logic()).1.slot(bound@.unwrap_logic().2 + offset) == Some(Some((^result)@[offset])))]
#[ensures(forall<index: Int>
    !(bound@.unwrap_logic().2 <= index && index < bound@.unwrap_logic().2 + len@) ==>
        (^packet.inner_logic()).1.slot(index) == packet.inner_logic().1.slot(index))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    super::raw_vec::slot_known((^packet.inner_logic()).1.slot(
        bound@.unwrap_logic().2 + offset)))]
pub(crate) unsafe fn borrow_packet_mut<'a>(
    bound: &'a super::raw_vec::BoundPtr,
    len: usize,
    mut packet: Ghost<&'a mut Packet>,
    status: Ghost<Snapshot<Status>>,
    left: Ghost<bool>,
) -> &'a mut [u8] {
    let _ = (status, left);
    unsafe { super::raw_vec::borrow_bound_mut(bound, len, ghost! { &mut packet.into_inner().1 }) }
}

/// Body-proved ticket-preserving adapter for B4-uninit.
#[requires(packet_matches(*packet.inner_logic(), status.inner_logic().allocation,
    status.inner_logic().capacity, status.inner_logic().split,
    status.inner_logic().registration, left.inner_logic()))]
#[requires(bound.invariant())]
#[requires(bound@ != None && bound@.unwrap_logic().0 == status.inner_logic().allocation &&
    bound@.unwrap_logic().1 == status.inner_logic().capacity)]
#[requires((if left.inner_logic() { 0int } else { status.inner_logic().split }) <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <=
    (if left.inner_logic() { status.inner_logic().split } else { status.inner_logic().capacity }))]
#[ensures(result@.len() == len@ && (^result)@.len() == len@)]
#[ensures(packet_matches(^packet.inner_logic(), status.inner_logic().allocation,
    status.inner_logic().capacity, status.inner_logic().split,
    status.inner_logic().registration, left.inner_logic()))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset) == Some(result@[offset]@))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    (^packet.inner_logic()).1.slot(bound@.unwrap_logic().2 + offset) == Some((^result)@[offset]@))]
#[ensures(forall<index: Int>
    !(bound@.unwrap_logic().2 <= index && index < bound@.unwrap_logic().2 + len@) ==>
        (^packet.inner_logic()).1.slot(index) == packet.inner_logic().1.slot(index))]
pub(crate) unsafe fn borrow_packet_uninit_mut<'a>(
    bound: super::raw_vec::BoundPtr,
    len: usize,
    mut packet: Ghost<&'a mut Packet>,
    status: Ghost<Snapshot<Status>>,
    left: Ghost<bool>,
) -> &'a mut [core::mem::MaybeUninit<u8>] {
    let _ = (status, left);
    unsafe { super::raw_vec::borrow_bound_uninit_mut(bound, len, ghost! { &mut packet.into_inner().1 }) }
}

/// Body-proved shared packet read; shared region borrowing freezes its slots.
#[requires(packet_matches(*packet.inner_logic(), status.inner_logic().allocation,
    status.inner_logic().capacity, status.inner_logic().split,
    status.inner_logic().registration, left.inner_logic()))]
#[requires(bound.invariant())]
#[requires(bound@ != None && bound@.unwrap_logic().0 == status.inner_logic().allocation &&
    bound@.unwrap_logic().1 == status.inner_logic().capacity)]
#[requires((if left.inner_logic() { 0int } else { status.inner_logic().split }) <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <=
    (if left.inner_logic() { status.inner_logic().split } else { status.inner_logic().capacity }))]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    super::raw_vec::slot_known(packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
pub(crate) unsafe fn borrow_packet<'a>(
    bound: &'a super::raw_vec::BoundPtr,
    len: usize,
    packet: Ghost<&'a Packet>,
    status: Ghost<Snapshot<Status>>,
    left: Ghost<bool>,
) -> &'a [u8] {
    let _ = (status, left);
    unsafe { super::raw_vec::borrow_bound(bound, len, ghost! { &packet.into_inner().1 }) }
}
