use vstd::prelude::*;

verus! {

pub open spec fn position_distance_spec(next: u64, tail: u64) -> u64 {
    vstd::wrapping::u64_specs::wrapping_sub(tail, next)
}

pub open spec fn advance_position_spec(position: u64) -> u64 {
    vstd::wrapping::u64_specs::wrapping_add(position, 1)
}

pub open spec fn oldest_position_spec(tail: u64, capacity: u64) -> u64 {
    vstd::wrapping::u64_specs::wrapping_sub(tail, capacity)
}

pub open spec fn advance_capacity_spec(position: u64, capacity: u64) -> u64 {
    vstd::wrapping::u64_specs::wrapping_add(position, capacity)
}

pub open spec fn masked_slot_spec(position: u64, capacity: u64) -> u64 {
    position & ((capacity - 1) as u64)
}

/// Production uses `u64` positions modulo 2^64.  This adapter keeps the
/// single-cycle distance explicit: equality means empty only while the number
/// of unobserved sends is strictly less than a complete `u64` cycle.
pub fn position_distance(next: u64, tail: u64) -> (distance: u64)
    ensures
        distance == position_distance_spec(next, tail),
        tail >= next ==> distance == tail - next,
        tail < next ==> distance == u64::MAX - next + tail + 1,
        distance == 0 <==> next == tail,
    no_unwind
{
    let distance = tail.wrapping_sub(next);
    assert(tail >= next ==> distance == tail - next);
    assert(tail < next ==> distance == u64::MAX - next + tail + 1);
    assert(distance == 0 <==> next == tail);
    distance
}

/// Exact production `wrapping_add(1)` position advance.
pub fn advance_position(position: u64) -> (next: u64)
    ensures
        next == advance_position_spec(position),
        position == u64::MAX ==> next == 0,
        position < u64::MAX ==> next == position + 1,
    no_unwind
{
    let next = position.wrapping_add(1);
    assert(position == u64::MAX ==> next == 0);
    assert(position < u64::MAX ==> next == position + 1);
    next
}

/// Exact production `tail.pos.wrapping_sub(capacity)` oldest-position adapter.
pub fn oldest_position(tail: u64, capacity: u64) -> (oldest: u64)
    ensures
        tail >= capacity ==> oldest == tail - capacity,
        tail < capacity ==> oldest == u64::MAX - capacity + tail + 1,
        oldest == oldest_position_spec(tail, capacity),
        position_distance_spec(oldest, tail) == capacity,
    no_unwind
{
    let oldest = tail.wrapping_sub(capacity);
    assert(tail >= capacity ==> oldest == tail - capacity);
    assert(tail < capacity ==> oldest == u64::MAX - capacity + tail + 1);
    let distance = position_distance(oldest, tail);
    assert(distance == capacity);
    oldest
}

pub fn advance_capacity(position: u64, capacity: u64) -> (next: u64)
    ensures next == advance_capacity_spec(position, capacity),
    no_unwind
{
    position.wrapping_add(capacity)
}

/// Exact mask operation used by both send and recv_ref.  The power-of-two
/// premise is the property established by `new_with_receiver_count` after
/// `next_power_of_two`.
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RingLookup {
    Empty,
    Lagged(u64),
    Ready(u64),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlotLookup {
    Empty,
    Ready,
    DifferentGeneration,
}

/// Position-tag projection of production `Slot<T>`. Payload ownership and
/// `rem` conservation are already body-proved in `broadcast_protocol`; this
/// type adds the production initial tag and generation comparison.
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

    /// Production initializes slot `i` with `i.wrapping_sub(capacity)`. Adding
    /// capacity therefore makes the initial receiver cursor `i` classify the
    /// slot as empty, not as a published value.
    pub fn initial(index: u64, capacity: u64) -> (result: Self)
        requires
            capacity > 0,
            index < capacity,
        ensures
            result.capacity() == capacity,
            result.index() == index,
            result.tag() == oldest_position_spec(index, capacity),
            advance_capacity_spec(result.tag(), capacity) == index,
            !result.occupied(),
        no_unwind
    {
        let tag = oldest_position(index, capacity);
        let initial_next = advance_capacity(tag, capacity);
        assert(initial_next == index);
        BroadcastGenerationSlot { capacity, index, tag, occupied: false }
    }

    /// Send selects this physical slot with the mask before writing the exact
    /// reservation ticket into `Slot::pos`. `evicted` records whether a prior
    /// generation occupied the slot; value destruction remains the frozen Drop
    /// boundary composed with `BroadcastSlot::publish`.
    pub fn publish(&mut self, position: u64) -> (evicted: bool)
        requires
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

    /// Exact position tests from `recv_ref`: tag equality is ready; otherwise
    /// `tag + capacity == receiver.next` is empty; every other tag is a
    /// different generation and enters locked lag classification.
    pub fn classify(&self, next: u64) -> (lookup: SlotLookup)
        ensures
            lookup == SlotLookup::Ready <==> self.tag() == next,
            lookup == SlotLookup::Empty <==>
                self.tag() != next
                    && advance_capacity_spec(self.tag(), self.capacity()) == next,
            lookup == SlotLookup::DifferentGeneration <==>
                self.tag() != next
                    && advance_capacity_spec(self.tag(), self.capacity()) != next,
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

/// A production-shaped single-receiver projection. `unread` is the canonical
/// progress measure and records the distance since the receiver was last
/// observed.  Excluding `u64::MAX + 1` unobserved sends is intentional: at a
/// complete cycle the production representation has an ABA collision.
pub struct BroadcastRing {
    capacity: u64,
    tail: u64,
    next: u64,
    unread: u64,
}

impl BroadcastRing {
    pub closed spec fn capacity(&self) -> u64 { self.capacity }
    pub closed spec fn tail(&self) -> u64 { self.tail }
    pub closed spec fn next(&self) -> u64 { self.next }
    pub closed spec fn unread(&self) -> u64 { self.unread }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.capacity() > 0
        &&& self.capacity() <= 0x8000_0000_0000_0000u64
        &&& self.capacity() & ((self.capacity() - 1) as u64) == 0
        &&& position_distance_spec(self.next(), self.tail()) == self.unread()
    }

    /// `subscribe` snapshots `tail.pos`, so it cannot receive a value that was
    /// reserved before the tail lock snapshot.
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
        let distance = position_distance(tail, tail);
        BroadcastRing { capacity, tail, next: tail, unread: distance }
    }

    /// Mirrors send's tail reservation. The precondition is an explicit proof
    /// condition for a candidate finite-window policy; it is not a claim that
    /// production enforces or that the user has selected that policy.
    pub fn reserve_send(&mut self) -> (position: u64)
        requires
            old(self).well_formed(),
            old(self).unread() < u64::MAX,
        ensures
            final(self).well_formed(),
            position == old(self).tail(),
            final(self).capacity() == old(self).capacity(),
            final(self).tail() == advance_position_spec(old(self).tail()),
            final(self).next() == old(self).next(),
            final(self).unread() == old(self).unread() + 1,
            masked_slot_spec(position, old(self).capacity()) < old(self).capacity(),
        no_unwind
    {
        let position = self.tail;
        let _slot = masked_slot(position, self.capacity);
        self.tail = advance_position(self.tail);
        self.unread += 1;
        let distance = position_distance(self.next, self.tail);
        assert(distance == self.unread);
        position
    }

    pub fn is_empty(&self) -> (empty: bool)
        requires self.well_formed(),
        ensures empty == (self.unread() == 0),
        no_unwind
    {
        self.next == self.tail
    }

    /// The lag branch corresponds to `tail.pos.wrapping_sub(capacity)` and its
    /// `missed = next.wrapping_sub(receiver.next)`. It moves to exactly the
    /// oldest retained generation and reports every skipped generation once.
    pub fn catch_up_if_lagged(&mut self) -> (lookup: RingLookup)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).tail() == old(self).tail(),
            final(self).capacity() == old(self).capacity(),
            old(self).unread() == 0 ==> lookup == RingLookup::Empty,
            old(self).unread() == 0 ==> final(self).next() == old(self).next(),
            old(self).unread() > 0 && old(self).unread() <= old(self).capacity() ==>
                lookup == RingLookup::Ready(old(self).next()),
            old(self).unread() <= old(self).capacity() ==>
                final(self).next() == old(self).next(),
            old(self).unread() > old(self).capacity() ==>
                lookup == RingLookup::Lagged(
                    (old(self).unread() - old(self).capacity()) as u64),
            old(self).unread() > old(self).capacity() ==>
                final(self).unread() == old(self).capacity(),
            old(self).unread() > old(self).capacity() ==>
                final(self).next() == oldest_position_spec(
                    old(self).tail(), old(self).capacity()),
        no_unwind
    {
        if self.unread == 0 {
            RingLookup::Empty
        } else if self.unread > self.capacity {
            let missed = self.unread - self.capacity;
            self.next = oldest_position(self.tail, self.capacity);
            self.unread = self.capacity;
            RingLookup::Lagged(missed)
        } else {
            RingLookup::Ready(self.next)
        }
    }

    /// Consuming a ready generation advances the modulo cursor exactly once.
    pub fn receive_ready(&mut self) -> (position: u64)
        requires
            old(self).well_formed(),
            old(self).unread() > 0,
            old(self).unread() <= old(self).capacity(),
        ensures
            final(self).well_formed(),
            position == old(self).next(),
            final(self).next() == advance_position_spec(old(self).next()),
            final(self).tail() == old(self).tail(),
            final(self).unread() + 1 == old(self).unread(),
            masked_slot_spec(position, old(self).capacity()) < old(self).capacity(),
        no_unwind
    {
        let position = self.next;
        let _slot = masked_slot(position, self.capacity);
        self.next = advance_position(self.next);
        self.unread -= 1;
        let distance = position_distance(self.next, self.tail);
        assert(distance == self.unread);
        position
    }

    /// Correct sequential bounded snapshot drain shape. Lagged, overwritten
    /// generations have already released their values on the send side, so
    /// this Receiver releases only the retained `min(unread, capacity)` slots.
    /// Progress is that retained count, not an ordering comparison between
    /// wrapping positions.
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
        if self.unread > self.capacity {
            self.next = oldest_position(self.tail, self.capacity);
            self.unread = self.capacity;
        }
        let initial_retained = self.unread;
        let mut released = 0u64;
        while self.unread > 0
            invariant
                self.well_formed(),
                self.capacity == old(self).capacity,
                self.tail == old(self).tail,
                released + self.unread == initial_retained,
            decreases self.unread,
        {
            self.next = advance_position(self.next);
            self.unread -= 1;
            released += 1;
            let distance = position_distance(self.next, self.tail);
            assert(distance == self.unread);
        }
        released
    }
}

pub fn verify_send_receive_across_wrap()
{
    assert(2u64 & ((2u64 - 1) as u64) == 0) by (bit_vector);
    let mut ring = BroadcastRing::subscribe(2, u64::MAX);
    let first = ring.reserve_send();
    assert(first == u64::MAX);
    assert(ring.tail() == 0);
    let nonempty = ring.is_empty();
    assert(!nonempty);
    let ready = ring.catch_up_if_lagged();
    assert(ready == RingLookup::Ready(u64::MAX));
    let received = ring.receive_ready();
    assert(received == u64::MAX);
    assert(ring.next() == 0);
    let empty = ring.is_empty();
    assert(empty);
}

pub fn verify_initial_and_overwritten_slot_generation_across_wrap()
{
    let mut slot = BroadcastGenerationSlot::initial(0, 2);
    assert(slot.tag() == u64::MAX - 1);
    let empty = slot.classify(0);
    assert(empty == SlotLookup::Empty);

    // `u64::MAX - 1` and zero are two send tickets exactly one capacity
    // apart. They map to index zero on opposite sides of the u64 wrap, while
    // the stored tag distinguishes which generation is currently published.
    assert(((u64::MAX - 1) as u64) & 1u64 == 0) by (bit_vector);
    let first_evicted = slot.publish(u64::MAX - 1);
    assert(!first_evicted);
    let first = slot.classify(u64::MAX - 1);
    assert(first == SlotLookup::Ready);

    assert(0u64 & 1u64 == 0) by (bit_vector);
    let second_evicted = slot.publish(0);
    assert(second_evicted);
    assert(slot.tag() == 0);
    let stale = slot.classify(u64::MAX - 1);
    assert(stale == SlotLookup::DifferentGeneration);
    let second = slot.classify(0);
    assert(second == SlotLookup::Ready);
}

pub fn verify_lag_recovery_across_wrap()
{
    assert(2u64 & ((2u64 - 1) as u64) == 0) by (bit_vector);
    let mut ring = BroadcastRing::subscribe(2, u64::MAX - 2);
    ring.reserve_send();
    ring.reserve_send();
    ring.reserve_send();
    assert(ring.tail() == 0);
    assert(ring.unread() == 3);
    assert(ring.capacity() == 2);
    let lagged = ring.catch_up_if_lagged();
    assert((3u64 - 2u64) as u64 == 1);
    assert(lagged == RingLookup::Lagged(1));
    assert(ring.next() == u64::MAX - 1);
    let first = ring.receive_ready();
    let second = ring.receive_ready();
    assert(first == u64::MAX - 1);
    assert(second == u64::MAX);
    let empty = ring.is_empty();
    assert(empty);
}

pub fn verify_snapshot_drain_across_wrap()
{
    assert(2u64 & ((2u64 - 1) as u64) == 0) by (bit_vector);
    let mut ring = BroadcastRing::subscribe(2, u64::MAX - 1);
    ring.reserve_send();
    ring.reserve_send();
    assert(ring.tail() == 0);
    assert(ring.unread() == 2);
    assert(ring.capacity() == 2);
    let released = ring.drain_snapshot();
    assert(released == 2);
    assert(ring.next() == 0);
    let empty = ring.is_empty();
    assert(empty);
}

} // verus!
