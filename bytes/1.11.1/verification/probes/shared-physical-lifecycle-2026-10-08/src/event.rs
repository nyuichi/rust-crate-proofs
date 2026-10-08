//! NEW GENERIC TCB: operation-bound atomic invariant; no ghost opening API.
use core::{marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};
use creusot_std::{prelude::*, ghost::{FnGhost, Perm, invariant::Protocol},
    logic::FMap, std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Relaxed,Release,Acquire,None as NoStore}},
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

    /// NEW rule. Native RMW is the only entry; no ghost-callable opener exists.
    /// The matching Perm and all protocol updates must be supplied by the caller
    /// inside the checked callback. Relaxed has no Acquire postcondition.
    #[trusted]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        (if c.val_load() == usize::MAX {c.val_store() == 0usize} else {c.val_store()@ == c.val_load()@ + 1}) ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        (if c.val_load() == usize::MAX {c.val_store() == 0usize} else {c.val_store()@ == c.val_load()@ + 1}) &&
        result == c.val_load() && f.postcondition_once((s,c),()))]
    pub fn increment<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>) {
        self.atomic.native.fetch_add(1,Ordering::Relaxed)
    }    /// NEW rule. Native RMW is the only entry; no ghost-callable opener exists.
    /// The matching Perm and all protocol updates must be supplied by the caller
    /// inside the checked callback. Relaxed has no Acquire postcondition.
    #[trusted]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        (if c.val_load() == 0usize {c.val_store() == usize::MAX} else {c.val_store()@ + 1 == c.val_load()@}) ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        !c.shot_store() && c.ward() == self.model() &&
        (if c.val_load() == 0usize {c.val_store() == usize::MAX} else {c.val_store()@ + 1 == c.val_load()@}) &&
        result == c.val_load() && f.postcondition_once((s,c),()))]
    pub fn decrement<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Release>) {
        self.atomic.native.fetch_sub(1,Ordering::Release)
    }
    /// Native Acquire load; opens the same encapsulated state only at this event.
    #[trusted]
    #[requires(forall<s:&mut S,c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        s.protocol() && s.public()==self.public() && s.atomic()==self.model() && inv(s) &&
        !c.shot_store() && c.ward()==self.model() ==>
        f.precondition((s,c)) && (f.postcondition_once((s,c),()) ==>
            (^s).protocol() && (^s).public()==self.public() && (^s).atomic()==self.model()))]
    #[ensures(exists<s:&mut S,c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        s.protocol() && s.public()==self.public() && s.atomic()==self.model() && inv(s) &&
        !c.shot_store() && c.ward()==self.model() && result==c.val_load() &&
        f.postcondition_once((s,c),()))]
    pub fn acquire<F>(&self,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&Committer<ModelAtomic,usize,Acquire,NoStore>) {
        self.atomic.native.load(Ordering::Acquire)
    }

}
