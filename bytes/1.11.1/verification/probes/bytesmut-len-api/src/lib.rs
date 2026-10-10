#![allow(dead_code, unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;

#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/view_region.rs"]
mod view_region;
mod ownership_proof {
    pub(crate) use crate::raw_vec;
    pub(crate) use crate::view_region;
}

include!(concat!(env!("OUT_DIR"), "/bytes_mut_api.rs"));
