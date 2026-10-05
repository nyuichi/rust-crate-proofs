//! Sequential adjacent merge: retire one registration, retain both byte regions.
use super::*;
use creusot_std::{ghost::perm::Perm, prelude::*};
use crate::ownership_proof::scalable_tickets;
use super::sequential_shared_control::{ControlContext, ControlPtr, HandleRegistration};

#[requires(left.inner_logic().control == control && right.inner_logic().control == control)]
#[requires(left.inner_logic().matches(*context.inner_logic()) && right.inner_logic().matches(*context.inner_logic()))]
#[requires(left.inner_logic().packet.0.logical_id() != right.inner_logic().packet.0.logical_id())]
#[requires(left.inner_logic().packet.1.hi() == right.inner_logic().packet.1.lo())]
#[ensures((^context.inner_logic()).valid(control) && (^context.inner_logic()).active())]
#[ensures(result.inner_logic().control == control && result.inner_logic().matches(^context.inner_logic()))]
#[ensures(result.inner_logic().packet.0.logical_id() == left.inner_logic().packet.0.logical_id())]
#[ensures((* (^context.inner_logic()).status.pending).len() + 1 == (*context.inner_logic().status.pending).len())]
#[ensures(result.inner_logic().packet.1.lo() == left.inner_logic().packet.1.lo())]
#[ensures(result.inner_logic().packet.1.hi() == right.inner_logic().packet.1.hi())]
#[ensures(result.inner_logic().status.capacity == context.inner_logic().status.capacity &&
    result.inner_logic().status.allocation == context.inner_logic().status.allocation)]
#[ensures(forall<index: Int> result.inner_logic().packet.1.slot(index) ==
    if left.inner_logic().packet.1.lo() <= index && index < left.inner_logic().packet.1.hi() {
        left.inner_logic().packet.1.slot(index)
    } else if right.inner_logic().packet.1.lo() <= index && index < right.inner_logic().packet.1.hi() {
        right.inner_logic().packet.1.slot(index)
    } else { None })]
#[ensures(forall<other: HandleRegistration> other.matches(*context.inner_logic()) &&
    other.packet.0.logical_id() != left.inner_logic().packet.0.logical_id() &&
    other.packet.0.logical_id() != right.inner_logic().packet.0.logical_id() ==>
        other.matches(^context.inner_logic()))]
pub(crate) fn merge(
    control: ControlPtr, mut context: Ghost<&mut ControlContext>,
    left: Ghost<HandleRegistration>, right: Ghost<HandleRegistration>,
) -> Ghost<HandleRegistration> {
    ghost! {
        let remainder = snapshot!((*context.status.pending).remove(left.packet.0.logical_id()));
        scalable_tickets::pending_cardinality(remainder);
        proof_assert!((*remainder).contains(right.packet.0.logical_id()));
        proof_assert!((*remainder).len() >= 1);
    };
    proof_assert!((*context.status.pending).len() >= 2);
    let (permission, counter) = ghost! {
        let context = &mut **context;
        (&**context.owner.as_ref().unwrap(), context.counter.as_mut().unwrap())
    }.split();
    let shared = unsafe { Perm::as_ref(control.pointer, permission) };
    let old = shared.ref_count.fetch_sub_release(1, counter);
    assert!(old > 1);
    ghost! {
        let left = left.into_inner();
        let right = right.into_inner();
        let packet = scalable_tickets::adjacent_region::merge(
            ghost! { context.registry.as_mut().unwrap() },
            Ghost::new(left.packet), Ghost::new(right.packet),
        ).into_inner();
        context.status = snapshot!(context.registry.unwrap_logic().public());
        HandleRegistration { control, status: context.status, packet, left: left.left }
    }
}

#[requires(left.proof_initialized() && right.proof_initialized())]
#[requires(left.shared_context.inner_logic() == None && right.shared_context.inner_logic() == None)]
#[requires(left.shared_registration.inner_logic() != None && right.shared_registration.inner_logic() != None)]
#[requires(left.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()))]
#[requires(right.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()))]
#[requires(left.shared_registration.inner_logic().unwrap_logic().control == right.shared_registration.inner_logic().unwrap_logic().control)]
#[requires(left.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != right.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id())]
#[requires(left.len == left.cap)]
#[requires(left.shared_registration.inner_logic().unwrap_logic().packet.1.hi() == right.shared_registration.inner_logic().unwrap_logic().packet.1.lo())]
#[requires(right.ptr@.unwrap_logic().2 == right.shared_registration.inner_logic().unwrap_logic().packet.1.lo())]
#[ensures((^left).proof_initialized() && (^left).shared_context.inner_logic() == None)]
#[ensures((^left).shared_registration.inner_logic().unwrap_logic().matches(^context.inner_logic()))]
#[ensures((^left).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==
    left.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id())]
#[ensures((^left).ptr == left.ptr && (^left).data == left.data)]
#[ensures((^left).len@ == left.len@ + right.len@ && (^left).cap@ == left.cap@ + right.cap@)]
#[ensures((* (^context.inner_logic()).status.pending).len() + 1 == (*context.inner_logic().status.pending).len())]
#[ensures(forall<index: Int> 0 <= index && index < (^left).len@ ==>
    (^left).proof_view_slot(index) == if index < left.len@ { left.proof_view_slot(index) }
        else { right.proof_view_slot(index - left.len@) })]
#[ensures(forall<other: HandleRegistration> other.matches(*context.inner_logic()) &&
    other.packet.0.logical_id() != left.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
    other.packet.0.logical_id() != right.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
        other.matches(^context.inner_logic()))]
pub(crate) fn merge_into(left: &mut BytesMut, mut right: BytesMut, context: Ghost<&mut ControlContext>) {
    let old_left = snapshot!(*left);
    let old_right = snapshot!(right);
    let left_registration = ghost! { left.shared_registration.take().unwrap() };
    let right_registration = ghost! { right.shared_registration.take().unwrap() };
    let identity = ghost! { left_registration.control.identity.into_inner() };
    let control = ControlPtr { pointer: left.data, identity };
    left.len += right.len;
    left.cap += right.cap;
    mem::forget(right);
    let registration = merge(control, context, left_registration, right_registration);
    left.shared_registration = ghost! { Some(registration.into_inner()) };
    proof_assert!(forall<i: Int> 0 <= i && i < left.len@ ==>
        left.proof_view_slot(i) == if i < old_left.len@ { old_left.proof_view_slot(i) }
            else { old_right.proof_view_slot(i - old_left.len@) });
    proof_assert!(left.proof_owned_valid());
    proof_assert!(forall<i: Int> 0 <= i && i < left.len@ ==>
        crate::ownership_proof::raw_vec::slot_known(left.proof_view_slot(i)));
}
