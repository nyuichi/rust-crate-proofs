//! Restricted proof gate for actual Shared control allocation and explicit release.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/sequential_counter.rs"]
mod sequential_counter;
#[path = "../../../../src/ownership_proof/shared_protocol.rs"]
mod shared_protocol;
#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;

pub(crate) mod ownership_proof {
    pub(crate) use crate::{owned_region, raw_vec, sequential_counter, shared_protocol, bound_ptr, boxed_alignment};
}
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_split.rs")); }
use creusot_std::prelude::*;
#[requires(split@ <= input@.len())]
pub fn left_then_right(input: Vec<u8>, split: usize) {
    actual::BytesMut::proof_split_then_release(input, split, false);
}
#[requires(split@ <= input@.len())]
pub fn right_then_left(input: Vec<u8>, split: usize) {
    actual::BytesMut::proof_split_then_release(input, split, true);
}
