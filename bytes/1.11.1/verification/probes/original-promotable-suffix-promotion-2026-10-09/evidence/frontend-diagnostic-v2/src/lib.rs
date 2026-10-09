#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]
extern crate alloc;
use creusot_std::{prelude::*, ghost::{Perm, lifetime_logic::{Lifetime, LifetimeToken}},
    logic::{FMap, Id, Int, real::{PositiveReal, Real}, ra::{RA, excl::Excl}},
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Relaxed, Release, Acquire, None as NoStore}},
        committer::Committer, view::{SyncView, ReleaseSyncView, HasTimestamp}}};
mod event;
#[path = "../../original-public-shared-gate-2026-10-08/src/relaxed.rs"] mod relaxed;
#[path = "../../../../src/ref_count_limit.rs"] mod ref_count_limit;
#[path = "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs"] mod fraction_map;
#[path = "../../shared-physical-lifecycle-2026-10-08/src/release.rs"] mod release;
mod lifecycle;
#[path = "../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"] mod boxed_alignment;
mod provenance_specs;
#[path = "../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs"] mod pointer_event;
mod field_event;
mod physical_projection;
mod free_effect;
mod erased_call;
#[cfg(creusot)] mod public_shared;
#[cfg(not(creusot))]
mod loom {pub mod sync {pub mod atomic {pub use core::sync::atomic::*;}}}
#[cfg(not(creusot))]
#[path = "../../../../src/ref_count_ops.rs"] mod native_ref_count_ops;
#[cfg(not(creusot))] fn abort()->! {std::process::abort()}

#[cfg(creusot)] mod owned_pointer;
#[cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;
#[path = "../../original-boxed-automatic-drop-2026-10-09/src/tag_specs.rs"] mod promotion_tags;

#[cfg(creusot)] mod view_pointer;

#[cfg(creusot)] mod cursor_pointer;

#[cfg(creusot)] mod suffix_pointer;
