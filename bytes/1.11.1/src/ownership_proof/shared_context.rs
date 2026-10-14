//! An explicit, affine proof context for sequential shared-header operations.
//!
//! The context carries the caller's existing token set and the registry that
//! owns the actual readiness fragments issued by shared-header producers. It
//! never creates tokens or duplicates a registry. Native `Bytes` and
//! `BytesMut` layouts do not contain this proof-only context.

use creusot_std::{
    ghost::{invariant::{Namespace, Tokens}, resource::Fragment, Ghost, NotObjective},
    logic::{Id, Int, Set, ra::UnitRA},
    prelude::*,
};

use super::ready_registry::{ReadyCap, ReadyKey, ReadyRegistry};

/// Owns one legitimate token set and the one affine registry associated with
/// it. This value is intentionally not `Clone` or `Copy`.
pub(crate) struct OwnedSharedContext<'ctx, R: UnitRA> {
    tokens: Ghost<Tokens<'ctx>>,
    ready: Ghost<ReadyRegistry<R>>,
    _not_objective: NotObjective,
}

/// An exclusive view into the actual context fields. Mutations through
/// `ready` are mutations of the owner's registry, not copies of its state.
pub(crate) struct BorrowedSharedContext<'view, R: UnitRA> {
    tokens: Ghost<Tokens<'view>>,
    /// The actual registry reference stays in ghost storage and cannot be
    /// retargeted by assigning through an exposed `&mut` field.
    ready: Ghost<&'view mut ReadyRegistry<R>>,
    _not_objective: NotObjective,
}

impl<'ctx, R: UnitRA> OwnedSharedContext<'ctx, R> {
    /// Exact projection used to frame the caller's token namespaces.
    #[logic(open(self))]
    pub(crate) fn token_namespaces(self) -> Set<Namespace> {
        self.tokens.inner_logic().namespaces()
    }

    /// Exact registry length for contracts on methods that receive the
    /// owning context. The registry itself remains private and affine.
    #[logic(open(self))]
    pub(crate) fn ready_len(self) -> Int {
        self.ready.inner_logic().len()
    }

    /// Exact logical state at one stable slot, including the actual fragment
    /// identity and complete RA value.
    #[logic(open(self))]
    pub(crate) fn ready_entry_state(self, slot: Int) -> Option<(Id, Int, Id, R)> {
        self.ready.inner_logic().entry_state(slot)
    }

    #[logic(open(self))]
    pub(crate) fn ready_has_key(self, key: ReadyKey) -> bool {
        self.ready.inner_logic().has_key(key)
    }

    /// Move an already-authorized token set into one context and initialize
    /// its empty registry. This function neither creates nor splits tokens.
    #[check(ghost)]
    #[requires(tokens.invariant())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().ready_len() == 0)]
    #[ensures(result.inner_logic().token_namespaces() == tokens.inner_logic().namespaces())]
    pub(crate) fn from_tokens(tokens: Ghost<Tokens<'ctx>>) -> Ghost<Self> {
        ghost! {
            let ready = ReadyRegistry::new();
            Self { tokens, ready, _not_objective: NotObjective {} }
        }
    }

    /// Reborrow both components from their actual owner fields. The view's
    /// registry reference has the same place identity as `self.ready`.
    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures(result.inner_logic().ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() ==>
        result.inner_logic().ready_entry_state(i) == context.inner_logic().ready_entry_state(i))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    // Registry mutations through the returned child flow back to this exact
    // field when its borrow ends; token ownership is restored unchanged.
    #[ensures((^context.inner_logic()).ready_len() == result.inner_logic().ready_final_len())]
    #[ensures(forall<i: Int> 0 <= i && i < (^context.inner_logic()).ready_len() ==>
        (^context.inner_logic()).ready_entry_state(i) == result.inner_logic().ready_final_entry_state(i))]
    pub(crate) fn reborrow_view<'view>(
        context: Ghost<&'view mut Self>,
    ) -> Ghost<BorrowedSharedContext<'view, R>> {
        ghost! {
            let context = context.into_inner();
            let tokens = context.tokens.borrow_mut().into_inner().reborrow();
            let ready = context.ready.borrow_mut().into_inner();
            BorrowedSharedContext { tokens: Ghost::new(tokens), ready: Ghost::new(ready), _not_objective: NotObjective {} }
        }
    }
}

impl<'view, R: UnitRA> BorrowedSharedContext<'view, R> {
    #[logic(open(self))]
    pub(crate) fn token_namespaces(self) -> Set<Namespace> {
        self.tokens.inner_logic().namespaces()
    }

    #[logic(open(self))]
    pub(crate) fn ready_len(self) -> Int {
        self.ready.inner_logic().len()
    }

    #[logic(open(self))]
    pub(crate) fn ready_entry_state(self, slot: Int) -> Option<(Id, Int, Id, R)> {
        self.ready.inner_logic().entry_state(slot)
    }

    #[logic(open(self))]
    pub(crate) fn ready_has_key(self, key: ReadyKey) -> bool {
        self.ready.inner_logic().has_key(key)
    }

    /// Exact registration frame over the actual borrowed registry. The key
    /// itself identifies the appended cap's real fragment value; each older
    /// slot and the token set remain unchanged.
    #[logic(opaque)]
    #[ensures(result == (
        after.token_namespaces() == self.token_namespaces() &&
        after.ready_len() == self.ready_len() + 1 &&
        key.slot() == self.ready_len() &&
        after.ready_has_key(key) &&
        after.ready_entry_state(key.slot()) == Some((
            key.resource_id(), key.slot(), key.resource_id(), fragment_value
        )) &&
        (forall<i: Int> 0 <= i && i < self.ready_len() ==>
            after.ready_entry_state(i) == self.ready_entry_state(i))
    ))]
    pub(crate) fn registered_cap(
        self,
        after: BorrowedSharedContext<'_, R>,
        key: ReadyKey,
        fragment_value: R,
    ) -> bool {
        pearlite! {
            after.token_namespaces() == self.token_namespaces() &&
            after.ready_len() == self.ready_len() + 1 &&
            key.slot() == self.ready_len() &&
            after.ready_has_key(key) &&
            after.ready_entry_state(key.slot()) == Some((
                key.resource_id(), key.slot(), key.resource_id(), fragment_value
            )) &&
            (forall<i: Int> 0 <= i && i < self.ready_len() ==>
                after.ready_entry_state(i) == self.ready_entry_state(i))
        }
    }

    /// Exact context frame when no readiness capability is registered, taken,
    /// or returned.
    #[logic(opaque)]
    #[ensures(result == (
        after.token_namespaces() == self.token_namespaces() &&
        after.ready_len() == self.ready_len() &&
        (forall<i: Int> 0 <= i && i < self.ready_len() ==>
            after.ready_entry_state(i) == self.ready_entry_state(i))
    ))]
    pub(crate) fn unchanged(self, after: BorrowedSharedContext<'_, R>) -> bool {
        pearlite! {
            after.token_namespaces() == self.token_namespaces() &&
            after.ready_len() == self.ready_len() &&
            (forall<i: Int> 0 <= i && i < self.ready_len() ==>
                after.ready_entry_state(i) == self.ready_entry_state(i))
        }
    }

    /// Read the registry's actual ghost model without consuming its exclusive
    /// view, for checked proof bodies that continue moving that view.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(result.inner_logic() == self.ready_has_key(key))]
    pub(crate) fn ready_has_key_ghost(&self, key: ReadyKey) -> Ghost<bool> {
        ghost! {
            snapshot!(self.ready_has_key(key)).into_ghost().into_inner()
        }
    }

    /// Prophetic observations describe the registry after the current view's
    /// borrow ends, so a caller can frame legitimate downstream mutations.
    #[logic(open(self), prophetic)]
    pub(crate) fn ready_final_len(self) -> Int {
        pearlite! { (^self.ready.inner_logic()).len() }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn ready_final_entry_state(self, slot: Int) -> Option<(Id, Int, Id, R)> {
        pearlite! { (^self.ready.inner_logic()).entry_state(slot) }
    }

    /// Reborrow the token capability from this view, preserving its namespace
    /// set and tying its lifetime to the exclusive view borrow.
    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[ensures(result.inner_logic().namespaces() == context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() ==>
        (^context.inner_logic()).ready_entry_state(i) == context.inner_logic().ready_entry_state(i))]
    pub(crate) fn reborrow_tokens<'short>(
        context: Ghost<&'short mut Self>,
    ) -> Ghost<Tokens<'short>>
    where
        'view: 'short,
    {
        ghost! {
            let context = context.into_inner();
            let tokens = context.tokens.borrow_mut().into_inner().reborrow();
            tokens
        }
    }

    /// Narrow a registry fragment into the context, assigning a stable slot.
    /// This forwards the producer's exact affine fragment; no mirror resource
    /// is created.
    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures(result.inner_logic().resource_id() == fragment.inner_logic().id())]
    #[ensures(result.inner_logic().slot() == context.inner_logic().ready_len())]
    #[ensures(result.inner_logic().slot() >= 0)]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len() + 1)]
    #[ensures((^context.inner_logic()).ready_entry_state(result.inner_logic().slot()) ==
        Some((result.inner_logic().resource_id(), result.inner_logic().slot(),
            fragment.inner_logic().id(), fragment.inner_logic()@)))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() ==>
        (^context.inner_logic()).ready_entry_state(i) == context.inner_logic().ready_entry_state(i))]
    #[ensures(context.inner_logic().registered_cap(
        (^context.inner_logic()),
        result.inner_logic(),
        fragment.inner_logic()@
    ))]
    pub(crate) fn register<'short>(
        context: Ghost<&'short mut Self>,
        fragment: Ghost<Fragment<R>>,
    ) -> Ghost<ReadyKey>
    where
        'view: 'short,
    {
        ghost! {
            let context = context.into_inner();
            let mut registry = context.ready.borrow_mut().into_inner();
            (**registry).register(fragment).into_inner()
        }
    }

    /// Move out the exact capability named by its producer-issued key,
    /// leaving a tombstone. All other registry slots are framed exactly.
    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().ready_has_key(key))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().key() == key)]
    #[ensures(result.inner_logic().state() == context.inner_logic().ready_entry_state(key.slot()).unwrap_logic())]
    #[ensures(0 <= key.slot() && key.slot() < context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_entry_state(key.slot()) == None)]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() && i != key.slot() ==>
        (^context.inner_logic()).ready_entry_state(i) == context.inner_logic().ready_entry_state(i))]
    pub(crate) fn take<'short>(
        context: Ghost<&'short mut Self>,
        key: ReadyKey,
    ) -> Ghost<ReadyCap<R>>
    where
        'view: 'short,
    {
        ghost! {
            let context = context.into_inner();
            let mut registry = context.ready.borrow_mut().into_inner();
            (**registry).take(key).into_inner()
        }
    }

    /// Return the same actual cap to its original empty slot. The registry
    /// length and every other slot remain unchanged.
    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key().slot() < context.inner_logic().ready_len())]
    #[requires(context.inner_logic().ready_entry_state(cap.inner_logic().key().slot()) == None)]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_has_key(cap.inner_logic().key()))]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures((^context.inner_logic()).ready_entry_state(cap.inner_logic().key().slot()) ==
        Some(cap.inner_logic().state()))]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
        i != cap.inner_logic().key().slot() ==>
        (^context.inner_logic()).ready_entry_state(i) == context.inner_logic().ready_entry_state(i))]
    pub(crate) fn return_cap<'short>(
        context: Ghost<&'short mut Self>,
        cap: Ghost<ReadyCap<R>>,
    ) -> Ghost<()>
    where
        'view: 'short,
    {
        ghost! {
            let context = context.into_inner();
            let mut registry = context.ready.borrow_mut().into_inner();
            (**registry).return_cap(cap).into_inner()
        }
    }

    /// Reborrow both existing capabilities without copying either one.
    #[check(ghost)]
    #[requires(context.inner_logic().invariant())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures(result.inner_logic().ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() ==>
        result.inner_logic().ready_entry_state(i) == context.inner_logic().ready_entry_state(i))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == result.inner_logic().ready_final_len())]
    #[ensures(forall<i: Int> 0 <= i && i < (^context.inner_logic()).ready_len() ==>
        (^context.inner_logic()).ready_entry_state(i) == result.inner_logic().ready_final_entry_state(i))]
    pub(crate) fn reborrow<'short>(
        context: Ghost<&'short mut Self>,
    ) -> Ghost<BorrowedSharedContext<'short, R>>
    where
        'view: 'short,
    {
        ghost! {
            let context = context.into_inner();
            let tokens = context.tokens.borrow_mut().into_inner().reborrow();
            let mut ready = context.ready.borrow_mut().into_inner();
            let ready: &mut ReadyRegistry<R> = &mut **ready;
            BorrowedSharedContext { tokens: Ghost::new(tokens), ready: Ghost::new(ready), _not_objective: NotObjective {} }
        }
    }
}

impl<R: UnitRA> Invariant for OwnedSharedContext<'_, R> {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.tokens.invariant() && self.ready.invariant()
    }
}

impl<R: UnitRA> Invariant for BorrowedSharedContext<'_, R> {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.tokens.invariant() && self.ready.invariant()
    }
}
