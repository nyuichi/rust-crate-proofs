use vstd::prelude::*;

verus! {

/// Tokio accepts capacities up to `usize::MAX / 2` and rounds them to a power
/// of two.  This proof representation records exactly the resulting buffer
/// length and mask.  `usize`/allocation are frozen adapters; all ring arithmetic
/// below is the channel-specific part.
pub struct BroadcastGeometry {
    exponent: u32,
    capacity: u64,
    mask: u64,
}

impl BroadcastGeometry {
    pub closed spec fn exponent(&self) -> u32 { self.exponent }
    pub closed spec fn capacity(&self) -> u64 { self.capacity }
    pub closed spec fn mask(&self) -> u64 { self.mask }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.capacity() > 0
        &&& self.capacity() <= 0x8000_0000_0000_0000u64
        &&& self.exponent() <= 63
        &&& self.capacity() == 1u64 << self.exponent()
        &&& self.mask() + 1 == self.capacity()
        &&& self.capacity() & self.mask() == 0
    }

    /// `capacity` is the post-`next_power_of_two` value from production.
    pub fn new(exponent: u32) -> (result: Self)
        requires
            exponent <= 63,
        ensures
            result.well_formed(),
            result.exponent() == exponent,
            result.capacity() == 1u64 << exponent,
            result.mask() == (1u64 << exponent) - 1,
        no_unwind
    {
        let capacity = 1u64 << exponent;
        assert(capacity > 0) by (bit_vector)
            requires exponent <= 63, capacity == 1u64 << exponent;
        assert(capacity <= 0x8000_0000_0000_0000u64) by (bit_vector)
            requires exponent <= 63, capacity == 1u64 << exponent;
        assert(capacity & ((capacity - 1) as u64) == 0) by (bit_vector)
            requires exponent <= 63, capacity == 1u64 << exponent;
        BroadcastGeometry { exponent, capacity, mask: capacity - 1 }
    }

    pub fn slot(&self, position: u64) -> (index: u64)
        requires self.well_formed(),
        ensures index == position & self.mask(), index < self.capacity(),
        no_unwind
    {
        let mask = self.mask;
        let index = position & mask;
        assert((position & mask) <= mask) by (bit_vector);
        index
    }

    /// Production initializes slot `i` with `i.wrapping_sub(capacity)`.  Its
    /// first empty observation adds one capacity and therefore identifies `i`.
    pub fn initial_tag(&self, index: u64) -> (tag: u64)
        requires self.well_formed(), index < self.capacity(),
        ensures
            tag == vstd::wrapping::u64_specs::wrapping_sub(index, self.capacity()),
            vstd::wrapping::u64_specs::wrapping_add(tag, self.capacity()) == index,
        no_unwind
    {
        let capacity = self.capacity;
        let tag = index.wrapping_sub(capacity);
        assert(index.wrapping_sub(capacity).wrapping_add(capacity) == index) by (bit_vector);
        tag
    }

    /// Exact same-generation relation used by send and recv.  This is stated
    /// for every supported power-of-two capacity, not just the cap=2 witness.
    pub fn next_generation_same_slot(&self, position: u64) -> (next: u64)
        requires
            self.well_formed(),
            position <= u64::MAX - self.capacity(),
        ensures
            next == position + self.capacity(),
            (next & self.mask()) == (position & self.mask()),
            next > position,
        no_unwind
    {
        let capacity = self.capacity;
        let mask = self.mask;
        let exponent = self.exponent;
        let next = position + capacity;
        assert((next & mask) == (position & mask)) by (bit_vector)
            requires
                exponent <= 63,
                capacity == 1u64 << exponent,
                mask == capacity - 1,
                next == position + capacity,
                position <= u64::MAX - capacity;
        next
    }

    /// Distinct retained tickets cannot alias. Full `slot.pos` tags classify
    /// older generations; this lemma supplies the no-alias fact inside the
    /// one-capacity retained window.
    pub fn retained_slot_unique(&self, earlier: u64, later: u64)
        requires
            self.well_formed(),
            earlier <= later,
            later - earlier < self.capacity(),
            (earlier & self.mask()) == (later & self.mask()),
        ensures earlier == later,
        no_unwind
    {
        let exponent = self.exponent;
        let capacity = self.capacity;
        let mask = self.mask;
        assert(earlier == later) by (bit_vector)
            requires
                exponent <= 63,
                capacity == 1u64 << exponent,
                mask == capacity - 1,
                earlier <= later,
                later - earlier < capacity,
                (earlier & mask) == (later & mask);
    }
}

/// Exact owned projection of production `Slot<T>`.  The Mutex and atomic
/// linearization are frozen adapters; `pos`, `rem`, and `val` are not.
pub struct PhysicalBroadcastSlot<T> {
    index: u64,
    pos: u64,
    rem: usize,
    val: Option<T>,
}

impl<T> PhysicalBroadcastSlot<T> {
    pub closed spec fn index(&self) -> u64 { self.index }
    pub closed spec fn pos(&self) -> u64 { self.pos }
    pub closed spec fn rem(&self) -> usize { self.rem }
    pub closed spec fn val(&self) -> Option<T> { self.val }
    pub closed spec fn well_formed(&self) -> bool {
        self.rem() == 0 <==> self.val().is_none()
    }

    pub fn initial(index: u64, capacity: u64) -> (result: Self)
        requires capacity > 0, index < capacity,
        ensures
            result.well_formed(),
            result.index() == index,
            result.pos() == vstd::wrapping::u64_specs::wrapping_sub(index, capacity),
            result.rem() == 0,
            result.val().is_none(),
        no_unwind
    {
        PhysicalBroadcastSlot {
            index,
            pos: index.wrapping_sub(capacity),
            rem: 0,
            val: None,
        }
    }

    /// Exact order of production send while holding tail then slot: assign
    /// position, receiver count, then value. The overwritten value is moved
    /// out exactly once and never duplicated by this transition.
    pub fn publish(&mut self, position: u64, mask: u64, receivers: usize, value: T)
        -> (overwritten: Option<T>)
        requires receivers > 0, (position & mask) == old(self).index(),
        ensures
            final(self).well_formed(),
            final(self).index() == old(self).index(),
            final(self).pos() == position,
            (final(self).pos() & mask) == final(self).index(),
            final(self).rem() == receivers,
            final(self).val() == Some(value),
            overwritten == old(self).val(),
        no_unwind
    {
        self.pos = position;
        self.rem = receivers;
        let mut incoming = Some(value);
        core::mem::swap(&mut self.val, &mut incoming);
        incoming
    }

    /// `RecvGuard::drop`: every acquired reader consumes exactly one `rem`.
    /// The last one moves the unique stored value out; earlier readers leave it
    /// in place. This transition is independent of Clone success or unwind.
    pub fn release_guard(&mut self) -> (released: Option<T>)
        requires old(self).well_formed(), old(self).rem() > 0,
        ensures
            final(self).well_formed(),
            final(self).pos() == old(self).pos(),
            final(self).rem() + 1 == old(self).rem(),
            old(self).rem() == 1 ==> released == old(self).val(),
            old(self).rem() == 1 ==> final(self).val().is_none(),
            old(self).rem() > 1 ==> released.is_none(),
            old(self).rem() > 1 ==> final(self).val() == old(self).val(),
        no_unwind
    {
        self.rem -= 1;
        if self.rem == 0 { self.val.take() } else { None }
    }

    /// Channel-specific part of `clone_value` followed by guard Drop. Clone is
    /// a frozen arbitrary-code boundary; either outcome performs the same
    /// exactly-once reader release during stack unwinding.
    pub fn finish_clone(&mut self, clone_succeeded: bool) -> (released: Option<T>)
        requires old(self).well_formed(), old(self).rem() > 0,
        ensures
            final(self).well_formed(),
            final(self).rem() + 1 == old(self).rem(),
            final(self).pos() == old(self).pos(),
            old(self).rem() == 1 ==> released == old(self).val(),
            old(self).rem() > 1 ==> released.is_none(),
        no_unwind
    {
        let _ = clone_succeeded;
        self.release_guard()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SendCleanupPhase {
    Published,
    Notified,
}

/// Linear ownership of the displaced payload across production send's
/// lock-free cleanup suffix. There is intentionally no channel-state field:
/// both waking a Waker and dropping `T` may reenter and mutate the channel.
pub struct SendCleanup<T> {
    phase: SendCleanupPhase,
    old_value: Option<T>,
    published_pos: u64,
    captured_receivers: usize,
}

impl<T> SendCleanup<T> {
    pub closed spec fn phase(&self) -> SendCleanupPhase { self.phase }
    pub closed spec fn old_value(&self) -> Option<T> { self.old_value }
    pub closed spec fn published_pos(&self) -> u64 { self.published_pos }
    pub closed spec fn captured_receivers(&self) -> usize { self.captured_receivers }

    pub fn after_slot_unlock(
        old_value: Option<T>,
        position: u64,
        receivers: usize,
    ) -> (result: Self)
        requires receivers > 0,
        ensures
            result.phase() == SendCleanupPhase::Published,
            result.old_value() == old_value,
            result.published_pos() == position,
            result.captured_receivers() == receivers,
        no_unwind
    {
        SendCleanup {
            phase: SendCleanupPhase::Published,
            old_value,
            published_pos: position,
            captured_receivers: receivers,
        }
    }

    /// Successful `notify_rx`. The old payload token remains linear while no
    /// assertion is made about post-Waker channel state.
    pub fn notification_complete(&mut self)
        requires old(self).phase() == SendCleanupPhase::Published,
        ensures
            final(self).phase() == SendCleanupPhase::Notified,
            final(self).old_value() == old(self).old_value(),
            final(self).published_pos() == old(self).published_pos(),
            final(self).captured_receivers() == old(self).captured_receivers(),
        no_unwind
    {
        self.phase = SendCleanupPhase::Notified;
    }

    /// Unwind out of `notify_rx`: Rust drops the still-owned local exactly
    /// once. Returning the token represents handing it to arbitrary Drop.
    pub fn unwind_notification(self) -> (drop_token: Option<T>)
        requires self.phase() == SendCleanupPhase::Published,
        ensures drop_token == self.old_value(),
        no_unwind
    {
        self.old_value
    }

    /// Normal notification path. Consuming `self` prevents a second cleanup;
    /// arbitrary old-value Drop may return, panic, or reenter after this point.
    pub fn begin_old_value_drop(self) -> (drop_token: Option<T>)
        requires self.phase() == SendCleanupPhase::Notified,
        ensures drop_token == self.old_value(),
        no_unwind
    {
        self.old_value
    }
}

/// Global conservation summary for physical payload ownership. A published
/// payload is either still stored, displaced by a later publish, or released by
/// the final RecvGuard. Each transition increments exactly one terminal bucket.
pub struct BroadcastValueConservation {
    published: u64,
    stored: u64,
    overwritten: u64,
    last_guard_released: u64,
}

impl BroadcastValueConservation {
    pub closed spec fn published(&self) -> u64 { self.published }
    pub closed spec fn stored(&self) -> u64 { self.stored }
    pub closed spec fn overwritten(&self) -> u64 { self.overwritten }
    pub closed spec fn last_guard_released(&self) -> u64 { self.last_guard_released }
    pub closed spec fn well_formed(&self) -> bool {
        self.published() as int == self.stored() as int
            + self.overwritten() as int + self.last_guard_released() as int
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.published() == 0,
        no_unwind
    {
        BroadcastValueConservation {
            published: 0,
            stored: 0,
            overwritten: 0,
            last_guard_released: 0,
        }
    }

    pub fn publish_into_empty(&mut self)
        requires old(self).well_formed(), old(self).published() < u64::MAX,
            old(self).stored() < u64::MAX,
        ensures final(self).well_formed(),
            final(self).published() == old(self).published() + 1,
            final(self).stored() == old(self).stored() + 1,
            final(self).overwritten() == old(self).overwritten(),
            final(self).last_guard_released() == old(self).last_guard_released(),
        no_unwind
    {
        self.published += 1;
        self.stored += 1;
    }

    pub fn overwrite(&mut self)
        requires old(self).well_formed(), old(self).stored() > 0,
            old(self).published() < u64::MAX, old(self).overwritten() < u64::MAX,
        ensures final(self).well_formed(),
            final(self).published() == old(self).published() + 1,
            final(self).stored() == old(self).stored(),
            final(self).overwritten() == old(self).overwritten() + 1,
            final(self).last_guard_released() == old(self).last_guard_released(),
        no_unwind
    {
        self.published += 1;
        self.overwritten += 1;
    }

    pub fn release_last_guard(&mut self)
        requires old(self).well_formed(), old(self).stored() > 0,
            old(self).last_guard_released() < u64::MAX,
        ensures final(self).well_formed(),
            final(self).published() == old(self).published(),
            final(self).stored() + 1 == old(self).stored(),
            final(self).overwritten() == old(self).overwritten(),
            final(self).last_guard_released() == old(self).last_guard_released() + 1,
        no_unwind
    {
        self.stored -= 1;
        self.last_guard_released += 1;
    }
}

pub fn verify_arbitrary_capacity_geometry(exponent: u32, position: u64)
    requires
        exponent <= 63,
        position <= u64::MAX - (1u64 << exponent),
{
    let geometry = BroadcastGeometry::new(exponent);
    let capacity = 1u64 << exponent;
    let index = geometry.slot(position);
    let next = geometry.next_generation_same_slot(position);
    assert(index < capacity);
    assert((next & geometry.mask()) == index);
}

pub fn verify_overwrite_then_last_release(old: u64, new: u64)
{
    let mut slot = PhysicalBroadcastSlot::initial(0, 2);
    assert(slot.index() == 0);
    assert((0u64 & 1u64) == 0u64) by (bit_vector);
    let none = slot.publish(0, 1, 1, old);
    assert(none.is_none());
    assert((2u64 & 1u64) == 0u64) by (bit_vector);
    assert((2u64 & 1u64) == slot.index());
    let overwritten = slot.publish(2, 1, 1, new);
    assert(overwritten == Some(old));
    let released = slot.finish_clone(false);
    assert(released == Some(new));
    assert(slot.val().is_none());
}

pub fn verify_send_cleanup_normal_and_unwind(old: u64)
{
    let cleanup_on_unwind = SendCleanup::after_slot_unlock(Some(old), 4, 2);
    let unwind_token = cleanup_on_unwind.unwind_notification();
    assert(unwind_token == Some(old));

    let mut cleanup = SendCleanup::after_slot_unlock(Some(old), 4, 2);
    cleanup.notification_complete();
    let normal_token = cleanup.begin_old_value_drop();
    assert(normal_token == Some(old));
}

pub fn verify_value_conservation_two_terminal_paths()
{
    let mut ledger = BroadcastValueConservation::new();
    ledger.publish_into_empty();
    ledger.overwrite();
    ledger.release_last_guard();
    assert(ledger.published() == 2);
    assert(ledger.overwritten() == 1);
    assert(ledger.last_guard_released() == 1);
    assert(ledger.stored() == 0);
}

} // verus!
