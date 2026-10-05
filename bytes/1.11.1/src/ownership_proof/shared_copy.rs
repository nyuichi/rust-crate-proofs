//! Sequential registered ARC copy reserve: copy the live view into a new unique
//! allocation, then explicitly retire only the source registration.
use super::*;
use creusot_std::{ghost::perm::Perm, prelude::*};
use crate::ownership_proof::{raw_vec, vec_capacity};
use super::sequential_shared_control::{ControlContext, HandleRegistration};

// Transfer the known-prefix fact from allocation coordinates to the visible view.
#[check(ghost)]
#[requires((*handle).proof_unique_at_zero_valid())]
#[ensures((*handle).proof_initialized())]
fn initialized_coordinates(handle:Snapshot<BytesMut>){
 proof_assert!(forall<i:Int> (*handle).proof_view_slot(i)==(*handle).proof_unique_slot(i));
}

#[requires(handle.proof_initialized() && handle.proof_registered_valid())]
#[requires(handle.shared_context.inner_logic() == None)]
#[requires(handle.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()))]
#[requires((*context.inner_logic().status.pending).len() >= 1)]
#[requires(handle.len@ + additional@ <= isize::MAX@)]
#[ensures(result.proof_unique_at_zero_valid() && result.proof_initialized())]
#[ensures(result.len == handle.len && result.cap@ >= handle.len@ + additional@)]
#[ensures(result.ptr@.unwrap_logic().2 == 0)]
#[ensures(forall<i: Int> 0 <= i && i < handle.len@ ==>
    result.proof_view_slot(i) == handle.proof_view_slot(i))]
#[ensures((^context.inner_logic()).valid(handle.shared_registration.inner_logic().unwrap_logic().control) &&
    ((*context.inner_logic().status.pending).len() >= 2 ==> (^context.inner_logic()).active()))]
#[ensures((* (^context.inner_logic()).status.pending).len() + 1 == (*context.inner_logic().status.pending).len())]
#[ensures(forall<other: HandleRegistration> other.matches(*context.inner_logic()) &&
    other.packet.0.logical_id() != handle.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
        other.matches(^context.inner_logic()))]
pub(crate) fn reserve_copy(
    handle: BytesMut, additional: usize, mut context: Ghost<&mut ControlContext>,
) -> BytesMut {
    let original_repr = {
        let shared = unsafe { Perm::as_ref(handle.data, ghost! { &**context.owner.as_ref().unwrap() }) };
        shared.original_capacity_repr
    };
    let minimum = crate::capacity_ops::original_capacity_from_repr(original_repr);
    let requested = cmp::max(handle.len + additional, minimum);
    let (pointer, _, capacity, mut caps) = vec_capacity::with_capacity_bound(requested);
    {
        let source = handle.as_slice();
        let destination = unsafe { raw_vec::borrow_bound_uninit_mut(pointer, capacity, ghost! { &mut caps.1 }) };
        crate::storage_ops::copy_to_uninit_prefix(destination, source);
    }
    let unique = BytesMut {
        ptr: pointer, len: handle.len, cap: capacity,
        data: invalid_ptr(crate::capacity_ops::pack_vec_metadata(original_repr)),
        unique_at_zero: ghost! { Some(caps.into_inner()) },
        pending_control: ghost! { None }, shared_registration: ghost! { None },
        shared_context: ghost! { None },
    };
    ghost!{initialized_coordinates(snapshot!(unique));};
    let _ = handle.proof_carrier_release(context);
    unique
}
