use crate::oneshot_value::OneshotValue;
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

pub const RX_TASK_SET: usize = 0b00001;
pub const VALUE_SENT: usize = 0b00010;
pub const CLOSED: usize = 0b00100;
pub const TX_TASK_SET: usize = 0b01000;
pub const KNOWN_BITS: usize = RX_TASK_SET | VALUE_SENT | CLOSED | TX_TASK_SET;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlotCapability { Sender, Staged, Receiver, InnerDrop, Released }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CompletionAttempt { Completed, Closed, Retry }

/// A production state snapshot contains exactly the four declared bits. This
/// representation excludes unknown bits by construction while `raw` fixes the
/// precise machine-word encoding consumed by the atomic adapter.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ExactStateSnapshot {
    rx_task_set: bool,
    value_sent: bool,
    closed: bool,
    tx_task_set: bool,
}

impl ExactStateSnapshot {
    pub closed spec fn raw(&self) -> nat {
        (if self.rx_task_set { RX_TASK_SET as nat } else { 0 })
            + (if self.value_sent { VALUE_SENT as nat } else { 0 })
            + (if self.closed { CLOSED as nat } else { 0 })
            + (if self.tx_task_set { TX_TASK_SET as nat } else { 0 })
    }
    pub closed spec fn value_sent(&self) -> bool { self.value_sent }
    pub closed spec fn closed(&self) -> bool { self.closed }
    pub closed spec fn rx_task_set(&self) -> bool { self.rx_task_set }
    pub closed spec fn tx_task_set(&self) -> bool { self.tx_task_set }

    pub fn new() -> (result: Self)
        ensures result.raw() == 0, !result.value_sent(), !result.closed(),
            !result.rx_task_set(), !result.tx_task_set(),
        no_unwind
    {
        ExactStateSnapshot {
            rx_task_set: false, value_sent: false, closed: false, tx_task_set: false,
        }
    }

    pub fn with_rx_task(self) -> (result: Self)
        ensures result.raw() == self.raw() + if self.rx_task_set() { 0nat } else { RX_TASK_SET as nat },
            result.value_sent() == self.value_sent(), result.closed() == self.closed(),
            result.tx_task_set() == self.tx_task_set(),
        no_unwind
    {
        let mut result = self;
        result.rx_task_set = true;
        result
    }

    pub fn with_closed(self) -> (result: Self)
        ensures result.closed(), result.value_sent() == self.value_sent(),
            result.rx_task_set() == self.rx_task_set(),
            result.tx_task_set() == self.tx_task_set(),
        no_unwind
    {
        let mut result = self;
        result.closed = true;
        result
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WakeEffects {
    pub receiver: Option<u64>,
    pub sender: Option<u64>,
}

#[derive(PartialEq, Eq)]
pub enum ProductionTryRecv<T> { Value(T), Empty, Closed }

/// One iteration of production's weak-CAS loop. `success == false` includes
/// both interference and a legal spurious `compare_exchange_weak` failure.
/// The unique active Sender has not published VALUE_SENT, and receiver/Waker
/// interference cannot publish it on the Sender's behalf.
pub fn completion_cas_attempt(
    observed: ExactStateSnapshot,
    actual: ExactStateSnapshot,
    cas_succeeded: bool,
) -> (result: (CompletionAttempt, ExactStateSnapshot))
    requires
        cas_succeeded ==> observed == actual,
        !observed.value_sent(),
        actual.value_sent() == observed.value_sent(),
        observed.closed() ==> actual.closed(),
    ensures
        observed.closed() ==>
            result == (CompletionAttempt::Closed, actual),
        !observed.closed() && !cas_succeeded ==>
            result == (CompletionAttempt::Retry, actual),
        !observed.closed() && cas_succeeded ==>
            result.0 == CompletionAttempt::Completed,
        result.0 == CompletionAttempt::Completed ==>
            result.1.value_sent(),
        result.0 == CompletionAttempt::Completed ==>
            result.1.closed() == actual.closed(),
        result.0 == CompletionAttempt::Completed ==>
            result.1.raw() == actual.raw() + VALUE_SENT as nat,
        result.0 != CompletionAttempt::Completed ==> result.1 == actual,
        result.0 == CompletionAttempt::Closed ==>
            result.1.value_sent() == actual.value_sent(),
    no_unwind
{
    if observed.closed {
        (CompletionAttempt::Closed, actual)
    } else if !cas_succeeded {
        (CompletionAttempt::Retry, actual)
    } else {
        let mut completed = actual;
        completed.value_sent = true;
        (CompletionAttempt::Completed, completed)
    }
}

/// Integrated production view of `Inner<T>` and the outer
/// `Option<Arc<Inner<T>>>` fields. The physical PCell owns exactly the
/// `Option<T>` stored behind production's shared `UnsafeCell`; `slot_capability`
/// states which capability the bits require. Connecting that capability to
/// Tokio's shared-`&self` wrapper remains an open representation refinement;
/// only the raw UnsafeCell semantics beneath it are frozen.
pub struct ProductionOneshot<T> {
    value: OneshotValue<T>,
    tx_alive: bool,
    sender_local_arc: bool,
    send_staged: bool,
    rx_alive: bool,
    inner_dropped: bool,
    value_sent: bool,
    closed: bool,
    rx_waker: Option<u64>,
    tx_waker: Option<u64>,
}

impl<T> ProductionOneshot<T> {
    pub closed spec fn value(&self) -> Option<T> { self.value.contents() }
    pub closed spec fn tx_alive(&self) -> bool { self.tx_alive }
    pub closed spec fn sender_local_arc(&self) -> bool { self.sender_local_arc }
    pub closed spec fn send_staged(&self) -> bool { self.send_staged }
    pub closed spec fn rx_alive(&self) -> bool { self.rx_alive }
    pub closed spec fn inner_dropped(&self) -> bool { self.inner_dropped }
    pub closed spec fn value_sent(&self) -> bool { self.value_sent }
    pub closed spec fn closed(&self) -> bool { self.closed }
    pub closed spec fn rx_waker(&self) -> Option<u64> { self.rx_waker }
    pub closed spec fn tx_waker(&self) -> Option<u64> { self.tx_waker }

    pub closed spec fn raw_state(&self) -> nat {
        (if self.rx_waker().is_some() { RX_TASK_SET as nat } else { 0 })
            + (if self.value_sent() { VALUE_SENT as nat } else { 0 })
            + (if self.closed() { CLOSED as nat } else { 0 })
            + (if self.tx_waker().is_some() { TX_TASK_SET as nat } else { 0 })
    }

    pub closed spec fn strong_count(&self) -> nat {
        (if self.tx_alive() { 1nat } else { 0nat })
            + (if self.sender_local_arc() { 1nat } else { 0nat })
            + (if self.rx_alive() { 1nat } else { 0nat })
    }

    pub closed spec fn slot_capability(&self) -> SlotCapability {
        if self.inner_dropped() { SlotCapability::Released }
        else if self.send_staged() { SlotCapability::Staged }
        else if self.tx_alive() { SlotCapability::Sender }
        else if self.value_sent() { SlotCapability::Receiver }
        else { SlotCapability::InnerDrop }
    }

    pub closed spec fn resources_well_formed(&self) -> bool {
        &&& self.inner_dropped() ==> self.value().is_none()
        &&& self.inner_dropped() ==> self.rx_waker().is_none()
        &&& self.inner_dropped() ==> self.tx_waker().is_none()
        &&& self.rx_waker().is_some() ==> self.rx_alive()
        &&& !(self.tx_alive() && self.sender_local_arc())
        &&& self.send_staged() ==> self.sender_local_arc() && !self.tx_alive()
        &&& self.send_staged() && !self.rx_alive() ==> self.closed()
        &&& self.sender_local_arc() ==> self.send_staged()
        &&& self.tx_alive() ==> !self.value_sent()
        &&& self.tx_alive() ==> self.value().is_none()
        &&& self.value().is_some() ==>
            !self.tx_alive()
                && (self.send_staged() || (self.value_sent() && self.rx_alive()))
        &&& self.send_staged() ==> self.value().is_some() && !self.value_sent()
        &&& !self.tx_alive() && !self.sender_local_arc() && !self.closed() ==> self.value_sent()
        &&& !self.rx_alive() && self.tx_alive() ==> self.closed()
        &&& self.closed() && !self.value_sent() ==> self.rx_waker().is_none()
    }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.resources_well_formed()
        &&& self.inner_dropped() ==
            (!self.tx_alive() && !self.sender_local_arc() && !self.rx_alive())
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.tx_alive(), result.rx_alive(),
            !result.inner_dropped(), !result.value_sent(), !result.closed(),
            result.value().is_none(), result.raw_state() == 0,
            result.strong_count() == 2,
            result.slot_capability() == SlotCapability::Sender,
    {
        ProductionOneshot {
            value: OneshotValue::empty(),
            tx_alive: true,
            sender_local_arc: false,
            send_staged: false,
            rx_alive: true,
            inner_dropped: false,
            value_sent: false,
            closed: false,
            rx_waker: None,
            tx_waker: None,
        }
    }

    pub fn exact_state_snapshot(&self) -> (result: ExactStateSnapshot)
        requires self.well_formed(),
        ensures result.raw() == self.raw_state(),
            result.value_sent() == self.value_sent(),
            result.closed() == self.closed(),
            result.rx_task_set() == self.rx_waker().is_some(),
            result.tx_task_set() == self.tx_waker().is_some(),
        no_unwind
    {
        ExactStateSnapshot {
            rx_task_set: self.rx_waker.is_some(),
            value_sent: self.value_sent,
            closed: self.closed,
            tx_task_set: self.tx_waker.is_some(),
        }
    }

    pub fn install_rx_waker(&mut self, waker: u64)
        requires old(self).well_formed(), old(self).rx_alive(),
            !old(self).value_sent(), !old(self).closed(),
        ensures final(self).well_formed(), final(self).rx_waker() == Some(waker),
            final(self).value() == old(self).value(),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).tx_waker() == old(self).tx_waker(),
        no_unwind
    {
        self.rx_waker = Some(waker);
    }

    pub fn install_tx_waker(&mut self, waker: u64)
        requires old(self).well_formed(), old(self).tx_alive(), !old(self).closed(),
        ensures final(self).well_formed(), final(self).tx_waker() == Some(waker),
            final(self).value() == old(self).value(),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).rx_waker() == old(self).rx_waker(),
        no_unwind
    {
        self.tx_waker = Some(waker);
    }

    /// First half of `Sender::send`: exact outer `self.inner.take()`, followed
    /// by the physical slot store. The local Arc keeps `Inner` alive while a
    /// concurrent receiver may set CLOSED before the completion CAS.
    pub fn sender_begin_store(&mut self, value: T)
        requires old(self).well_formed(), old(self).tx_alive(),
        ensures final(self).well_formed(), !final(self).tx_alive(),
            final(self).sender_local_arc(), final(self).send_staged(),
            final(self).value() == Some(value), !final(self).value_sent(),
            final(self).closed() == old(self).closed(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).rx_waker() == old(self).rx_waker(),
            final(self).tx_waker() == old(self).tx_waker(),
            final(self).strong_count() == old(self).strong_count(),
        no_unwind
    {
        self.value.store(value);
        self.tx_alive = false;
        self.sender_local_arc = true;
        self.send_staged = true;
    }

    /// Second half of `Sender::send`: the completion CAS either publishes the
    /// staged slot or observes CLOSED and takes the exact value back, followed
    /// by release of the local Arc.
    pub fn sender_finish_complete(&mut self)
        -> (result: (Result<(), T>, WakeEffects))
        requires old(self).well_formed(), old(self).sender_local_arc(),
            old(self).send_staged(),
        ensures final(self).well_formed(), !final(self).tx_alive(),
            !final(self).sender_local_arc(), !final(self).send_staged(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).closed() == old(self).closed(),
            old(self).rx_alive() ==> final(self).tx_waker() == old(self).tx_waker(),
            old(self).closed() ==> match result.0 {
                Err(value) => old(self).value() == Some(value),
                Ok(()) => false,
            },
            old(self).closed() ==> final(self).value().is_none(),
            old(self).closed() ==> !final(self).value_sent(),
            old(self).closed() ==> result.1.receiver.is_none(),
            !old(self).closed() ==> result.0 == Ok(()),
            !old(self).closed() ==> final(self).value() == old(self).value(),
            !old(self).closed() ==> final(self).value_sent(),
            !old(self).closed() ==> result.1.receiver == old(self).rx_waker(),
            result.1.sender.is_none(),
            !old(self).rx_alive() ==> final(self).inner_dropped(),
        no_unwind
    {
        if self.closed {
            let returned = self.value.take();
            self.sender_local_arc = false;
            self.send_staged = false;
            if !self.rx_alive {
                self.rx_waker = None;
                self.tx_waker = None;
                self.inner_dropped = true;
            }
            match returned {
                Some(value) => (Err(value), WakeEffects { receiver: None, sender: None }),
                None => unreached(),
            }
        } else {
            self.value_sent = true;
            self.sender_local_arc = false;
            self.send_staged = false;
            (Ok(()), WakeEffects { receiver: self.rx_waker, sender: None })
        }
    }

    /// Non-interfered convenience path; the split methods above expose the
    /// production store/CLOSED/CAS race to refinement callers.
    pub fn sender_send(&mut self, value: T)
        -> (result: (Result<(), T>, WakeEffects))
        requires old(self).well_formed(), old(self).tx_alive(),
        ensures final(self).well_formed(), !final(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).closed() == old(self).closed(),
            old(self).rx_alive() ==> final(self).tx_waker() == old(self).tx_waker(),
            old(self).closed() ==> result.0 == Err(value),
            old(self).closed() ==> final(self).value().is_none(),
            old(self).closed() ==> !final(self).value_sent(),
            old(self).closed() ==> result.1.receiver.is_none(),
            !old(self).closed() ==> result.0 == Ok(()),
            !old(self).closed() ==> final(self).value() == Some(value),
            !old(self).closed() ==> final(self).value_sent(),
            !old(self).closed() ==> result.1.receiver == old(self).rx_waker(),
            result.1.sender.is_none(),
            !old(self).rx_alive() ==> final(self).inner_dropped(),
        no_unwind
    {
        self.sender_begin_store(value);
        self.sender_finish_complete()
    }

    /// Sender Drop uses the same completion CAS but publishes no payload.
    pub fn sender_drop(&mut self) -> (effects: WakeEffects)
        requires old(self).well_formed(), old(self).tx_alive(),
        ensures final(self).well_formed(), !final(self).tx_alive(),
            final(self).rx_alive() == old(self).rx_alive(),
            final(self).value().is_none(),
            old(self).closed() ==> !final(self).value_sent(),
            !old(self).closed() ==> final(self).value_sent(),
            effects.receiver == if old(self).closed() { None } else { old(self).rx_waker() },
            effects.sender.is_none(),
            final(self).closed() == old(self).closed(),
            !old(self).rx_alive() ==> final(self).inner_dropped(),
        no_unwind
    {
        self.tx_alive = false;
        let receiver = if self.closed { None } else {
            self.value_sent = true;
            self.rx_waker
        };
        if !self.rx_alive {
            self.rx_waker = None;
            self.tx_waker = None;
            self.inner_dropped = true;
        }
        WakeEffects { receiver, sender: None }
    }

    /// `Inner::close`: set CLOSED, wake an installed sender only before
    /// completion, and destroy an RX task only when completion can no longer
    /// win. VALUE_SENT and CLOSED may both be true when send won first.
    pub fn receiver_close(&mut self) -> (effects: WakeEffects)
        requires old(self).well_formed(), old(self).rx_alive(),
        ensures final(self).well_formed(), final(self).closed(),
            final(self).value_sent() == old(self).value_sent(),
            final(self).value() == old(self).value(),
            final(self).tx_alive() == old(self).tx_alive(),
            final(self).rx_alive(),
            final(self).tx_waker() == old(self).tx_waker(),
            !old(self).value_sent() ==> final(self).rx_waker().is_none(),
            old(self).value_sent() ==> final(self).rx_waker() == old(self).rx_waker(),
            effects.sender == if old(self).value_sent() { None } else { old(self).tx_waker() },
            effects.receiver.is_none(),
        no_unwind
    {
        self.closed = true;
        let sender = if self.value_sent { None } else {
            self.rx_waker = None;
            self.tx_waker
        };
        WakeEffects { receiver: None, sender }
    }

    /// Exact nonblocking receive and outer `self.inner = None`. Unlike the old
    /// sequential model, successful/closed try_recv does not invent a CLOSED
    /// bit; it only releases the Receiver's Arc.
    pub fn receiver_try_recv(&mut self) -> (result: ProductionTryRecv<T>)
        requires old(self).well_formed(), old(self).rx_alive(),
        ensures final(self).well_formed(),
            result == ProductionTryRecv::Empty ==>
                final(self).rx_alive() && final(self).value() == old(self).value(),
            result != ProductionTryRecv::Empty ==> !final(self).rx_alive(),
            match result {
                ProductionTryRecv::Value(value) => old(self).value() == Some(value),
                ProductionTryRecv::Empty => !old(self).value_sent() && !old(self).closed(),
                ProductionTryRecv::Closed =>
                    (old(self).value_sent() && old(self).value().is_none())
                        || (!old(self).value_sent() && old(self).closed()),
            },
            final(self).closed() == old(self).closed(),
            result != ProductionTryRecv::Empty
                && !old(self).tx_alive() && !old(self).sender_local_arc() ==>
                final(self).inner_dropped(),
        no_unwind
    {
        if self.value_sent {
            let value = self.value.take();
            self.rx_alive = false;
            if !self.tx_alive && !self.sender_local_arc {
                self.rx_waker = None;
                self.tx_waker = None;
                self.inner_dropped = true;
            }
            match value {
                Some(value) => ProductionTryRecv::Value(value),
                None => ProductionTryRecv::Closed,
            }
        } else if self.closed {
            self.rx_alive = false;
            if !self.tx_alive && !self.sender_local_arc {
                self.rx_waker = None;
                self.tx_waker = None;
                self.inner_dropped = true;
            }
            ProductionTryRecv::Closed
        } else {
            ProductionTryRecv::Empty
        }
    }

    /// Ready branch shared by Future poll after the proved poll-core recheck.
    pub fn receiver_poll_ready(&mut self) -> (result: ProductionTryRecv<T>)
        requires old(self).well_formed(), old(self).rx_alive(),
            old(self).value_sent() || old(self).closed(),
        ensures final(self).well_formed(), !final(self).rx_alive(),
            result != ProductionTryRecv::Empty,
            !old(self).tx_alive() && !old(self).sender_local_arc() ==>
                final(self).inner_dropped(),
            match result {
                ProductionTryRecv::Value(value) => old(self).value() == Some(value),
                ProductionTryRecv::Closed =>
                    (old(self).value_sent() && old(self).value().is_none())
                        || (!old(self).value_sent() && old(self).closed()),
                ProductionTryRecv::Empty => false,
            },
        no_unwind
    {
        self.receiver_try_recv()
    }

    pub fn receiver_drop(&mut self) -> (result: (Option<T>, WakeEffects))
        requires old(self).well_formed(), old(self).rx_alive(),
        ensures final(self).well_formed(), !final(self).rx_alive(), final(self).closed(),
            result.0 == if old(self).value_sent() { old(self).value() } else { None },
            old(self).send_staged() ==>
                final(self).value() == old(self).value(),
            !old(self).send_staged() ==> final(self).value().is_none(),
            result.1.sender == if old(self).value_sent() { None } else { old(self).tx_waker() },
            result.1.receiver.is_none(),
            !old(self).tx_alive() && !old(self).sender_local_arc() ==>
                final(self).inner_dropped(),
        no_unwind
    {
        let effects = self.receiver_close();
        let value = if self.value_sent { self.value.take() } else { None };
        self.rx_alive = false;
        if !self.tx_alive && !self.sender_local_arc {
            self.rx_waker = None;
            self.tx_waker = None;
            self.inner_dropped = true;
        }
        (value, effects)
    }

    /// Production `try_recv` is repeatable after its outer `inner` option has
    /// been cleared: the terminal branch returns Closed without touching the
    /// already released endpoint.
    pub fn receiver_try_recv_terminal(&mut self) -> (result: ProductionTryRecv<T>)
        requires old(self).well_formed(), !old(self).rx_alive(),
        ensures *final(self) == *old(self), result == ProductionTryRecv::Closed,
        no_unwind
    {
        ProductionTryRecv::Closed
    }

    /// Production `Receiver::close` is a no-op after the outer option is None.
    pub fn receiver_close_terminal(&mut self)
        requires old(self).well_formed(), !old(self).rx_alive(),
        ensures *final(self) == *old(self),
        no_unwind
    {
    }

    pub fn is_empty(&self) -> (result: bool)
        requires self.well_formed(),
        ensures result == (!self.rx_alive() || !self.value_sent() || self.value().is_none()),
        no_unwind
    {
        if !self.rx_alive || !self.value_sent { true } else { self.value.is_empty() }
    }

    pub fn is_terminated(&self) -> (result: bool)
        requires self.well_formed(),
        ensures result == !self.rx_alive(),
        no_unwind
    { !self.rx_alive }
}

pub fn verify_send_receive_outer_roundtrip(value: u64, waker: u64)
{
    let mut channel = ProductionOneshot::new();
    channel.install_rx_waker(waker);
    let (sent, effects) = channel.sender_send(value);
    assert(sent == Ok(()));
    assert(effects.receiver == Some(waker));
    assert(channel.slot_capability() == SlotCapability::Receiver);
    let received = channel.receiver_poll_ready();
    assert(received == ProductionTryRecv::Value(value));
    assert(channel.inner_dropped());
    assert(channel.strong_count() == 0);
}

pub fn verify_close_rejects_send_and_returns_value(value: u64, tx_waker: u64)
{
    let mut channel = ProductionOneshot::new();
    channel.install_tx_waker(tx_waker);
    let effects = channel.receiver_close();
    assert(effects.sender == Some(tx_waker));
    let (sent, _) = channel.sender_send(value);
    assert(sent == Err(value));
    assert(!channel.value_sent());
    let received = channel.receiver_try_recv();
    assert(received == ProductionTryRecv::Closed);
    assert(channel.inner_dropped());
}

pub fn verify_close_between_store_and_completion_returns_exact_value(
    value: u64,
    tx_waker: u64,
)
{
    let mut channel = ProductionOneshot::new();
    channel.install_tx_waker(tx_waker);
    channel.sender_begin_store(value);
    assert(channel.slot_capability() == SlotCapability::Staged);
    assert(channel.strong_count() == 2);
    let observed_before_close = channel.exact_state_snapshot();
    let close_effects = channel.receiver_close();
    assert(close_effects.sender == Some(tx_waker));
    let actual_after_close = channel.exact_state_snapshot();
    let (retry, retry_state) = completion_cas_attempt(
        observed_before_close,
        actual_after_close,
        false,
    );
    assert(retry == CompletionAttempt::Retry);
    assert(retry_state == actual_after_close);
    let (closed, _) = completion_cas_attempt(
        actual_after_close,
        actual_after_close,
        true,
    );
    assert(closed == CompletionAttempt::Closed);
    let (sent, send_effects) = channel.sender_finish_complete();
    assert(sent == Err(value));
    assert(send_effects.receiver.is_none());
    assert(channel.value().is_none());
    assert(!channel.value_sent());
    let received = channel.receiver_try_recv();
    assert(received == ProductionTryRecv::Closed);
    assert(channel.inner_dropped());
}

pub fn verify_send_wins_then_close_preserves_value(value: u64)
{
    let mut channel = ProductionOneshot::new();
    let (sent, _) = channel.sender_send(value);
    assert(sent == Ok(()));
    channel.receiver_close();
    assert(channel.closed() && channel.value_sent());
    assert(channel.value() == Some(value));
    let received = channel.receiver_try_recv();
    match received {
        ProductionTryRecv::Value(received_value) => assert(received_value == value),
        ProductionTryRecv::Empty => assert(false),
        ProductionTryRecv::Closed => assert(false),
    }
}

pub fn verify_sender_drop_yields_closed_without_setting_closed()
{
    let mut channel = ProductionOneshot::<u64>::new();
    channel.sender_drop();
    assert(channel.value_sent());
    assert(!channel.closed());
    let received = channel.receiver_try_recv();
    assert(received == ProductionTryRecv::Closed);
}

pub fn verify_receiver_drop_consumes_sent_value(value: u64)
{
    let mut channel = ProductionOneshot::new();
    let (sent, _) = channel.sender_send(value);
    assert(sent == Ok(()));
    let (dropped, _) = channel.receiver_drop();
    assert(dropped == Some(value));
    assert(channel.inner_dropped());
}

pub fn verify_receiver_drop_between_store_and_completion_returns_exact_value(
    value: u64,
    tx_waker: u64,
)
{
    let mut channel = ProductionOneshot::new();
    channel.install_tx_waker(tx_waker);
    channel.sender_begin_store(value);
    let (receiver_value, drop_effects) = channel.receiver_drop();
    assert(receiver_value.is_none());
    assert(drop_effects.sender == Some(tx_waker));
    assert(!channel.rx_alive());
    assert(channel.sender_local_arc());
    assert(channel.value() == Some(value));
    let (sent, _) = channel.sender_finish_complete();
    assert(sent == Err(value));
    assert(channel.inner_dropped());
    assert(channel.strong_count() == 0);
    assert(channel.tx_waker().is_none());
}

pub fn verify_tx_waker_released_by_receiver_terminal_drop(value: u64, tx_waker: u64)
{
    let mut channel = ProductionOneshot::new();
    channel.install_tx_waker(tx_waker);
    let (sent, _) = channel.sender_send(value);
    assert(sent == Ok(()));
    assert(!channel.tx_alive());
    assert(channel.tx_waker() == Some(tx_waker));
    let close_effects = channel.receiver_close();
    assert(close_effects.sender.is_none());
    assert(channel.tx_waker() == Some(tx_waker));
    let (dropped, _) = channel.receiver_drop();
    assert(dropped == Some(value));
    assert(channel.tx_waker().is_none());
    assert(channel.inner_dropped());
}

pub fn verify_repeated_terminal_try_recv_and_close()
{
    let mut channel = ProductionOneshot::<u64>::new();
    channel.sender_drop();
    let first = channel.receiver_try_recv();
    assert(first == ProductionTryRecv::Closed);
    let second = channel.receiver_try_recv_terminal();
    assert(second == ProductionTryRecv::Closed);
    channel.receiver_close_terminal();
    let third = channel.receiver_try_recv_terminal();
    assert(third == ProductionTryRecv::Closed);
    assert(channel.inner_dropped());
}

pub fn verify_cas_retry_then_close()
{
    let initial = ExactStateSnapshot::new();
    let registered = initial.with_rx_task();
    assert(registered.raw() == RX_TASK_SET as nat);
    assert(!registered.value_sent());
    let (first, _) = completion_cas_attempt(initial, registered, false);
    assert(first == CompletionAttempt::Retry);
    let closed = registered.with_closed();
    let (second, _) = completion_cas_attempt(
        registered, closed, false);
    assert(second == CompletionAttempt::Retry);
    let (third, final_state) = completion_cas_attempt(closed, closed, true);
    assert(third == CompletionAttempt::Closed);
    assert(!final_state.value_sent());
}

pub fn verify_cas_retry_then_complete()
{
    let initial = ExactStateSnapshot::new();
    let registered = initial.with_rx_task();
    assert(registered.raw() == RX_TASK_SET as nat);
    assert(!registered.value_sent());
    let (first, _) = completion_cas_attempt(initial, registered, false);
    assert(first == CompletionAttempt::Retry);
    let (second, final_state) = completion_cas_attempt(
        registered, registered, true);
    assert(second == CompletionAttempt::Completed);
    assert(final_state.value_sent());
    assert(final_state.raw() == RX_TASK_SET as nat + VALUE_SENT as nat);
    assert((RX_TASK_SET | VALUE_SENT) as nat
        == RX_TASK_SET as nat + VALUE_SENT as nat) by (bit_vector);
}

pub proof fn verify_exact_state_mutants_rejected()
{
    assert(KNOWN_BITS == 0b01111usize) by (bit_vector);
    assert((KNOWN_BITS & 0b10000usize) == 0) by (bit_vector);
    assert((CLOSED | VALUE_SENT) != CLOSED) by (bit_vector);
    assert(RX_TASK_SET != TX_TASK_SET);
}

} // verus!
