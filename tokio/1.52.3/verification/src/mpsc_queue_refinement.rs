use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RawRead {
    Empty,
    Busy,
    Value,
    Closed,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RawSlotPhase { Claimed, Ready, Consumed }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TraversalLeasePhase { TailOwned, Released, Detached, Reused, Freed }

/// Linear sender/receiver handoff represented by production
/// `observed_tail_position` plus the RELEASED ready bit. A raw block cannot be
/// detached merely because the logical cursor passed it: the tail-advance
/// release must first publish its sender-access boundary, and the receiver must
/// acquire that release and reach the recorded position.
pub tracked struct MpscTraversalLease {
    ghost block_generation: nat,
    ghost observed_tail_position: nat,
    ghost receiver_index: nat,
    ghost phase: TraversalLeasePhase,
}

impl MpscTraversalLease {
    pub closed spec fn block_generation(&self) -> nat { self.block_generation }
    pub closed spec fn observed_tail_position(&self) -> nat {
        self.observed_tail_position
    }
    pub closed spec fn receiver_index(&self) -> nat { self.receiver_index }
    pub closed spec fn phase(&self) -> TraversalLeasePhase { self.phase }

    pub proof fn tail_owned(block_generation: nat) -> (tracked result: Self)
        ensures result.block_generation() == block_generation,
            result.observed_tail_position() == 0,
            result.receiver_index() == 0,
            result.phase() == TraversalLeasePhase::TailOwned,
    {
        let tracked result = MpscTraversalLease {
            block_generation, observed_tail_position: 0, receiver_index: 0,
            phase: TraversalLeasePhase::TailOwned,
        };
        result
    }

    /// Successful `block_tail` CAS followed by the Release zero-add snapshot
    /// and `Block::tx_release` write/RELEASED publication.
    pub proof fn tx_release(tracked &mut self, observed_tail_position: nat)
        requires old(self).phase() == TraversalLeasePhase::TailOwned,
        ensures final(self).phase() == TraversalLeasePhase::Released,
            final(self).block_generation() == old(self).block_generation(),
            final(self).observed_tail_position() == observed_tail_position,
            final(self).receiver_index() == old(self).receiver_index(),
    {
        self.observed_tail_position = observed_tail_position;
        self.phase = TraversalLeasePhase::Released;
    }

    /// `Rx::reclaim_blocks` acquires RELEASED, reads the observation, and only
    /// detaches after its unique cursor reaches that boundary.
    pub proof fn receiver_detach(tracked &mut self, receiver_index: nat)
        requires old(self).phase() == TraversalLeasePhase::Released,
            old(self).observed_tail_position() <= receiver_index,
        ensures final(self).phase() == TraversalLeasePhase::Detached,
            final(self).block_generation() == old(self).block_generation(),
            final(self).observed_tail_position() == old(self).observed_tail_position(),
            final(self).receiver_index() == receiver_index,
    {
        self.receiver_index = receiver_index;
        self.phase = TraversalLeasePhase::Detached;
    }

    /// The detached unique pointer is consumed exactly once, either by one of
    /// the bounded reinsert attempts or by `Box::from_raw`.
    pub proof fn reuse(tracked &mut self, next_generation: nat)
        requires old(self).phase() == TraversalLeasePhase::Detached,
            next_generation > old(self).block_generation(),
        ensures final(self).phase() == TraversalLeasePhase::Reused,
            final(self).block_generation() == next_generation,
    {
        self.block_generation = next_generation;
        self.phase = TraversalLeasePhase::Reused;
    }

    pub proof fn free(tracked &mut self)
        requires old(self).phase() == TraversalLeasePhase::Detached,
        ensures final(self).phase() == TraversalLeasePhase::Freed,
            final(self).block_generation() == old(self).block_generation(),
    {
        self.phase = TraversalLeasePhase::Freed;
    }
}

/// Linear permission for the `MaybeUninit<T>` cell selected by a raw queue
/// claim. It is separate from the queue topology so arbitrary payloads do not
/// enlarge the list traversal VCs.
pub tracked struct RawValueSlot<T> {
    ghost index: nat,
    ghost value: Option<T>,
    ghost phase: RawSlotPhase,
}

impl<T> RawValueSlot<T> {
    pub closed spec fn index(&self) -> nat { self.index }
    pub closed spec fn value(&self) -> Option<T> { self.value }
    pub closed spec fn phase(&self) -> RawSlotPhase { self.phase }
    pub closed spec fn well_formed(&self) -> bool {
        (self.phase() == RawSlotPhase::Ready) == self.value().is_some()
    }

    pub proof fn claimed(index: nat) -> (tracked result: Self)
        ensures result.well_formed(), result.index() == index,
            result.phase() == RawSlotPhase::Claimed, result.value().is_none(),
    {
        let tracked result = RawValueSlot {
            index, value: None, phase: RawSlotPhase::Claimed,
        };
        result
    }

    pub proof fn publish(tracked &mut self, value: T)
        requires old(self).well_formed(), old(self).phase() == RawSlotPhase::Claimed,
        ensures final(self).well_formed(), final(self).index() == old(self).index(),
            final(self).phase() == RawSlotPhase::Ready,
            final(self).value() == Some(value),
    {
        self.value = Some(value);
        self.phase = RawSlotPhase::Ready;
    }

    pub proof fn consume(tracked &mut self) -> (result: T)
        requires old(self).well_formed(), old(self).phase() == RawSlotPhase::Ready,
        ensures final(self).well_formed(), final(self).index() == old(self).index(),
            final(self).phase() == RawSlotPhase::Consumed,
            final(self).value().is_none(), old(self).value() == Some(result),
    {
        let value = match self.value {
            Some(value) => value,
            None => proof_from_false(),
        };
        self.value = None;
        self.phase = RawSlotPhase::Consumed;
        value
    }
}

pub open spec fn supported_block_capacity(cap: usize) -> bool {
    cap == 2 || cap == 16 || cap == 32
}

/// Arithmetic used by production `block::{start_index,offset,try_push,grow}`.
/// The absolute logical generation is kept by `RawMpscQueue`; this type is the
/// exact erased `usize` representation stored in each production block.
pub struct MpscBlockGeometry {
    capacity: usize,
    mask: usize,
}

impl MpscBlockGeometry {
    pub closed spec fn capacity(&self) -> usize { self.capacity }
    pub closed spec fn mask(&self) -> usize { self.mask }
    pub closed spec fn well_formed(&self) -> bool {
        &&& supported_block_capacity(self.capacity())
        &&& self.mask() + 1 == self.capacity()
    }
    pub open spec fn offset_spec(&self, index: usize) -> usize {
        index & self.mask()
    }

    pub fn new(capacity: usize) -> (result: Self)
        requires supported_block_capacity(capacity),
        ensures result.well_formed(), result.capacity() == capacity,
            result.mask() == capacity - 1,
        no_unwind
    {
        MpscBlockGeometry { capacity, mask: capacity - 1 }
    }

    pub fn offset(&self, index: usize) -> (result: usize)
        requires self.well_formed(),
        ensures result == self.offset_spec(index), result < self.capacity(),
        no_unwind
    {
        let capacity = self.capacity;
        let mask = self.mask;
        let result = index & mask;
        assert(result < capacity) by (bit_vector)
            requires
                capacity == 2 || capacity == 16 || capacity == 32,
                mask + 1 == capacity,
                result == index & mask;
        result
    }

    pub fn start_index(&self, index: usize) -> (result: usize)
        requires self.well_formed(),
        ensures result == index & !self.mask(),
            result | self.offset_spec(index) == index,
            self.offset_spec(result) == 0,
        no_unwind
    {
        let mask = self.mask;
        let result = index & !mask;
        assert(mask == 1 || mask == 15 || mask == 31);
        assert((result | (index & mask)) == index) by (bit_vector)
            requires mask == 1 || mask == 15 || mask == 31,
                result == index & !mask;
        assert((result & mask) == 0) by (bit_vector)
            requires result == index & !mask;
        result
    }

    /// Both `Block::try_push` and `Block::grow` advance a block generation
    /// modulo the machine word. Production must use explicit wrapping in both
    /// debug and release configurations.
    pub fn next_start(&self, start: usize) -> (result: usize)
        requires self.well_formed(), self.offset_spec(start) == 0,
        ensures result == start.wrapping_add(self.capacity()),
        no_unwind
    {
        start.wrapping_add(self.capacity)
    }

    /// Exact overflow-safe membership test for `Block::has_value`; subtraction
    /// is used instead of an overflowing `start + BLOCK_CAP` upper bound.
    pub fn contains(&self, start: usize, slot: usize) -> (result: bool)
        requires self.well_formed(), self.offset_spec(start) == 0,
        ensures result == (slot.wrapping_sub(start) < self.capacity()),
        no_unwind
    {
        slot.wrapping_sub(start) < self.capacity
    }
}

/// Exact `BlockHeader::ready_slots` layout for loom/32/64-bit block sizes.
/// Value slots occupy bits `0..BLOCK_CAP`, followed by RELEASED and TX_CLOSED.
pub struct MpscReadyBits {
    capacity: usize,
    bits: u64,
}

impl MpscReadyBits {
    pub closed spec fn capacity(&self) -> usize { self.capacity }
    pub closed spec fn bits(&self) -> u64 { self.bits }
    pub open spec fn released_mask(&self) -> u64 {
        1u64 << self.capacity()
    }
    pub open spec fn closed_mask(&self) -> u64 {
        1u64 << (self.capacity() + 1)
    }
    pub open spec fn ready(&self, slot: usize) -> bool {
        self.bits() & (1u64 << slot) == (1u64 << slot)
    }
    pub open spec fn released(&self) -> bool {
        self.bits() & self.released_mask() == self.released_mask()
    }
    pub open spec fn tx_closed(&self) -> bool {
        self.bits() & self.closed_mask() == self.closed_mask()
    }

    pub fn new(capacity: usize) -> (result: Self)
        requires supported_block_capacity(capacity),
        ensures result.capacity() == capacity, result.bits() == 0,
        no_unwind
    {
        MpscReadyBits { capacity, bits: 0 }
    }

    pub fn set_ready(&mut self, slot: usize)
        requires supported_block_capacity(old(self).capacity()),
            slot < old(self).capacity(), !old(self).ready(slot),
        ensures final(self).capacity() == old(self).capacity(),
            final(self).bits() == old(self).bits() | (1u64 << slot),
        no_unwind
    {
        self.bits |= 1u64 << slot;
    }

    pub fn release(&mut self)
        requires supported_block_capacity(old(self).capacity()),
        ensures final(self).capacity() == old(self).capacity(),
            final(self).bits() == old(self).bits() | old(self).released_mask(),
        no_unwind
    {
        self.bits |= 1u64 << self.capacity;
    }

    pub fn close(&mut self)
        requires supported_block_capacity(old(self).capacity()),
        ensures final(self).capacity() == old(self).capacity(),
            final(self).bits() == old(self).bits() | old(self).closed_mask(),
        no_unwind
    {
        self.bits |= 1u64 << (self.capacity + 1);
    }
}

pub proof fn ready_flag_masks_are_disjoint(capacity: usize, slot: usize)
    requires supported_block_capacity(capacity), slot < capacity,
    ensures
        (1u64 << slot) & (1u64 << capacity) == 0,
        (1u64 << slot) & (1u64 << (capacity + 1)) == 0,
        (1u64 << capacity) & (1u64 << (capacity + 1)) == 0,
{
    if capacity == 2 {
        assert((1u64 << slot) & (1u64 << 2) == 0) by (bit_vector)
            requires slot < 2;
        assert((1u64 << slot) & (1u64 << 3) == 0) by (bit_vector)
            requires slot < 2;
    } else if capacity == 16 {
        assert((1u64 << slot) & (1u64 << 16) == 0) by (bit_vector)
            requires slot < 16;
        assert((1u64 << slot) & (1u64 << 17) == 0) by (bit_vector)
            requires slot < 16;
    } else {
        assert(capacity == 32);
        assert((1u64 << slot) & (1u64 << 32) == 0) by (bit_vector)
            requires slot < 32;
        assert((1u64 << slot) & (1u64 << 33) == 0) by (bit_vector)
            requires slot < 32;
    }
    assert((1u64 << capacity) & (1u64 << (capacity + 1)) == 0) by (bit_vector)
        requires capacity == 2 || capacity == 16 || capacity == 32;
}

/// Channel-specific ownership and classification of the production raw list.
/// Allocation and pointer dereference are frozen foundations, but no list
/// result is supplied as an oracle: `observe_head` derives Value/Closed/Busy/
/// Empty from claimed slots, ready bits, the close bit, and the Rx cursor.
pub tracked struct RawMpscQueue {
    ghost claimed_values: Set<nat>,
    ghost ready_values: Set<nat>,
    ghost consumed_values: Set<nat>,
    ghost head: nat,
    ghost tail: nat,
    ghost close_slot: Option<nat>,
    ghost close_ready: bool,
    ghost released_blocks: Set<nat>,
    ghost reclaimed_blocks: Set<nat>,
    ghost block_capacity: nat,
    ghost wake_count: nat,
}

impl RawMpscQueue {
    pub closed spec fn claimed_values(&self) -> Set<nat> { self.claimed_values }
    pub closed spec fn ready_values(&self) -> Set<nat> { self.ready_values }
    pub closed spec fn consumed_values(&self) -> Set<nat> { self.consumed_values }
    pub closed spec fn head(&self) -> nat { self.head }
    pub closed spec fn tail(&self) -> nat { self.tail }
    pub closed spec fn close_slot(&self) -> Option<nat> { self.close_slot }
    pub closed spec fn close_ready(&self) -> bool { self.close_ready }
    pub closed spec fn released_blocks(&self) -> Set<nat> { self.released_blocks }
    pub closed spec fn reclaimed_blocks(&self) -> Set<nat> { self.reclaimed_blocks }
    pub closed spec fn block_capacity(&self) -> nat { self.block_capacity }
    pub closed spec fn wake_count(&self) -> nat { self.wake_count }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.block_capacity() > 0
        &&& self.head() <= self.tail()
        &&& self.ready_values().subset_of(self.claimed_values())
        &&& self.consumed_values().subset_of(self.claimed_values())
        &&& self.ready_values().disjoint(self.consumed_values())
        &&& forall|i: nat| self.claimed_values().contains(i) ==> i < self.tail()
        &&& forall|i: nat| self.consumed_values().contains(i) ==> i < self.head()
        &&& match self.close_slot() {
            Some(i) => i + 1 == self.tail()
                && !self.claimed_values().contains(i),
            None => !self.close_ready(),
        }
        &&& self.close_ready() ==> self.close_slot().is_some()
        &&& self.reclaimed_blocks().subset_of(self.released_blocks())
    }

    pub proof fn new(block_capacity: nat) -> (tracked result: Self)
        requires block_capacity > 0,
        ensures result.well_formed(), result.head() == 0, result.tail() == 0,
            result.claimed_values().is_empty(), result.ready_values().is_empty(),
            result.consumed_values().is_empty(), result.close_slot().is_none(),
            !result.close_ready(), result.released_blocks().is_empty(),
            result.reclaimed_blocks().is_empty(), result.wake_count() == 0,
    {
        let tracked result = RawMpscQueue {
            claimed_values: Set::empty(), ready_values: Set::empty(),
            consumed_values: Set::empty(), head: 0, tail: 0,
            close_slot: None, close_ready: false,
            released_blocks: Set::empty(), reclaimed_blocks: Set::empty(),
            block_capacity, wake_count: 0,
        };
        result
    }

    /// `tail_position.fetch_add(1)` for a value. The returned logical index is
    /// unique even when its erased `usize` representation cycles.
    pub proof fn claim_value(tracked &mut self) -> (index: nat)
        requires old(self).well_formed(), old(self).close_slot().is_none(),
        ensures final(self).well_formed(), index == old(self).tail(),
            final(self).tail() == old(self).tail() + 1,
            final(self).claimed_values() == old(self).claimed_values().insert(index),
            final(self).ready_values() == old(self).ready_values(),
            final(self).consumed_values() == old(self).consumed_values(),
            final(self).head() == old(self).head(),
            final(self).close_slot().is_none(),
            !final(self).close_ready(),
    {
        let index = self.tail;
        self.tail = self.tail + 1;
        self.claimed_values = self.claimed_values.insert(index);
        index
    }

    /// Release publication of a written `Values<T>` cell followed by its ready
    /// bit. The raw payload permission is indexed by the same logical slot.
    pub proof fn publish_value(tracked &mut self, index: nat)
        requires old(self).well_formed(),
            old(self).claimed_values().contains(index),
            !old(self).ready_values().contains(index),
            !old(self).consumed_values().contains(index),
        ensures final(self).well_formed(),
            final(self).ready_values() == old(self).ready_values().insert(index),
            final(self).claimed_values() == old(self).claimed_values(),
            final(self).head() == old(self).head(), final(self).tail() == old(self).tail(),
            final(self).consumed_values() == old(self).consumed_values(),
            final(self).close_slot() == old(self).close_slot(),
            final(self).close_ready() == old(self).close_ready(),
    {
        self.ready_values = self.ready_values.insert(index);
    }

    pub proof fn publish_slot<T>(
        tracked &mut self,
        tracked slot: &mut RawValueSlot<T>,
        value: T,
    )
        requires old(self).well_formed(), old(slot).well_formed(),
            old(slot).phase() == RawSlotPhase::Claimed,
            old(self).claimed_values().contains(old(slot).index()),
            !old(self).ready_values().contains(old(slot).index()),
            !old(self).consumed_values().contains(old(slot).index()),
        ensures final(self).well_formed(), final(slot).well_formed(),
            final(slot).index() == old(slot).index(),
            final(slot).phase() == RawSlotPhase::Ready,
            final(slot).value() == Some(value),
            final(self).ready_values()
                == old(self).ready_values().insert(old(slot).index()),
            final(self).head() == old(self).head(), final(self).tail() == old(self).tail(),
            final(self).claimed_values() == old(self).claimed_values(),
            final(self).consumed_values() == old(self).consumed_values(),
            final(self).close_slot() == old(self).close_slot(),
            final(self).close_ready() == old(self).close_ready(),
    {
        let index = slot.index;
        slot.publish(value);
        self.publish_value(index);
    }

    /// The final strong sender claims the unique next slot before setting the
    /// block's `TX_CLOSED` flag. No subsequent value claim is permitted.
    pub proof fn claim_close(tracked &mut self) -> (index: nat)
        requires old(self).well_formed(), old(self).close_slot().is_none(),
        ensures final(self).well_formed(), index == old(self).tail(),
            final(self).tail() == old(self).tail() + 1,
            final(self).close_slot() == Some(index), !final(self).close_ready(),
            final(self).claimed_values() == old(self).claimed_values(),
            final(self).ready_values() == old(self).ready_values(),
            final(self).head() == old(self).head(),
    {
        let index = self.tail;
        self.tail = self.tail + 1;
        self.close_slot = Some(index);
        index
    }

    pub proof fn publish_close(tracked &mut self)
        requires old(self).well_formed(), old(self).close_slot().is_some(),
            !old(self).close_ready(),
        ensures final(self).well_formed(), final(self).close_ready(),
            final(self).close_slot() == old(self).close_slot(),
            final(self).head() == old(self).head(), final(self).tail() == old(self).tail(),
            final(self).claimed_values() == old(self).claimed_values(),
            final(self).ready_values() == old(self).ready_values(),
            final(self).consumed_values() == old(self).consumed_values(),
    {
        self.close_ready = true;
    }

    pub open spec fn observe_head(&self) -> RawRead {
        if self.ready_values().contains(self.head()) {
            RawRead::Value
        } else if self.close_ready() && self.close_slot() == Some(self.head()) {
            RawRead::Closed
        } else if self.tail() == self.head() {
            RawRead::Empty
        } else {
            RawRead::Busy
        }
    }

    pub proof fn classify_head(tracked &self) -> (result: RawRead)
        requires self.well_formed(),
        ensures result == self.observe_head(),
            result == RawRead::Value <==> self.ready_values().contains(self.head()),
            result == RawRead::Closed <==>
                !self.ready_values().contains(self.head())
                    && self.close_ready() && self.close_slot() == Some(self.head()),
            result == RawRead::Empty <==>
                !self.ready_values().contains(self.head())
                    && !(self.close_ready() && self.close_slot() == Some(self.head()))
                    && self.tail() == self.head(),
    {
        self.observe_head()
    }

    pub proof fn pop_value(tracked &mut self) -> (index: nat)
        requires old(self).well_formed(), old(self).observe_head() == RawRead::Value,
        ensures final(self).well_formed(), index == old(self).head(),
            final(self).head() == old(self).head() + 1,
            final(self).ready_values() == old(self).ready_values().remove(index),
            final(self).consumed_values() == old(self).consumed_values().insert(index),
            final(self).tail() == old(self).tail(),
            final(self).close_slot() == old(self).close_slot(),
            final(self).close_ready() == old(self).close_ready(),
            final(self).claimed_values() == old(self).claimed_values(),
    {
        let index = self.head;
        self.ready_values = self.ready_values.remove(index);
        self.consumed_values = self.consumed_values.insert(index);
        self.head = self.head + 1;
        index
    }

    pub proof fn pop_slot<T>(
        tracked &mut self,
        tracked slot: &mut RawValueSlot<T>,
    ) -> (result: T)
        requires old(self).well_formed(), old(slot).well_formed(),
            old(self).observe_head() == RawRead::Value,
            old(slot).index() == old(self).head(),
            old(slot).phase() == RawSlotPhase::Ready,
        ensures final(self).well_formed(), final(slot).well_formed(),
            final(self).head() == old(self).head() + 1,
            final(self).tail() == old(self).tail(),
            final(self).claimed_values() == old(self).claimed_values(),
            final(self).ready_values()
                == old(self).ready_values().remove(old(self).head()),
            final(self).consumed_values()
                == old(self).consumed_values().insert(old(self).head()),
            final(self).close_slot() == old(self).close_slot(),
            final(self).close_ready() == old(self).close_ready(),
            final(slot).index() == old(slot).index(),
            final(slot).phase() == RawSlotPhase::Consumed,
            old(slot).value() == Some(result),
    {
        let index = self.pop_value();
        assert(index == slot.index);
        slot.consume()
    }

    pub proof fn observe_close(tracked &mut self)
        requires old(self).well_formed(), old(self).observe_head() == RawRead::Closed,
        ensures final(self).well_formed(), final(self).head() == old(self).head(),
            final(self).tail() == old(self).tail(),
            final(self).close_slot() == old(self).close_slot(),
            final(self).close_ready() == old(self).close_ready(),
    { }

    pub proof fn wake_receiver(tracked &mut self)
        requires old(self).well_formed(),
        ensures final(self).well_formed(),
            final(self).wake_count() == old(self).wake_count() + 1,
            final(self).head() == old(self).head(), final(self).tail() == old(self).tail(),
            final(self).close_slot() == old(self).close_slot(),
    {
        self.wake_count = self.wake_count + 1;
    }

    pub proof fn release_block(tracked &mut self, block: nat)
        requires old(self).well_formed(),
            !old(self).released_blocks().contains(block),
        ensures final(self).well_formed(),
            final(self).released_blocks() == old(self).released_blocks().insert(block),
            final(self).reclaimed_blocks() == old(self).reclaimed_blocks(),
    {
        self.released_blocks = self.released_blocks.insert(block);
    }

    /// Receiver-only reclamation follows the acquired pop cursor. The raw
    /// pointer may be recycled only after sender release and consumption of all
    /// value slots in that logical block generation.
    pub proof fn reclaim_block(tracked &mut self, block: nat)
        requires old(self).well_formed(),
            old(self).released_blocks().contains(block),
            !old(self).reclaimed_blocks().contains(block),
            (block + 1) * old(self).block_capacity() <= old(self).head(),
        ensures final(self).well_formed(),
            final(self).reclaimed_blocks() == old(self).reclaimed_blocks().insert(block),
            final(self).released_blocks() == old(self).released_blocks(),
            final(self).head() == old(self).head(), final(self).tail() == old(self).tail(),
    {
        self.reclaimed_blocks = self.reclaimed_blocks.insert(block);
    }
}

pub proof fn verify_raw_busy_then_fifo_value()
{
    let tracked mut queue = RawMpscQueue::new(2);
    let first = queue.claim_value();
    let second = queue.claim_value();
    queue.publish_value(second);
    let busy = queue.classify_head();
    assert(busy == RawRead::Busy);
    queue.publish_value(first);
    let first_ready = queue.classify_head();
    assert(first_ready == RawRead::Value);
    let popped_first = queue.pop_value();
    assert(popped_first == first);
    let popped_second = queue.pop_value();
    assert(popped_second == second);
}

pub proof fn verify_last_strong_close_is_fifo_and_wakes()
{
    let tracked mut queue = RawMpscQueue::new(2);
    let value = queue.claim_value();
    assert(queue.claimed_values().contains(value));
    assert(!queue.ready_values().contains(value));
    let close = queue.claim_close();
    queue.publish_close();
    assert(queue.claimed_values().contains(value));
    assert(queue.head() == value);
    assert(!queue.ready_values().contains(value));
    assert(queue.close_slot() == Some(close));
    assert(close == value + 1);
    let busy = queue.classify_head();
    assert(busy == RawRead::Busy);
    queue.publish_value(value);
    let popped = queue.pop_value();
    assert(popped == value);
    assert(queue.head() == close);
    assert(queue.close_ready());
    assert(queue.close_slot() == Some(queue.head()));
    assert(!queue.ready_values().contains(queue.head()));
    let closed = queue.classify_head();
    assert(closed == RawRead::Closed);
    assert(close == queue.head());
    let before = queue.wake_count();
    queue.wake_receiver();
    assert(queue.wake_count() == before + 1);
}

pub proof fn verify_raw_slot_moves_payload_exactly_once(value: u64)
{
    let tracked mut queue = RawMpscQueue::new(2);
    let index = queue.claim_value();
    let tracked mut slot = RawValueSlot::claimed(index);
    queue.publish_slot(&mut slot, value);
    let read = queue.classify_head();
    assert(read == RawRead::Value);
    let received = queue.pop_slot(&mut slot);
    assert(received == value);
    assert(slot.phase() == RawSlotPhase::Consumed);
}

pub proof fn verify_release_observation_precedes_reuse_or_free(
    block: nat, observed: nat, receiver_index: nat, reuse: bool,
)
    requires observed <= receiver_index,
{
    let tracked mut lease = MpscTraversalLease::tail_owned(block);
    lease.tx_release(observed);
    lease.receiver_detach(receiver_index);
    if reuse {
        lease.reuse(block + 1);
        assert(lease.phase() == TraversalLeasePhase::Reused);
    } else {
        lease.free();
        assert(lease.phase() == TraversalLeasePhase::Freed);
    }
}

} // verus!
