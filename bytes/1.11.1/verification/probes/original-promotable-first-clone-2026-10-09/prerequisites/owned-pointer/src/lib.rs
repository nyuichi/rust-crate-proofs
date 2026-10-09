#![allow(unexpected_cfgs, unused_variables, dead_code)]
#[path = "../../../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs"]
mod pointer_event;
#[path = "../../../src/owned_pointer.rs"]
mod owned_pointer;
pub use owned_pointer::first_strong_cas;

#[cfg(any(feature="wrong_expected",feature="missing_store",feature="stale_readonly"))] mod controls;
