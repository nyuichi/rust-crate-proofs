//! Concrete slice specialization of the exact IntoIter bodies and core Iterator refinement.
#![allow(unexpected_cfgs)]
#[path = "../../../../../src/slice_ops.rs"]
mod slice_ops;
include!(concat!(env!("OUT_DIR"), "/actual_iterator.rs"));
