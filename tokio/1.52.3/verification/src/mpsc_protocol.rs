use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReserveResult {
    Permit,
    Full,
    Closed,
}

/// Bounded-channel semaphore and lifecycle state. This is the accounting
/// relation shared by `send`, `try_send`, borrowed/owned permits, receive, and
/// receiver close.
pub struct MpscCapacity {
    capacity: u64,
    available: u64,
    reserved: u64,
    queued: u64,
    senders: u64,
    accepting: bool,
    receiver_alive: bool,
}

impl MpscCapacity {
    pub closed spec fn capacity(&self) -> u64 { self.capacity }
    pub closed spec fn available(&self) -> u64 { self.available }
    pub closed spec fn reserved(&self) -> u64 { self.reserved }
    pub closed spec fn queued(&self) -> u64 { self.queued }
    pub closed spec fn senders(&self) -> u64 { self.senders }
    pub closed spec fn accepting(&self) -> bool { self.accepting }
    pub closed spec fn receiver_alive(&self) -> bool { self.receiver_alive }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.capacity() > 0
        &&& self.available() + self.reserved() + self.queued() == self.capacity()
        &&& self.accepting() ==> self.receiver_alive()
    }

    pub fn new(capacity: u64) -> (result: Self)
        requires capacity > 0,
        ensures
            result.well_formed(),
            result.capacity() == capacity,
            result.available() == capacity,
            result.reserved() == 0,
            result.queued() == 0,
            result.senders() == 1,
            result.accepting(),
            result.receiver_alive(),
        no_unwind
    {
        MpscCapacity {
            capacity,
            available: capacity,
            reserved: 0,
            queued: 0,
            senders: 1,
            accepting: true,
            receiver_alive: true,
        }
    }

    pub fn clone_sender(&mut self)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
            old(self).senders() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).senders() == old(self).senders() + 1,
            final(self).available() == old(self).available(),
            final(self).reserved() == old(self).reserved(),
            final(self).queued() == old(self).queued(),
        no_unwind
    {
        self.senders += 1;
    }

    pub fn drop_sender(&mut self) -> (last: bool)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
        ensures
            final(self).well_formed(),
            final(self).senders() + 1 == old(self).senders(),
            final(self).available() == old(self).available(),
            final(self).reserved() == old(self).reserved(),
            final(self).queued() == old(self).queued(),
            last == (old(self).senders() == 1),
        no_unwind
    {
        self.senders -= 1;
        self.senders == 0
    }

    pub fn reserve(&mut self) -> (result: ReserveResult)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            result == ReserveResult::Closed ==>
                (!old(self).accepting() || old(self).senders() == 0),
            result == ReserveResult::Full ==>
                old(self).accepting() && old(self).senders() > 0
                    && old(self).available() == 0,
            result == ReserveResult::Permit ==>
                old(self).accepting() && old(self).senders() > 0
                    && old(self).available() > 0,
            result == ReserveResult::Permit ==>
                final(self).available() + 1 == old(self).available(),
            result == ReserveResult::Permit ==>
                final(self).reserved() == old(self).reserved() + 1,
            result != ReserveResult::Permit ==>
                final(self).available() == old(self).available(),
            result != ReserveResult::Permit ==>
                final(self).reserved() == old(self).reserved(),
            final(self).queued() == old(self).queued(),
            final(self).senders() == old(self).senders(),
            final(self).accepting() == old(self).accepting(),
            final(self).receiver_alive() == old(self).receiver_alive(),
            final(self).capacity() == old(self).capacity(),
        no_unwind
    {
        if !self.accepting || self.senders == 0 {
            ReserveResult::Closed
        } else if self.available == 0 {
            ReserveResult::Full
        } else {
            self.available -= 1;
            self.reserved += 1;
            ReserveResult::Permit
        }
    }

    pub fn commit_permit(&mut self)
        requires
            old(self).well_formed(),
            old(self).reserved() > 0,
        ensures
            final(self).well_formed(),
            final(self).reserved() + 1 == old(self).reserved(),
            final(self).queued() == old(self).queued() + 1,
            final(self).available() == old(self).available(),
            final(self).senders() == old(self).senders(),
            final(self).accepting() == old(self).accepting(),
            final(self).receiver_alive() == old(self).receiver_alive(),
            final(self).capacity() == old(self).capacity(),
        no_unwind
    {
        self.reserved -= 1;
        self.queued += 1;
    }

    pub fn cancel_permit(&mut self)
        requires
            old(self).well_formed(),
            old(self).reserved() > 0,
        ensures
            final(self).well_formed(),
            final(self).reserved() + 1 == old(self).reserved(),
            final(self).available() == old(self).available() + 1,
            final(self).queued() == old(self).queued(),
            final(self).senders() == old(self).senders(),
            final(self).accepting() == old(self).accepting(),
            final(self).receiver_alive() == old(self).receiver_alive(),
            final(self).capacity() == old(self).capacity(),
        no_unwind
    {
        self.reserved -= 1;
        self.available += 1;
    }

    pub fn receive(&mut self) -> (received: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            received == (old(self).queued() > 0),
            received ==> final(self).queued() + 1 == old(self).queued(),
            received ==> final(self).available() == old(self).available() + 1,
            !received ==> final(self).queued() == old(self).queued(),
            !received ==> final(self).available() == old(self).available(),
            final(self).reserved() == old(self).reserved(),
            final(self).senders() == old(self).senders(),
            final(self).accepting() == old(self).accepting(),
            final(self).receiver_alive() == old(self).receiver_alive(),
            final(self).capacity() == old(self).capacity(),
        no_unwind
    {
        if self.queued > 0 {
            self.queued -= 1;
            self.available += 1;
            true
        } else {
            false
        }
    }

    /// Models one production `Semaphore::add_permits(number_added)` after a
    /// preallocated `recv_many` batch. The queued-count precondition connects
    /// the batch to values already removed from the logical queue.
    pub fn receive_many(&mut self, count: u64)
        requires
            old(self).well_formed(),
            count > 0,
            count <= old(self).queued(),
        ensures
            final(self).well_formed(),
            final(self).queued() + count == old(self).queued(),
            final(self).available() == old(self).available() + count,
            final(self).reserved() == old(self).reserved(),
            final(self).senders() == old(self).senders(),
            final(self).accepting() == old(self).accepting(),
            final(self).receiver_alive() == old(self).receiver_alive(),
            final(self).capacity() == old(self).capacity(),
        no_unwind
    {
        self.queued -= count;
        self.available += count;
    }

    /// `Receiver::close`: new reservations stop, but already reserved permits
    /// may still publish and the receiver drains queued values.
    pub fn close_receiver(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            !final(self).accepting(),
            final(self).receiver_alive() == old(self).receiver_alive(),
            final(self).available() == old(self).available(),
            final(self).reserved() == old(self).reserved(),
            final(self).queued() == old(self).queued(),
        no_unwind
    {
        self.accepting = false;
    }

    /// Dropping the unique receiver drains every currently queued value. Any
    /// outstanding permit remains linear and must later commit or cancel.
    pub fn drop_receiver(&mut self) -> (drained: u64)
        requires
            old(self).well_formed(),
            old(self).receiver_alive(),
        ensures
            final(self).well_formed(),
            !final(self).accepting(),
            !final(self).receiver_alive(),
            drained == old(self).queued(),
            final(self).queued() == 0,
            final(self).available() == old(self).available() + old(self).queued(),
            final(self).reserved() == old(self).reserved(),
            final(self).senders() == old(self).senders(),
        no_unwind
    {
        let drained = self.queued;
        self.available += self.queued;
        self.queued = 0;
        self.accepting = false;
        self.receiver_alive = false;
        drained
    }

    pub fn drained_and_closed(&self) -> (result: bool)
        requires self.well_formed(),
        ensures
            result == (self.queued() == 0 && self.reserved() == 0
                && (!self.accepting() || self.senders() == 0)),
        no_unwind
    {
        self.queued == 0 && self.reserved == 0 && (!self.accepting || self.senders == 0)
    }
}

#[derive(PartialEq, Eq)]
pub enum QueueRead<T> {
    Empty,
    Busy,
    Value(T),
    Closed,
}

/// Two consecutive cells of production's block list. It captures the crucial
/// lock-free property: senders claim FIFO indices independently, but the sole
/// receiver cannot pass an earlier claim that is still being written.
pub struct TwoSlotQueue<T> {
    claimed: u8,
    head: u8,
    first: Option<T>,
    second: Option<T>,
    closed_at: Option<u8>,
}

impl<T> TwoSlotQueue<T> {
    pub closed spec fn claimed(&self) -> u8 { self.claimed }
    pub closed spec fn head(&self) -> u8 { self.head }
    pub closed spec fn first(&self) -> Option<T> { self.first }
    pub closed spec fn second(&self) -> Option<T> { self.second }
    pub closed spec fn closed_at(&self) -> Option<u8> { self.closed_at }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.head() <= self.claimed()
        &&& self.claimed() <= 2
        &&& self.first().is_some() ==> self.claimed() >= 1
        &&& self.second().is_some() ==> self.claimed() >= 2
        &&& self.head() >= 1 ==> self.first().is_none()
        &&& self.head() >= 2 ==> self.second().is_none()
        &&& match self.closed_at() {
            Some(position) => position == self.claimed(),
            None => true,
        }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.claimed() == 0,
            result.head() == 0,
            result.first().is_none(),
            result.second().is_none(),
            result.closed_at().is_none(),
        no_unwind
    {
        TwoSlotQueue { claimed: 0, head: 0, first: None, second: None, closed_at: None }
    }

    pub fn claim(&mut self) -> (index: u8)
        requires
            old(self).well_formed(),
            old(self).claimed() < 2,
            old(self).closed_at().is_none(),
        ensures
            final(self).well_formed(),
            index == old(self).claimed(),
            final(self).claimed() == old(self).claimed() + 1,
            final(self).head() == old(self).head(),
            final(self).first() == old(self).first(),
            final(self).second() == old(self).second(),
            final(self).closed_at() == old(self).closed_at(),
        no_unwind
    {
        let index = self.claimed;
        self.claimed += 1;
        index
    }

    pub fn publish(&mut self, index: u8, value: T)
        requires
            old(self).well_formed(),
            index < old(self).claimed(),
            index >= old(self).head(),
            index == 0 ==> old(self).first().is_none(),
            index == 1 ==> old(self).second().is_none(),
        ensures
            final(self).well_formed(),
            final(self).claimed() == old(self).claimed(),
            final(self).head() == old(self).head(),
            index == 0 ==> final(self).first() == Some(value),
            index == 0 ==> final(self).second() == old(self).second(),
            index == 1 ==> final(self).second() == Some(value),
            index == 1 ==> final(self).first() == old(self).first(),
            final(self).closed_at() == old(self).closed_at(),
        no_unwind
    {
        if index == 0 {
            self.first = Some(value);
        } else {
            self.second = Some(value);
        }
    }

    pub fn close(&mut self)
        requires
            old(self).well_formed(),
            old(self).closed_at().is_none(),
        ensures
            final(self).well_formed(),
            final(self).closed_at() == Some(old(self).claimed()),
            final(self).claimed() == old(self).claimed(),
            final(self).head() == old(self).head(),
            final(self).first() == old(self).first(),
            final(self).second() == old(self).second(),
        no_unwind
    {
        self.closed_at = Some(self.claimed);
    }

    pub fn pop(&mut self) -> (result: QueueRead<T>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).claimed() == old(self).claimed(),
            final(self).closed_at() == old(self).closed_at(),
            matches!(result, QueueRead::Busy | QueueRead::Closed | QueueRead::Empty) ==>
                final(self).first() == old(self).first(),
            matches!(result, QueueRead::Busy | QueueRead::Closed | QueueRead::Empty) ==>
                final(self).second() == old(self).second(),
            matches!(result, QueueRead::Value(_)) && old(self).head() == 0 ==>
                final(self).first().is_none() && final(self).second() == old(self).second(),
            matches!(result, QueueRead::Value(_)) && old(self).head() == 1 ==>
                final(self).second().is_none() && final(self).first() == old(self).first(),
            result == if old(self).head() < old(self).claimed() {
                if old(self).head() == 0 {
                    match old(self).first() {
                        Some(value) => QueueRead::Value(value),
                        None => QueueRead::Busy,
                    }
                } else {
                    match old(self).second() {
                        Some(value) => QueueRead::Value(value),
                        None => QueueRead::Busy,
                    }
                }
            } else if old(self).closed_at() == Some(old(self).head()) {
                QueueRead::Closed
            } else {
                QueueRead::Empty
            },
            match result {
                QueueRead::Value(value) => {
                    old(self).head() < old(self).claimed()
                        && final(self).head() == old(self).head() + 1
                        && (old(self).head() == 0 ==> old(self).first() == Some(value))
                        && (old(self).head() == 1 ==> old(self).second() == Some(value))
                },
                QueueRead::Busy => {
                    old(self).head() < old(self).claimed()
                        && final(self).head() == old(self).head()
                },
                QueueRead::Closed => {
                    old(self).closed_at() == Some(old(self).head())
                        && final(self).head() == old(self).head()
                },
                QueueRead::Empty => {
                    old(self).head() == old(self).claimed()
                        && old(self).closed_at().is_none()
                        && final(self).head() == old(self).head()
                },
            },
        no_unwind
    {
        if self.head < self.claimed {
            if self.head == 0 {
                match self.first.take() {
                    Some(value) => {
                        self.head += 1;
                        QueueRead::Value(value)
                    },
                    None => QueueRead::Busy,
                }
            } else {
                match self.second.take() {
                    Some(value) => {
                        self.head += 1;
                        QueueRead::Value(value)
                    },
                    None => QueueRead::Busy,
                }
            }
        } else if self.closed_at == Some(self.head) {
            QueueRead::Closed
        } else {
            QueueRead::Empty
        }
    }
}

/// Registration projection of `Chan::recv`: a producer can publish between
/// the first pop and AtomicWaker registration, therefore the second pop is a
/// required part of the protocol.
pub struct MpscPoll {
    registered: bool,
}

impl MpscPoll {
    pub closed spec fn registered(&self) -> bool { self.registered }

    pub fn new() -> (result: Self)
        ensures !result.registered(),
        no_unwind
    {
        MpscPoll { registered: false }
    }

    pub fn register_after_empty(&mut self)
        requires !old(self).registered(),
        ensures final(self).registered(),
        no_unwind
    {
        self.registered = true;
    }

    pub fn finish_recheck<T>(&mut self, result: &QueueRead<T>) -> (pending: bool)
        requires old(self).registered(),
        ensures
            pending == matches!(result, QueueRead::Empty | QueueRead::Busy),
            pending ==> final(self).registered(),
            !pending ==> !final(self).registered(),
        no_unwind
    {
        match result {
            QueueRead::Empty | QueueRead::Busy => true,
            QueueRead::Value(_) | QueueRead::Closed => {
                self.registered = false;
                false
            },
        }
    }
}

pub fn verify_bounded_permit_roundtrip()
{
    let mut state = MpscCapacity::new(1);
    let reserved = state.reserve();
    assert(reserved == ReserveResult::Permit);
    assert(state.available() == 0);
    state.commit_permit();
    assert(state.queued() == 1);
    let received = state.receive();
    assert(received);
    assert(state.available() == 1);
}

pub fn verify_cancelled_permit_restores_capacity()
{
    let mut state = MpscCapacity::new(2);
    let reserved = state.reserve();
    assert(reserved == ReserveResult::Permit);
    state.cancel_permit();
    assert(state.available() == 2);
    assert(state.reserved() == 0);
}

pub fn verify_fifo_blocks_out_of_order_publication(first: u64, second: u64)
{
    let mut queue = TwoSlotQueue::new();
    let first_index = queue.claim();
    let second_index = queue.claim();
    assert(first_index == 0 && second_index == 1);
    queue.publish(second_index, second);
    let busy = queue.pop();
    assert(busy == QueueRead::Busy);
    queue.publish(first_index, first);
    let first_read = queue.pop();
    assert(first_read == QueueRead::Value(first));
    let second_read = queue.pop();
    assert(second_read == QueueRead::Value(second));
}

pub fn verify_values_before_close_marker(value: u64)
{
    let mut queue = TwoSlotQueue::new();
    let index = queue.claim();
    queue.publish(index, value);
    queue.close();
    let value_read = queue.pop();
    assert(value_read == QueueRead::Value(value));
    let closed = queue.pop();
    assert(closed == QueueRead::Closed);
}

pub fn verify_receiver_close_drains_existing_permit()
{
    let mut state = MpscCapacity::new(1);
    let permit = state.reserve();
    assert(permit == ReserveResult::Permit);
    state.close_receiver();
    state.commit_permit();
    let received = state.receive();
    assert(received);
    let done = state.drained_and_closed();
    assert(done);
}

} // verus!
