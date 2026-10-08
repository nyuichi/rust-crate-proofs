#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]

use creusot_std::{
    ghost::{Perm, lifetime_logic::{Lifetime, LifetimeToken}},
    logic::{FMap, Id, Int, real::{PositiveReal, Real}, ra::{RA, excl::Excl}},
    prelude::*,
    std::sync::{
        atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire, None as NoStore, Relaxed, Release}},
        committer::Committer,
        view::{HasTimestamp, ReleaseSyncView, SyncView},
    },
};

#[path = "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs"]
mod fraction_map;
#[path = "../../../../src/ref_count_limit.rs"]
mod ref_count_limit;
#[path = "../../original-public-shared-gate-2026-10-08/src/event.rs"]
mod event;
#[path = "../../original-public-shared-gate-2026-10-08/src/relaxed.rs"]
mod relaxed;
#[path = "../../shared-physical-lifecycle-2026-10-08/src/release.rs"]
mod release;
mod lifecycle;
#[cfg(feature = "driver")]
mod driver;

pub use lifecycle::{Registry, RecoveryPayload, Ticket};
#[cfg(feature = "driver")]
pub use driver::three_clones_sparse_hole_mixed_retirement;

