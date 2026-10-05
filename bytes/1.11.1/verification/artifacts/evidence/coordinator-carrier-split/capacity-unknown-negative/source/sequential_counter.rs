//! Local sequential bridge for the exact native atomic operations used by Shared.
//!
//! TCB C1-C4: `new`, Relaxed fetch-add, Release fetch-sub, and Acquire load bind
//! one sealed logical identity to one native atomic cell and its current value.
//! CounterOwn is exclusive, affine, non-Send/non-Sync and NotObjective. All
//! access to the private cell requires that authority. This is a sequential
//! scalar model: it asserts no release sequence, synchronization, or data
//! visibility, and gives no byte/control allocation recovery authority.
//! TODO: remove this local trust only when native core atomics have verified
//! standard contracts exposing exclusive ownership and these exact operations.

use core::{marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};
use creusot_std::{
    ghost::{NotObjective, resource::Resource},
    logic::{Id, ra::excl::{Excl, ExclUpdate}},
    prelude::*,
};

#[cfg_attr(not(creusot), repr(transparent))]
pub(crate) struct SequentialCounter {
    inner: AtomicUsize,
    #[cfg(creusot)]
    identity: Ghost<Id>,
}

pub(crate) struct CounterOwn {
    resource: Resource<Excl<usize>>,
    _not_objective: NotObjective,
    _not_send_sync: PhantomData<*mut ()>,
}

impl View for SequentialCounter {
    type ViewTy = Id;
    #[logic]
    fn view(self) -> Id { *self.identity }
}
impl View for CounterOwn {
    type ViewTy = (Id, Int);
    #[logic]
    fn view(self) -> Self::ViewTy { pearlite! { (self.resource.id(), (self.resource@.0)@) } }
}

impl SequentialCounter {
    /// C1: allocate a fresh exclusive identity for this native cell.
    #[trusted]
    #[ensures(result.0@ == result.1.inner_logic()@.0)]
    #[ensures(result.1.inner_logic()@.1 == value@)]
    pub(crate) fn new(value: usize) -> (Self, Ghost<CounterOwn>) {
        let inner = AtomicUsize::new(value);
        let binding = ghost! {
            let resource = Resource::alloc(snapshot!(Excl(value))).into_inner();
            let identity = resource.id_ghost();
            (identity, CounterOwn { resource, _not_objective: NotObjective {}, _not_send_sync: PhantomData })
        };
        let (_identity, own) = binding.split();
        (Self { inner, #[cfg(creusot)] identity: _identity }, own)
    }

    /// C2: exact nonwrapping scalar update; the native ordering is Relaxed.
    #[trusted]
    #[requires(own.inner_logic()@.0 == self@)]
    #[requires(own.inner_logic()@.1 + amount@ <= usize::MAX@)]
    #[ensures(result@ == own.inner_logic()@.1)]
    #[ensures((^own.inner_logic())@.0 == own.inner_logic()@.0)]
    #[ensures((^own.inner_logic())@.1 == own.inner_logic()@.1 + amount@)]
    pub(crate) fn fetch_add_relaxed(&self, amount: usize, own: Ghost<&mut CounterOwn>) -> usize {
        let old = self.inner.fetch_add(amount, Ordering::Relaxed);
        ghost! { own.into_inner().resource.update(ExclUpdate(snapshot!(old + amount))); };
        old
    }

    /// C3: exact nonwrapping scalar update; the native ordering is Release.
    #[trusted]
    #[requires(own.inner_logic()@.0 == self@)]
    #[requires(amount@ <= own.inner_logic()@.1)]
    #[ensures(result@ == own.inner_logic()@.1)]
    #[ensures((^own.inner_logic())@.0 == own.inner_logic()@.0)]
    #[ensures((^own.inner_logic())@.1 == own.inner_logic()@.1 - amount@)]
    pub(crate) fn fetch_sub_release(&self, amount: usize, own: Ghost<&mut CounterOwn>) -> usize {
        let old = self.inner.fetch_sub(amount, Ordering::Release);
        ghost! { own.into_inner().resource.update(ExclUpdate(snapshot!(old - amount))); };
        old
    }

    /// C4: read the scalar under exclusive ownership; no visibility transfer.
    #[trusted]
    #[requires(own.inner_logic()@.0 == self@)]
    #[ensures(result@ == own.inner_logic()@.1)]
    pub(crate) fn load_acquire(&self, own: Ghost<&CounterOwn>) -> usize {
        self.inner.load(Ordering::Acquire)
    }
}

#[cfg(not(creusot))]
const _: () = {
    assert!(core::mem::size_of::<SequentialCounter>() == core::mem::size_of::<AtomicUsize>());
    assert!(core::mem::align_of::<SequentialCounter>() == core::mem::align_of::<AtomicUsize>());
};
