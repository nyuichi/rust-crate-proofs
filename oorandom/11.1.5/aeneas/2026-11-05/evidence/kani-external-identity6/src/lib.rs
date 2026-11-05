use oorandom::{Rand32,Rand64};
use core::borrow::{Borrow,BorrowMut};

#[cfg(kani)]
#[kani::proof]
fn concrete_identity_rand32() {
 let state: (u64,u64)=kani::any();
 let replacement: (u64,u64)=kani::any();
 let original=Rand32::from_state(state);
 let converted=<Rand32 as From<Rand32>>::from(original);
 assert_eq!(converted.state(),state);
 let borrowed=<Rand32 as Borrow<Rand32>>::borrow(&converted);
 assert_eq!(borrowed.state(),state);
 let mut destination=converted;
 {
  let mutable=<Rand32 as BorrowMut<Rand32>>::borrow_mut(&mut destination);
  assert_eq!(mutable.state(),state);
  *mutable=Rand32::from_state(replacement);
 }
 assert_eq!(destination.state(),replacement);
}

#[cfg(kani)]
#[kani::proof]
fn concrete_identity_rand64() {
 let state: (u128,u128)=kani::any();
 let replacement: (u128,u128)=kani::any();
 let original=Rand64::from_state(state);
 let converted=<Rand64 as From<Rand64>>::from(original);
 assert_eq!(converted.state(),state);
 let borrowed=<Rand64 as Borrow<Rand64>>::borrow(&converted);
 assert_eq!(borrowed.state(),state);
 let mut destination=converted;
 {
  let mutable=<Rand64 as BorrowMut<Rand64>>::borrow_mut(&mut destination);
  assert_eq!(mutable.state(),state);
  *mutable=Rand64::from_state(replacement);
 }
 assert_eq!(destination.state(),replacement);
}
