//! NEW GENERIC TCB: closed-cursor events bound to the exact leased native field.
//! No untracked interference/reentrancy; old unscoped adapters do not accept this descriptor.
//!
//! Unlike `EventAtomic`, this adapter does not own or wrap the native atomic.
//! `ScopedFieldInvariant::bind` associates one consumed protocol with a borrowed core
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

pub use crate::event::{EventProtocol, ScopedProtocol, ScopeCursor};

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
pub struct ScopedFieldInvariant<S: ScopedProtocol> {
    state: PhantomData<S>,
}

#[cfg(creusot)]
#[trusted]
unsafe impl<S: ScopedProtocol + Send + Objective> Sync for ScopedFieldInvariant<S> {}
#[cfg(not(creusot))]
unsafe impl<S: ScopedProtocol + Send> Sync for ScopedFieldInvariant<S> {}
#[trusted]
unsafe impl<S: ScopedProtocol + Send> Send for ScopedFieldInvariant<S> {}

impl<S: ScopedProtocol> ScopedFieldInvariant<S> {
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
    #[ensures(result.inner_logic().0.model() == atomic_model(atomic))]
    #[ensures(result.inner_logic().0.public() == state.public())]
    #[ensures(result.inner_logic().1.model() == atomic_model(atomic))]
    #[ensures(result.inner_logic().1.public() == state.public())]
    #[ensures(*result.inner_logic().1.observation() == state.observe())]
    pub fn bind(atomic: &CoreAtomicUsize, state: Ghost<S>) -> Ghost<(Self, ScopeCursor<S>)> {
        // The body is ghost-erased; the trusted contract identifies the actual
        // native field passed here.
        Ghost::conjure()
    }
}

/// Perform an event on the count field selected through a live typed control
/// lease. The native reference exists only inside this operation. In the
/// trusted interpretation, the FullBorrow/typed-Box permission justifies that
/// dereference, `AtomicField` selects the actual field, the native/model
/// identity is the one in `ScopedFieldInvariant`, and the operation's atomic model
/// is the corresponding Committer event. The loan ends before the ghost
/// callback runs, so the callback may consume the ticket token.
///
/// This boundary assumes no bytes ownership, refcount-to-ticket, last-owner,
/// or reclamation fact. The selected `AtomicField` implementation and lease
/// wellformedness must be body proved by the caller.
#[trusted]
#[requires(cursor.inner_logic().model() == invariant.inner_logic().model() &&
    cursor.inner_logic().public() == invariant.inner_logic().public())]
#[ensures((^cursor).model() == invariant.inner_logic().model() &&
    (^cursor).public() == invariant.inner_logic().public())]
#[requires(control.inner_logic().cur().wellformed())]
#[requires(control.inner_logic().cur().pointer() == pointer)]
#[requires(control.inner_logic().lft() == lease.inner_logic().lft())]
#[requires(control.inner_logic().cur().model() == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    s.observe() == *cursor.inner_logic().observation() &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT &&
     c.val_store()@ == c.val_load()@ + 1) ==>
    f.precondition((s, c)) &&
    (f.postcondition_once((s, c), ()) ==>
        (^s).protocol() && (^s).public() == invariant.inner_logic().public() &&
        (^s).atomic() == invariant.inner_logic().model() && (^c).shot_store()))]
#[ensures(exists<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    s.observe() == *cursor.inner_logic().observation() &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT &&
     c.val_store()@ == c.val_load()@ + 1) &&
    result == c.val_load() && f.postcondition_once((s, c), ()) &&
    *(^cursor).observation() == (^s).observe())]
pub(crate) fn increment_owned<T, S, F>(
    pointer: *const T,
    control: Ghost<&FullBorrow<OwnedControl<T>>>,
    lease: Ghost<&LifetimeToken>,
    invariant: Ghost<&ScopedFieldInvariant<S>>,
    cursor: Ghost<&mut ScopeCursor<S>>,
    f: Ghost<F>,
) -> usize
where
    T: AtomicField,
    S: ScopedProtocol,
    F: FnGhost + FnOnce(&mut S, &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>),
{
    #[cfg(not(creusot))]
    { match crate::native_ref_count_ops::try_increment(unsafe { &*pointer }.atomic_field()) {
        Ok(old) => old,
        Err(_) => std::process::abort(),
    }}
    #[cfg(creusot)]
    { unreachable!("generic exact-field native/model bridge") }
}

/// Release decrement using the actual nested native field of the typed
/// control allocation. This token-owning variant ends its scoped control
/// borrow before giving the token to the protocol callback.
#[trusted]
#[requires(cursor.inner_logic().model() == invariant.inner_logic().model() &&
    cursor.inner_logic().public() == invariant.inner_logic().public())]
#[ensures((^cursor).model() == invariant.inner_logic().model() &&
    (^cursor).public() == invariant.inner_logic().public())]
#[requires(control.inner_logic().cur().wellformed())]
#[requires(control.inner_logic().cur().pointer() == pointer)]
#[requires(control.inner_logic().lft() == lease.inner_logic().lft())]
#[requires(control.inner_logic().cur().model() == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &mut Committer<ModelAtomic, usize, Relaxed, Release>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    s.observe() == *cursor.inner_logic().observation() &&
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
    s.observe() == *cursor.inner_logic().observation() &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    (if c.val_load() == 0usize { c.val_store() == usize::MAX }
     else { c.val_store()@ + 1 == c.val_load()@ }) &&
    result == c.val_load() &&
    f.postcondition_once((s, c, lease.inner_logic()), ()) &&
    *(^cursor).observation() == (^s).observe())]
pub(crate) fn decrement_owned<T, S, F>(
    pointer: *const T,
    control: Ghost<&FullBorrow<OwnedControl<T>>>,
    lease: Ghost<LifetimeToken>,
    invariant: Ghost<&ScopedFieldInvariant<S>>,
    cursor: Ghost<&mut ScopeCursor<S>>,
    f: Ghost<F>,
) -> usize
where
    T: AtomicField,
    S: ScopedProtocol,
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
#[requires(cursor.inner_logic().model() == invariant.inner_logic().model() &&
    cursor.inner_logic().public() == invariant.inner_logic().public())]
#[ensures((^cursor).model() == invariant.inner_logic().model() &&
    (^cursor).public() == invariant.inner_logic().public())]
#[requires(control.inner_logic().cur().wellformed())]
#[requires(control.inner_logic().cur().pointer() == pointer)]
#[requires(control.inner_logic().lft() == lease.inner_logic().lft())]
#[requires(control.inner_logic().cur().model() == invariant.inner_logic().model())]
#[requires(forall<s: &mut S, c: &Committer<ModelAtomic, usize, Acquire, NoStore>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    s.observe() == *cursor.inner_logic().observation() &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() ==>
    f.precondition((s, c)) &&
    (f.postcondition_once((s, c), ()) ==>
        (^s).protocol() && (^s).public() == invariant.inner_logic().public() &&
        (^s).atomic() == invariant.inner_logic().model()))]
#[ensures(exists<s: &mut S, c: &Committer<ModelAtomic, usize, Acquire, NoStore>>
    s.protocol() && s.public() == invariant.inner_logic().public() &&
    s.atomic() == invariant.inner_logic().model() && inv(s) &&
    s.observe() == *cursor.inner_logic().observation() &&
    !c.shot_store() && c.ward() == invariant.inner_logic().model() &&
    result == c.val_load() && f.postcondition_once((s, c), ()) &&
    *(^cursor).observation() == (^s).observe())]
pub(crate) fn acquire_owned<T, S, F>(
    pointer: *const T,
    control: Ghost<&FullBorrow<OwnedControl<T>>>,
    lease: Ghost<&LifetimeToken>,
    invariant: Ghost<&ScopedFieldInvariant<S>>,
    cursor: Ghost<&mut ScopeCursor<S>>,
    f: Ghost<F>,
) -> usize
where
    T: AtomicField,
    S: ScopedProtocol,
    F: FnGhost + FnOnce(&mut S, &Committer<ModelAtomic, usize, Acquire, NoStore>),
{
    unsafe { &*pointer }.atomic_field().load(Ordering::Acquire)
}
