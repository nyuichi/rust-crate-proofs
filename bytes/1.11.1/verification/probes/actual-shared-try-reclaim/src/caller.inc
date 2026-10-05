#[requires(input@.len()+additional@<=isize::MAX@)]
pub(crate) fn reclaim_caller(input:Vec<u8>,at:usize,additional:usize,singleton:bool,prefix:bool)->bool{
 #[cfg(not(creusot))]
 let allocation_capacity=input.capacity();
 let mut right=BytesMut::from_vec(input);ghost!{BytesMut::proof_initialization_coordinates(snapshot!(right));};
 let at=cmp::min(at,right.len);let left=right.split_to(at);let mut context=right.proof_take_coordinator();
 let (mut right,left)=if prefix{(left,right)}else{(right,left)};
 let mut sibling=if singleton{assert!(!left.proof_carrier_release(context.borrow_mut()));None}else{Some(left)};
 let mut coordinator=ghost!{Some(context.into_inner())};
 #[cfg(not(creusot))]
 let expected=right.as_slice().to_vec();
 #[cfg(not(creusot))]
 let expected_success=additional<=right.cap-right.len || (singleton &&
  (allocation_capacity>=right.len+additional+if prefix{0}else{at} ||
   (allocation_capacity>=right.len+additional && (if prefix{0}else{at})>=right.len)));
 let before=snapshot!(coordinator.inner_logic().unwrap_logic().status);
 let success=right.try_reclaim(additional,coordinator.borrow_mut());
 #[cfg(feature="negative_allocation_changed")]
 proof_assert!(coordinator.inner_logic().unwrap_logic().status.allocation != before.allocation);
 #[cfg(not(creusot))]
 {assert_eq!(right.as_slice(),expected.as_slice());assert_eq!(success,expected_success);}
 let mut context=ghost!{coordinator.take().unwrap()};
 if let Some(left)=sibling.take(){assert!(!left.proof_carrier_release(context.borrow_mut()));}
 assert!(right.proof_carrier_release(context.borrow_mut()));
 success
}
