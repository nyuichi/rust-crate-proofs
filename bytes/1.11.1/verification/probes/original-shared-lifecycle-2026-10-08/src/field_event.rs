//! NEW GENERIC TCB: atomic events bound to the actual atomic field.
//!
//! Unlike `EventAtomic`, this adapter does not own or wrap the native atomic.
//! `FieldInvariant::bind` associates one consumed protocol with a borrowed core
//! `AtomicUsize`; each event must name that same field and its model. The
//! trusted relation is a generic native-field/model relation only. It states
//! no bytes ownership, refcount, or reclamation law.

use alloc::boxed::Box;
use core::{marker::PhantomData, sync::atomic::{AtomicUsize as CoreAtomicUsize, Ordering}};
use creusot_std::{
    ghost::{FnGhost, Perm, invariant::Protocol, lifetime_logic::{FullBorrow, LifetimeToken}},
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

/// Projection from a typed control allocation to its actual native count
/// field. Implementations are ordinary Rust bodies and must prove the model
/// projection contract; the lease TCB does not trust a bytes-specific field
/// selector.
pub(crate) trait AtomicField {
    #[cfg_attr(creusot, ensures(atomic_model(result) == self.field_model()))]
    fn atomic_field(&self) -> &CoreAtomicUsize;

    #[cfg(creusot)]
    #[logic]
    fn field_model(&self) -> ModelAtomic;
}

/// Proof-only owner for one typed control allocation. This value lives in a
/// `FullBorrow`; each event requires a matching live `LifetimeToken`. It does
/// not alter the native control-block layout.
pub(crate) struct OwnedControl<T: AtomicField> {
    pub(crate) pointer: *const T,
    pub(crate) owner: Ghost<Box<Perm<*const T>>>,
}

impl<T: AtomicField> OwnedControl<T> {
    #[logic(open(crate))]
    pub(crate) fn pointer(self) -> *const T { self.pointer }

    #[logic(open(crate))]
    pub(crate) fn model(self) -> ModelAtomic { self.owner.val().field_model() }

    #[logic(open(crate), prophetic)]
    pub(crate) fn wellformed(self) -> bool {
        self.pointer == *self.owner.ward()
    }
}

/// Turn the exact typed Box permission into a lease payload. The field model
/// is always projected from the permission itself, so no second model value
/// or bytes-specific pointer/counter axiom is introduced.
#[check(ghost)]
#[requires(*owner.ward() == pointer)]
#[ensures(result.inner_logic().wellformed())]
#[ensures(result.inner_logic().pointer() == pointer)]
#[ensures(result.inner_logic().owner == owner)]
pub(crate) fn own_control<T: AtomicField>(
    pointer: *const T,
    owner: Ghost<Box<Perm<*const T>>>,
) -> Ghost<OwnedControl<T>> {
    ghost! { OwnedControl { pointer, owner } }
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
    #[ensures(result.inner_logic().model() == atomic_model(atomic))]
    #[ensures(result.inner_logic().public() == state.public())]
    pub fn bind(atomic: &CoreAtomicUsize, state: Ghost<S>) -> Ghost<Self> {
        // The body is ghost-erased; the trusted contract identifies the actual
        // native field passed here.
        ghost! { Self { state: PhantomData } }
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

/// Perform an event on the count field selected through a live typed control
/// lease. The native reference exists only inside this operation. In the
/// trusted interpretation, the FullBorrow/typed-Box permission justifies that
/// dereference, `AtomicField` selects the actual field, the native/model
/// identity is the one in `FieldInvariant`, and the operation's atomic model
/// is the corresponding Committer event. The loan ends before the ghost
/// callback runs, so the callback may consume the ticket token.
///
/// This boundary assumes no bytes ownership, refcount-to-ticket, last-owner,
/// or reclamation fact. The selected `AtomicField` implementation and lease
/// wellformedness must be body proved by the caller.
#[trusted]
#[requires(control.inner_logic().cur().wellformed())]
#[requires(control.inner_logic().cur().pointer() == pointer)]
#[requires(control.inner_logic().lft() == lease.inner_logic().lft())]
#[requires(control.inner_logic().cur().model() == invariant.inner_logic().model())]
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
pub(crate) fn increment_owned<T, S, F>(
    pointer: *const T,
    control: Ghost<&FullBorrow<OwnedControl<T>>>,
    lease: Ghost<&LifetimeToken>,
    invariant: Ghost<&FieldInvariant<S>>,
    f: Ghost<F>,
) -> usize
where
    T: AtomicField,
    S: EventProtocol,
    F: FnGhost + FnOnce(&mut S, &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>),
{
    unsafe { &*pointer }.atomic_field().fetch_add(1, Ordering::Relaxed)
}

/// Release decrement using the actual nested native field of the typed
/// control allocation. This token-owning variant ends its scoped control
/// borrow before giving the token to the protocol callback.
#[trusted]
#[requires(control.inner_logic().cur().wellformed())]
#[requires(control.inner_logic().cur().pointer() == pointer)]
#[requires(control.inner_logic().lft() == lease.inner_logic().lft())]
#[requires(control.inner_logic().cur().model() == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Release>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == 0usize { c.val_store() == usize::MAX }
     else { c.val_store()@ + 1 == c.val_load()@ }) ==>
    f.precondition((s, c, lease.inner_logic())) &&
    (f.postcondition_once((s, c, lease.inner_logic()), ()) ==>
        (^s).protocol() && (^s).public() == invariant.inner_logic().public() &&
        (^s).atomic() == invariant.inner_logic().model() && (^c).shot_store()))]
#[ensures(exists<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Release>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == 0usize { c.val_store() == usize::MAX }
     else { c.val_store()@ + 1 == c.val_load()@ }) &&
    result == c.val_load() &&
    f.postcondition_once((s, c, lease.inner_logic()), ()))]
pub(crate) fn decrement_owned<T, S, F>(
    pointer: *const T,
    control: Ghost<&FullBorrow<OwnedControl<T>>>,
    lease: Ghost<LifetimeToken>,
    invariant: Ghost<&FieldInvariant<S>>,
    f: Ghost<F>,
) -> usize
where
    T: AtomicField,
    S: EventProtocol,
    F: FnGhost + FnOnce(
        &mut S,
        &mut Committer<ModelAtomic, usize, Relaxed, Release>,
        LifetimeToken,
    ),
{
    unsafe { &*pointer }.atomic_field().fetch_sub(1, Ordering::Release)
}

/// Acquire load through a still-live full lifetime fraction after the Release
/// decrement. The callback can synchronize the published payload, but this
/// boundary itself makes no assertion about which decrement was last.
#[trusted]
#[requires(control.inner_logic().cur().wellformed())]
#[requires(control.inner_logic().cur().pointer() == pointer)]
#[requires(control.inner_logic().lft() == lease.inner_logic().lft())]
#[requires(control.inner_logic().cur().model() == invariant.inner_logic().model())]
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
pub(crate) fn acquire_owned<T, S, F>(
    pointer: *const T,
    control: Ghost<&FullBorrow<OwnedControl<T>>>,
    lease: Ghost<&LifetimeToken>,
    invariant: Ghost<&FieldInvariant<S>>,
    f: Ghost<F>,
) -> usize
where
    T: AtomicField,
    S: EventProtocol,
    F: FnGhost + FnOnce(&mut S, &Committer<ModelAtomic, usize, Acquire, NoStore>),
{
    unsafe { &*pointer }.atomic_field().load(Ordering::Acquire)
}
