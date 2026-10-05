#![allow(unexpected_cfgs, dead_code)]
#![recursion_limit = "512"]
extern crate alloc;
#[cfg(creusot)] use creusot_std::prelude::*;
#[path = "../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path = "../../../../src/ownership_proof/bound_ptr.rs"] mod bound_ptr;
#[path = "../../../../src/ownership_proof/frozen_region.rs"] mod frozen_region;
mod ownership_proof { pub(crate) use crate::{raw_vec, frozen_region, bound_ptr, sequential_counter, shared_protocol}; }
#[path = "../../../../src/ownership_proof/sequential_counter.rs"] mod sequential_counter;
#[path = "../../../../src/ownership_proof/shared_protocol.rs"] mod shared_protocol;
#[path = "../../../../src/capacity_ops.rs"] mod capacity_ops;
pub use actual::BytesMut;
mod bytes { pub(crate) use crate::actual::Vtable; }
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_frozen.rs")); }

// Generic physical metadata construction only: this contract supplies no
// atomic value, permission, refcount, ordering, or protocol postcondition.
#[cfg(creusot)]
extern_spec! {
    impl<T> core::sync::atomic::AtomicPtr<T> {
        fn new(value: *mut T) -> Self;
    }
}
