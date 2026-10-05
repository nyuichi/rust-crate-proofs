//! Sequential control protocol for actual coordinator-carrier ARC splits.
//! Native counter operations use the existing scalar bridge. All registration
//! updates are proved compositions of the scalable affine ticket inventory.
use super::*;
use creusot_std::{ghost::perm::Perm, logic::Id, prelude::*};
use crate::ownership_proof::{raw_vec::{self, BoundPtr}, sequential_counter::CounterOwn, scalable_tickets};
pub(crate) use crate::ownership_proof::scalable_tickets::{Coordinator, Packet, Status};

#[derive(core::clone::Clone, Copy)]
pub(crate) struct ControlPtr {
    pub(super) pointer: *mut Shared,
    pub(super) identity: Ghost<Id>,
}
pub(crate) struct ControlContext {
    pub(super) registry: Option<Coordinator>,
    pub(super) counter: Option<CounterOwn>,
    pub(super) owner: Option<Box<Perm<*const Shared>>>,
    pub(super) status: Snapshot<Status>,
}
impl ControlContext {
    #[logic(open(super), prophetic)]
    pub(super) fn valid_count(self, control: ControlPtr, extra: Int) -> bool {
        pearlite! {
            match (self.registry, self.counter, self.owner) {
                (Some(registry), Some(counter), Some(owner)) =>
                    registry.public() == *self.status &&
                    counter@.0 == *control.identity &&
                    counter@.1 == (*self.status.pending).len() + extra &&
                    *owner.ward() == control.pointer &&
                    owner.val().ref_count@ == counter@.0 &&
                    owner.val().buffer.base.invariant() &&
                    owner.val().buffer.base@ == Some((self.status.allocation, self.status.capacity, 0int)) &&
                    owner.val().buffer.capacity@ == self.status.capacity,
                (None, None, None) => extra == 0 && (*self.status.pending).is_empty(),
                _ => false,
            }
        }
    }
    #[logic(open(super), prophetic)]
    pub(super) fn valid(self, control: ControlPtr) -> bool { self.valid_count(control, 0) }
    #[logic(open(super))]
    pub(super) fn active(self) -> bool { pearlite! { self.registry != None } }
}
pub(super) struct PendingControl {
    pub(super) counter: CounterOwn,
    pub(super) owner: Box<Perm<*const Shared>>,
    pub(super) caps: (raw_vec::Recovery, raw_vec::PhysicalRegion),
}
impl PendingControl {
    #[logic(open(super), prophetic)]
    pub(super) fn valid(self, pointer: *mut Shared, base: BoundPtr, capacity: usize) -> bool {
        pearlite! {
            base.invariant() && base@ == Some((self.caps.0.namespace(), capacity@, 0int)) &&
            self.caps.0.invariant() && self.caps.1.invariant() &&
            self.caps.0.capacity() == capacity@ && self.caps.1.capacity() == capacity@ &&
            self.caps.1.namespace() == self.caps.0.namespace() &&
            self.caps.1.resource_id() == self.caps.0.namespace() &&
            self.caps.1.lo() == 0 && self.caps.1.hi() == capacity@ &&
            *self.owner.ward() == pointer && self.counter@.1 == 2 &&
            self.owner.val().ref_count@ == self.counter@.0 &&
            self.owner.val().buffer.base == base && self.owner.val().buffer.capacity == capacity
        }
    }
}
pub(super) struct HandleRegistration {
    pub(super) control: ControlPtr,
    // This creation snapshot supplies stable ticket/region geometry. Matching
    // an evolving context additionally checks its CURRENT pending map below.
    pub(super) status: Snapshot<Status>,
    pub(super) packet: Packet,
    pub(super) left: bool,
}
impl HandleRegistration {
    #[logic(open(super), prophetic)]
    pub(super) fn view_lo(self) -> Int { self.packet.1.lo() }
    #[logic(open(super), prophetic)]
    pub(super) fn view_hi(self) -> Int { self.packet.1.hi() }

    #[logic(open(super), prophetic)]
    pub(super) fn valid(self) -> bool { scalable_tickets::packet_matches(self.packet, *self.status) }
    #[logic(open(super), prophetic)]
    pub(super) fn matches(self, context: ControlContext) -> bool {
        pearlite! { self.valid() && context.valid(self.control) && context.active() &&
            scalable_tickets::packet_matches(self.packet, *context.status) }
    }
    #[logic(open(super), prophetic)]
    pub(super) fn matches_incremented(self, context: ControlContext) -> bool {
        pearlite! { self.valid() && context.valid_count(self.control, 1) && context.active() &&
            scalable_tickets::packet_matches(self.packet, *context.status) }
    }
}

#[check(ghost)]
#[requires(pending.inner_logic().valid(pointer, base, capacity))]
#[requires(0 <= *cut && *cut <= capacity@)]
#[ensures(result.inner_logic().1.matches(result.inner_logic().0) && result.inner_logic().2.matches(result.inner_logic().0))]
#[ensures(result.inner_logic().1.control.pointer == pointer && result.inner_logic().2.control == result.inner_logic().1.control)]
#[ensures(result.inner_logic().0.status.capacity == capacity@ && result.inner_logic().0.status.allocation == pending.inner_logic().caps.0.namespace())]
#[ensures((*result.inner_logic().0.status.pending).len() == 2)]
#[ensures(result.inner_logic().1.packet.1.lo() == 0 && result.inner_logic().1.packet.1.hi() == *cut)]
#[ensures(result.inner_logic().2.packet.1.lo() == *cut && result.inner_logic().2.packet.1.hi() == capacity@)]
#[ensures(result.inner_logic().1.packet.0.logical_id() != result.inner_logic().2.packet.0.logical_id())]
#[ensures(forall<index: Int> result.inner_logic().1.packet.1.slot(index) ==
    if 0 <= index && index < *cut { pending.inner_logic().caps.1.slot(index) } else { None })]
#[ensures(forall<index: Int> result.inner_logic().2.packet.1.slot(index) ==
    if *cut <= index && index < capacity@ { pending.inner_logic().caps.1.slot(index) } else { None })]
pub(super) fn activate(pointer: *mut Shared, base: BoundPtr, capacity: usize,
    cut: Snapshot<Int>, pending: Ghost<PendingControl>) -> Ghost<(ControlContext, HandleRegistration, HandleRegistration)> {
    ghost! {
        let pending = pending.into_inner();
        let created = scalable_tickets::initialize_from_caps(capacity, Ghost::new(pending.caps));
        let (mut registry, root) = created.split();
        let children = scalable_tickets::replace_at(registry.borrow_mut(), root, cut);
        let (left, right) = children.into_inner();
        let registry = registry.into_inner();
        let identity: Snapshot<Id> = snapshot!(pending.counter@.0);
        let control = ControlPtr { pointer, identity: identity.into_ghost() };
        let status: Snapshot<Status> = snapshot!(registry.public());
        (ControlContext { registry: Some(registry), counter: Some(pending.counter), owner: Some(pending.owner), status },
         HandleRegistration { control, status, packet: left, left: true },
         HandleRegistration { control, status, packet: right, left: false })
    }
}

#[requires(context.inner_logic().valid(control) && context.inner_logic().active())]
#[requires((*context.inner_logic().status.pending).len() < isize::MAX@)]
#[ensures(result@ == (*context.inner_logic().status.pending).len())]
#[ensures((^context.inner_logic()).valid_count(control, 1) && (^context.inner_logic()).active())]
#[ensures((^context.inner_logic()).status == context.inner_logic().status)]
pub(super) fn increment(control: ControlPtr, mut context: Ghost<&mut ControlContext>) -> usize {
    let (permission, counter) = ghost! {
        let context = context.into_inner();
        (&**context.owner.as_ref().unwrap(), context.counter.as_mut().unwrap())
    }.split();
    let shared = unsafe { Perm::as_ref(control.pointer, permission) };
    shared.ref_count.fetch_add_relaxed(1, counter)
}

#[check(ghost)]
#[requires(parent.inner_logic().matches_incremented(*context.inner_logic()))]
#[requires(parent.inner_logic().packet.1.lo() <= *cut && *cut <= parent.inner_logic().packet.1.hi())]
#[ensures(result.inner_logic().0.matches(^context.inner_logic()) && result.inner_logic().1.matches(^context.inner_logic()))]
#[ensures(result.inner_logic().0.control == parent.inner_logic().control && result.inner_logic().1.control == parent.inner_logic().control)]
#[ensures((^context.inner_logic()).status.capacity == context.inner_logic().status.capacity &&
    (^context.inner_logic()).status.allocation == context.inner_logic().status.allocation &&
    (^context.inner_logic()).status.registration == context.inner_logic().status.registration)]
#[ensures((* (^context.inner_logic()).status.pending).len() == (*context.inner_logic().status.pending).len() + 1)]
#[ensures(result.inner_logic().0.packet.1.lo() == parent.inner_logic().packet.1.lo() && result.inner_logic().0.packet.1.hi() == *cut)]
#[ensures(result.inner_logic().1.packet.1.lo() == *cut && result.inner_logic().1.packet.1.hi() == parent.inner_logic().packet.1.hi())]
#[ensures(result.inner_logic().0.packet.0.logical_id() != result.inner_logic().1.packet.0.logical_id())]
#[ensures(forall<other: HandleRegistration>
    other.valid() && other.control == parent.inner_logic().control &&
    scalable_tickets::packet_matches(other.packet, *context.inner_logic().status) &&
    other.packet.0.logical_id() != parent.inner_logic().packet.0.logical_id() ==>
        other.matches(^context.inner_logic()))]
#[ensures(forall<index: Int> result.inner_logic().0.packet.1.slot(index) ==
    if parent.inner_logic().packet.1.lo() <= index && index < *cut { parent.inner_logic().packet.1.slot(index) } else { None })]
#[ensures(forall<index: Int> result.inner_logic().1.packet.1.slot(index) ==
    if *cut <= index && index < parent.inner_logic().packet.1.hi() { parent.inner_logic().packet.1.slot(index) } else { None })]
#[ensures(forall<other: HandleRegistration>
    other.valid() && scalable_tickets::packet_matches(other.packet, *context.inner_logic().status) ==>
        other.packet.0.logical_id() != result.inner_logic().0.packet.0.logical_id() &&
        other.packet.0.logical_id() != result.inner_logic().1.packet.0.logical_id())]
pub(super) fn split_more(mut context: Ghost<&mut ControlContext>, parent: Ghost<HandleRegistration>, cut: Snapshot<Int>)
    -> Ghost<(HandleRegistration, HandleRegistration)> {
    ghost! {
        let parent = parent.into_inner();
        let children = scalable_tickets::replace_at(ghost! { context.registry.as_mut().unwrap() }, Ghost::new(parent.packet), cut);
        let (left, right) = children.into_inner();
        context.status = snapshot!(context.registry.unwrap_logic().public());
        (HandleRegistration { control: parent.control, status: context.status, packet: left, left: true },
         HandleRegistration { control: parent.control, status: context.status, packet: right, left: false })
    }
}

#[requires(registration.inner_logic().control == control && registration.inner_logic().matches(*context.inner_logic()))]
#[ensures((^context.inner_logic()).valid(control))]
#[ensures(result == !(^context.inner_logic()).active())]
#[ensures(result == ((*context.inner_logic().status.pending).len() == 1))]
#[ensures(result == (* (^context.inner_logic()).status.pending).is_empty())]
#[ensures((^context.inner_logic()).status.capacity == context.inner_logic().status.capacity &&
    (^context.inner_logic()).status.allocation == context.inner_logic().status.allocation &&
    (^context.inner_logic()).status.registration == context.inner_logic().status.registration)]
#[ensures(* (^context.inner_logic()).status.pending ==
    (*context.inner_logic().status.pending).remove(registration.inner_logic().packet.0.logical_id()))]
#[ensures((* (^context.inner_logic()).status.pending).len() + 1 == (*context.inner_logic().status.pending).len())]
#[ensures(forall<other: HandleRegistration> other.matches(*context.inner_logic()) &&
    other.packet.0.logical_id() != registration.inner_logic().packet.0.logical_id() ==> other.matches(^context.inner_logic()))]
pub(super) fn release(control: ControlPtr, mut context: Ghost<&mut ControlContext>, registration: Ghost<HandleRegistration>) -> bool {
    ghost! { scalable_tickets::pending_cardinality(snapshot!(*context.status.pending)); };
    let old = {
        let (permission, counter) = ghost! {
            let context = &mut **context;
            (&**context.owner.as_ref().unwrap(), context.counter.as_mut().unwrap())
        }.split();
        let shared = unsafe { Perm::as_ref(control.pointer, permission) };
        shared.ref_count.fetch_sub_release(1, counter)
    };
    scalable_tickets::retire(ghost! { context.registry.as_mut().unwrap() }, ghost! { registration.into_inner().packet });
    ghost! { context.status = snapshot!(context.registry.unwrap_logic().public()); };
    if old != 1 { return false; }
    ghost! { scalable_tickets::pending_cardinality(snapshot!(*context.status.pending)); };
    {
        let permission = ghost! { &**context.owner.as_ref().unwrap() };
        let shared = unsafe { Perm::as_ref(control.pointer, permission) };
        let observed = shared.ref_count.load_acquire(ghost! { context.counter.as_ref().unwrap() });
        assert!(observed == 0);
    }
    let full = scalable_tickets::finish(ghost! { context.registry.take().unwrap() });
    let (base, capacity) = {
        let permission = ghost! { &mut **context.owner.as_mut().unwrap() };
        let shared = unsafe { Perm::as_mut(control.pointer, permission) };
        let base = shared.buffer.base;
        let capacity = shared.buffer.capacity;
        shared.buffer.base = BoundPtr::unbound(NonNull::dangling());
        shared.buffer.capacity = 0;
        (base, capacity)
    };
    unsafe { raw_vec::deallocate_bound_vec(base, capacity, full); }
    let owner = ghost! { let _counter = context.counter.take().unwrap(); context.owner.take().unwrap() };
    unsafe { Perm::drop(control.pointer, owner); }
    true
}

/// Open the private ticket/region relation at its defining module boundary.
/// This body composes B4-bound and preserves the affine ticket unchanged.
#[requires(scalable_tickets::packet_matches(*packet.inner_logic(), *status.inner_logic()))]
#[requires(bound.invariant())]
#[requires(bound@ != None && bound@.unwrap_logic().0 == status.inner_logic().allocation &&
    bound@.unwrap_logic().1 == status.inner_logic().capacity)]
#[requires(packet.inner_logic().1.lo() <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <=
    packet.inner_logic().1.hi())]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    raw_vec::slot_known(packet.inner_logic().1.slot(
        bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@ && (^result)@.len() == len@)]
#[ensures(scalable_tickets::packet_matches(^packet.inner_logic(), *status.inner_logic()))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    (^packet.inner_logic()).1.slot(bound@.unwrap_logic().2 + offset) == Some(Some((^result)@[offset])))]
#[ensures(forall<index: Int>
    !(bound@.unwrap_logic().2 <= index && index < bound@.unwrap_logic().2 + len@) ==>
        (^packet.inner_logic()).1.slot(index) == packet.inner_logic().1.slot(index))]
#[ensures((^packet.inner_logic()).0 == packet.inner_logic().0)]
#[ensures((^packet.inner_logic()).1.lo() == packet.inner_logic().1.lo())]
#[ensures((^packet.inner_logic()).1.hi() == packet.inner_logic().1.hi())]
#[ensures(scalable_tickets::packet_matches(^packet.inner_logic(), *status.inner_logic()))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    raw_vec::slot_known((^packet.inner_logic()).1.slot(
        bound@.unwrap_logic().2 + offset)))]
pub(crate) unsafe fn borrow_packet_mut<'a>(
    bound: &'a raw_vec::BoundPtr,
    len: usize,
    mut packet: Ghost<&'a mut Packet>,
    status: Ghost<Snapshot<Status>>,
    left: Ghost<bool>,
) -> &'a mut [u8] {
    let _ = (status, left);
    unsafe { raw_vec::borrow_bound_mut(bound, len, ghost! { &mut packet.into_inner().1 }) }
}


/// Body-proved shared packet read; shared region borrowing freezes its slots.
#[requires(scalable_tickets::packet_matches(*packet.inner_logic(), *status.inner_logic()))]
#[requires(bound.invariant())]
#[requires(bound@ != None && bound@.unwrap_logic().0 == status.inner_logic().allocation &&
    bound@.unwrap_logic().1 == status.inner_logic().capacity)]
#[requires(packet.inner_logic().1.lo() <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <=
    packet.inner_logic().1.hi())]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    raw_vec::slot_known(packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    packet.inner_logic().1.slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
pub(crate) unsafe fn borrow_packet<'a>(
    bound: &'a raw_vec::BoundPtr,
    len: usize,
    packet: Ghost<&'a Packet>,
    status: Ghost<Snapshot<Status>>,
    left: Ghost<bool>,
) -> &'a [u8] {
    let _ = (status, left);
    unsafe { raw_vec::borrow_bound(bound, len, ghost! { &packet.into_inner().1 }) }
}

/// Body-proved ticket-preserving adapter for B4-uninit.
#[requires(scalable_tickets::packet_matches(*packet.inner_logic(), *status.inner_logic()))]
#[requires(base.invariant())]
#[requires(base@ != None && base@.unwrap_logic().0 == status.inner_logic().allocation &&
    base@.unwrap_logic().1 == status.inner_logic().capacity)]
#[requires(packet.inner_logic().1.lo() <= base@.unwrap_logic().2)]
#[requires(base@.unwrap_logic().2 + capacity@ <=
    packet.inner_logic().1.hi())]
#[requires(visible <= capacity)]
#[requires(forall<index: Int> 0 <= index && index < visible@ ==>
    raw_vec::slot_known(packet.inner_logic().1.slot(base@.unwrap_logic().2 + index)))]
#[ensures(forall<index: Int> 0 <= index && index < visible@ ==>
    raw_vec::slot_known((^packet.inner_logic()).1.slot(base@.unwrap_logic().2 + index)))]
#[ensures(result@.len() == (capacity@ - visible@) && (^result)@.len() == (capacity@ - visible@))]
#[ensures(scalable_tickets::packet_matches(^packet.inner_logic(), *status.inner_logic()))]
#[ensures(forall<offset: Int> 0 <= offset && offset < (capacity@ - visible@) ==>
    packet.inner_logic().1.slot(base@.unwrap_logic().2 + visible@ + offset) == Some(result@[offset]@))]
#[ensures(forall<offset: Int> 0 <= offset && offset < (capacity@ - visible@) ==>
    (^packet.inner_logic()).1.slot(base@.unwrap_logic().2 + visible@ + offset) == Some((^result)@[offset]@))]
#[ensures(forall<index: Int>
    !(base@.unwrap_logic().2 + visible@ <= index && index < base@.unwrap_logic().2 + capacity@) ==>
        (^packet.inner_logic()).1.slot(index) == packet.inner_logic().1.slot(index))]
#[ensures((^packet.inner_logic()).0 == packet.inner_logic().0)]
#[ensures((^packet.inner_logic()).1.lo() == packet.inner_logic().1.lo())]
#[ensures((^packet.inner_logic()).1.hi() == packet.inner_logic().1.hi())]
pub(crate) unsafe fn borrow_packet_uninit_mut<'a>(
    base: raw_vec::BoundPtr,
    visible: usize,
    capacity: usize,
    mut packet: Ghost<&'a mut Packet>,
    status: Ghost<Snapshot<Status>>,
    left: Ghost<bool>,
) -> &'a mut [core::mem::MaybeUninit<u8>] {
    let _ = (status, left);
    unsafe { raw_vec::borrow_spare_preserving_prefix(base, visible, capacity, ghost! { &mut packet.into_inner().1 }) }
}

