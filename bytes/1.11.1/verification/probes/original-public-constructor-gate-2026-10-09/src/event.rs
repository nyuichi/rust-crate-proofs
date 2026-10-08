//! NEW GENERIC TCB: operation-bound atomic invariant; no ghost opening API.
use core::{marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};
use creusot_std::{prelude::*, ghost::{FnGhost, Perm, invariant::Protocol},
    logic::FMap, std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Relaxed, Release, Acquire, None as NoStore}},
        committer::Committer, view::{HasTimestamp,SyncView}}};
#[cfg(creusot)] use creusot_std::ghost::Objective;

pub trait EventProtocol: Protocol {
    #[logic] fn atomic(self) -> ModelAtomic;
}

/// Unbound actual atomic; bind consumes this value and the state's actual Perm.
pub struct RawAtomic { native: AtomicUsize }
impl RawAtomic {
    #[logic(opaque)] pub fn model(self) -> ModelAtomic { dead }
    #[trusted]
    #[ensures(*result.1.ward() == result.0.model())]
    #[ensures(**current <= ^current)]
    #[ensures(result.1.val() == FMap::singleton(result.0.model().get_timestamp(^current), (value,^current)))]
    pub fn new(value:usize,current:Ghost<&mut SyncView>)->(Self,Ghost<Perm<ModelAtomic>>) {
        (Self {native:AtomicUsize::new(value)},Ghost::conjure())
    }
}

/// Its private protocol cannot be reached through Tokens or stock invariants.
#[opaque]
pub struct EventAtomic<S:EventProtocol> { atomic:RawAtomic, state:PhantomData<S> }
#[cfg(creusot)]
#[trusted]
unsafe impl<S:EventProtocol+Send+Objective> Sync for EventAtomic<S> {}
#[cfg(not(creusot))]
unsafe impl<S:EventProtocol+Send> Sync for EventAtomic<S> {}
#[trusted]
unsafe impl<S:EventProtocol+Send> Send for EventAtomic<S> {}

impl<S:EventProtocol> EventAtomic<S> {
    #[logic(opaque)] pub fn model(self)->ModelAtomic { dead }
    #[logic(opaque)] pub fn public(self)->S::Public { dead }

    /// Consumes the protocol once; never copies its affine contents.
    #[trusted]
    #[requires(state.protocol() && state.atomic() == atomic.model())]
    #[ensures(result.model() == atomic.model() && result.public() == state.public())]
    pub fn bind(atomic:RawAtomic,state:Ghost<S>)->Self {
        Self {atomic,state:PhantomData}
    }

    /// One success commits exactly once at the actual Relaxed CAS. The update
    /// closure passed to fetch_update contains only pure guard/arithmetic.
    /// Failed/spurious retries do not invoke f; refusal is a Relaxed read and
    /// makes no store BY THIS OPERATION. Other threads may still modify state.
    /// Err carries no callback postcondition and cannot create a registration.
    #[trusted]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(match result {
        Ok(old) => exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 &&
        old == c.val_load() && f.postcondition_once((s,c),()),
        Err(observed) => observed > crate::ref_count_limit::MAX_REF_COUNT &&
            exists<c:&Committer<ModelAtomic,usize,Relaxed,NoStore>>
                !c.shot_store() && c.ward() == self.model() && c.val_load() == observed
    })]
    pub fn try_increment<F>(&self,f:Ghost<F>)->Result<usize,usize>
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>) {
        self.atomic.native.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
            crate::ref_count_limit::next_ref_count)
    }

    /// Normal-return guarded increment; refusal follows the native abort path.
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 &&
        result == c.val_load() && f.postcondition_once((s,c),()))]
    pub fn increment<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>) {
        match self.try_increment(f) {
            Ok(old) => old,
            Err(_) => { abort_overflow(); 0 }
        }
    }

    #[trusted]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() && (if c.val_load() == 0usize { c.val_store() == usize::MAX } else { c.val_store()@ + 1 == c.val_load()@ }) ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() && (if c.val_load() == 0usize { c.val_store() == usize::MAX } else { c.val_store()@ + 1 == c.val_load()@ }) &&
        result == c.val_load() && f.postcondition_once((s,c),()))]
    pub fn decrement<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Release>) {
        self.atomic.native.fetch_sub(1, Ordering::Release)
    }

    #[trusted]
    #[requires(forall<s:&mut S,c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() && true ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model()))]
    #[ensures(exists<s:&mut S,c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() && true &&
        result == c.val_load() && f.postcondition_once((s,c),()))]
    pub fn acquire<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&Committer<ModelAtomic,usize,Acquire,NoStore>) {
        self.atomic.native.load(Ordering::Acquire)
    }
}

#[trusted]
#[ensures(false)]
#[cold]
fn abort_overflow() { std::process::abort() }
