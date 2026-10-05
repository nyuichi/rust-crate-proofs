//! Restricted proof gate for actual unique-handle AsRef and AsMut implementations.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
#[cfg(not(creusot))]
#[path = "../../../../src/allocation_ops.rs"]
mod allocation_ops;
#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;
#[cfg(not(feature = "readonly-deref"))]
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[cfg(not(feature = "readonly-deref"))]
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/sequential_counter.rs"]
mod sequential_counter;
#[cfg(not(feature = "readonly-deref"))]
#[path = "../../../../src/ownership_proof/shared_protocol.rs"]
mod shared_protocol;
#[cfg(feature = "readonly-deref")]
include!(concat!(env!("OUT_DIR"), "/readonly_modules.rs"));
#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;

pub(crate) mod ownership_proof {
    pub(crate) use crate::{owned_region, raw_vec, sequential_counter, shared_protocol, bound_ptr, boxed_alignment};
}
#[path = "../../../../src/comparison_ops.rs"]
mod comparison_ops;
#[cfg(feature = "concrete-iterator")]
#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;
#[cfg(feature = "concrete-iterator")]
pub mod concrete_iterator { include!(concat!(env!("OUT_DIR"), "/actual_iterator.rs")); }
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_traits.rs")); }

pub fn compare_unique(left: Vec<u8>, right: Vec<u8>) -> (bool, core::cmp::Ordering) {
    actual::compare_unique(left, right)
}

#[cfg(feature = "str-adapters")]
mod str_contract;

#[cfg(feature = "str-adapters")]
pub fn compare_text_unique(left: Vec<u8>, right: &str) -> (bool, core::cmp::Ordering) {
    actual::compare_text_unique(left, right)
}
