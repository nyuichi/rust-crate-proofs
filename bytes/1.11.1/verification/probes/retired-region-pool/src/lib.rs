//! Explicit physical retirement; no actual Shared, atomics, or Drop proof.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
use alloc::vec::Vec;
use creusot_std::prelude::*;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
use raw_vec::{detach_vec, deallocate_vec, PhysicalPool};

#[requires(split@ <= input@.len())]
pub fn retire_left_then_right(input: Vec<u8>, split: usize) {
    let (raw, _len, caps) = detach_vec(input);
    let full = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let boundary: Int = *Int::new(split as i128);
        let (left, right) = region.split_at(boundary);
        let pool = pool.retire(left);
        let pool = pool.retire(right);
        (recovery, pool.finish(0int))
    };
    unsafe { deallocate_vec(raw, full); }
}

#[requires(split@ <= input@.len())]
pub fn retire_right_then_left(input: Vec<u8>, split: usize) {
    let (raw, _len, caps) = detach_vec(input);
    let full = ghost! {
        let (recovery, region) = caps.into_inner();
        let pool = PhysicalPool::empty_from(&region);
        let boundary: Int = *Int::new(split as i128);
        let (left, right) = region.split_at(boundary);
        let pool = pool.retire(right);
        let pool = pool.retire(left);
        (recovery, pool.finish(0int))
    };
    unsafe { deallocate_vec(raw, full); }
}
