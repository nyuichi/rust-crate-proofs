use vstd::prelude::*;

verus! {

pub open spec fn initial_slot_tag_spec(index: u64, capacity: u64) -> u64 {
    vstd::wrapping::u64_specs::wrapping_sub(index, capacity)
}

pub open spec fn advance_capacity_spec(position: u64, capacity: u64) -> u64 {
    vstd::wrapping::u64_specs::wrapping_add(position, capacity)
}

pub open spec fn masked_slot_spec(position: u64, capacity: u64) -> u64 {
    position & ((capacity - 1) as u64)
}

pub fn initial_slot_tag(index: u64, capacity: u64) -> (tag: u64)
    ensures tag == initial_slot_tag_spec(index, capacity),
    no_unwind
{
    index.wrapping_sub(capacity)
}

pub fn advance_capacity(position: u64, capacity: u64) -> (next: u64)
    ensures next == advance_capacity_spec(position, capacity),
    no_unwind
{
    position.wrapping_add(capacity)
}

/// Send and recv use the same power-of-two mask established by channel
/// construction. Position arithmetic itself is monotonic and non-wrapping;
/// only the physical index is masked.
pub fn masked_slot(position: u64, capacity: u64) -> (index: u64)
    requires
        capacity > 0,
        capacity <= 0x8000_0000_0000_0000u64,
        capacity & ((capacity - 1) as u64) == 0,
    ensures
        index == masked_slot_spec(position, capacity),
        index < capacity,
    no_unwind
{
    let mask = capacity - 1;
    let index = position & mask;
    assert((position & mask) <= mask) by (bit_vector);
    assert(index <= mask);
    assert(mask < capacity);
    index
}

pub fn saturating_distance(distance: u64, maximum: u64) -> (result: u64)
    ensures
        distance <= maximum ==> result == distance,
        distance > maximum ==> result == maximum,
        result == 0 <==> distance == 0 || maximum == 0,
    no_unwind
{
    if distance > maximum { maximum } else { distance }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SendDecision {
    NoReceivers,
    Terminal,
    Reserved(u64),
}

/// Tail-lock projection of production send's two pre-mutation gates. The
/// receiver check has priority over terminal exhaustion. `writes` abstracts
/// all slot/rem/value/waiter mutation, so both rejecting outcomes prove that
/// the entire channel state remains unchanged before the input value unwinds.
pub struct TerminalSendGate {
    tail: u64,
    receivers: u64,
    writes: u64,
}

impl TerminalSendGate {
    pub closed spec fn tail(&self) -> u64 { self.tail }
    pub closed spec fn receivers(&self) -> u64 { self.receivers }
    pub closed spec fn writes(&self) -> u64 { self.writes }

    pub fn new(tail: u64, receivers: u64) -> (result: Self)
        ensures
            result.tail() == tail,
            result.receivers() == receivers,
            result.writes() == 0,
        no_unwind
    {
        TerminalSendGate { tail, receivers, writes: 0 }
    }

    pub fn reserve(&mut self) -> (decision: SendDecision)
        requires
            old(self).receivers() > 0 && old(self).tail() < u64::MAX ==>
                old(self).writes() < u64::MAX,
        ensures
            old(self).receivers() == 0 ==> decision == SendDecision::NoReceivers,
            old(self).receivers() == 0 ==> final(self).tail() == old(self).tail(),
            old(self).receivers() == 0 ==> final(self).writes() == old(self).writes(),
            old(self).receivers() > 0 && old(self).tail() == u64::MAX ==>
                decision == SendDecision::Terminal,
            old(self).receivers() > 0 && old(self).tail() == u64::MAX ==>
                final(self).tail() == old(self).tail(),
            old(self).receivers() > 0 && old(self).tail() == u64::MAX ==>
                final(self).writes() == old(self).writes(),
            old(self).receivers() > 0 && old(self).tail() < u64::MAX ==>
                decision == SendDecision::Reserved(old(self).tail()),
            old(self).receivers() > 0 && old(self).tail() < u64::MAX ==>
                final(self).tail() == old(self).tail() + 1,
            old(self).receivers() > 0 && old(self).tail() < u64::MAX ==>
                final(self).writes() == old(self).writes() + 1,
            final(self).receivers() == old(self).receivers(),
        no_unwind
    {
        if self.receivers == 0 {
            SendDecision::NoReceivers
        } else if self.tail == u64::MAX {
            SendDecision::Terminal
        } else {
            let position = self.tail;
            self.tail += 1;
            self.writes += 1;
            SendDecision::Reserved(position)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RingLookup {
    Empty,
    Lagged(u64),
    Ready(u64),
}

/// Monotonic single-receiver projection selected for production. `tail ==
/// u64::MAX` is an irreversible sentinel; no successful send reserves that
/// position, so every reachable cursor satisfies `next <= tail` without wrap.
pub struct BroadcastRing {
    capacity: u64,
    tail: u64,
    next: u64,
}

impl BroadcastRing {
    pub closed spec fn capacity(&self) -> u64 { self.capacity }
    pub closed spec fn tail(&self) -> u64 { self.tail }
    pub closed spec fn next(&self) -> u64 { self.next }
    pub closed spec fn unread(&self) -> int { self.tail() - self.next() }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.capacity() > 0
        &&& self.capacity() <= 0x8000_0000_0000_0000u64
        &&& self.capacity() & ((self.capacity() - 1) as u64) == 0
        &&& self.next() <= self.tail()
    }

    pub fn subscribe(capacity: u64, tail: u64) -> (result: Self)
        requires
            capacity > 0,
            capacity <= 0x8000_0000_0000_0000u64,
            capacity & ((capacity - 1) as u64) == 0,
        ensures
            result.well_formed(),
            result.capacity() == capacity,
            result.tail() == tail,
            result.next() == tail,
            result.unread() == 0,
        no_unwind
    {
        BroadcastRing { capacity, tail, next: tail }
    }

    /// Active-receiver send. `None` is the exact terminal gate and changes no
    /// state; `Some` reserves at most MAX-1 and advances monotonically.
    pub fn reserve_send(&mut self) -> (position: Option<u64>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).capacity() == old(self).capacity(),
            final(self).next() == old(self).next(),
            old(self).tail() == u64::MAX ==> position.is_none(),
            old(self).tail() == u64::MAX ==> final(self).tail() == old(self).tail(),
            old(self).tail() < u64::MAX ==> position == Some(old(self).tail()),
            old(self).tail() < u64::MAX ==> final(self).tail() == old(self).tail() + 1,
            position.is_some() ==> position.unwrap() < u64::MAX,
        no_unwind
    {
        if self.tail == u64::MAX {
            None
        } else {
            let position = self.tail;
            let _slot = masked_slot(position, self.capacity);
            self.tail += 1;
            Some(position)
        }
    }

    pub fn len_with_maximum(&self, maximum: u64) -> (length: u64)
        requires self.well_formed(),
        ensures
            self.unread() <= maximum ==> length == self.unread(),
            self.unread() > maximum ==> length == maximum,
        no_unwind
    {
        let distance = self.tail - self.next;
        saturating_distance(distance, maximum)
    }

    pub fn is_empty(&self) -> (empty: bool)
        requires self.well_formed(),
        ensures empty == (self.unread() == 0),
        no_unwind
    {
        self.next == self.tail
    }

    pub fn catch_up_if_lagged(&mut self) -> (lookup: RingLookup)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).tail() == old(self).tail(),
            final(self).capacity() == old(self).capacity(),
            old(self).unread() == 0 ==> lookup == RingLookup::Empty,
            0 < old(self).unread() <= old(self).capacity() ==>
                lookup == RingLookup::Ready(old(self).next()),
            old(self).unread() <= old(self).capacity() ==>
                final(self).next() == old(self).next(),
            old(self).unread() > old(self).capacity() ==>
                lookup == RingLookup::Lagged(
                    (old(self).unread() - old(self).capacity()) as u64),
            old(self).unread() > old(self).capacity() ==>
                final(self).next() == old(self).tail() - old(self).capacity(),
            old(self).unread() > old(self).capacity() ==>
                final(self).unread() == old(self).capacity(),
        no_unwind
    {
        let unread = self.tail - self.next;
        if unread == 0 {
            RingLookup::Empty
        } else if unread > self.capacity {
            let oldest = self.tail - self.capacity;
            let missed = oldest - self.next;
            self.next = oldest;
            RingLookup::Lagged(missed)
        } else {
            RingLookup::Ready(self.next)
        }
    }

    pub fn receive_ready(&mut self) -> (position: u64)
        requires
            old(self).well_formed(),
            0 < old(self).unread() <= old(self).capacity(),
        ensures
            final(self).well_formed(),
            position == old(self).next(),
            position < u64::MAX,
            final(self).next() == old(self).next() + 1,
            final(self).tail() == old(self).tail(),
            final(self).capacity() == old(self).capacity(),
            final(self).unread() + 1 == old(self).unread(),
            masked_slot_spec(position, old(self).capacity()) < old(self).capacity(),
        no_unwind
    {
        let position = self.next;
        let _slot = masked_slot(position, self.capacity);
        self.next += 1;
        position
    }

    pub fn drain_snapshot(&mut self) -> (released: u64)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).tail() == old(self).tail(),
            final(self).next() == old(self).tail(),
            final(self).unread() == 0,
            old(self).unread() <= old(self).capacity() ==>
                released == old(self).unread(),
            old(self).unread() > old(self).capacity() ==>
                released == old(self).capacity(),
        no_unwind
    {
        let unread = self.tail - self.next;
        if unread > self.capacity {
            self.next = self.tail - self.capacity;
        }
        let initial_retained = self.tail - self.next;
        let mut released = 0u64;
        while self.next < self.tail
            invariant
                self.well_formed(),
                self.capacity == old(self).capacity,
                self.tail == old(self).tail,
                released + self.tail - self.next == initial_retained,
            decreases self.tail - self.next,
        {
            self.next += 1;
            released += 1;
        }
        released
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlotLookup {
    Empty,
    Ready,
    DifferentGeneration,
}

/// Position-tag projection of production's exact equality order. At a
/// reachable terminal tail, the queried slot was most recently published at
/// `MAX - capacity`; its `tag + capacity == next` relation classifies Empty.
pub struct BroadcastGenerationSlot {
    capacity: u64,
    index: u64,
    tag: u64,
    occupied: bool,
}

impl BroadcastGenerationSlot {
    pub closed spec fn capacity(&self) -> u64 { self.capacity }
    pub closed spec fn index(&self) -> u64 { self.index }
    pub closed spec fn tag(&self) -> u64 { self.tag }
    pub closed spec fn occupied(&self) -> bool { self.occupied }

    pub fn initial(index: u64, capacity: u64) -> (result: Self)
        requires capacity > 0, index < capacity,
        ensures
            result.capacity() == capacity,
            result.index() == index,
            result.tag() == initial_slot_tag_spec(index, capacity),
            advance_capacity_spec(result.tag(), capacity) == index,
            !result.occupied(),
        no_unwind
    {
        let tag = initial_slot_tag(index, capacity);
        let initial_next = advance_capacity(tag, capacity);
        assert(initial_next == index);
        BroadcastGenerationSlot { capacity, index, tag, occupied: false }
    }

    pub fn publish(&mut self, position: u64) -> (evicted: bool)
        requires
            position < u64::MAX,
            old(self).capacity() > 0,
            old(self).index() < old(self).capacity(),
            masked_slot_spec(position, old(self).capacity()) == old(self).index(),
        ensures
            final(self).capacity() == old(self).capacity(),
            final(self).index() == old(self).index(),
            final(self).tag() == position,
            final(self).occupied(),
            evicted == old(self).occupied(),
        no_unwind
    {
        let evicted = self.occupied;
        self.tag = position;
        self.occupied = true;
        evicted
    }

    pub fn classify(&self, next: u64, tail: u64) -> (lookup: SlotLookup)
        requires next <= tail,
        ensures
            self.tag() == next ==> lookup == SlotLookup::Ready,
            self.tag() != next
                && advance_capacity_spec(self.tag(), self.capacity()) == next
                ==> lookup == SlotLookup::Empty,
            self.tag() != next
                && advance_capacity_spec(self.tag(), self.capacity()) != next
                ==> lookup == SlotLookup::DifferentGeneration,
        no_unwind
    {
        if self.tag == next {
            SlotLookup::Ready
        } else if advance_capacity(self.tag, self.capacity) == next {
            SlotLookup::Empty
        } else {
            SlotLookup::DifferentGeneration
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DrainStep {
    Done,
    Lagged,
    Released(u64),
}

/// One iteration of Receiver::drop after its tail snapshot. Concurrent sends
/// may increase `observed_tail`, but their tickets are at least `until`. A
/// Ready step releases only the current ticket `< until`; a lag jump releases
/// nothing and may move beyond the snapshot, causing the next step to stop.
pub struct ConcurrentDropDrain {
    next: u64,
    until: u64,
    released: u64,
}

impl ConcurrentDropDrain {
    pub closed spec fn next(&self) -> u64 { self.next }
    pub closed spec fn until(&self) -> u64 { self.until }
    pub closed spec fn released(&self) -> u64 { self.released }

    pub closed spec fn well_formed(&self) -> bool {
        self.released() <= self.next()
    }

    pub fn new(next: u64, until: u64) -> (result: Self)
        requires next <= until,
        ensures
            result.well_formed(),
            result.next() == next,
            result.until() == until,
            result.released() == 0,
        no_unwind
    {
        ConcurrentDropDrain { next, until, released: 0 }
    }

    pub fn step(&mut self, observed_tail: u64, capacity: u64) -> (step: DrainStep)
        requires
            old(self).well_formed(),
            old(self).until() <= observed_tail,
            capacity > 0,
        ensures
            final(self).well_formed(),
            final(self).until() == old(self).until(),
            old(self).next() >= old(self).until() ==> step == DrainStep::Done,
            old(self).next() >= old(self).until() ==>
                final(self).next() == old(self).next(),
            old(self).next() < old(self).until()
                && observed_tail - old(self).next() > capacity ==>
                step == DrainStep::Lagged,
            old(self).next() < old(self).until()
                && observed_tail - old(self).next() <= capacity ==>
                step == DrainStep::Released(old(self).next()),
            step == DrainStep::Done ==> final(self).released() == old(self).released(),
            step == DrainStep::Released(old(self).next()) ==>
                old(self).next() < old(self).until(),
            step == DrainStep::Released(old(self).next()) ==>
                final(self).next() == old(self).next() + 1,
            step == DrainStep::Released(old(self).next()) ==>
                final(self).released() == old(self).released() + 1,
            step == DrainStep::Lagged ==> final(self).next() > old(self).next(),
            step == DrainStep::Lagged ==>
                final(self).next() == observed_tail - capacity,
            step == DrainStep::Lagged ==>
                final(self).released() == old(self).released(),
            step != DrainStep::Done ==> final(self).next() > old(self).next(),
        no_unwind
    {
        if self.next >= self.until {
            DrainStep::Done
        } else if observed_tail - self.next > capacity {
            self.next = observed_tail - capacity;
            DrainStep::Lagged
        } else {
            let ticket = self.next;
            self.next += 1;
            self.released += 1;
            DrainStep::Released(ticket)
        }
    }
}

pub fn verify_terminal_send_gate_priority()
{
    let mut no_receivers = TerminalSendGate::new(u64::MAX, 0);
    let no_rx = no_receivers.reserve();
    assert(no_rx == SendDecision::NoReceivers);
    assert(no_receivers.tail() == u64::MAX);
    assert(no_receivers.writes() == 0);

    let mut terminal = TerminalSendGate::new(u64::MAX, 1);
    let rejected = terminal.reserve();
    assert(rejected == SendDecision::Terminal);
    assert(terminal.tail() == u64::MAX);
    assert(terminal.writes() == 0);
}

pub fn verify_terminal_boundary_send_receive()
{
    assert(2u64 & ((2u64 - 1) as u64) == 0) by (bit_vector);
    let mut ring = BroadcastRing::subscribe(2, u64::MAX - 2);
    let first = ring.reserve_send();
    let second = ring.reserve_send();
    assert(first == Some((u64::MAX - 2) as u64));
    assert(second == Some((u64::MAX - 1) as u64));
    assert(ring.tail() == u64::MAX);
    let terminal = ring.reserve_send();
    assert(terminal.is_none());
    let first_value = ring.receive_ready();
    let second_value = ring.receive_ready();
    assert(first_value == u64::MAX - 2);
    assert(second_value == u64::MAX - 1);
    assert(ring.next() == u64::MAX);
    let empty = ring.is_empty();
    assert(empty);
}

pub fn verify_terminal_len_saturation()
{
    assert(1u64 & ((1u64 - 1) as u64) == 0) by (bit_vector);
    let ring = BroadcastRing { capacity: 1, tail: u64::MAX, next: 0 };
    assert(ring.well_formed());
    let host_32 = ring.len_with_maximum(u32::MAX as u64);
    assert(host_32 == u32::MAX as u64);
    let host_64 = ring.len_with_maximum(u64::MAX);
    assert(host_64 == u64::MAX);
    let empty = ring.is_empty();
    assert(!empty);
}

pub fn verify_terminal_queried_slot_is_empty()
{
    let mut slot = BroadcastGenerationSlot::initial(1, 2);
    assert(((u64::MAX - 2) as u64) & 1u64 == 1) by (bit_vector);
    slot.publish(u64::MAX - 2);
    assert(slot.tag() == u64::MAX - 2);
    let terminal = slot.classify(u64::MAX, u64::MAX);
    assert(terminal == SlotLookup::Empty);
}

pub fn verify_positive_lag_and_snapshot_bound()
{
    let mut drain = ConcurrentDropDrain::new(2, 4);
    let lag = drain.step(7, 2);
    assert(lag == DrainStep::Lagged);
    assert(drain.next() == 5);
    assert(drain.released() == 0);
    let done = drain.step(8, 2);
    assert(done == DrainStep::Done);
    assert(drain.released() == 0);
}

} // verus!
