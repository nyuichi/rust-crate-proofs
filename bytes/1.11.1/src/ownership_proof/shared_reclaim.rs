//! Sequential singleton ownership transition, retaining the live Shared cell.
//! This module is a child of bytes_mut in the proof extraction.
use super::*;
use creusot_std::{ghost::perm::Perm, prelude::*};
use crate::ownership_proof::{
    raw_vec::{Recovery, PhysicalRegion},
    sequential_counter::CounterOwn,
    scalable_tickets,
};
use super::sequential_shared_control::{ControlContext, ControlPtr, HandleRegistration};

pub(crate) struct SingletonLease {
    pub(crate) counter: CounterOwn,
    pub(crate) owner: Box<Perm<*const Shared>>,
    pub(crate) caps: (Recovery, PhysicalRegion),
}

impl SingletonLease {
    #[logic(open(super), prophetic)]
    pub(crate) fn valid(self, control: ControlPtr) -> bool {
        pearlite! {
            self.counter@.0 == *control.identity && self.counter@.1 == 1 &&
            *self.owner.ward() == control.pointer &&
            self.owner.val().ref_count@ == self.counter@.0 &&
            self.owner.val().original_capacity_repr <= 7usize &&
            self.owner.val().buffer.base.invariant() &&
            self.owner.val().buffer.base@ == Some((self.caps.0.namespace(), self.caps.0.capacity(), 0int)) &&
            self.owner.val().buffer.capacity@ == self.caps.0.capacity() &&
            self.caps.0.invariant() && self.caps.1.invariant() &&
            self.caps.1.capacity() == self.caps.0.capacity() &&
            self.caps.1.namespace() == self.caps.0.namespace() &&
            self.caps.1.resource_id() == self.caps.0.namespace() &&
            self.caps.1.lo() == 0 && self.caps.1.hi() == self.caps.0.capacity()
        }
    }
}

/// The actual Acquire observation used to select the singleton reserve branch.
#[requires(context.inner_logic().valid(control) && context.inner_logic().active())]
#[ensures(result == ((*context.inner_logic().status.pending).len() == 1))]
pub(crate) fn is_unique(control: ControlPtr, context: Ghost<&ControlContext>) -> bool {
    let shared = unsafe { Perm::as_ref(control.pointer, ghost! { &**context.owner.as_ref().unwrap() }) };
    shared.ref_count.load_acquire(ghost! { context.counter.as_ref().unwrap() }) == 1
}

#[requires(registration.inner_logic().control == control && registration.inner_logic().matches(context.inner_logic()))]
#[requires((*context.inner_logic().status.pending).len() == 1)]
#[ensures(result.inner_logic().valid(control))]
#[ensures(result.inner_logic().owner == context.inner_logic().owner.unwrap_logic())]
#[ensures(result.inner_logic().counter == context.inner_logic().counter.unwrap_logic())]
#[ensures(result.inner_logic().caps.0.namespace() == context.inner_logic().status.allocation)]
#[ensures(result.inner_logic().caps.0.capacity() == context.inner_logic().status.capacity)]
#[ensures(forall<index: Int> registration.inner_logic().packet.1.lo() <= index && index < registration.inner_logic().packet.1.hi() ==>
    result.inner_logic().caps.1.slot(index) == registration.inner_logic().packet.1.slot(index))]
pub(crate) fn acquire(
    control: ControlPtr,
    context: Ghost<ControlContext>,
    registration: Ghost<HandleRegistration>,
) -> Ghost<SingletonLease> {
    let permission = ghost! { &**context.owner.as_ref().unwrap() };
    let shared = unsafe { Perm::as_ref(control.pointer, permission) };
    let observed = shared.ref_count.load_acquire(ghost! { context.counter.as_ref().unwrap() });
    assert!(observed == 1);
    ghost! {
        let context = context.into_inner();
        let registration = registration.into_inner();
        let caps = scalable_tickets::singleton_region::recover(
            Ghost::new(context.registry.unwrap()), Ghost::new(registration.packet),
        ).into_inner();
        SingletonLease { counter: context.counter.unwrap(), owner: context.owner.unwrap(), caps }
    }
}

#[requires(lease.inner_logic().valid(control))]
#[ensures(result.inner_logic().1.control == control)]
#[ensures(result.inner_logic().1.matches(result.inner_logic().0))]
#[ensures((*result.inner_logic().0.status.pending).len() == 1)]
#[ensures(result.inner_logic().0.status.capacity == lease.inner_logic().caps.0.capacity())]
#[ensures(result.inner_logic().0.status.allocation == lease.inner_logic().caps.0.namespace())]
#[ensures(result.inner_logic().0.owner == Some(lease.inner_logic().owner))]
#[ensures(result.inner_logic().0.counter == Some(lease.inner_logic().counter))]
#[ensures(result.inner_logic().1.packet.1.lo() == 0 &&
    result.inner_logic().1.packet.1.hi() == lease.inner_logic().caps.0.capacity())]
#[ensures(forall<index: Int> result.inner_logic().1.packet.1.slot(index) == lease.inner_logic().caps.1.slot(index))]
pub(crate) fn reactivate(
    control: ControlPtr,
    lease: Ghost<SingletonLease>,
) -> Ghost<(ControlContext, HandleRegistration)> {
    let shared = unsafe { Perm::as_ref(control.pointer, ghost! { &*lease.owner }) };
    let capacity = shared.buffer.capacity;
    ghost! {
        let lease = lease.into_inner();
        let (registry, packet) = scalable_tickets::initialize_from_caps(capacity, Ghost::new(lease.caps)).into_inner();
        let status = snapshot!(registry.public());
        (ControlContext { registry: Some(registry), counter: Some(lease.counter), owner: Some(lease.owner), status },
         HandleRegistration { control, status, packet, left: false })
    }
}
