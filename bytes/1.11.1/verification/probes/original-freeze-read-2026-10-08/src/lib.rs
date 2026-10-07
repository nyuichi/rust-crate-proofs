#![recursion_limit="512"]
#![allow(unexpected_cfgs,dead_code,unused_imports,unused_variables,unused_unsafe,unused_mut)]
extern crate alloc;
#[cfg(creusot)] use creusot_std::prelude::*;
#[cfg(creusot)] #[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[cfg(creusot)] #[path="../../../../src/storage_ops.rs"] mod storage_ops;
#[cfg(creusot)] #[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[cfg(creusot)] #[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[cfg(creusot)] #[path="../../../../src/ownership_proof/boxed_alignment.rs"] mod boxed_alignment;
#[cfg(creusot)] mod ownership_proof { pub(crate) use crate::{raw_vec,boxed_alignment}; }
#[cfg(creusot)] pub struct TryGetError { requested:usize,available:usize }
#[cfg(creusot)] #[requires(false)] fn panic_advance(_e:&TryGetError)->! {panic!("excluded invalid advance")}
#[cfg(creusot)] mod bytes_mut { include!(concat!(env!("OUT_DIR"),"/bytes_mut.rs")); }
#[cfg(creusot)] mod bytes { include!(concat!(env!("OUT_DIR"),"/bytes.rs")); }
#[cfg(creusot)] pub use bytes_mut::BytesMut;
#[cfg(creusot)] pub use bytes::Bytes;
