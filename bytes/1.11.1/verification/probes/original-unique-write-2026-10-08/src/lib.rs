#![recursion_limit="512"]
#![allow(unexpected_cfgs,dead_code,unused_imports,unused_unsafe)]
extern crate alloc;
use creusot_std::prelude::*;
#[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path="../../../../src/storage_ops.rs"] mod storage_ops;
#[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
mod ownership_proof { pub(crate) use crate::raw_vec; }
pub struct TryGetError { requested:usize,available:usize }
#[requires(false)]
fn panic_advance(_error:&TryGetError)->! { panic!("excluded invalid advance") }
mod actual { include!(concat!(env!("OUT_DIR"),"/actual.rs")); }
