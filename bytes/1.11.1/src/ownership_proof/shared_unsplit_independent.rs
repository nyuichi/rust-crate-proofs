//! Sequential unsplit copying across two independently owned control contexts.
use super::*;
use creusot_std::prelude::*;
use sequential_shared_control::{ControlContext,HandleRegistration};
// Normalize the visible and allocation-relative coordinates for a fresh unique owner.
#[check(ghost)]
#[requires((*handle).proof_initialized() && (*handle).proof_unique_owned())]
#[requires(handle.ptr@.unwrap_logic().2 == 0)]
#[ensures((*handle).proof_unique_at_zero_valid())]
fn zero_coordinate_valid(handle:Snapshot<BytesMut>){
 proof_assert!(forall<i:Int> (*handle).proof_unique_slot(i)==(*handle).proof_view_slot(i));
}

#[requires(left.proof_initialized() && right.proof_initialized())]
#[requires(left.shared_context.inner_logic()==None && right.shared_context.inner_logic()==None)]
#[requires(left.shared_registration.inner_logic()!=None && right.shared_registration.inner_logic()!=None)]
#[requires(left.shared_registration.inner_logic().unwrap_logic().matches(*left_context.inner_logic()))]
#[requires(right.shared_registration.inner_logic().unwrap_logic().matches(*right_context.inner_logic()))]
#[requires(left.shared_registration.inner_logic().unwrap_logic().control!=right.shared_registration.inner_logic().unwrap_logic().control)]
#[requires((*left_context.inner_logic().status.pending).len()>=1)]
#[requires(left.len@+right.len@<=isize::MAX@)]
#[ensures(result.proof_initialized() && result.proof_unique_at_zero_valid())]
#[ensures(result.len@==left.len@+right.len@)]
#[ensures(forall<i:Int> 0<=i && i<result.len@ ==> result.proof_view_slot(i)==
 if i<left.len@{left.proof_view_slot(i)}else{right.proof_view_slot(i-left.len@)})]
#[ensures((* (^left_context.inner_logic()).status.pending).len()+1==(*left_context.inner_logic().status.pending).len())]
#[ensures((^left_context.inner_logic()).valid(left.shared_registration.inner_logic().unwrap_logic().control))]
#[ensures(forall<other:HandleRegistration> other.matches(*left_context.inner_logic()) &&
 other.packet.0.logical_id()!=left.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
 other.matches(^left_context.inner_logic()))]
#[ensures((* (^right_context.inner_logic()).status.pending).len()+1==(*right_context.inner_logic().status.pending).len())]
#[ensures((^right_context.inner_logic()).valid(right.shared_registration.inner_logic().unwrap_logic().control))]
#[ensures(forall<other:HandleRegistration> other.matches(*right_context.inner_logic()) &&
 other.packet.0.logical_id()!=right.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
 other.matches(^right_context.inner_logic()))]
pub(crate) fn independent_concat(left:BytesMut,right:BytesMut,left_context:Ghost<&mut ControlContext>,right_context:Ghost<&mut ControlContext>)->BytesMut{
 let mut output=shared_copy::reserve_copy(left,right.len,left_context);
 let old_len=output.len;
 let total=old_len+right.len;
 {
  let source=right.as_slice();
  let destination=output.spare_capacity_mut();
  crate::storage_ops::copy_to_uninit_prefix(destination,source);
 }
 unsafe{output.set_len(total);}
 let _=right.proof_carrier_release(right_context);
 ghost!{zero_coordinate_valid(snapshot!(output));};
 output
}


