use vstd::prelude::*;

verus! {

/// Aggregate ownership of all unread logical messages for one Receiver. One
/// canonical cursor drives the production drop loop irrespective of how many
/// physical ring wraps occur.
pub struct BroadcastDrain {
    next: u64,
    tail: u64,
    released: u64,
}

impl BroadcastDrain {
    pub closed spec fn next(&self) -> u64 { self.next }
    pub closed spec fn tail(&self) -> u64 { self.tail }
    pub closed spec fn released(&self) -> u64 { self.released }

    pub closed spec fn well_formed(&self) -> bool {
        self.next() <= self.tail()
    }

    pub fn new(next: u64, tail: u64) -> (result: Self)
        requires next <= tail,
        ensures
            result.well_formed(),
            result.next() == next,
            result.tail() == tail,
            result.released() == 0,
        no_unwind
    {
        BroadcastDrain { next, tail, released: 0 }
    }

    /// Abstracts Receiver::drop's repeated recv_ref. Each iteration releases
    /// exactly one unread logical Receiver reference; lag recovery may skip
    /// physical values but cannot change the logical interval consumed here.
    pub fn drain_all(&mut self)
        requires
            old(self).well_formed(),
            old(self).released() <= u64::MAX - (old(self).tail() - old(self).next()),
        ensures
            final(self).well_formed(),
            final(self).next() == old(self).tail(),
            final(self).tail() == old(self).tail(),
            final(self).released() == old(self).released() + old(self).tail() - old(self).next(),
        no_unwind
    {
        let initial_next = self.next;
        let initial_released = self.released;
        while self.next < self.tail
            invariant
                initial_next <= self.next,
                self.next <= self.tail,
                self.tail == old(self).tail,
                self.released == initial_released + self.next - initial_next,
                self.released <= u64::MAX - (self.tail - self.next),
            decreases self.tail - self.next,
        {
            self.next += 1;
            self.released += 1;
        }
    }
}

/// Clone execution is a common trait boundary. Given its exact-value outcome,
/// this body proves broadcast's channel-specific guard behavior: cursor
/// consumption and remaining-reader decrement happen even when Clone panics.
pub struct BroadcastCloneGuard<T> {
    slot_value: Option<T>,
    remaining: u64,
}

impl<T> BroadcastCloneGuard<T> {
    pub closed spec fn slot_value(&self) -> Option<T> { self.slot_value }
    pub closed spec fn remaining(&self) -> u64 { self.remaining }

    pub closed spec fn well_formed(&self) -> bool {
        self.slot_value().is_some() && self.remaining() > 0
    }

    pub fn new(value: T, remaining: u64) -> (result: Self)
        requires remaining > 0,
        ensures
            result.well_formed(),
            result.slot_value() == Some(value),
            result.remaining() == remaining,
        no_unwind
    {
        BroadcastCloneGuard { slot_value: Some(value), remaining }
    }

    pub fn finish_clone(&mut self, outcome: Result<T, ()>) -> (released: Option<T>)
        requires
            old(self).well_formed(),
            match outcome {
                Ok(cloned) => old(self).slot_value() == Some(cloned),
                Err(()) => true,
            },
        ensures
            final(self).remaining() + 1 == old(self).remaining(),
            old(self).remaining() == 1 ==> released == old(self).slot_value(),
            old(self).remaining() == 1 ==> final(self).slot_value().is_none(),
            old(self).remaining() > 1 ==> released.is_none(),
            old(self).remaining() > 1 ==> final(self).slot_value() == old(self).slot_value(),
        no_unwind
    {
        self.remaining -= 1;
        if self.remaining == 0 {
            self.slot_value.take()
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastWaitResult {
    Pending,
    Recheck,
    Cancelled,
}

/// Non-pointer projection of broadcast Recv's intrusive waiter. Tail-lock
/// ownership is the common lock adapter; queue membership and Waker identity
/// are channel-specific and body-proved here.
pub struct BroadcastWaiter {
    queued: bool,
    waker: Option<u64>,
}

impl BroadcastWaiter {
    pub closed spec fn queued(&self) -> bool { self.queued }
    pub closed spec fn waker(&self) -> Option<u64> { self.waker }

    pub closed spec fn well_formed(&self) -> bool {
        self.queued() ==> self.waker().is_some()
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            !result.queued(),
            result.waker().is_none(),
        no_unwind
    {
        BroadcastWaiter { queued: false, waker: None }
    }

    pub fn register_empty(&mut self, waker: u64) -> (replaced: Option<u64>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).queued(),
            final(self).waker() == Some(waker),
            replaced == old(self).waker(),
        no_unwind
    {
        let mut replacement = Some(waker);
        core::mem::swap(&mut self.waker, &mut replacement);
        self.queued = true;
        replacement
    }

    pub fn notify(&mut self) -> (wake: Option<u64>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            !final(self).queued(),
            final(self).waker().is_none(),
            wake == old(self).waker(),
        no_unwind
    {
        self.queued = false;
        self.waker.take()
    }

    pub fn cancel(&mut self) -> (removed: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            !final(self).queued(),
            final(self).waker().is_none(),
            removed == old(self).queued(),
        no_unwind
    {
        let removed = self.queued;
        self.queued = false;
        self.waker = None;
        removed
    }
}

pub fn verify_broadcast_arbitrary_receiver_drain(next: u64, tail: u64)
    requires next <= tail,
{
    let mut drain = BroadcastDrain::new(next, tail);
    drain.drain_all();
    assert(drain.next() == tail);
    assert(drain.released() == tail - next);
}

pub fn verify_broadcast_clone_panic_releases_reader(value: u64)
{
    let mut guard = BroadcastCloneGuard::new(value, 1);
    let released = guard.finish_clone(Err(()));
    assert(released == Some(value));
    assert(guard.remaining() == 0);
    assert(guard.slot_value().is_none());
}

pub fn verify_broadcast_waiter_replacement_and_cancel()
{
    let mut waiter = BroadcastWaiter::new();
    let first = waiter.register_empty(10);
    assert(first.is_none());
    let replaced = waiter.register_empty(20);
    assert(replaced == Some(10));
    let removed = waiter.cancel();
    assert(removed);
    assert(!waiter.queued());
}

} // verus!
