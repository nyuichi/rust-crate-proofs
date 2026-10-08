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
