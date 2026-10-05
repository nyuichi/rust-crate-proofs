//! Restricted proof gate for actual unique-handle AsRef and AsMut implementations.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
#[cfg(not(creusot))]
#[path = "../../../../src/allocation_ops.rs"]
mod allocation_ops;
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
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_traits.rs")); }

pub fn traits_unique(input: Vec<u8>, value: u8) {
    actual::BytesMut::proof_traits_unique(input, value);
}

// The build script relocates this proof-caller template into the generated
// `actual` module. Keeping it disabled here ensures the caller is translated
// in the same module as the private-field method contracts it consumes.
#[cfg(any())]
fn proof_observe_is_empty(owner: &actual::BytesMut) {
    assert!(owner.is_empty() == (owner.len() == 0));
}

#[cfg(not(creusot))]
pub fn unique_is_empty(input: Vec<u8>) -> bool {
    let owner = actual::BytesMut::from_vec(input);
    let result = owner.is_empty();
    owner.proof_release_unique_at_zero();
    result
}
