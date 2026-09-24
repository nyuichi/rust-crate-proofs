use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastLookup {
    Empty,
    Closed,
    Lagged(u64),
    Ready(u64),
}

/// Locked tail state for the bounded broadcast ring. Positions are logical
/// non-wrapping generations; production's wrapping-u64 arithmetic is confined
/// to its position adapter.
pub struct BroadcastTail {
    capacity: u64,
    position: u64,
    receivers: u64,
    senders: u64,
    closed: bool,
}

impl BroadcastTail {
    pub closed spec fn capacity(&self) -> u64 { self.capacity }
    pub closed spec fn position(&self) -> u64 { self.position }
    pub closed spec fn receivers(&self) -> u64 { self.receivers }
    pub closed spec fn senders(&self) -> u64 { self.senders }
    pub closed spec fn closed(&self) -> bool { self.closed }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.capacity() > 0
        &&& self.closed() == (self.receivers() == 0 || self.senders() == 0)
    }

    pub fn new(capacity: u64) -> (result: Self)
        requires capacity > 0,
        ensures
            result.well_formed(),
            result.capacity() == capacity,
            result.position() == 0,
            result.receivers() == 1,
            result.senders() == 1,
        no_unwind
    {
        BroadcastTail { capacity, position: 0, receivers: 1, senders: 1, closed: false }
    }

    pub fn subscribe(&mut self) -> (next: u64)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
            old(self).receivers() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).receivers() == old(self).receivers() + 1,
            final(self).senders() == old(self).senders(),
            final(self).position() == old(self).position(),
            next == old(self).position(),
            !final(self).closed(),
        no_unwind
    {
        self.receivers += 1;
        self.closed = false;
        self.position
    }

    pub fn clone_sender(&mut self)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
            old(self).senders() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).senders() == old(self).senders() + 1,
            final(self).receivers() == old(self).receivers(),
            final(self).position() == old(self).position(),
        no_unwind
    {
        self.senders += 1;
    }

    pub fn drop_sender(&mut self) -> (closed_now: bool)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
        ensures
            final(self).well_formed(),
            final(self).senders() + 1 == old(self).senders(),
            final(self).receivers() == old(self).receivers(),
            final(self).position() == old(self).position(),
            closed_now == (old(self).senders() == 1),
        no_unwind
    {
        self.senders -= 1;
        if self.senders == 0 {
            self.closed = true;
            true
        } else {
            false
        }
    }

    pub fn drop_receiver(&mut self) -> (closed_now: bool)
        requires
            old(self).well_formed(),
            old(self).receivers() > 0,
        ensures
            final(self).well_formed(),
            final(self).receivers() + 1 == old(self).receivers(),
            final(self).senders() == old(self).senders(),
            final(self).position() == old(self).position(),
            closed_now == (old(self).receivers() == 1),
        no_unwind
    {
        self.receivers -= 1;
        if self.receivers == 0 {
            self.closed = true;
            true
        } else {
            false
        }
    }

    /// Reserves the unique next ring position while holding the tail lock.
    pub fn reserve_send(&mut self) -> (result: Option<(u64, u64)>)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
            old(self).position() < u64::MAX,
        ensures
            final(self).well_formed(),
            result.is_none() ==> old(self).receivers() == 0,
            result.is_none() ==> final(self).position() == old(self).position(),
            old(self).receivers() > 0 ==>
                result == Some((old(self).position(), old(self).receivers())),
            result == Some((old(self).position(), old(self).receivers())) ==>
                final(self).position() == old(self).position() + 1,
            final(self).receivers() == old(self).receivers(),
            final(self).senders() == old(self).senders(),
            final(self).capacity() == old(self).capacity(),
            final(self).closed() == old(self).closed(),
        no_unwind
    {
        if self.receivers == 0 {
            None
        } else {
            let ticket = (self.position, self.receivers);
            self.position += 1;
            Some(ticket)
        }
    }

    pub fn classify(&self, next: u64) -> (result: BroadcastLookup)
        requires
            self.well_formed(),
            next <= self.position(),
        ensures
            match result {
                BroadcastLookup::Empty => next == self.position() && !self.closed(),
                BroadcastLookup::Closed => next == self.position() && self.closed(),
                BroadcastLookup::Lagged(missed) => {
                    self.position() > self.capacity()
                        && next < self.position() - self.capacity()
                        && missed == self.position() - self.capacity() - next
                },
                BroadcastLookup::Ready(position) => {
                    position == next && next < self.position()
                        && (self.position() <= self.capacity()
                            || next >= self.position() - self.capacity())
                },
            },
        no_unwind
    {
        if next == self.position {
            if self.closed { BroadcastLookup::Closed } else { BroadcastLookup::Empty }
        } else if self.position > self.capacity && next < self.position - self.capacity {
            BroadcastLookup::Lagged(self.position - self.capacity - next)
        } else {
            BroadcastLookup::Ready(next)
        }
    }
}

/// One physical ring slot. `remaining` conserves the receivers captured by the
/// send ticket; overwriting returns the evicted value exactly once.
pub struct BroadcastSlot<T> {
    position: Option<u64>,
    remaining: u64,
    value: Option<T>,
}

impl<T> BroadcastSlot<T> {
    pub closed spec fn position(&self) -> Option<u64> { self.position }
    pub closed spec fn remaining(&self) -> u64 { self.remaining }
    pub closed spec fn value(&self) -> Option<T> { self.value }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.position().is_none() == self.value().is_none()
        &&& self.value().is_none() ==> self.remaining() == 0
    }

    pub fn empty() -> (result: Self)
        ensures
            result.well_formed(),
            result.position().is_none(),
            result.value().is_none(),
        no_unwind
    {
        BroadcastSlot { position: None, remaining: 0, value: None }
    }

    pub fn publish(&mut self, position: u64, receivers: u64, value: T)
        -> (evicted: Option<T>)
        requires
            old(self).well_formed(),
            receivers > 0,
        ensures
            final(self).well_formed(),
            evicted == old(self).value(),
            final(self).position() == Some(position),
            final(self).remaining() == receivers,
            final(self).value() == Some(value),
        no_unwind
    {
        let mut replacement = Some(value);
        core::mem::swap(&mut self.value, &mut replacement);
        self.position = Some(position);
        self.remaining = receivers;
        replacement
    }

    pub fn receive_copy(&mut self, position: u64) -> (result: Option<T>)
        where T: Copy
        requires
            old(self).well_formed(),
            old(self).position() == Some(position),
            old(self).remaining() > 0,
        ensures
            result == old(self).value(),
            final(self).position() == old(self).position(),
            final(self).value() == old(self).value(),
            final(self).remaining() + 1 == old(self).remaining(),
            final(self).well_formed(),
        no_unwind
    {
        self.remaining -= 1;
        self.value
    }
}

pub struct BroadcastReceiver {
    next: u64,
}

impl BroadcastReceiver {
    pub closed spec fn next(&self) -> u64 { self.next }

    pub fn new(next: u64) -> (result: Self)
        ensures result.next() == next,
        no_unwind
    {
        BroadcastReceiver { next }
    }

    pub fn apply_lookup(&mut self, lookup: BroadcastLookup) -> (result: BroadcastLookup)
        requires
            match lookup {
                BroadcastLookup::Ready(position) =>
                    position == old(self).next() && old(self).next() < u64::MAX,
                BroadcastLookup::Lagged(missed) =>
                    missed <= u64::MAX - old(self).next(),
                BroadcastLookup::Empty | BroadcastLookup::Closed => true,
            },
            lookup != BroadcastLookup::Lagged(0),
        ensures
            result == lookup,
            match lookup {
                BroadcastLookup::Ready(_) => final(self).next() == old(self).next() + 1,
                BroadcastLookup::Lagged(missed) => final(self).next() == old(self).next() + missed,
                BroadcastLookup::Empty | BroadcastLookup::Closed =>
                    final(self).next() == old(self).next(),
            },
        no_unwind
    {
        match lookup {
            BroadcastLookup::Ready(_) => self.next += 1,
            BroadcastLookup::Lagged(missed) => self.next += missed,
            BroadcastLookup::Empty | BroadcastLookup::Closed => {},
        }
        lookup
    }
}

pub fn verify_broadcast_two_receivers(value: u64)
{
    let mut tail = BroadcastTail::new(2);
    let second_next = tail.subscribe();
    let mut first = BroadcastReceiver::new(0);
    let mut second = BroadcastReceiver::new(second_next);
    let mut slot = BroadcastSlot::empty();
    let ticket = tail.reserve_send();
    assert(ticket == Some((0, 2)));
    let evicted = slot.publish(0, 2, value);
    assert(evicted.is_none());
    let first_lookup = tail.classify(first.next);
    assert(first_lookup == BroadcastLookup::Ready(0));
    first.apply_lookup(first_lookup);
    let first_value = slot.receive_copy(0);
    assert(first_value == Some(value));
    let second_lookup = tail.classify(second.next);
    second.apply_lookup(second_lookup);
    let second_value = slot.receive_copy(0);
    assert(second_value == Some(value));
    assert(slot.remaining() == 0);
}

pub fn verify_broadcast_lag()
{
    let mut tail = BroadcastTail::new(2);
    let mut receiver = BroadcastReceiver::new(0);
    tail.reserve_send();
    tail.reserve_send();
    tail.reserve_send();
    let lookup = tail.classify(receiver.next);
    assert(lookup == BroadcastLookup::Lagged(1));
    receiver.apply_lookup(lookup);
    assert(receiver.next() == 1);
    let ready = tail.classify(receiver.next);
    assert(ready == BroadcastLookup::Ready(1));
}

pub fn verify_broadcast_reopens()
{
    let mut tail = BroadcastTail::new(4);
    let last = tail.drop_receiver();
    assert(last);
    assert(tail.closed());
    let next = tail.subscribe();
    assert(next == tail.position());
    assert(!tail.closed());
}

} // verus!
