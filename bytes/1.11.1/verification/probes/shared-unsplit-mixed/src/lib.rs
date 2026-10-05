//! Exact public mixed and Unique unsplit cfg bodies; sequential explicit cleanup.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
#[cfg(not(creusot))]
#[path = "../../../../src/allocation_ops.rs"]
mod allocation_ops;
use creusot_std::prelude::*;
#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;
#[path = "../../../../src/storage_ops.rs"]
mod storage_ops;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/sequential_counter.rs"]
mod sequential_counter;
include!(concat!(env!("OUT_DIR"), "/protocol_modules.rs"));
#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;
#[path = "../../../../src/ownership_proof/vec_capacity.rs"]
mod vec_capacity;
#[path = "../../../../src/ownership_proof/unique_reclaim.rs"]
mod unique_reclaim;
pub(crate) mod ownership_proof {
    pub(crate) use crate::{owned_region, raw_vec, sequential_counter, scalable_tickets, bound_ptr, boxed_alignment, vec_capacity, unique_reclaim};
    pub(crate) use crate::actual::sequential_shared_control as shared_protocol;
}
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_carrier.rs")); }
#[requires(false)]
fn abort() -> ! { std::process::abort() }
#[cfg(not(feature="unique_pair"))]
#[requires(a@.len()+b@.len()<=isize::MAX@)]
pub fn exercise_mixed(a:Vec<u8>,b:Vec<u8>,unique_left:bool,advance:usize,canonical:bool){actual::mixed_caller(a,b,unique_left,advance,canonical)}
#[cfg(all(test,not(creusot),not(feature="unique_pair")))]
mod mixed_tests{
 #[test]fn both_mixed_orders(){for a in [0,1,4,9]{for b in [0,1,4,9]{for spare in [0,8]{for unique_left in [false,true]{for advance in [0,1,usize::MAX]{for canonical in [false,true]{
  let mut left=Vec::with_capacity(a+spare);left.extend((0..a).map(|i|(3*i)as u8));
  let mut right=Vec::with_capacity(b+spare);right.extend((0..b).map(|i|(7*i)as u8));
  super::exercise_mixed(left,right,unique_left,advance,canonical);
 }}}}}}}
}

#[cfg(feature="unique_pair")]
#[requires(a@.len()+b@.len()<=isize::MAX@)]
pub fn exercise_unique(a:Vec<u8>,b:Vec<u8>,advance:usize,left_empty:bool,right_empty:bool){actual::unique_caller(a,b,advance,left_empty,right_empty)}
#[cfg(all(test,not(creusot),feature="unique_pair"))]
mod unique_tests{
 #[test]fn both_unique_offsets_and_empties(){for a in [0,1,4,9]{for b in [0,1,4,9]{for spare in [0,8]{for advance in [0,1,usize::MAX]{for left_empty in [false,true]{for right_empty in [false,true]{
  let mut left=Vec::with_capacity(a+spare);left.extend((0..a).map(|i|(3*i)as u8));
  let mut right=Vec::with_capacity(b+spare);right.extend((0..b).map(|i|(7*i)as u8));
  super::exercise_unique(left,right,advance,left_empty,right_empty);
 }}}}}}}
}
