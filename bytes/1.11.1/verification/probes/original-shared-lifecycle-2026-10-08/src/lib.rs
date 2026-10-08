#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]

extern crate alloc;

use creusot_std::{ghost::invariant::Protocol, prelude::*};

pub mod field_event;

pub(crate) use bytes_shared_physical_lifecycle::bounded as bounded;

impl<T: bounded::RecoveryPayload> field_event::EventProtocol for bounded::State<T> {
    #[logic]
    fn atomic(self) -> creusot_std::std::sync::atomic::AtomicUsize {
        self.public().0
    }
}

mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;

mod source_adapter;
mod pointer_event;
