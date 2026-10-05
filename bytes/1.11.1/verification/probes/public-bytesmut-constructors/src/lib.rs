//! Exact-source probe for the public zeroed and slice-copy BytesMut constructors.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;

use creusot_std::prelude::*;

#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/sequential_counter.rs"]
mod sequential_counter;
#[path = "../../../../src/ownership_proof/shared_protocol.rs"]
mod shared_protocol;

mod ownership_proof {
    pub(crate) use crate::{bound_ptr, owned_region, raw_vec, sequential_counter, shared_protocol};
}

#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

mod actual_public_constructors;
