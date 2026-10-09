//! NEW GENERIC TCB: closed, noninterfering scope and event observation alignment.
//! Only closed checker-admitted callers may use this experimental adapter.
//! No cursor state/resource getter; no bytes lastness or ownership assumption.
use core::{marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};
use creusot_std::{prelude::*, ghost::{FnGhost, Perm, invariant::Protocol},
    logic::FMap, std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Relaxed, Release, Acquire, None as NoStore}},
        committer::Committer, view::{HasTimestamp,SyncView}}};
#[cfg(creusot)] use creusot_std::ghost::Objective;

pub trait EventProtocol: Protocol {
    #[logic] fn atomic(self) -> ModelAtomic;
}

/// In this closed probe the sole implementation projects only pure map/Int data.
pub trait ScopedProtocol: EventProtocol {
    type Observation;
    #[logic] fn observe(self) -> Self::Observation;
}

/// Affine erased trace channel. Native scope checker supplies noninterference.
/// Raw-pointer marker denies Send/Sync; no Clone or resource extraction exists.
#[opaque]
pub struct ScopeCursor<S: ScopedProtocol> { marker: PhantomData<*mut S> }
impl<S: ScopedProtocol> ScopeCursor<S> {
    #[logic(opaque)] pub fn model(self) -> ModelAtomic { dead }
    #[logic(opaque)] pub fn public(self) -> S::Public { dead }
    #[logic(opaque)] pub fn observation(self) -> Snapshot<S::Observation> { dead }
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
pub struct ScopedEventAtomic<S:ScopedProtocol> { atomic:RawAtomic, state:PhantomData<S> }
#[cfg(creusot)]
#[trusted]
unsafe impl<S:ScopedProtocol+Send+Objective> Sync for ScopedEventAtomic<S> {}
#[cfg(not(creusot))]
unsafe impl<S:ScopedProtocol+Send> Sync for ScopedEventAtomic<S> {}
#[trusted]
unsafe impl<S:ScopedProtocol+Send> Send for ScopedEventAtomic<S> {}

impl<S:ScopedProtocol> ScopedEventAtomic<S> {
    #[logic(opaque)] pub fn model(self)->ModelAtomic { dead }
    #[logic(opaque)] pub fn public(self)->S::Public { dead }

    /// Consumes the protocol once; never copies its affine contents.
    #[trusted]
    #[requires(state.protocol() && state.atomic() == atomic.model())]
    #[ensures(result.0.model() == atomic.model() && result.0.public() == state.public())]
    #[ensures(result.1.inner_logic().model() == atomic.model())]
    #[ensures(result.1.inner_logic().public() == state.public())]
    #[ensures(*result.1.inner_logic().observation() == state.observe())]
    pub fn bind(atomic:RawAtomic,state:Ghost<S>)->(Self,Ghost<ScopeCursor<S>>) {
        (Self {atomic,state:PhantomData}, ghost! { ScopeCursor {marker:PhantomData} })
    }

    /// One success commits exactly once at the actual Relaxed CAS. The update
    /// closure passed to fetch_update contains only pure guard/arithmetic.
    /// Failed/spurious retries do not invoke f; refusal is a Relaxed read and
    /// makes no store BY THIS OPERATION. This scoped facade excludes untracked mutation and reentrancy; every
    /// event is serialized through the unique erased cursor.
    /// Err carries no callback postcondition and cannot create a registration.
    #[trusted]
    #[requires(cursor.inner_logic().model() == self.model() && cursor.inner_logic().public() == self.public())]
    #[ensures((^cursor).model() == self.model() && (^cursor).public() == self.public())]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(match result {
        Ok(old) => exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 &&
        old == c.val_load() && f.postcondition_once((s,c),()) &&
        *(^cursor).observation() == (^s).observe(),
        Err(observed) => (^cursor).observation() == cursor.inner_logic().observation() && observed > crate::ref_count_limit::MAX_REF_COUNT &&
            exists<c:&Committer<ModelAtomic,usize,Relaxed,NoStore>>
                !c.shot_store() && c.ward() == self.model() && c.val_load() == observed
    })]
    pub fn try_increment<F>(&self,cursor:Ghost<&mut ScopeCursor<S>>,f:Ghost<F>)->Result<usize,usize>
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>) {
        self.atomic.native.fetch_update(Ordering::Relaxed, Ordering::Relaxed,
            crate::ref_count_limit::next_ref_count)
    }

    /// Normal-return guarded increment; refusal follows the native abort path.
    #[requires(cursor.inner_logic().model() == self.model() && cursor.inner_logic().public() == self.public())]
    #[ensures((^cursor).model() == self.model() && (^cursor).public() == self.public())]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT && c.val_store()@ == c.val_load()@ + 1 &&
        result == c.val_load() && f.postcondition_once((s,c),()) &&
        *(^cursor).observation() == (^s).observe())]
    pub fn increment<F>(&self,cursor:Ghost<&mut ScopeCursor<S>>,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>) {
        match self.try_increment(cursor,f) {
            Ok(old) => old,
            Err(_) => { abort_overflow(); 0 }
        }
    }

    #[trusted]
    #[requires(cursor.inner_logic().model() == self.model() && cursor.inner_logic().public() == self.public())]
    #[ensures((^cursor).model() == self.model() && (^cursor).public() == self.public())]
    #[requires(forall<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() && (if c.val_load() == 0usize { c.val_store() == usize::MAX } else { c.val_store()@ + 1 == c.val_load()@ }) ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model() && (^c).shot_store()))]
    #[ensures(exists<s:&mut S,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() && (if c.val_load() == 0usize { c.val_store() == usize::MAX } else { c.val_store()@ + 1 == c.val_load()@ }) &&
        result == c.val_load() && f.postcondition_once((s,c),()) &&
        *(^cursor).observation() == (^s).observe())]
    pub fn decrement<F>(&self,cursor:Ghost<&mut ScopeCursor<S>>,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&mut Committer<ModelAtomic,usize,Relaxed,Release>) {
        self.atomic.native.fetch_sub(1, Ordering::Release)
    }

    #[trusted]
    #[requires(cursor.inner_logic().model() == self.model() && cursor.inner_logic().public() == self.public())]
    #[ensures((^cursor).model() == self.model() && (^cursor).public() == self.public())]
    #[requires(forall<s:&mut S,c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() && true ==>
        f.precondition((s,c)) &&
        (f.postcondition_once((s,c),()) ==> (^s).protocol() && (^s).public() == self.public() &&
            (^s).atomic() == self.model()))]
    #[ensures(exists<s:&mut S,c:&Committer<ModelAtomic,usize,Acquire,NoStore>>
        s.protocol() && s.public() == self.public() && s.atomic() == self.model() && inv(s) &&
        s.observe() == *cursor.inner_logic().observation() &&
        !c.shot_store() && c.ward() == self.model() && true &&
        result == c.val_load() && f.postcondition_once((s,c),()) &&
        *(^cursor).observation() == (^s).observe())]
    pub fn acquire<F>(&self,cursor:Ghost<&mut ScopeCursor<S>>,f:Ghost<F>)->usize
    where F:FnGhost+FnOnce(&mut S,&Committer<ModelAtomic,usize,Acquire,NoStore>) {
        self.atomic.native.load(Ordering::Acquire)
    }
}

#[trusted]
#[ensures(false)]
#[cold]
fn abort_overflow() { std::process::abort() }
