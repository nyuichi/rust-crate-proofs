//! Restricted sequential mixed and unique unsplit copying with explicit cleanup.
use super::*;
use creusot_std::prelude::*;
use sequential_shared_control::{ControlContext,HandleRegistration};
// Constant bit facts remain explicit outside bitwise proof mode.
#[check(ghost)]
#[cfg_attr(creusot, bitwise_proof)]
#[ensures(KIND_VEC & KIND_MASK == KIND_VEC)]
#[ensures(KIND_VEC >> crate::capacity_ops::VEC_POS_OFFSET == 0usize)]
pub(crate) fn empty_metadata_bits(){}

#[check(ghost)]
#[requires((*handle).proof_unique_at_zero_valid())]
#[ensures((*handle).proof_initialized())]
pub(crate) fn initial_coordinates(handle:Snapshot<BytesMut>){
 proof_assert!(forall<i:Int> (*handle).proof_view_slot(i)==(*handle).proof_unique_slot(i));
}
#[check(ghost)]
#[requires((*handle).proof_initialized() && (*handle).proof_unique_owned())]
#[requires(handle.ptr@.unwrap_logic().2==0)]
#[ensures((*handle).proof_unique_at_zero_valid())]
fn final_coordinates(handle:Snapshot<BytesMut>){
 proof_assert!(forall<i:Int> (*handle).proof_unique_slot(i)==(*handle).proof_view_slot(i));
}
#[requires((left.proof_initialized() || left.proof_empty_valid()) && (right.proof_initialized() || right.proof_empty_valid()))]
#[requires(left.len@+right.len@<=isize::MAX@)]
#[ensures(result.proof_initialized() && result.proof_unique_at_zero_valid())]
#[ensures(result.len@==left.len@+right.len@)]
#[ensures(forall<i:Int> 0<=i && i<result.len@ ==> result.proof_view_slot(i)==
 if i<left.len@{left.proof_view_slot(i)}else{right.proof_view_slot(i-left.len@)})]
pub(crate) fn copy_pair(left:&BytesMut,right:&BytesMut)->BytesMut{
 let total=left.len+right.len;
 let (pointer,_,capacity,caps)=crate::ownership_proof::vec_capacity::with_capacity_bound(total);
 let repr=original_capacity_to_repr(capacity);
 let mut output=BytesMut{ptr:pointer,len:0,cap:capacity,data:invalid_ptr(crate::capacity_ops::pack_vec_metadata(repr)),
  unique_at_zero:ghost!{Some(caps.into_inner())},pending_control:ghost!{None},shared_registration:ghost!{None},shared_context:ghost!{None}};
 ghost!{initial_coordinates(snapshot!(output));};
 if left.len>0 {let source=left.as_slice();let destination=output.spare_capacity_mut();crate::storage_ops::copy_to_uninit_prefix(destination,source);}
 unsafe{output.set_len(left.len);}
 if right.len>0 {let source=right.as_slice();let destination=output.spare_capacity_mut();crate::storage_ops::copy_to_uninit_prefix(destination,source);}
 unsafe{output.set_len(total);}
 ghost!{final_coordinates(snapshot!(output));};
 output
}
#[requires(handle.proof_unique_owned() || handle.proof_empty_valid())]
pub(crate) fn retire_unique(mut handle:BytesMut){
 ghost!{empty_metadata_bits();};
 let offset=unsafe{handle.get_vec_pos()};let pointer=handle.ptr;let capacity=handle.cap;
 // Zero total capacity denotes no physical allocation, including canonical empties.
 if capacity==0 && offset==0 {mem::forget(handle);return;}
 let caps=ghost!{handle.unique_at_zero.take().unwrap()};mem::forget(handle);
 unsafe{release_unique_storage(pointer,capacity,offset,caps);}
}
#[requires(((left.proof_unique_owned() || left.proof_empty_valid()) && right.proof_registered_valid()) || (left.proof_registered_valid() && (right.proof_unique_owned() || right.proof_empty_valid())))]
#[requires(left.shared_context.inner_logic()==None && right.shared_context.inner_logic()==None)]
#[requires(left.shared_registration.inner_logic()!=None ==> left.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()))]
#[requires(right.shared_registration.inner_logic()!=None ==> right.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()))]
#[requires((*context.inner_logic().status.pending).len()>=1)]
#[ensures((^context.inner_logic()).valid(if left.shared_registration.inner_logic()!=None {left.shared_registration.inner_logic().unwrap_logic().control}else{right.shared_registration.inner_logic().unwrap_logic().control}))]
#[ensures((* (^context.inner_logic()).status.pending).len()+1==(*context.inner_logic().status.pending).len())]
#[ensures((^context.inner_logic()).active()==((*context.inner_logic().status.pending).len()>=2))]
#[ensures(forall<other:HandleRegistration> other.matches(*context.inner_logic()) &&
 other.packet.0.logical_id()!=(if left.shared_registration.inner_logic()!=None {left.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()}else{right.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()}) ==>
 other.matches(^context.inner_logic()))]
pub(crate) fn retire_pair(left:BytesMut,right:BytesMut,context:Ghost<&mut ControlContext>){
 ghost!{empty_metadata_bits();};
 if left.kind()==KIND_VEC {retire_unique(left);let _=right.proof_carrier_release(context);}
 else {let _=left.proof_carrier_release(context);retire_unique(right);}
}

