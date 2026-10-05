#![allow(unexpected_cfgs,unused_variables,dead_code)]
use creusot_std::{prelude::*,ghost::Perm,
    std::sync::{atomic::{AtomicUsize as ModelAtomic,ordering::{Acquire,Relaxed,Release,None as NoStore}},
        committer::Committer,view::{AtView,HasTimestamp,SyncView}}};
mod primitive;
use primitive::NativeAtomic;

/// Restricted physical publication slice: resources are moved before native
/// release, remain sealed, and are accessible only after matching acquire.
/// The exclusive permission here does not yet claim concurrent access.
#[requires(*own.ward() == atomic.model())]
#[requires(own.val().get(latest.inner_logic()) == Some((2usize,*current)))]
#[requires(forall<t:Int> own.val().get(t) != None ==> t <= latest.inner_logic())]
pub fn two_retirements<T>(atomic:&NativeAtomic,mut own:Ghost<Perm<ModelAtomic>>,
    mut latest:Ghost<Int>,mut current:Ghost<SyncView>,first:Ghost<T>,second:Ghost<T>)->Ghost<(T,T)> {
    let first=AtView::new(first);
    let (mut first_view,first)=first.split();
    let old=atomic.decrement(ghost! {|c:&mut Committer<ModelAtomic,usize,Relaxed,Release>| {
        let publication=primitive::release_rmw(c,&mut own,&mut first_view);
        proof_assert!(c.timestamp() == *latest);
        *latest=*snapshot!(c.timestamp()+1);
        proof_assert!(first.view() <= publication);
    }});
    proof_assert!(old == 2usize);
    let second=AtView::new(second);
    let (mut second_view,second)=second.split();
    let old=atomic.decrement(ghost! {|c:&mut Committer<ModelAtomic,usize,Relaxed,Release>| {
        let publication=primitive::release_rmw(c,&mut own,&mut second_view);
        proof_assert!(c.timestamp() == *latest);
        *latest=*snapshot!(c.timestamp()+1);
        proof_assert!(first.view() <= publication && second.view() <= publication);
    }});
    proof_assert!(old == 1usize);
    #[cfg(not(feature="negative_no_acquire"))]
    let zero=atomic.acquire(ghost! {|c:&Committer<ModelAtomic,usize,Acquire,NoStore>| {
        c.shoot_load(&own,&mut second_view);
        proof_assert!(c.timestamp() == *latest);
    }});
    ghost! { (first.into_inner().sync(*second_view),second.into_inner().sync(*second_view)) }
}
mod concurrent;
