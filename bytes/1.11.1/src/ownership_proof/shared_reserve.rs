//! Sequential reserve of a recovered Shared singleton.
use super::*;
use creusot_std::{ghost::perm::Perm, prelude::*};
use crate::ownership_proof::{raw_vec::{self, BoundPtr}, scalable_tickets, unique_reclaim};
use super::shared_reclaim::SingletonLease;
use super::sequential_shared_control::{ControlContext, ControlPtr, HandleRegistration};

#[requires(lease.inner_logic().valid(control))]
#[requires(view.invariant())]
#[requires(view@ == Some((lease.inner_logic().caps.0.namespace(), lease.inner_logic().caps.0.capacity(), offset@)))]
#[requires(offset@ + len@ <= lease.inner_logic().caps.0.capacity())]
#[requires(offset@ + len@ + additional@ <= isize::MAX@)]
#[requires(forall<i: Int> 0 <= i && i < len@ ==>
    raw_vec::slot_known(lease.inner_logic().caps.1.slot(offset@ + i)))]
#[ensures(result.2.inner_logic().valid(control))]
#[ensures(result.2.inner_logic().counter == lease.inner_logic().counter)]
#[ensures(result.0.invariant() && result.0@ != None)]
#[ensures(result.0@.unwrap_logic().0 == result.2.inner_logic().caps.0.namespace())]
#[ensures(result.0@.unwrap_logic().1 == result.2.inner_logic().caps.0.capacity())]
#[ensures(result.0@.unwrap_logic().2 + result.1@ <= result.2.inner_logic().caps.0.capacity())]
#[ensures(len@ + additional@ <= result.1@)]
// A fitting request takes only the reuse/movement branches, preserving the
// original physical allocation authority rather than invoking reallocation.
#[ensures((lease.inner_logic().caps.0.capacity() >= offset@ + len@ + additional@ ||
    (lease.inner_logic().caps.0.capacity() >= len@ + additional@ && offset >= len)) ==>
    result.2.inner_logic().caps.0 == lease.inner_logic().caps.0)]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==>
    result.2.inner_logic().caps.1.slot(result.0@.unwrap_logic().2 + i) == lease.inner_logic().caps.1.slot(offset@ + i))]
pub(crate) fn reserve_lease(
    control: ControlPtr, lease: Ghost<SingletonLease>, view: BoundPtr,
    offset: usize, len: usize, additional: usize,
) -> (BoundPtr, usize, Ghost<SingletonLease>) {
    let (base, old_capacity) = {
        let shared = unsafe { Perm::as_ref(control.pointer, ghost! { &*lease.owner }) };
        (shared.buffer.base, shared.buffer.capacity)
    };
    let required = len + additional;
    let required_with_offset = required + offset;
    if old_capacity >= required_with_offset {
        return (view, required, lease);
    }
    let (control_owner, caps) = ghost! {
        let lease = lease.into_inner();
        ((lease.counter, lease.owner), lease.caps)
    }.split();
    let (new_base, new_view, new_allocation_capacity, visible_capacity, new_caps) =
        if old_capacity >= required && offset >= len {
            let (recovery, region) = caps.split();
            let (new_base, region) = unique_reclaim::reclaim(view, offset, len, region);
            (new_base, new_base, old_capacity, old_capacity,
                ghost! { (recovery.into_inner(), region.into_inner()) })
        } else {
            // Match unique growth's bounded policy: doubling is an optimization,
            // while the requested allocation may still fit above half MAX.
            let double = if old_capacity > isize::MAX as usize / 2 {
                required_with_offset
            } else { old_capacity * 2 };
            let capacity = cmp::max(cmp::max(double, required_with_offset), 8);
            let (new_base, caps) = unsafe { raw_vec::reallocate_bound(base, old_capacity, capacity, caps) };
            (new_base, new_base.advance_within(offset), capacity, capacity - offset, caps)
        };
    let mut lease = ghost! {
        let (counter, owner) = control_owner.into_inner();
        SingletonLease { counter, owner, caps: new_caps.into_inner() }
    };
    {
        let shared = unsafe { Perm::as_mut(control.pointer, ghost! { &mut *lease.owner }) };
        shared.buffer.base = new_base;
        shared.buffer.capacity = new_allocation_capacity;
    }
    (new_view, visible_capacity, lease)
}

/// Register only the visible-capacity prefix, retaining its complement in the
/// retired pool. The temporary two-entry ghost inventory is never packaged as
/// a ControlContext: the unchanged native counter remains one throughout.
#[requires(lease.inner_logic().valid(control))]
#[requires(0 <= *end && *end <= lease.inner_logic().caps.0.capacity())]
#[ensures(result.inner_logic().1.control == control)]
#[ensures(result.inner_logic().1.matches(result.inner_logic().0))]
#[ensures((*result.inner_logic().0.status.pending).len() == 1)]
#[ensures(result.inner_logic().0.status.capacity == lease.inner_logic().caps.0.capacity())]
#[ensures(result.inner_logic().0.status.allocation == lease.inner_logic().caps.0.namespace())]
#[ensures(result.inner_logic().0.owner == Some(lease.inner_logic().owner))]
#[ensures(result.inner_logic().0.counter == Some(lease.inner_logic().counter))]
#[ensures(result.inner_logic().1.packet.1.lo() == 0 && result.inner_logic().1.packet.1.hi() == *end)]
#[ensures(forall<index: Int> 0 <= index && index < *end ==>
    result.inner_logic().1.packet.1.slot(index) == lease.inner_logic().caps.1.slot(index))]
pub(crate) fn reactivate_view(
    control: ControlPtr, lease: Ghost<SingletonLease>, end: Snapshot<Int>,
) -> Ghost<(ControlContext, HandleRegistration)> {
    let shared = unsafe { Perm::as_ref(control.pointer, ghost! { &*lease.owner }) };
    let capacity = shared.buffer.capacity;
    let (control_owner, caps) = ghost! {
        let lease = lease.into_inner();
        ((lease.counter, lease.owner), lease.caps)
    }.split();
    let (mut registry, root) = ghost! {
        scalable_tickets::initialize_from_caps(capacity, caps).into_inner()
    }.split();
    let (left, right) = ghost! {
        scalable_tickets::replace_at(registry.borrow_mut(), root, end).into_inner()
    }.split();
    scalable_tickets::retire(registry.borrow_mut(), right);
    ghost! {
        let (counter, owner) = control_owner.into_inner();
        let registry = registry.into_inner();
        let status = snapshot!(registry.public());
        (ControlContext { registry: Some(registry), counter: Some(counter), owner: Some(owner), status },
         HandleRegistration { control, status, packet: left.into_inner(), left: false })
    }
}
