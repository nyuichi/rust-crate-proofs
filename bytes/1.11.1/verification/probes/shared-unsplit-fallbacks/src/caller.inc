pub(crate) fn empty_caller(input:Vec<u8>,cut:usize){
 let mut right=BytesMut::from_vec(input);
 ghost!{BytesMut::proof_initialization_coordinates(snapshot!(right));};
 let mut left=right.split_to(cmp::min(cut,right.len()));
 let mut context=right.proof_take_coordinator();
 left.truncate(0);
 #[cfg(not(creusot))]
 let expected=right.as_slice().to_vec();
 left.unsplit(right,context.borrow_mut());
 #[cfg(not(creusot))]
 assert_eq!(left.as_slice(),expected.as_slice());
 assert!(left.proof_carrier_release(context.borrow_mut()));
}

pub(crate) fn copy_caller(input:Vec<u8>,first:usize,second:usize){
 let mut tail=BytesMut::from_vec(input);
 ghost!{BytesMut::proof_initialization_coordinates(snapshot!(tail));};
 let mut left=tail.split_to(cmp::min(first,tail.len()));
 let middle=tail.split_to(cmp::min(second,tail.len()));
 let mut context=tail.proof_take_coordinator();
 #[cfg(not(creusot))]
 let expected={let mut v=left.as_slice().to_vec();v.extend_from_slice(tail.as_slice());v};
 if left.len==0 || middle.len==0 || tail.cap==0 {
  let _=left.proof_carrier_release(context.borrow_mut());
  let _=tail.proof_carrier_release(context.borrow_mut());
  assert!(middle.proof_carrier_release(context.borrow_mut()));
  return;
 }
 left.unsplit(tail,context.borrow_mut());
 let mut output=left;
 #[cfg(not(creusot))]
 assert_eq!(output.as_slice(),expected.as_slice());
 assert!(middle.proof_carrier_release(context.borrow_mut()));
 let pointer=output.ptr;let capacity=output.cap;
 let caps=ghost!{output.unique_at_zero.take().unwrap()};
 mem::forget(output);
 unsafe{release_unique_storage(pointer,capacity,0,caps);}
}

pub(crate) fn empty_other_caller(input:Vec<u8>){
 let mut left=BytesMut::from_vec(input);
 ghost!{BytesMut::proof_initialization_coordinates(snapshot!(left));};
 let right=left.split_to(0);
 let mut context=left.proof_take_coordinator();
 #[cfg(not(creusot))]
 let expected=left.as_slice().to_vec();
 left.unsplit(right,context.borrow_mut());
 #[cfg(not(creusot))]
 assert_eq!(left.as_slice(),expected.as_slice());
 assert!(left.proof_carrier_release(context.borrow_mut()));
}
