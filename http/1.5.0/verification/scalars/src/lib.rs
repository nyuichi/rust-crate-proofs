#![cfg_attr(creusot, feature(nonzero_internals))]
#![allow(dead_code, unexpected_cfgs)]

// These modules are the upstream runtime implementations. This harness only
// narrows the translated crate so scalar proofs can be developed independently
// of the rest of http's unrelated pointer and type-erasure code.
#[cfg(feature = "status")]
#[path = "../../../src/status.rs"]
pub mod status;

#[path = "../../../src/version.rs"]
pub mod version;
