//! Exact-source gate for `BytesMut::with_capacity` and its B1 cleanup path.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]

extern crate alloc;

#[cfg(not(creusot))]
#[path = "../../../../src/allocation_ops.rs"]
mod allocation_ops;
#[path = "../../../../src/storage_ops.rs"]
mod storage_ops;
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
#[path = "../../../../src/ownership_proof/vec_capacity.rs"]
mod vec_capacity;

pub(crate) mod ownership_proof {
    pub(crate) use crate::{
        bound_ptr, boxed_alignment, owned_region, raw_vec, sequential_counter, shared_protocol,
        vec_capacity,
    };
}

mod actual {
    include!(concat!(env!("OUT_DIR"), "/actual_with_capacity.rs"));

    impl BytesMut {
        /// Construct through the exact source `with_capacity` body, check its
        /// empty Unknown allocation model, then consume the full B1 authority.
        #[cfg(creusot)]
        #[ensures(result.0@ == 0)]
        #[ensures(result.1@ >= requested@)]
        pub(crate) fn capacity_then_release(requested: usize) -> (usize, usize) {
            let mut owner = Self::with_capacity(requested);
            let len = owner.len();
            let capacity = owner.capacity();
            proof_assert!(owner.proof_unique_at_zero_valid());
            proof_assert!(forall<index: Int> 0 <= index && index < capacity@ ==>
                owner.proof_unique_slot(index) == Some(None));
            owner.proof_release_unique();
            (len, capacity)
        }

        /// Native execution of the exact constructor and explicit release.
        #[cfg(not(creusot))]
        pub(crate) fn native_capacity_then_release(requested: usize) -> (usize, usize) {
            let mut owner = Self::with_capacity(requested);
            let len = owner.len();
            let capacity = owner.capacity();
            owner.proof_release_unique();
            (len, capacity)
        }
    }
}

use creusot_std::prelude::*;

/// Prove the exact extracted runtime constructor's length and requested-capacity
/// guarantee through field construction and explicit B1 cleanup.
#[cfg(creusot)]
#[ensures(result.0@ == 0)]
#[ensures(result.1@ >= requested@)]
pub fn actual_with_capacity_then_release(requested: usize) -> (usize, usize) {
    actual::BytesMut::capacity_then_release(requested)
}

/// The extracted constructor does not promise capacity for a different,
/// strictly larger request.
#[cfg(all(creusot, feature = "wrong_requested_capacity"))]
#[requires(requested@ < larger_request@)]
#[ensures(result.1@ >= larger_request@)]
pub fn wrong_requested_capacity(requested: usize, larger_request: usize) -> (usize, usize) {
    actual::BytesMut::capacity_then_release(requested)
}

/// Run the exact extracted runtime constructor and explicit release natively.
#[cfg(not(creusot))]
pub fn native_actual_with_capacity_then_release(requested: usize) -> (usize, usize) {
    actual::BytesMut::native_capacity_then_release(requested)
}
