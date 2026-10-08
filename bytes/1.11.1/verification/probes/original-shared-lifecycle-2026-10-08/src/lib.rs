#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]

extern crate alloc;

pub mod field_event;

#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;

mod source_adapter;
mod pointer_event;
