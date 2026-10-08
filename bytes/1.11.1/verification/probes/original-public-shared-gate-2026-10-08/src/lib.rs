#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit = "512"]
mod event;
mod relaxed;
mod model_cas;
#[path = "../../../../src/ref_count_limit.rs"]
mod ref_count_limit;
use creusot_std::{prelude::*, ghost::Perm,
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::Relaxed},
        committer::Committer, view::{SyncView, ReleaseSyncView}}};

/// Generic publication preservation client, not a bytes protocol theorem.
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
#[requires(match own.val().get(c.timestamp()) {Some((_,prior)) => *payload_view <= prior, None => false})]
#[ensures(match (^own).val().get(c.timestamp()+1) {Some((_,publication)) => *payload_view <= publication, None => false})]
fn retain_payload_publication(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,
    own:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView,payload_view:Snapshot<SyncView>) {
    let publication = relaxed::relaxed_rmw(c, own, current, release);
}

#[cfg(feature="negative_relaxed_acquire")]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
#[requires(match own.val().get(c.timestamp()) {Some((_,prior)) => *payload_view <= prior, None => false})]
#[ensures(*payload_view <= ^current)]
fn cannot_acquire_with_relaxed(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,
    own:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView,payload_view:Snapshot<SyncView>) {
    let publication = relaxed::relaxed_rmw(c, own, current, release);
}

#[cfg(feature="negative_relaxed_publish_current")]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
#[ensures(*current <= *result)]
fn cannot_publish_current_with_relaxed(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,
    own:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView)->Snapshot<SyncView> {
    relaxed::relaxed_rmw(c, own, current, release)
}

#[cfg(feature="negative_double_commit")]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
fn cannot_commit_twice(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,
    own:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView) {
    let _ = relaxed::relaxed_rmw(c, own, current, release);
    let _ = relaxed::relaxed_rmw(c, own, current, release);
}

#[cfg(feature="negative_wrong_field")]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() != *other.ward())]
fn cannot_use_wrong_field(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,
    other:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView) {
    let _ = relaxed::relaxed_rmw(c, other, current, release);
}

#[cfg(feature="negative_ticket_on_refusal")]
#[requires(*own.ward() == *at)]
fn cannot_issue_on_refusal(at:&ModelAtomic,own:Ghost<Perm<ModelAtomic>>) {
    let mut own = own;
    let mut current = ghost! {SyncView::new().into_inner()};
    let release = ghost! {ReleaseSyncView::new().into_inner()};
    let mut issued = ghost! {false};
    let result = model_cas::guarded_increment(at,ghost! {|c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
        let _ = relaxed::relaxed_rmw(c,&mut own,&mut current,release.into_inner());
        *issued = true;
    }});
    if let Err(_) = result {
        proof_assert!(*issued);
    }
}

#[cfg(feature="public_shared")]
extern crate alloc;
#[cfg(feature="public_shared")]
use creusot_std::{ghost::lifetime_logic::{Lifetime,LifetimeToken},
    logic::{FMap,Id,real::{PositiveReal,Real},ra::{RA,excl::Excl}},
    std::sync::{atomic::ordering::{Acquire,Release,None as NoStore},view::HasTimestamp}};
#[cfg(feature="public_shared")]
#[path="../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs"] mod fraction_map;
#[cfg(feature="public_shared")]
#[path="../../shared-physical-lifecycle-2026-10-08/src/release.rs"] mod release;
#[cfg(feature="public_shared")]
#[path="../../guarded-shared-protocol-2026-10-08/src/lifecycle.rs"] mod lifecycle;
#[cfg(feature="public_shared")]
#[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[cfg(feature="public_shared")]
#[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[cfg(feature="public_shared")]
#[path="../../../../src/ownership_proof/boxed_alignment.rs"] mod boxed_alignment;
#[cfg(feature="public_shared")]
#[path="../../original-shared-lifecycle-2026-10-08/src/provenance_specs.rs"] mod provenance_specs;
#[cfg(feature="public_shared")]
#[path="../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs"] mod pointer_event;
#[cfg(feature="public_shared")]
mod field_event;
#[cfg(feature="public_shared")]
mod physical_projection;
#[cfg(all(feature="public_shared",not(creusot)))]
mod loom {pub mod sync {pub mod atomic {pub use core::sync::atomic::*;}}}
#[cfg(all(feature="public_shared",not(creusot)))]
#[path="../../../../src/ref_count_ops.rs"] mod native_ref_count_ops;
#[cfg(all(feature="public_shared",not(creusot)))]
fn abort()->! {std::process::abort()}
#[cfg(feature="public_shared")]
mod erased_call;
#[cfg(all(feature="public_shared",creusot))]
mod public_shared;
