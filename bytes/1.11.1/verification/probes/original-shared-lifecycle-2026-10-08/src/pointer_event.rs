//! Generic TCB for a core `AtomicPtr` initialized once and only read.
//!
//! The affine permission is consumed into `ReadOnlyPointer`; this module
//! exposes no store operation. The TCB maps the actual core field to its
//! atomic model, preserves its initialized pointer value, and interprets one
//! native Relaxed load. It states no bytes ownership, vtable, refcount, or
//! reclamation law.

use core::{marker::PhantomData, sync::atomic::{AtomicPtr as CoreAtomicPtr, Ordering}};
use creusot_std::{
    ghost::Perm,
    logic::FMap,
    prelude::*,
    std::sync::{
        atomic::AtomicPtr as ModelAtomicPtr,
        view::{HasTimestamp, SyncView},
    },
};

#[logic(opaque)]
pub(crate) fn pointer_model(field: &CoreAtomicPtr<()>) -> ModelAtomicPtr<()> {
    dead
}

/// Construct the actual core atomic pointer field and its unique model
/// permission. The returned field can be moved into its final record before
/// `bind_read_only` consumes the permission.
#[trusted]
#[ensures(*result.1.ward() == pointer_model(&result.0))]
#[ensures(**current <= ^current)]
#[ensures(result.1.val() == FMap::singleton(
    pointer_model(&result.0).get_timestamp(^current), (value, ^current)))]
pub(crate) fn new_pointer(
    value: *mut (),
    current: Ghost<&mut SyncView>,
) -> (CoreAtomicPtr<()>, Ghost<Perm<ModelAtomicPtr<()>>>) {
    (CoreAtomicPtr::new(value), Ghost::conjure())
}

/// An affine read-only interpretation of one initialized pointer field.
#[opaque]
pub(crate) struct ReadOnlyPointer {
    state: PhantomData<ModelAtomicPtr<()>>,
}

impl ReadOnlyPointer {
    #[logic(opaque)]
    pub(crate) fn model(self) -> ModelAtomicPtr<()> {
        dead
    }

    #[logic(opaque)]
    pub(crate) fn value(self) -> *mut () {
        dead
    }
}

/// Consume the sole write permission after initialization. Requiring every
/// history entry to retain the same pointer establishes the restricted
/// immutable-cell premise needed by the load adapter.
#[trusted]
#[check(ghost)]
#[requires(*permission.inner_logic().ward() == pointer_model(field))]
#[requires(forall<t: Int> permission.inner_logic().val().get(t) != None ==>
    permission.inner_logic().val().get(t).unwrap_logic().0 == value)]
#[ensures(result.model() == pointer_model(field))]
#[ensures(result.value() == value)]
pub(crate) fn bind_read_only(
    field: &CoreAtomicPtr<()>,
    value: *mut (),
    permission: Ghost<Perm<ModelAtomicPtr<()>>>,
) -> Ghost<ReadOnlyPointer> {
    Ghost::conjure()
}

/// Read the named actual native field. The consumed affine permission and the
/// absence of any store operation in this adapter ensure that the field still
/// contains its initialized pointer.
#[trusted]
#[requires(pointer_model(field) == binding.inner_logic().model())]
#[ensures(result == binding.inner_logic().value())]
pub(crate) fn load_relaxed(
    field: &CoreAtomicPtr<()>,
    binding: Ghost<&ReadOnlyPointer>,
) -> *mut () {
    field.load(Ordering::Relaxed)
}

/// Select the initialized pointer through the exact mutable core atomic field,
/// matching `AtomicMut::with_mut` in the selected production `loom.rs` alias.
/// The returned raw pointer is a value copy; no mutable reference escapes this
/// generic bridge. This is used only when the source consumes a `Bytes` handle
/// for cleanup and has exclusive access to `data`. This is the restricted
/// read-only counterpart of Std 0.13's `AtomicPtr::into_inner`: that API
/// consumes the unique permission and exposes the latest modeled pointer
/// value. Here an immutable binding proves there were no stores, and Rust's
/// exclusive borrow supports `get_mut`; the native-to-model interpretation
/// remains a generic TCB.
#[trusted]
#[requires(pointer_model(field) == binding.inner_logic().model())]
#[ensures(result == binding.inner_logic().value())]
#[ensures(^field == *field)]
pub(crate) fn get_mut(field: &mut CoreAtomicPtr<()>, binding: Ghost<&ReadOnlyPointer>) -> *mut () {
    *field.get_mut()
}
