//! Compile-fail fixtures for Rust move behavior of the B1/B2 capability types.
#![allow(unexpected_cfgs)]
extern crate alloc;

#[path = "../../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;

#[cfg(feature = "duplicate_descriptor")]
mod duplicate_descriptor;
#[cfg(feature = "use_after_transfer")]
mod use_after_transfer;
