#![allow(non_shorthand_field_patterns)]

use core::sync::atomic::{AtomicUsize, Ordering};
use vstd::atomic::AtomicCellId;
use vstd::prelude::*;

verus! {

#[verifier::external_body]
pub struct RawOrderedUsize {
    atomic: AtomicUsize,
}

#[verifier::external_body]
pub tracked struct RawUsizePermission {
    no_copy: NoCopy,
    unused: bool,
}

pub ghost struct RawUsizePermissionData {
    pub atomic_id: int,
    pub value: usize,
}

impl RawUsizePermission {
    #[verifier::external_body]
    pub uninterp spec fn view(self) -> RawUsizePermissionData;

    pub open spec fn id(&self) -> AtomicCellId { self.view().atomic_id }
    pub open spec fn value(&self) -> usize { self.view().value }
}

impl RawOrderedUsize {
    pub uninterp spec fn id(&self) -> AtomicCellId;

    #[verifier::external_body]
    pub fn new(value: usize) -> (result: (Self, Tracked<RawUsizePermission>))
        ensures
            result.1@.id() == result.0.id(),
            result.1@.value() == value,
        no_unwind
    {
        (
            RawOrderedUsize { atomic: AtomicUsize::new(value) },
            Tracked::assume_new(),
        )
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn load_relaxed(&self, Tracked(permission): Tracked<&RawUsizePermission>)
        -> (result: usize)
        requires permission.id() == self.id(),
        ensures result == permission.value(),
        opens_invariants none
        no_unwind
    {
        self.atomic.load(Ordering::Relaxed)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn load_acquire(&self, Tracked(permission): Tracked<&RawUsizePermission>)
        -> (result: usize)
        requires permission.id() == self.id(),
        ensures result == permission.value(),
        opens_invariants none
        no_unwind
    {
        self.atomic.load(Ordering::Acquire)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn compare_exchange_weak_acqrel_acquire(
        &self,
        Tracked(permission): Tracked<&mut RawUsizePermission>,
        current: usize,
        new: usize,
    ) -> (result: Result<usize, usize>)
        requires old(permission).id() == self.id(),
        ensures
            final(permission).id() == self.id(),
            match result {
                Ok(previous) => {
                    previous == current
                        && old(permission).value() == current
                        && final(permission).value() == new
                },
                Err(actual) => {
                    actual == old(permission).value()
                        && final(permission).value() == old(permission).value()
                },
            },
        opens_invariants none
        no_unwind
    {
        self.atomic.compare_exchange_weak(
            current,
            new,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn fetch_or_acquire(
        &self,
        Tracked(permission): Tracked<&mut RawUsizePermission>,
        mask: usize,
    ) -> (previous: usize)
        requires old(permission).id() == self.id(),
        ensures
            previous == old(permission).value(),
            final(permission).id() == self.id(),
            final(permission).value() == previous | mask,
        opens_invariants none
        no_unwind
    {
        self.atomic.fetch_or(mask, Ordering::Acquire)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn fetch_or_acqrel(
        &self,
        Tracked(permission): Tracked<&mut RawUsizePermission>,
        mask: usize,
    ) -> (previous: usize)
        requires old(permission).id() == self.id(),
        ensures
            previous == old(permission).value(),
            final(permission).id() == self.id(),
            final(permission).value() == previous | mask,
        opens_invariants none
        no_unwind
    {
        self.atomic.fetch_or(mask, Ordering::AcqRel)
    }

    #[verifier::external_body]
    #[verifier::atomic]
    pub fn fetch_and_acqrel(
        &self,
        Tracked(permission): Tracked<&mut RawUsizePermission>,
        mask: usize,
    ) -> (previous: usize)
        requires old(permission).id() == self.id(),
        ensures
            previous == old(permission).value(),
            final(permission).id() == self.id(),
            final(permission).value() == previous & mask,
        opens_invariants none
        no_unwind
    {
        self.atomic.fetch_and(mask, Ordering::AcqRel)
    }
}

/// Operation-specific ordered bitfield corresponding to production `State`.
pub struct OrderedOneshotBits {
    raw: RawOrderedUsize,
    permission: Tracked<RawUsizePermission>,
}

impl OrderedOneshotBits {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.permission@.id() == self.raw.id()
    }

    pub closed spec fn value(&self) -> usize { self.permission@.value() }

    pub fn new() -> (result: Self)
        ensures result.value() == 0,
        no_unwind
    {
        let (raw, permission) = RawOrderedUsize::new(0);
        OrderedOneshotBits { raw, permission }
    }

    pub fn load_acquire(&self) -> (result: usize)
        ensures result == self.value(),
        no_unwind
    {
        proof { use_type_invariant(self); }
        self.raw.load_acquire(Tracked(self.permission.borrow()))
    }

    pub fn load_relaxed(&self) -> (result: usize)
        ensures result == self.value(),
        no_unwind
    {
        proof { use_type_invariant(self); }
        self.raw.load_relaxed(Tracked(self.permission.borrow()))
    }

    pub fn set_closed(&mut self, closed_mask: usize) -> (previous: usize)
        ensures
            previous == old(self).value(),
            final(self).value() == previous | closed_mask,
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        self.raw.fetch_or_acquire(
            Tracked(self.permission.borrow_mut()),
            closed_mask,
        )
    }

    pub fn set_task(&mut self, task_mask: usize) -> (result: usize)
        ensures
            final(self).value() == old(self).value() | task_mask,
            result == final(self).value(),
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        let previous = self.raw.fetch_or_acqrel(
            Tracked(self.permission.borrow_mut()),
            task_mask,
        );
        previous | task_mask
    }

    pub fn unset_task(&mut self, keep_mask: usize) -> (result: usize)
        ensures
            final(self).value() == old(self).value() & keep_mask,
            result == final(self).value(),
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        let previous = self.raw.fetch_and_acqrel(
            Tracked(self.permission.borrow_mut()),
            keep_mask,
        );
        previous & keep_mask
    }

    pub fn complete_once(&mut self, expected: usize, completed: usize)
        -> (result: Result<usize, usize>)
        ensures
            match result {
                Ok(previous) => {
                    previous == expected
                        && old(self).value() == expected
                        && final(self).value() == completed
                },
                Err(actual) => {
                    actual == old(self).value()
                        && final(self).value() == old(self).value()
                },
            },
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        self.raw.compare_exchange_weak_acqrel_acquire(
            Tracked(self.permission.borrow_mut()),
            expected,
            completed,
        )
    }
}

pub fn verify_ordered_bit_operations(
    closed_mask: usize,
    rx_task_mask: usize,
    tx_task_mask: usize,
)
{
    let mut state = OrderedOneshotBits::new();
    assert(0usize | rx_task_mask == rx_task_mask) by (bit_vector);
    let rx = state.set_task(rx_task_mask);
    assert(rx == rx_task_mask);
    let both = state.set_task(tx_task_mask);
    assert(both == rx_task_mask | tx_task_mask);
    let previous = state.set_closed(closed_mask);
    assert(previous == both);
    assert(state.value() == both | closed_mask);
    let loaded = state.load_acquire();
    assert(loaded == state.value());
}

} // verus!
