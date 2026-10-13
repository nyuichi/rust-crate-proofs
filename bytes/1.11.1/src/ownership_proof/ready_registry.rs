//! Append-only affine storage for producer-issued readiness fragments.
//!
//! This module does not allocate or duplicate resource-algebra fragments. A
//! producer transfers the exact fragment returned by its authority into the
//! registry. Taking a capability leaves a tombstone at its stable slot;
//! returning it fills only that slot. The registry has no authority of its
//! own and does not interpret the fragment's state.

use creusot_std::{
    ghost::{Ghost, NotObjective, resource::{Authority, Fragment}},
    logic::{Id, Int, Seq, ra::{UnitRA, update::LocalUpdate}},
    prelude::*,
};

/// Stable registry identity. The slot is assigned only by append and never
/// reused, including after a capability is taken or retired.
#[derive(creusot_std::prelude::Clone, Copy)]
pub(crate) struct ReadyKey {
    resource_id: Id,
    slot: Int,
}

impl ReadyKey {
    #[logic(open(self))]
    pub(crate) fn resource_id(self) -> Id {
        self.resource_id
    }

    #[logic(open(self))]
    pub(crate) fn slot(self) -> Int {
        self.slot
    }
}

/// One actual affine fragment together with the key assigned when it was
/// registered. This type is intentionally not `Clone` or `Copy`.
pub(crate) struct ReadyCap<R: UnitRA> {
    key: ReadyKey,
    fragment: Fragment<R>,
    _not_objective: NotObjective,
}

impl<R: UnitRA> ReadyCap<R> {
    /// Opaque logical view of the key and the exact fragment identity and RA
    /// state. Callers can rely on this tuple without opening the cap layout.
    #[logic(opaque)]
    #[ensures(result.0 == self.key().resource_id())]
    #[ensures(result.1 == self.key().slot())]
    #[ensures(result.2 == self.fragment_id())]
    #[ensures(result.3 == self.fragment_value())]
    pub(crate) fn state(self) -> (Id, Int, Id, R) {
        pearlite! {
            (self.key.resource_id, self.key.slot, self.fragment.id(), self.fragment@)
        }
    }

    /// Public logical projections used in state contracts without exposing
    /// the affine fragment field to sibling proof modules.
    #[logic(open(self))]
    pub(crate) fn fragment_id(self) -> Id {
        self.fragment.id()
    }

    #[logic(open(self))]
    pub(crate) fn fragment_value(self) -> R {
        pearlite! { self.fragment@ }
    }

    #[logic(open(self))]
    pub(crate) fn key(self) -> ReadyKey {
        self.key
    }

    /// Body-checked access to the actual affine fragment for authority
    /// compatibility lemmas. The reference is confined to ghost code.
    #[check(ghost)]
    #[ensures(result.inner_logic().id() == self.fragment_id())]
    #[ensures(result.inner_logic()@ == self.fragment_value())]
    pub(crate) fn fragment_ref(&self) -> Ghost<&Fragment<R>> {
        ghost! { &self.fragment }
    }

    /// Apply an authority update through the actual paired fragment. Forwarding
    /// through `Authority::update` keeps the fragment resource id stable, which
    /// preserves the key-to-fragment binding.
    #[check(ghost)]
    #[requires(authority.id() == self.key.resource_id)]
    #[requires(authority.id() == self.fragment.id())]
    #[requires(update.premise(authority@, self.fragment@))]
    #[ensures((^self).key.resource_id == self.key.resource_id)]
    #[ensures((^self).key.slot == self.key.slot)]
    #[ensures((^authority).id() == authority.id())]
    #[ensures((^self).fragment.id() == self.fragment.id())]
    #[ensures((^authority)@ == update.update(authority@, self.fragment@).0)]
    #[ensures((^self).fragment@ == update.update(authority@, self.fragment@).1)]
    pub(crate) fn update<U: LocalUpdate<R>>(
        &mut self,
        authority: &mut Authority<R>,
        update: U,
    ) -> Ghost<()> {
        authority.update(&mut self.fragment, update);
        ghost!(())
    }
}

impl<R: UnitRA> Invariant for ReadyCap<R> {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.key.slot >= 0 && self.fragment.id() == self.key.resource_id
        }
    }
}

/// One caller-context registry. Each occupied slot owns the exact
/// `Fragment<R>` moved into it; `None` entries are permanent tombstones or
/// currently outstanding loans. There is deliberately no second authority.
pub(crate) struct ReadyRegistry<R: UnitRA> {
    entries: Ghost<Seq<Option<ReadyCap<R>>>>,
    _not_objective: NotObjective,
}

#[logic(open)]
fn same_entry_state<R: UnitRA>(
    left: Option<(Id, Int, Id, R)>,
    right: Option<(Id, Int, Id, R)>,
) -> bool {
    pearlite! {
        match (left, right) {
            (None, None) => true,
            (Some((left_id, left_slot, left_fragment_id, left_value)),
             Some((right_id, right_slot, right_fragment_id, right_value))) =>
                left_id == right_id && left_slot == right_slot &&
                left_fragment_id == right_fragment_id && left_value == right_value,
            _ => false,
        }
    }
}

impl<R: UnitRA> ReadyRegistry<R> {
    #[logic(open(self))]
    fn entries(self) -> Seq<Option<ReadyCap<R>>> {
        self.entries.inner_logic()
    }

    /// Opaque length observation for exact context reborrow framing. This
    /// keeps append position and trailing tombstones visible without exposing
    /// the affine entries sequence itself.
    #[logic(opaque)]
    #[ensures(result == self.entries().len())]
    pub(crate) fn len(self) -> Int {
        self.entries().len()
    }

    /// Private representation view used to body-prove the public opaque slot
    /// observation. Each occupied entry contributes its full cap state.
    #[logic(open(self))]
    fn entry_state_model(self, slot: Int) -> Option<(Id, Int, Id, R)> {
        match self.entries().get(slot) {
            Some(Some(cap)) => Some(cap.state()),
            _ => None,
        }
    }

    /// Opaque logical observation of the exact cap at a stable slot. It
    /// records the issued key, fragment resource id, and complete RA value.
    #[logic(opaque)]
    #[ensures(result == self.entry_state_model(slot))]
    pub(crate) fn entry_state(self, slot: Int) -> Option<(Id, Int, Id, R)> {
        self.entry_state_model(slot)
    }

    #[logic(open(self))]
    fn contains_key(self, key: ReadyKey) -> bool {
        pearlite! {
            0 <= key.slot && key.slot < self.entries().len() &&
            match self.entry_state(key.slot) {
                Some((resource_id, slot, fragment_id, _)) =>
                    resource_id == key.resource_id && slot == key.slot &&
                    fragment_id == key.resource_id,
                None => false,
            }
        }
    }

    /// Public opaque projection of the exact occupied slot named by `key`.
    /// This is only a registration lookup; callers must still inspect the
    /// returned cap's actual RA state before treating it as an idle lease.
    #[logic(opaque)]
    #[ensures(result == self.contains_key(key))]
    pub(crate) fn has_key(self, key: ReadyKey) -> bool {
        self.contains_key(key)
    }

    #[check(ghost)]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(result.inner_logic().entries().len() == 0)]
    pub(crate) fn new() -> Ghost<Self> {
        ghost! {
            Self {
                entries: Seq::new(),
                _not_objective: NotObjective {},
            }
        }
    }

    /// Move a producer's actual fragment into a fresh, stable slot.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures((^self).invariant())]
    #[ensures(result.inner_logic().resource_id() == fragment.inner_logic().id())]
    #[ensures(result.inner_logic().slot() == self.entries().len())]
    #[ensures((^self).entries().len() == self.entries().len() + 1)]
    #[ensures((^self).contains_key(result.inner_logic()))]
    #[ensures(same_entry_state((^self).entry_state(result.inner_logic().slot()),
        Some((result.inner_logic().resource_id(), result.inner_logic().slot(),
            fragment.inner_logic().id(), fragment.inner_logic()@))))]
    #[ensures(forall<i: Int> 0 <= i && i < self.entries().len() ==>
        same_entry_state((^self).entry_state(i), self.entry_state(i)))]
    pub(crate) fn register(&mut self, fragment: Ghost<Fragment<R>>) -> Ghost<ReadyKey> {
        ghost! {
            let fragment = fragment.into_inner();
            let entries = self.entries.borrow_mut().into_inner();
            let slot = entries.len_ghost();
            let key = ReadyKey { resource_id: fragment.id_ghost(), slot };
            entries.push_back_ghost(Some(ReadyCap {
                key,
                fragment,
                _not_objective: NotObjective {},
            }));
            key
        }
    }

    /// Take the exact fragment named by a producer-issued key. The returned
    /// cap owns the same fragment; every other slot remains unchanged.
    #[check(ghost)]
    #[requires(self.invariant() && self.contains_key(key))]
    #[ensures((^self).invariant())]
    #[ensures(result.inner_logic().invariant())]
    #[ensures(same_entry_state(Some(result.inner_logic().state()), self.entry_state(key.slot())))]
    #[ensures((^self).entry_state(key.slot()) == None)]
    #[ensures((^self).entries().len() == self.entries().len())]
    #[ensures(forall<i: Int> 0 <= i && i < self.entries().len() && i != key.slot() ==>
        same_entry_state((^self).entry_state(i), self.entry_state(i)))]
    pub(crate) fn take(&mut self, key: ReadyKey) -> Ghost<ReadyCap<R>> {
        ghost! {
            let entries = self.entries.borrow_mut().into_inner();
            let slot = entries.get_mut_ghost(key.slot).unwrap();
            let cap = core::mem::replace(slot, None).unwrap();
            proof_assert!(cap.key.resource_id == key.resource_id && cap.key.slot == key.slot);
            cap
        }
    }

    /// Return a loaned capability to its original tombstone. The key cannot
    /// be changed and no occupied slot can be overwritten.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(cap.inner_logic().invariant())]
    #[requires(cap.inner_logic().key().slot() < self.entries().len())]
    #[requires(self.entry_state(cap.inner_logic().key().slot()) == None)]
    #[ensures((^self).invariant())]
    #[ensures((^self).contains_key(cap.inner_logic().key()))]
    #[ensures((^self).entries().len() == self.entries().len())]
    #[ensures(same_entry_state((^self).entry_state(cap.inner_logic().key().slot()),
        Some(cap.inner_logic().state())))]
    #[ensures(forall<i: Int> 0 <= i && i < self.entries().len() &&
        i != cap.inner_logic().key().slot() ==>
        same_entry_state((^self).entry_state(i), self.entry_state(i)))]
    pub(crate) fn return_cap(&mut self, cap: Ghost<ReadyCap<R>>) -> Ghost<()> {
        ghost! {
            let cap = cap.into_inner();
            let slot = cap.key.slot;
            let entries = self.entries.borrow_mut().into_inner();
            let entry = entries.get_mut_ghost(slot).unwrap();
            *entry = Some(cap);
            ()
        }
    }
}

impl<R: UnitRA> Invariant for ReadyRegistry<R> {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.entries.invariant() &&
            forall<i: Int> 0 <= i && i < self.entries().len() ==>
                match self.entries().get(i) {
                    Some(Some(cap)) =>
                        cap.key.slot == i && cap.key.resource_id == cap.fragment.id(),
                    _ => true,
                }
        }
    }
}
