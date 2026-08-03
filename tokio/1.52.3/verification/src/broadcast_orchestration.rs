use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastLockPhase {
    Unlocked,
    SlotProbe,
    Tail,
    TailAndSlot,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastWaiterPhase {
    Detached,
    Queued,
    Extracted,
    CleanupUnlinked,
    Done,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LockedRecheck {
    Ready,
    Empty,
    Closed,
    Lagged,
}

/// One-waiter projection of `recv_ref`, `notify_rx`, and `Recv::drop`.
/// Pointer validity, Mutex operations, and Waker execution are frozen
/// adapters. Lock order, membership, extraction order, and cleanup are the
/// broadcast-specific obligations proved here.
pub struct BroadcastOrchestration {
    locks: BroadcastLockPhase,
    waiter: BroadcastWaiterPhase,
    waker: Option<u64>,
    old_waker_after_unlock: Option<u64>,
    queued_flag: bool,
    tail_closed: bool,
    slot_matches: bool,
    empty_generation: bool,
}

impl BroadcastOrchestration {
    pub closed spec fn locks(&self) -> BroadcastLockPhase { self.locks }
    pub closed spec fn waiter(&self) -> BroadcastWaiterPhase { self.waiter }
    pub closed spec fn waker(&self) -> Option<u64> { self.waker }
    pub closed spec fn old_waker_after_unlock(&self) -> Option<u64> {
        self.old_waker_after_unlock
    }
    pub closed spec fn queued_flag(&self) -> bool { self.queued_flag }
    pub closed spec fn tail_closed(&self) -> bool { self.tail_closed }
    pub closed spec fn slot_matches(&self) -> bool { self.slot_matches }
    pub closed spec fn empty_generation(&self) -> bool { self.empty_generation }

    pub closed spec fn well_formed(&self) -> bool {
        &&& (self.waiter() == BroadcastWaiterPhase::Queued
            ==> self.queued_flag() && self.waker().is_some())
        &&& (self.waiter() == BroadcastWaiterPhase::CleanupUnlinked
            ==> self.queued_flag() && self.waker().is_some())
        &&& (self.waiter() == BroadcastWaiterPhase::Extracted
            ==> !self.queued_flag() && self.waker().is_none())
        &&& (self.waiter() == BroadcastWaiterPhase::Detached
            ==> !self.queued_flag())
        &&& (self.old_waker_after_unlock().is_some()
            ==> self.locks() == BroadcastLockPhase::TailAndSlot)
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(),
            result.locks() == BroadcastLockPhase::Unlocked,
            result.waiter() == BroadcastWaiterPhase::Detached,
            result.waker().is_none(), result.old_waker_after_unlock().is_none(),
        no_unwind
    {
        BroadcastOrchestration {
            locks: BroadcastLockPhase::Unlocked,
            waiter: BroadcastWaiterPhase::Detached,
            waker: None,
            old_waker_after_unlock: None,
            queued_flag: false,
            tail_closed: false,
            slot_matches: false,
            empty_generation: true,
        }
    }

    pub fn probe_slot(&mut self, matches: bool)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Unlocked,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::SlotProbe,
            final(self).slot_matches() == matches,
            final(self).waiter() == old(self).waiter(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.locks = BroadcastLockPhase::SlotProbe;
        self.slot_matches = matches;
    }

    /// Required deadlock-avoidance edge: a mismatching slot probe is released
    /// before attempting the tail lock.
    pub fn release_mismatching_probe(&mut self)
        requires old(self).well_formed(),
            old(self).locks() == BroadcastLockPhase::SlotProbe,
            !old(self).slot_matches(),
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Unlocked,
            final(self).waiter() == old(self).waiter(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.locks = BroadcastLockPhase::Unlocked;
    }

    pub fn lock_tail(&mut self, closed: bool)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Unlocked,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Tail,
            final(self).tail_closed() == closed,
            final(self).waiter() == old(self).waiter(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.locks = BroadcastLockPhase::Tail;
        self.tail_closed = closed;
    }

    pub fn relock_slot(&mut self, matches: bool, empty_generation: bool)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Tail,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::TailAndSlot,
            final(self).slot_matches() == matches,
            final(self).empty_generation() == empty_generation,
            final(self).tail_closed() == old(self).tail_closed(),
            final(self).old_waker_after_unlock() == old(self).old_waker_after_unlock(),
            final(self).waiter() == old(self).waiter(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.locks = BroadcastLockPhase::TailAndSlot;
        self.slot_matches = matches;
        self.empty_generation = empty_generation;
    }

    pub fn locked_recheck(&self) -> (result: LockedRecheck)
        requires self.well_formed(), self.locks() == BroadcastLockPhase::TailAndSlot,
        ensures
            self.slot_matches() ==> result == LockedRecheck::Ready,
            !self.slot_matches() && self.empty_generation() && self.tail_closed()
                ==> result == LockedRecheck::Closed,
            !self.slot_matches() && self.empty_generation() && !self.tail_closed()
                ==> result == LockedRecheck::Empty,
            !self.slot_matches() && !self.empty_generation()
                ==> result == LockedRecheck::Lagged,
        no_unwind
    {
        if self.slot_matches {
            LockedRecheck::Ready
        } else if !self.empty_generation {
            LockedRecheck::Lagged
        } else if self.tail_closed {
            LockedRecheck::Closed
        } else {
            LockedRecheck::Empty
        }
    }

    /// Registration happens only after the slot has been rechecked while both
    /// locks are held. Replaced Wakers remain in a separate token until unlock.
    pub fn register_empty(&mut self, new_waker: u64, same_task: bool)
        requires old(self).well_formed(),
            old(self).locks() == BroadcastLockPhase::TailAndSlot,
            !old(self).slot_matches(), old(self).empty_generation(),
            !old(self).tail_closed(), old(self).old_waker_after_unlock().is_none(),
            old(self).waiter() == BroadcastWaiterPhase::Detached
                || old(self).waiter() == BroadcastWaiterPhase::Queued,
            same_task ==> old(self).waker().is_some(),
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::TailAndSlot,
            final(self).waiter() == BroadcastWaiterPhase::Queued,
            final(self).queued_flag(),
            same_task ==> final(self).waker() == old(self).waker(),
            same_task ==> final(self).old_waker_after_unlock().is_none(),
            !same_task ==> final(self).waker() == Some(new_waker),
            !same_task ==> final(self).old_waker_after_unlock() == old(self).waker(),
        no_unwind
    {
        if !same_task {
            self.old_waker_after_unlock = self.waker.take();
            self.waker = Some(new_waker);
        }
        self.queued_flag = true;
        self.waiter = BroadcastWaiterPhase::Queued;
    }

    pub fn unlock_pending(&mut self) -> (old_waker: Option<u64>)
        requires old(self).well_formed(),
            old(self).locks() == BroadcastLockPhase::TailAndSlot,
            old(self).waiter() == BroadcastWaiterPhase::Queued,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Unlocked,
            old_waker == old(self).old_waker_after_unlock(),
            final(self).old_waker_after_unlock().is_none(),
            final(self).waiter() == BroadcastWaiterPhase::Queued,
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.locks = BroadcastLockPhase::Unlocked;
        self.old_waker_after_unlock.take()
    }

    pub fn begin_notify(&mut self)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Unlocked,
            old(self).waiter() == BroadcastWaiterPhase::Queued,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Tail,
            final(self).waiter() == BroadcastWaiterPhase::Queued,
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.locks = BroadcastLockPhase::Tail;
    }

    /// Production extracts the Waker before its Release store of queued=false.
    pub fn extract_for_wake(&mut self) -> (wake_token: Option<u64>)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Tail,
            old(self).waiter() == BroadcastWaiterPhase::Queued,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Tail,
            final(self).waiter() == BroadcastWaiterPhase::Extracted,
            !final(self).queued_flag(), final(self).waker().is_none(),
            wake_token == old(self).waker(),
        no_unwind
    {
        let wake = self.waker.take();
        self.queued_flag = false;
        self.waiter = BroadcastWaiterPhase::Extracted;
        wake
    }

    pub fn unlock_before_wake(&mut self)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Tail,
            old(self).waiter() == BroadcastWaiterPhase::Extracted,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Unlocked,
            final(self).waiter() == BroadcastWaiterPhase::Extracted,
        no_unwind
    {
        self.locks = BroadcastLockPhase::Unlocked;
    }

    /// Both normal wake completion and Waker-panic list cleanup leave no linked
    /// node. The arbitrary Waker execution itself is a frozen boundary.
    pub fn finish_wake_or_unwind_cleanup(&mut self)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Unlocked,
            old(self).waiter() == BroadcastWaiterPhase::Extracted,
        ensures final(self).well_formed(),
            final(self).waiter() == BroadcastWaiterPhase::Done,
            !final(self).queued_flag(), final(self).waker().is_none(),
        no_unwind
    {
        self.waiter = BroadcastWaiterPhase::Done;
    }

    /// If one Waker panics, `WaitersList::drop` unlinks later guarded-list
    /// entries without clearing their queued flags or taking their Wakers.
    /// This state is intentionally distinct from Extracted.
    pub fn cleanup_later_after_waker_panic(&mut self)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Unlocked,
            old(self).waiter() == BroadcastWaiterPhase::Queued,
        ensures final(self).well_formed(),
            final(self).locks() == BroadcastLockPhase::Unlocked,
            final(self).waiter() == BroadcastWaiterPhase::CleanupUnlinked,
            final(self).queued_flag(), final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.waiter = BroadcastWaiterPhase::CleanupUnlinked;
    }

    /// Recv::drop's Acquire fast path: extracted nodes observe false and never
    /// touch the list. Queued nodes must take tail and unlink under the lock.
    pub fn cancel(&mut self) -> (took_tail_lock: bool)
        requires old(self).well_formed(), old(self).locks() == BroadcastLockPhase::Unlocked,
            old(self).waiter() == BroadcastWaiterPhase::Queued
                || old(self).waiter() == BroadcastWaiterPhase::Extracted
                || old(self).waiter() == BroadcastWaiterPhase::CleanupUnlinked
                || old(self).waiter() == BroadcastWaiterPhase::Detached,
        ensures final(self).well_formed(),
            final(self).waiter() == BroadcastWaiterPhase::Done,
            !final(self).queued_flag(), final(self).waker().is_none(),
            took_tail_lock == (old(self).waiter() == BroadcastWaiterPhase::Queued
                || old(self).waiter() == BroadcastWaiterPhase::CleanupUnlinked),
            final(self).locks() == BroadcastLockPhase::Unlocked,
        no_unwind
    {
        let took = match self.waiter {
            BroadcastWaiterPhase::Queued | BroadcastWaiterPhase::CleanupUnlinked => true,
            _ => false,
        };
        self.queued_flag = false;
        self.waker = None;
        self.waiter = BroadcastWaiterPhase::Done;
        took
    }
}

pub fn verify_register_recheck_notify_cycle(waker: u64)
{
    let mut state = BroadcastOrchestration::new();
    state.probe_slot(false);
    state.release_mismatching_probe();
    state.lock_tail(false);
    state.relock_slot(false, true);
    let recheck = state.locked_recheck();
    assert(recheck == LockedRecheck::Empty);
    state.register_empty(waker, false);
    let old = state.unlock_pending();
    assert(old.is_none());
    state.begin_notify();
    let wake = state.extract_for_wake();
    assert(wake == Some(waker));
    state.unlock_before_wake();
    state.finish_wake_or_unwind_cleanup();
    assert(state.waiter() == BroadcastWaiterPhase::Done);
}

/// Two-entry projection of production `push_front` registration followed by
/// `pop_back` notification. This is the only list-order fact needed by
/// broadcast; the intrusive pointer representation remains frozen.
pub struct BroadcastWaiterOrder2 {
    oldest: Option<u64>,
    newest: Option<u64>,
}

impl BroadcastWaiterOrder2 {
    pub closed spec fn oldest(&self) -> Option<u64> { self.oldest }
    pub closed spec fn newest(&self) -> Option<u64> { self.newest }

    pub fn new() -> (result: Self)
        ensures result.oldest().is_none(), result.newest().is_none(),
        no_unwind
    {
        BroadcastWaiterOrder2 { oldest: None, newest: None }
    }

    pub fn push_front(&mut self, waiter: u64)
        requires old(self).newest().is_none(),
        ensures
            old(self).oldest().is_none() ==> final(self).oldest() == Some(waiter),
            old(self).oldest().is_none() ==> final(self).newest().is_none(),
            old(self).oldest().is_some() ==> final(self).oldest() == old(self).oldest(),
            old(self).oldest().is_some() ==> final(self).newest() == Some(waiter),
        no_unwind
    {
        if self.oldest.is_none() {
            self.oldest = Some(waiter);
        } else {
            self.newest = Some(waiter);
        }
    }

    pub fn pop_back(&mut self) -> (waiter: Option<u64>)
        ensures waiter == old(self).oldest(),
            final(self).oldest() == old(self).newest(),
            final(self).newest().is_none(),
        no_unwind
    {
        let waiter = self.oldest.take();
        self.oldest = self.newest.take();
        waiter
    }
}

pub fn verify_waiters_are_notified_registration_order(first: u64, second: u64)
{
    let mut order = BroadcastWaiterOrder2::new();
    order.push_front(first);
    order.push_front(second);
    let popped_first = order.pop_back();
    let popped_second = order.pop_back();
    assert(popped_first == Some(first));
    assert(popped_second == Some(second));
}

} // verus!
