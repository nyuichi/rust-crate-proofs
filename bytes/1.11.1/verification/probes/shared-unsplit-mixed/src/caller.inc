use shared_unsplit_mixed::{retire_unique,initial_coordinates};
#[ensures((result.proof_initialized() && result.proof_unique_owned()) || result.proof_empty_valid())]
#[ensures(result.len@<=input@.len())]
fn prepare_unique(input:Vec<u8>,advance:usize,canonical:bool)->BytesMut{
 let mut unique=BytesMut::from_vec(input);
 ghost!{initial_coordinates(snapshot!(unique));};
 let count=cmp::min(advance,cmp::min(unique.cap,MAX_VEC_POS));
 unsafe{unique.advance_unchecked(count);}
 if canonical {
  let empty=BytesMut{ptr:crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling()),len:0,cap:0,data:invalid_ptr(KIND_VEC),
   unique_at_zero:ghost!{None},pending_control:ghost!{None},shared_registration:ghost!{None},shared_context:ghost!{None}};
  retire_unique(mem::replace(&mut unique,empty));
 }
 unique
}
#[cfg(not(feature="unique_pair"))]
#[requires(a@.len()+b@.len()<=isize::MAX@)]
pub(crate) fn mixed_caller(a:Vec<u8>,b:Vec<u8>,unique_left:bool,advance:usize,canonical:bool){
 ghost!{shared_unsplit_mixed::empty_metadata_bits();};
 let unique=prepare_unique(a,advance,canonical);let mut shared=BytesMut::from_vec(b);
 ghost!{initial_coordinates(snapshot!(shared));};
 let empty=shared.split_to(0);let mut context=shared.proof_take_coordinator();
 assert!(!empty.proof_carrier_release(context.borrow_mut()));
 let (mut left,right)=if unique_left{(unique,shared)}else{(shared,unique)};
 #[cfg(not(creusot))]
 let expected={let mut v=if left.len==0{Vec::new()}else{left.as_slice().to_vec()};if right.len>0{v.extend_from_slice(right.as_slice());}v};
 left.unsplit(right,context.borrow_mut());
 let output=left;
 #[cfg(not(creusot))]
 if output.len==0 {assert!(expected.is_empty());}else{assert_eq!(output.as_slice(),expected.as_slice());}
 if output.kind()==KIND_VEC {retire_unique(output);proof_assert!(!context.active());}
 else {assert!(output.proof_carrier_release(context.borrow_mut()));}
}
#[cfg(feature="unique_pair")]
#[requires(a@.len()+b@.len()<=isize::MAX@)]
pub(crate) fn unique_caller(a:Vec<u8>,b:Vec<u8>,advance:usize,left_empty:bool,right_empty:bool){
 let mut left=prepare_unique(a,advance,left_empty);let right=prepare_unique(b,advance,right_empty);
 #[cfg(not(creusot))]
 let expected={let mut v=if left.len==0{Vec::new()}else{left.as_slice().to_vec()};if right.len>0{v.extend_from_slice(right.as_slice());}v};
 left.unsplit(right);
 let output=left;
 #[cfg(not(creusot))]
 if output.len==0 {assert!(expected.is_empty());}else{assert_eq!(output.as_slice(),expected.as_slice());}
 retire_unique(output);
}
