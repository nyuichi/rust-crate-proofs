//! Isolated immutable B4 classification candidate. Mutable B4 remains ordinary.
#![allow(unexpected_cfgs,dead_code,unused_variables)]
#![recursion_limit="512"]
extern crate alloc;
use creusot_std::prelude::*;
#[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path="../../../../src/ownership_proof/bound_ptr.rs"] mod bound_ptr;
include!(concat!(env!("OUT_DIR"),"/raw_module.rs"));
use raw_vec::{BoundPtr,PhysicalRegion};
struct Reader<'a>{bound:&'a BoundPtr,len:usize,region:Ghost<&'a PhysicalRegion>}
impl Invariant for Reader<'_>{
 #[logic(prophetic)]
 fn invariant(self)->bool { pearlite!{
  self.bound.invariant() && self.bound@ != None && self.region.inner_logic().invariant() &&
  self.bound@.unwrap_logic().0 == self.region.inner_logic().namespace() &&
  self.bound@.unwrap_logic().1 == self.region.inner_logic().capacity() &&
  self.region.inner_logic().lo() <= self.bound@.unwrap_logic().2 &&
  self.bound@.unwrap_logic().2+self.len@ <= self.region.inner_logic().hi() &&
  forall<i:Int> 0<=i && i<self.len@ ==> raw_vec::slot_known(self.region.inner_logic().slot(self.bound@.unwrap_logic().2+i))
 } }
}
impl core::ops::Deref for Reader<'_>{
 type Target=[u8];
 #[check(ghost)]
 #[ensures(result@.len()==self.len@)]
 #[ensures(forall<i:Int> 0<=i && i<self.len@ ==> self.region.inner_logic().slot(self.bound@.unwrap_logic().2+i)==Some(Some(result@[i])))]
 fn deref(&self)->&[u8]{unsafe{raw_vec::borrow_bound(self.bound,self.len,self.region)}}
}
#[requires(input@.len()>0)]
#[ensures(result == (first,second))]
pub fn read_after_real_writes(input:Vec<u8>,first:u8,second:u8)->(u8,u8){
 let (base,len,capacity,caps)=bound_ptr::detach_bound_vec(input);
 let (recovery,mut region)=caps.split();
 {
  let slice=unsafe{raw_vec::borrow_bound_mut(&base,len,region.borrow_mut())};
  slice[0]=first;
 }
 let first_read={
  let reader=Reader{bound:&base,len,region:region.borrow()};
  let ghost_read=ghost! { reader[0] };
  let native_read=reader[0];
  proof_assert!(ghost_read.inner_logic()==native_read && native_read==first);
  native_read
 };
 {
  let slice=unsafe{raw_vec::borrow_bound_mut(&base,len,region.borrow_mut())};
  slice[0]=second;
 }
 let second_read={
  let reader=Reader{bound:&base,len,region:region.borrow()};
  let ghost_read=ghost! { reader[0] };
  let native_read=reader[0];
  proof_assert!(ghost_read.inner_logic()==native_read && native_read==second);
  native_read
 };
 unsafe{raw_vec::deallocate_bound_vec(base,capacity,ghost!{(recovery.into_inner(),region.into_inner())});}
 (first_read,second_read)
}
#[cfg(all(creusot,feature="negative_ghost_write"))]
#[requires(input@.len()>0)]
pub fn forbidden_ghost_write(input:Vec<u8>){
 let (base,len,capacity,caps)=bound_ptr::detach_bound_vec(input);
 let (recovery,mut region)=caps.split();
 ghost!{let slice=unsafe{raw_vec::borrow_bound_mut(&base,len,region.borrow_mut())};slice[0]=99;};
}
#[cfg(all(creusot,feature="negative_stale_read"))]
#[requires(input@.len()>0 && first!=second)]
pub fn forbidden_stale_read(input:Vec<u8>,first:u8,second:u8){
 let (_,observed)=read_after_real_writes(input,first,second);
 proof_assert!(observed==first);
}
#[cfg(all(test,not(creusot)))]
mod tests{
 #[test] fn native_reads_follow_native_writes(){for first in [0,17,255]{for second in [0,18,254]{for cap in [1,8]{let mut v=Vec::with_capacity(cap);v.push(9);assert_eq!(super::read_after_real_writes(v,first,second),(first,second));}}}}
}
