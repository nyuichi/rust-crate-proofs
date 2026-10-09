#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]

#[cfg(feature = "public_constructor")]
mod relaxed;
#[cfg(feature = "public_constructor")]
#[path = "../../../../src/ref_count_limit.rs"]
mod ref_count_limit;

#[cfg(feature = "public_constructor")]
use creusot_std::prelude::*;
#[cfg(feature = "public_constructor")]
use creusot_std::{
    ghost::Perm,
    std::sync::{
        atomic::{AtomicUsize as ModelAtomic, ordering::Relaxed},
        committer::Committer,
        view::{ReleaseSyncView, SyncView},
    },
};
#[cfg(feature = "public_constructor")]
use creusot_std::{
    ghost::lifetime_logic::{Lifetime, LifetimeToken},
    logic::{FMap, Id, ra::{RA, excl::Excl}, real::PositiveReal},
    std::sync::{
        atomic::ordering::{Acquire, None as NoStore, Release},
        view::HasTimestamp,
    },
};

#[cfg(any(feature = "public_constructor", feature = "native_b1box"))]
extern crate alloc;

#[cfg(any(feature = "public_constructor", feature = "native_b1box"))]
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[cfg(any(feature = "public_constructor", feature = "native_b1box"))]
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;

#[cfg(all(feature = "public_constructor", creusot))]
mod event;
#[cfg(all(feature = "public_constructor", creusot))]
mod field_event;
#[cfg(all(feature = "public_constructor", creusot))]
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;
#[cfg(any(feature = "public_constructor", feature = "native_b1box"))]
mod provenance_specs;
#[cfg(all(feature = "public_constructor", creusot))]
#[path = "../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs"]
mod pointer_event;
#[cfg(all(feature = "public_constructor", creusot))]
#[path = "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs"]
mod fraction_map;
#[cfg(all(feature = "public_constructor", creusot))]
#[path = "../../shared-physical-lifecycle-2026-10-08/src/release.rs"]
mod release;
#[cfg(all(feature = "public_constructor", creusot))]
#[path = "../../guarded-shared-protocol-2026-10-08/src/lifecycle.rs"]
mod lifecycle;
#[cfg(all(feature = "public_constructor", creusot))]
mod read_projection;
#[cfg(all(feature = "public_constructor", creusot))]
mod public_constructor;

#[cfg(feature = "native_b1box")]
mod native_b1box;

#[cfg(all(feature="public_constructor",creusot))] mod physical_projection;
#[cfg(all(feature="public_constructor",creusot))] mod tag_specs;
#[cfg(all(feature="public_constructor",creusot))] mod erased_call;
