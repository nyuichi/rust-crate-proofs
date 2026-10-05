//! Exact actual two-split coordinator-carrier lifecycle; sequential explicit release only.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
use creusot_std::prelude::*;
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
#[path = "../../../../src/ownership_proof/scalable_tickets.rs"]
mod scalable_tickets;
#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;
pub(crate) mod ownership_proof {
    pub(crate) use crate::{owned_region, raw_vec, sequential_counter, scalable_tickets, bound_ptr, boxed_alignment};
    pub(crate) use crate::actual::sequential_shared_control as shared_protocol;
}
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_carrier.rs")); }
#[requires(false)]
fn abort() -> ! { std::process::abort() }
pub fn carrier_split(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
    actual::BytesMut::proof_carrier_split(input, first, second, order, value);
}

#[cfg(feature = "negative_carrier_missing_ticket")]
pub fn missing_empty_ticket() {
    actual::BytesMut::proof_carrier_missing_empty_ticket();
}
