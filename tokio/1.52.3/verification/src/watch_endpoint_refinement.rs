use vstd::prelude::*;

verus! {

pub open spec fn receiver_notification_limit() -> nat {
    usize::MAX as nat / 4
}

/// Production endpoint order: Sender clones Arc before its count; Receiver
/// creation reserves a cumulative last-drop notification before cloning Arc
/// and incrementing the live count.
pub struct WatchEndpoints {
    tx_count: usize,
    rx_count: usize,
    arc_owners: usize,
    tx_arc_gaps: usize,
    rx_arc_gaps: usize,
    pending_rx_reservations: usize,
    receiver_reservations: usize,
    receiver_close_calls: usize,
    sender_closed: bool,
}

impl WatchEndpoints {
    pub closed spec fn tx_count(&self) -> usize { self.tx_count }
    pub closed spec fn rx_count(&self) -> usize { self.rx_count }
    pub closed spec fn arc_owners(&self) -> usize { self.arc_owners }
    pub closed spec fn tx_arc_gaps(&self) -> usize { self.tx_arc_gaps }
    pub closed spec fn rx_arc_gaps(&self) -> usize { self.rx_arc_gaps }
    pub closed spec fn pending_rx_reservations(&self) -> usize {
        self.pending_rx_reservations
    }
    pub closed spec fn receiver_reservations(&self) -> usize {
        self.receiver_reservations
    }
    pub closed spec fn receiver_close_calls(&self) -> usize {
        self.receiver_close_calls
    }
    pub closed spec fn sender_closed(&self) -> bool { self.sender_closed }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.arc_owners() as int == self.tx_count() as int
            + self.rx_count() as int + self.tx_arc_gaps() as int
            + self.rx_arc_gaps() as int
        &&& self.arc_owners() <= usize::MAX / 2
        &&& (self.receiver_reservations() as nat) <= receiver_notification_limit()
        &&& self.receiver_close_calls() as int
            + self.pending_rx_reservations() as int + self.rx_arc_gaps() as int
            + if self.rx_count() > 0 { 1int } else { 0int }
            <= self.receiver_reservations() as int
        &&& self.sender_closed() == (self.tx_count() == 0)
    }

    pub fn channel() -> (result: Self)
        ensures result.well_formed(), result.tx_count() == 1,
            result.rx_count() == 1, result.arc_owners() == 2,
            result.receiver_reservations() == 1,
            result.receiver_close_calls() == 0, !result.sender_closed(),
        no_unwind
    {
        WatchEndpoints {
            tx_count: 1,
            rx_count: 1,
            arc_owners: 2,
            tx_arc_gaps: 0,
            rx_arc_gaps: 0,
            pending_rx_reservations: 0,
            receiver_reservations: 1,
            receiver_close_calls: 0,
            sender_closed: false,
        }
    }

    pub fn begin_clone_sender(&mut self)
        requires old(self).well_formed(), old(self).tx_count() > 0,
            old(self).arc_owners() < usize::MAX / 2,
        ensures final(self).well_formed(),
            final(self).arc_owners() == old(self).arc_owners() + 1,
            final(self).tx_arc_gaps() == old(self).tx_arc_gaps() + 1,
            final(self).tx_count() == old(self).tx_count(),
        no_unwind
    {
        self.arc_owners += 1;
        self.tx_arc_gaps += 1;
    }

    pub fn finish_clone_sender(&mut self)
        requires old(self).well_formed(), old(self).tx_arc_gaps() > 0,
            old(self).tx_count() > 0,
        ensures final(self).well_formed(),
            final(self).tx_arc_gaps() + 1 == old(self).tx_arc_gaps(),
            final(self).tx_count() == old(self).tx_count() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
        no_unwind
    {
        assert(self.tx_count < usize::MAX);
        self.tx_arc_gaps -= 1;
        self.tx_count += 1;
    }

    /// CAS success path for Receiver clone/subscribe reservation.
    pub fn reserve_receiver(&mut self) -> (reserved: bool)
        requires old(self).well_formed(),
        ensures
            (old(self).receiver_reservations() as nat) == receiver_notification_limit()
                ==> !reserved && final(self).well_formed()
                    && final(self).receiver_reservations()
                        == old(self).receiver_reservations()
                    && final(self).arc_owners() == old(self).arc_owners()
                    && final(self).rx_count() == old(self).rx_count(),
            (old(self).receiver_reservations() as nat) < receiver_notification_limit()
                ==> reserved && final(self).well_formed()
                    && final(self).receiver_reservations()
                        == old(self).receiver_reservations() + 1
                    && final(self).pending_rx_reservations()
                        == old(self).pending_rx_reservations() + 1
                    && final(self).arc_owners() == old(self).arc_owners()
                    && final(self).rx_count() == old(self).rx_count(),
        no_unwind
    {
        if self.receiver_reservations == usize::MAX / 4 {
            false
        } else {
            self.receiver_reservations += 1;
            self.pending_rx_reservations += 1;
            true
        }
    }

    pub fn begin_receiver_arc(&mut self)
        requires old(self).well_formed(), old(self).pending_rx_reservations() > 0,
            old(self).arc_owners() < usize::MAX / 2,
        ensures final(self).well_formed(),
            final(self).pending_rx_reservations() + 1
                == old(self).pending_rx_reservations(),
            final(self).rx_arc_gaps() == old(self).rx_arc_gaps() + 1,
            final(self).arc_owners() == old(self).arc_owners() + 1,
            final(self).rx_count() == old(self).rx_count(),
        no_unwind
    {
        self.pending_rx_reservations -= 1;
        self.rx_arc_gaps += 1;
        self.arc_owners += 1;
    }

    pub fn finish_receiver(&mut self)
        requires old(self).well_formed(), old(self).rx_arc_gaps() > 0,
        ensures final(self).well_formed(),
            final(self).rx_arc_gaps() + 1 == old(self).rx_arc_gaps(),
            final(self).rx_count() == old(self).rx_count() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
        no_unwind
    {
        assert(self.rx_count < usize::MAX);
        self.rx_arc_gaps -= 1;
        self.rx_count += 1;
    }

    pub fn drop_receiver(&mut self) -> (last: bool)
        requires old(self).well_formed(), old(self).rx_count() > 0,
        ensures final(self).well_formed(),
            final(self).rx_count() + 1 == old(self).rx_count(),
            final(self).arc_owners() + 1 == old(self).arc_owners(),
            last == (final(self).rx_count() == 0),
            last ==> final(self).receiver_close_calls()
                == old(self).receiver_close_calls() + 1,
            !last ==> final(self).receiver_close_calls()
                == old(self).receiver_close_calls(),
            last ==> (old(self).receiver_close_calls() as nat)
                < receiver_notification_limit(),
        no_unwind
    {
        self.rx_count -= 1;
        self.arc_owners -= 1;
        let last = self.rx_count == 0;
        if last {
            self.receiver_close_calls += 1;
        }
        last
    }

    pub fn drop_sender(&mut self) -> (last: bool)
        requires old(self).well_formed(), old(self).tx_count() > 0,
        ensures final(self).well_formed(),
            final(self).tx_count() + 1 == old(self).tx_count(),
            final(self).arc_owners() + 1 == old(self).arc_owners(),
            last == final(self).sender_closed(),
        no_unwind
    {
        self.tx_count -= 1;
        self.arc_owners -= 1;
        if self.tx_count == 0 { self.sender_closed = true; }
        self.tx_count == 0
    }
}

pub fn verify_terminal_receiver_reservation_preserves_counts()
{
    let mut endpoints = WatchEndpoints {
        tx_count: 1,
        rx_count: 1,
        arc_owners: 2,
        tx_arc_gaps: 0,
        rx_arc_gaps: 0,
        pending_rx_reservations: 0,
        receiver_reservations: usize::MAX / 4,
        receiver_close_calls: 0,
        sender_closed: false,
    };
    assert(endpoints.well_formed());
    let reserved = endpoints.reserve_receiver();
    assert(!reserved);
    assert(endpoints.rx_count() == 1);
    assert(endpoints.arc_owners() == 2);
}

pub fn verify_last_reserved_receiver_drop_has_notify_room()
{
    let mut endpoints = WatchEndpoints {
        tx_count: 1,
        rx_count: 1,
        arc_owners: 2,
        tx_arc_gaps: 0,
        rx_arc_gaps: 0,
        pending_rx_reservations: 0,
        receiver_reservations: usize::MAX / 4,
        receiver_close_calls: usize::MAX / 4 - 1,
        sender_closed: false,
    };
    assert(endpoints.well_formed());
    let last = endpoints.drop_receiver();
    assert(last);
    assert(endpoints.receiver_close_calls() == usize::MAX / 4);
}

pub fn verify_arc_first_sender_clone_has_counter_room()
{
    let mut endpoints = WatchEndpoints::channel();
    endpoints.begin_clone_sender();
    endpoints.finish_clone_sender();
    assert(endpoints.tx_count() == 2);
    assert(endpoints.arc_owners() == 3);
}

} // verus!
