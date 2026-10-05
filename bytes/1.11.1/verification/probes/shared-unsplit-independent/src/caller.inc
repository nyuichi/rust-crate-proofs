#[requires(a@.len()+b@.len()<=isize::MAX@)]
pub(crate) fn independent_caller(a:Vec<u8>,b:Vec<u8>){
 let mut left=BytesMut::from_vec(a);let mut right=BytesMut::from_vec(b);
 ghost!{BytesMut::proof_initialization_coordinates(snapshot!(left));BytesMut::proof_initialization_coordinates(snapshot!(right));};
 let left_empty=left.split_to(0);let right_empty=right.split_to(0);
 let mut left_context=left.proof_take_coordinator();let mut right_context=right.proof_take_coordinator();
 assert!(!left_empty.proof_carrier_release(left_context.borrow_mut()));
 assert!(!right_empty.proof_carrier_release(right_context.borrow_mut()));
 #[cfg(not(creusot))]
 let expected={let mut v=left.as_slice().to_vec();v.extend_from_slice(right.as_slice());v};
 if left.data==right.data {
  assert!(left.proof_carrier_release(left_context.borrow_mut()));
  assert!(right.proof_carrier_release(right_context.borrow_mut()));
  return;
 }
 let adopt=left.len==0;let discard=right.cap==0;
 left.unsplit(right,left_context.borrow_mut(),right_context.borrow_mut());
 let mut output=left;
 #[cfg(not(creusot))]
 assert_eq!(output.as_slice(),expected.as_slice());
 if adopt {assert!(output.proof_carrier_release(right_context.borrow_mut()));return;}
 if discard {assert!(output.proof_carrier_release(left_context.borrow_mut()));return;}
 let pointer=output.ptr;let capacity=output.cap;
 let caps=ghost!{output.unique_at_zero.take().unwrap()};mem::forget(output);
 unsafe{release_unique_storage(pointer,capacity,0,caps);}
}
