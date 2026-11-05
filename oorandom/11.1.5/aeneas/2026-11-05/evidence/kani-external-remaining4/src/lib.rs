#![feature(clone_to_uninit)]
use oorandom::{Rand32,Rand64};
use core::{any::{Any,TypeId},clone::CloneToUninit,mem::MaybeUninit};

#[cfg(kani)]
#[kani::proof]
fn clone_slot_rand32() {
 let state:(u64,u64)=kani::any();
 let source=Rand32::from_state(state);
 let mut destination=MaybeUninit::<Rand32>::uninit();
 // Separate live source and writable aligned destination stack slots, exact Self size.
 unsafe { <Rand32 as CloneToUninit>::clone_to_uninit(&source,destination.as_mut_ptr().cast::<u8>()); }
 let copied=unsafe { destination.assume_init() };
 assert_eq!(source.state(),state);
 assert_eq!(copied.state(),state);
}
#[cfg(kani)]
#[kani::proof]
fn any_typeid_rand32() {
 let state:(u64,u64)=kani::any();
 let source=Rand32::from_state(state);
 assert_eq!(<Rand32 as Any>::type_id(&source),TypeId::of::<Rand32>());
 assert_ne!(<Rand32 as Any>::type_id(&source),TypeId::of::<Rand64>());
 assert_eq!(source.state(),state);
}

#[cfg(kani)]
#[kani::proof]
fn clone_slot_rand64() {
 let state:(u128,u128)=kani::any();
 let source=Rand64::from_state(state);
 let mut destination=MaybeUninit::<Rand64>::uninit();
 // Separate live source and writable aligned destination stack slots, exact Self size.
 unsafe { <Rand64 as CloneToUninit>::clone_to_uninit(&source,destination.as_mut_ptr().cast::<u8>()); }
 let copied=unsafe { destination.assume_init() };
 assert_eq!(source.state(),state);
 assert_eq!(copied.state(),state);
}
#[cfg(kani)]
#[kani::proof]
fn any_typeid_rand64() {
 let state:(u128,u128)=kani::any();
 let source=Rand64::from_state(state);
 assert_eq!(<Rand64 as Any>::type_id(&source),TypeId::of::<Rand64>());
 assert_ne!(<Rand64 as Any>::type_id(&source),TypeId::of::<Rand32>());
 assert_eq!(source.state(),state);
}
