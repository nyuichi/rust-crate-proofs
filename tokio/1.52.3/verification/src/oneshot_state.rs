use vstd::prelude::*;

verus! {

pub const RX_TASK_SET: usize = 0b00001;
pub const VALUE_SENT: usize = 0b00010;
pub const CLOSED: usize = 0b00100;
pub const TX_TASK_SET: usize = 0b01000;

/// Executable proof view of production oneshot's four orthogonal state bits.
/// `value_sent` and `closed` are monotonic. The task bits own initialization
/// knowledge for their corresponding waker slots and may be replaced.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct OneshotBits {
    rx_task_set: bool,
    value_sent: bool,
    closed: bool,
    tx_task_set: bool,
}

impl OneshotBits {
    pub closed spec fn rx_task_set(&self) -> bool { self.rx_task_set }
    pub closed spec fn value_sent(&self) -> bool { self.value_sent }
    pub closed spec fn closed(&self) -> bool { self.closed }
    pub closed spec fn tx_task_set(&self) -> bool { self.tx_task_set }

    pub closed spec fn encoded(&self) -> nat {
        (if self.rx_task_set { RX_TASK_SET as nat } else { 0 })
            + (if self.value_sent { VALUE_SENT as nat } else { 0 })
            + (if self.closed { CLOSED as nat } else { 0 })
            + (if self.tx_task_set { TX_TASK_SET as nat } else { 0 })
    }

    pub fn new() -> (result: Self)
        ensures
            !result.rx_task_set(),
            !result.value_sent(),
            !result.closed(),
            !result.tx_task_set(),
            result.encoded() == 0,
        no_unwind
    {
        OneshotBits {
            rx_task_set: false,
            value_sent: false,
            closed: false,
            tx_task_set: false,
        }
    }

    /// Abstracts production `State::set_complete`'s CAS loop. A close which
    /// linearized first rejects completion and leaves VALUE_SENT clear.
    pub fn set_complete(&mut self) -> (completed: bool)
        ensures
            final(self).closed() == old(self).closed(),
            final(self).rx_task_set() == old(self).rx_task_set(),
            final(self).tx_task_set() == old(self).tx_task_set(),
            completed == !old(self).closed(),
            completed ==> final(self).value_sent(),
            !completed ==> final(self).value_sent() == old(self).value_sent(),
            old(self).value_sent() ==> final(self).value_sent(),
        no_unwind
    {
        if self.closed {
            false
        } else {
            self.value_sent = true;
            true
        }
    }

    pub fn set_closed(&mut self) -> (was_closed: bool)
        ensures
            was_closed == old(self).closed(),
            final(self).closed(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).rx_task_set() == old(self).rx_task_set(),
            final(self).tx_task_set() == old(self).tx_task_set(),
        no_unwind
    {
        let previous = self.closed;
        self.closed = true;
        previous
    }

    pub fn set_rx_task(&mut self)
        ensures
            final(self).rx_task_set(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).tx_task_set() == old(self).tx_task_set(),
        no_unwind
    {
        self.rx_task_set = true;
    }

    pub fn unset_rx_task(&mut self)
        ensures
            !final(self).rx_task_set(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).tx_task_set() == old(self).tx_task_set(),
        no_unwind
    {
        self.rx_task_set = false;
    }

    pub fn set_tx_task(&mut self)
        ensures
            final(self).tx_task_set(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).rx_task_set() == old(self).rx_task_set(),
        no_unwind
    {
        self.tx_task_set = true;
    }

    pub fn unset_tx_task(&mut self)
        ensures
            !final(self).tx_task_set(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).rx_task_set() == old(self).rx_task_set(),
        no_unwind
    {
        self.tx_task_set = false;
    }
}

pub fn verify_send_wins_close_race()
{
    let mut bits = OneshotBits::new();
    let completed = bits.set_complete();
    assert(completed);
    bits.set_closed();
    assert(bits.value_sent());
    assert(bits.closed());
}

pub fn verify_close_wins_send_race()
{
    let mut bits = OneshotBits::new();
    bits.set_closed();
    let completed = bits.set_complete();
    assert(!completed);
    assert(!bits.value_sent());
    assert(bits.closed());
}

pub fn verify_task_bits_do_not_change_lifecycle()
{
    let mut bits = OneshotBits::new();
    bits.set_rx_task();
    bits.set_tx_task();
    assert(!bits.value_sent());
    assert(!bits.closed());
    bits.unset_rx_task();
    bits.unset_tx_task();
    assert(bits.encoded() == 0);
}

} // verus!
