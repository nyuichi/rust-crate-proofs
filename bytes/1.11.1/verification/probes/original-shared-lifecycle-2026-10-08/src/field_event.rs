//! NEW GENERIC TCB: atomic events bound to the actual atomic field.
//!
//! Unlike `EventAtomic`, this adapter does not own or wrap the native atomic.
//! `FieldInvariant::bind` associates one consumed protocol with a borrowed core
//! `AtomicUsize`; each event must name that same field and its model. The
//! trusted relation is a generic native-field/model relation only. It states
//! no bytes ownership, refcount, or reclamation law.

use core::{marker::PhantomData, sync::atomic::{AtomicUsize as CoreAtomicUsize, Ordering}};
use creusot_std::{
    ghost::{FnGhost, Perm, invariant::Protocol},
    logic::FMap,
    prelude::*,
    std::sync::{
        atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire, None as NoStore, Relaxed, Release}},
        committer::Committer,
        view::{HasTimestamp, SyncView},
    },
};
#[cfg(creusot)]
use creusot_std::ghost::Objective;

pub trait EventProtocol: Protocol {
    #[logic]
    fn atomic(self) -> ModelAtomic;
}

/// Opaque model identity for one actual `core::sync::atomic::AtomicUsize`
/// field. The generic native/model boundary preserves this identity when the
/// value returned by `new` is moved into its final owner. It does not name a
/// second counter or assert any bytes ownership/refcount law.
#[logic(opaque)]
pub fn atomic_model(atomic: &CoreAtomicUsize) -> ModelAtomic {
    dead
}

/// Creates the actual core atomic field used by the caller and its unique
/// model permission. Callers move this returned field into their native record;
/// all later events borrow that same field directly.
#[trusted]
#[ensures(*result.1.ward() == atomic_model(&result.0))]
#[ensures(**current <= ^current)]
#[ensures(result.1.val() == FMap::singleton(
    atomic_model(&result.0).get_timestamp(^current), (value, ^current)))]
pub fn new(value: usize, current: Ghost<&mut SyncView>) -> (CoreAtomicUsize, Ghost<Perm<ModelAtomic>>) {
    (CoreAtomicUsize::new(value), Ghost::conjure())
}

/// Opaque, non-copyable descriptor for the protocol state associated with one
/// actual core atomic field. Its runtime representation carries no authority;
/// the trusted interpretation binds the consumed state to that named field.
#[opaque]
pub struct FieldInvariant<S: EventProtocol> {
    state: PhantomData<S>,
}

#[cfg(creusot)]
#[trusted]
unsafe impl<S: EventProtocol + Send + Objective> Sync for FieldInvariant<S> {}
#[cfg(not(creusot))]
unsafe impl<S: EventProtocol + Send> Sync for FieldInvariant<S> {}
#[trusted]
unsafe impl<S: EventProtocol + Send> Send for FieldInvariant<S> {}

impl<S: EventProtocol> FieldInvariant<S> {
    #[logic(opaque)]
    pub fn model(&self) -> ModelAtomic {
        dead
    }

    #[logic(opaque)]
    pub fn public(&self) -> S::Public {
        dead
    }

    /// Consumes the protocol once and binds it to this exact atomic model.
    /// The resulting descriptor has no state getter or ghost opener.
    #[trusted]
    #[check(ghost)]
    #[requires(state.protocol() && state.atomic() == atomic_model(atomic))]
    #[ensures(result.model() == atomic_model(atomic))]
    #[ensures(result.public() == state.public())]
    pub fn bind(atomic: &CoreAtomicUsize, state: Ghost<S>) -> Self {
        // The body is ghost-erased; the trusted contract identifies the actual
        // native field passed here.
        Self { state: PhantomData }
    }
}

/// Performs a Relaxed RMW on the borrowed native field itself, opening only
/// the invariant supplied for that exact field and only for the restricted
/// checked callback.
#[trusted]
#[requires(atomic_model(atomic) == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == usize::MAX { c.val_store() == 0usize }
     else { c.val_store()@ == c.val_load()@ + 1 }) ==>
    f.precondition((s, c)) &&
    (f.postcondition_once((s, c), ()) ==>
        (^s).protocol() && (^s).public() == invariant.inner_logic().public() &&
        (^s).atomic() == invariant.inner_logic().model() && (^c).shot_store()))]
#[ensures(exists<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == usize::MAX { c.val_store() == 0usize }
     else { c.val_store()@ == c.val_load()@ + 1 }) &&
    result == c.val_load() && f.postcondition_once((s, c), ()))]
pub fn increment<S, F>(
    atomic: &CoreAtomicUsize,
    invariant: Ghost<&FieldInvariant<S>>,
    f: Ghost<F>,
) -> usize
where
    S: EventProtocol,
    F: FnGhost + FnOnce(&mut S, &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>),
{
    atomic.fetch_add(1, Ordering::Relaxed)
}

/// Performs the actual Release fetch-sub RMW. The model admits its native
/// wrapping behavior, including old value zero; a lifecycle protocol must put
/// any nonzero requirement in its callback precondition.
#[trusted]
#[requires(atomic_model(atomic) == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Release>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == 0usize { c.val_store() == usize::MAX }
     else { c.val_store()@ + 1 == c.val_load()@ }) ==>
    f.precondition((s, c)) &&
    (f.postcondition_once((s, c), ()) ==>
        (^s).protocol() && (^s).public() == invariant.inner_logic().public() &&
        (^s).atomic() == invariant.inner_logic().model() && (^c).shot_store()))]
#[ensures(exists<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Release>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == 0usize { c.val_store() == usize::MAX }
     else { c.val_store()@ + 1 == c.val_load()@ }) &&
    result == c.val_load() && f.postcondition_once((s, c), ()))]
pub fn decrement<S, F>(
    atomic: &CoreAtomicUsize,
    invariant: Ghost<&FieldInvariant<S>>,
    f: Ghost<F>,
) -> usize
where
    S: EventProtocol,
    F: FnGhost + FnOnce(&mut S, &mut Committer<ModelAtomic, usize, Relaxed, Release>),
{
    atomic.fetch_sub(1, Ordering::Release)
}

/// Performs the actual Acquire load on the same named native field. The
/// callback receives only the encapsulated state and the typed load history;
/// no public operation can expose that state independently of this event.
#[trusted]
#[requires(atomic_model(atomic) == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &Committer<ModelAtomic, usize, Acquire, NoStore>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() ==>
    f.precondition((s, c)) &&
    (f.postcondition_once((s, c), ()) ==>
        (^s).protocol() && (^s).public() == invariant.inner_logic().public() &&
        (^s).atomic() == invariant.inner_logic().model()))]
#[ensures(exists<s: &mut S, c: &Committer<ModelAtomic, usize, Acquire, NoStore>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    result == c.val_load() && f.postcondition_once((s, c), ()))]
pub fn acquire<S, F>(
    atomic: &CoreAtomicUsize,
    invariant: Ghost<&FieldInvariant<S>>,
    f: Ghost<F>,
) -> usize
where
    S: EventProtocol,
    F: FnGhost + FnOnce(&mut S, &Committer<ModelAtomic, usize, Acquire, NoStore>),
{
    atomic.load(Ordering::Acquire)
}
