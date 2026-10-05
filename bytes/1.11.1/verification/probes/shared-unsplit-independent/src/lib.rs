//! Exact actual two-split coordinator-carrier lifecycle; sequential explicit release only.
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
#[requires(a@.len()+b@.len()<=isize::MAX@)]
pub fn exercise_independent(a:Vec<u8>,b:Vec<u8>){actual::independent_caller(a,b)}
#[cfg(all(test,not(creusot)))]
mod independent_tests{
 #[test]fn two_singleton_controls(){for a in [0,1,4,9]{for b in [0,1,4,9]{for spare in [0,8]{
  let mut left=Vec::with_capacity(a+spare);left.extend((0..a).map(|i|(3*i)as u8));
  let mut right=Vec::with_capacity(b+spare);right.extend((0..b).map(|i|(7*i)as u8));
  super::exercise_independent(left,right);
 }}}}
}
