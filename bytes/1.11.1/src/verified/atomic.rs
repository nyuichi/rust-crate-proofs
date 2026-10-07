//! Generic native-atomic semantics TCB, not a bytes/refcount protocol.
//! ModelAtomic supplies the existing stock Perm/Committer/SyncView sorts.
#[cfg(not(feature = "extra-platforms"))]
use core::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "extra-platforms")]
use extra_platforms::{AtomicUsize, Ordering};
use creusot_std::{prelude::*, ghost::{FnGhost,Perm},logic::FMap,
    std::sync::{atomic::{AtomicUsize as ModelAtomic,ordering::{Acquire,Relaxed,Release,None as NoStore}},
        committer::Committer,view::{HasTimestamp,SyncView}}};

pub struct NativeAtomic { native:AtomicUsize }
impl NativeAtomic {
    #[logic(opaque)]
    pub fn model(self)->ModelAtomic { dead }

    #[trusted]
    #[ensures(*result.1.ward() == result.0.model())]
    #[ensures(**current <= ^current)]
    #[ensures(result.1.val() == FMap::singleton(result.0.model().get_timestamp(^current),(n,^current)))]
    pub fn new(n:usize,current:Ghost<&mut SyncView>)->(Self,Ghost<Perm<ModelAtomic>>) {
        (Self{native:AtomicUsize::new(n)},Ghost::conjure())
    }

    #[trusted]
    #[requires(forall<c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        !c.shot_store() ==> c.ward() == self.model() ==>
        (if c.val_load() == 0usize { c.val_store() == usize::MAX } else { c.val_store()@ + 1 == c.val_load()@ }) ==>
        f.precondition((c,)) && (f.postcondition_once((c,),()) ==> (^c).shot_store()))]
    #[ensures(exists<c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        !c.shot_store() && c.ward() == self.model() &&
        (if c.val_load() == 0usize { c.val_store() == usize::MAX } else { c.val_store()@ + 1 == c.val_load()@ }) && result == c.val_load() && f.postcondition_once((c,),()))]
    pub fn decrement<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut Committer<ModelAtomic,usize,Relaxed,Release>) {
        self.native.fetch_sub(1,Ordering::Release)
    }

    #[trusted]
    #[requires(forall<c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        !c.shot_store() ==> c.ward() == self.model() ==> f.precondition((c,)))]
    #[ensures(exists<c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        !c.shot_store() && c.ward() == self.model() && c.val_load() == result && f.postcondition_once((c,),()))]
    pub fn acquire<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&Committer<ModelAtomic,usize,Acquire,NoStore>) {
        self.native.load(Ordering::Acquire)
    }
}

/// RMW-specific completion: publication joins the previous modification's
/// carried view and this thread's release view, without acquiring the former.
/// The returned snapshot is publication metadata, never a current-view witness:
/// returning `SyncView` itself would allow recovery without an Acquire.
#[trusted]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
#[ensures((*c).hist_inv(^c) && (^c).shot_store())]
#[ensures((*own).ward() == (^own).ward())]
#[ensures(*current <= ^current && ^current <= *result)]
#[ensures(c.ward().get_timestamp(*current) <= c.timestamp())]
#[ensures(c.timestamp() < c.ward().get_timestamp(^current))]
#[ensures(match (*own).val().get(c.timestamp()) { Some((v,_)) => v == c.val_load(), None=>false })]
#[cfg_attr(not(feature="negative_drop_prior_publication"), ensures(match (*own).val().get(c.timestamp()) { Some((_,prior)) => prior <= *result, None=>false }))]
#[ensures(forall<t:Int> (*own).val().get(t) != None ==> t <= c.timestamp())]
#[ensures((^own).val() == (*own).val().insert(c.timestamp()+1,(c.val_store(),*result)))]
pub fn release_rmw(c:&mut Committer<ModelAtomic,usize,Relaxed,Release>,own:&mut Perm<ModelAtomic>,current:&mut SyncView)->Snapshot<SyncView> {
    panic!("ghost atomic semantics rule")
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;

    #[test]
    fn two_native_release_rmws_have_one_last_observer() {
        for _ in 0..64 {
            let atomic = NativeAtomic { native: AtomicUsize::new(2) };
            let (left, right) = std::thread::scope(|scope| {
                let left = scope.spawn(|| atomic.decrement(ghost! {
                    |_: &mut Committer<ModelAtomic, usize, Relaxed, Release>| {}
                }));
                let right = scope.spawn(|| atomic.decrement(ghost! {
                    |_: &mut Committer<ModelAtomic, usize, Relaxed, Release>| {}
                }));
                (left.join().unwrap(), right.join().unwrap())
            });
            assert!((left == 2 && right == 1) || (left == 1 && right == 2));
            assert_eq!(atomic.acquire(ghost! {
                |_: &Committer<ModelAtomic, usize, Acquire, NoStore>| {}
            }), 0);
        }
    }
}

// The core backend continues to use the exact stock primitive.
#[cfg(not(feature = "extra-platforms"))]
pub use creusot_std::std::sync::atomic::fence_acquire;

/// Generic portable-atomic Acquire fence boundary, with the exact stock view
/// contract. portable-atomic supplies the platform-specific native fence;
/// no bytes ownership, retirement, or last-owner fact is assumed here.
#[cfg(feature = "extra-platforms")]
#[trusted]
#[ensures(acq_view@ == *result)]
#[allow(unused_variables)]
pub fn fence_acquire(acq_view: Ghost<creusot_std::std::sync::view::AcquireSyncView>) -> Ghost<SyncView> {
    extra_platforms::fence(Ordering::Acquire);
    Ghost::conjure()
}
