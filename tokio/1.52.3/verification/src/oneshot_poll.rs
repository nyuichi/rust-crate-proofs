use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReceiverPollState {
    Idle,
    Registering,
    Waiting,
    Done,
    Dropped,
}

#[derive(PartialEq, Eq)]
pub enum ReceiverPollResult<T> {
    Pending,
    Value(T),
    Closed,
}

/// Non-Pin, non-Waker projection of production `Inner::poll_recv`. Wakers are
/// represented by stable ids; cloning, wake execution, and Context access stay
/// at the poll-surface adapter.
pub struct ReceiverPollCore<T> {
    state: ReceiverPollState,
    complete: bool,
    closed: bool,
    value: Option<T>,
    installed_waker: Option<u64>,
    pending_waker: Option<u64>,
}

impl<T> ReceiverPollCore<T> {
    pub closed spec fn state(&self) -> ReceiverPollState { self.state }
    pub closed spec fn complete(&self) -> bool { self.complete }
    pub closed spec fn closed(&self) -> bool { self.closed }
    pub closed spec fn value(&self) -> Option<T> { self.value }
    pub closed spec fn installed_waker(&self) -> Option<u64> { self.installed_waker }
    pub closed spec fn pending_waker(&self) -> Option<u64> { self.pending_waker }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.value().is_some() ==> self.complete()
        &&& match self.state() {
            ReceiverPollState::Idle => {
                self.installed_waker().is_none() && self.pending_waker().is_none()
            },
            ReceiverPollState::Registering => self.pending_waker().is_some(),
            ReceiverPollState::Waiting => {
                self.pending_waker().is_none()
                    && ((!self.complete() && !self.closed())
                        ==> self.installed_waker().is_some())
            },
            ReceiverPollState::Done | ReceiverPollState::Dropped => {
                self.installed_waker().is_none() && self.pending_waker().is_none()
            },
        }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.state() == ReceiverPollState::Idle,
            !result.complete(),
            !result.closed(),
            result.value().is_none(),
    {
        ReceiverPollCore {
            state: ReceiverPollState::Idle,
            complete: false,
            closed: false,
            value: None,
            installed_waker: None,
            pending_waker: None,
        }
    }

    fn finish_ready(&mut self) -> (result: ReceiverPollResult<T>)
        requires
            old(self).well_formed(),
            old(self).complete() || old(self).closed(),
            old(self).state() != ReceiverPollState::Done,
            old(self).state() != ReceiverPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).complete() == old(self).complete(),
            final(self).closed() == old(self).closed(),
            final(self).state() == ReceiverPollState::Done,
            final(self).value().is_none(),
            match result {
                ReceiverPollResult::Value(value) => old(self).value() == Some(value),
                ReceiverPollResult::Closed => old(self).value().is_none(),
                ReceiverPollResult::Pending => false,
            },
        no_unwind
    {
        let value = self.value.take();
        self.installed_waker = None;
        self.pending_waker = None;
        self.state = ReceiverPollState::Done;
        match value {
            Some(value) => ReceiverPollResult::Value(value),
            None => ReceiverPollResult::Closed,
        }
    }

    /// First half of polling. `None` means the caller must perform the atomic
    /// task-bit publication and then call `finish_registration`.
    pub fn begin_poll(&mut self, waker: u64)
        -> (result: Option<ReceiverPollResult<T>>)
        requires
            old(self).well_formed(),
            old(self).state() == ReceiverPollState::Idle
                || old(self).state() == ReceiverPollState::Waiting,
        ensures
            final(self).well_formed(),
            final(self).complete() == old(self).complete(),
            final(self).closed() == old(self).closed(),
            match result {
                Some(ReceiverPollResult::Value(value)) => {
                    old(self).value() == Some(value)
                        && final(self).state() == ReceiverPollState::Done
                },
                Some(ReceiverPollResult::Closed) => {
                    (old(self).complete() || old(self).closed())
                        && old(self).value().is_none()
                        && final(self).state() == ReceiverPollState::Done
                },
                Some(ReceiverPollResult::Pending) => false,
                None => {
                    !old(self).complete()
                        && !old(self).closed()
                        && final(self).value() == old(self).value()
                        && final(self).state() == ReceiverPollState::Registering
                        && final(self).installed_waker().is_none()
                        && final(self).pending_waker() == Some(waker)
                },
            },
        no_unwind
    {
        if self.complete || self.closed {
            Some(self.finish_ready())
        } else {
            self.installed_waker = None;
            self.pending_waker = Some(waker);
            self.state = ReceiverPollState::Registering;
            None
        }
    }

    /// Locked/atomic recheck after the waker has been written but before the
    /// RX_TASK_SET publication is considered stable.
    pub fn finish_registration(&mut self) -> (result: ReceiverPollResult<T>)
        requires
            old(self).well_formed(),
            old(self).state() == ReceiverPollState::Registering,
        ensures
            final(self).well_formed(),
            final(self).complete() == old(self).complete(),
            final(self).closed() == old(self).closed(),
            match result {
                ReceiverPollResult::Pending => {
                    !old(self).complete()
                        && !old(self).closed()
                        && final(self).value() == old(self).value()
                        && final(self).state() == ReceiverPollState::Waiting
                        && final(self).installed_waker() == old(self).pending_waker()
                },
                ReceiverPollResult::Value(value) => {
                    old(self).value() == Some(value)
                        && final(self).state() == ReceiverPollState::Done
                },
                ReceiverPollResult::Closed => {
                    (old(self).complete() || old(self).closed())
                        && old(self).value().is_none()
                        && final(self).state() == ReceiverPollState::Done
                },
            },
        no_unwind
    {
        if self.complete || self.closed {
            self.finish_ready()
        } else {
            let pending = self.pending_waker.take();
            self.installed_waker = pending;
            self.state = ReceiverPollState::Waiting;
            ReceiverPollResult::Pending
        }
    }

    /// Sender publication. The returned id is exactly the installed receiver
    /// waker which production wakes after setting VALUE_SENT.
    pub fn send(&mut self, value: T) -> (result: Result<Option<u64>, T>)
        requires
            old(self).well_formed(),
            !old(self).complete(),
            old(self).state() != ReceiverPollState::Done,
            old(self).state() != ReceiverPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).state() == old(self).state(),
            final(self).pending_waker() == old(self).pending_waker(),
            final(self).installed_waker() == old(self).installed_waker(),
            old(self).closed() ==> result == Err(value),
            !old(self).closed() ==> final(self).complete(),
            !old(self).closed() ==> final(self).value() == Some(value),
            !old(self).closed() ==> result == Ok(old(self).installed_waker()),
        no_unwind
    {
        if self.closed {
            Err(value)
        } else {
            self.value = Some(value);
            self.complete = true;
            Ok(self.installed_waker)
        }
    }

    pub fn sender_drop(&mut self) -> (wake: Option<u64>)
        requires
            old(self).well_formed(),
            !old(self).complete(),
            old(self).state() != ReceiverPollState::Done,
            old(self).state() != ReceiverPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).complete() == !old(self).closed(),
            final(self).value().is_none(),
            wake == if old(self).closed() { None } else { old(self).installed_waker() },
        no_unwind
    {
        if self.closed {
            None
        } else {
            self.complete = true;
            self.installed_waker
        }
    }

    pub fn close(&mut self)
        requires
            old(self).well_formed(),
            old(self).state() != ReceiverPollState::Done,
            old(self).state() != ReceiverPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).closed(),
            final(self).complete() == old(self).complete(),
            final(self).value() == old(self).value(),
        no_unwind
    {
        self.closed = true;
        if !self.complete {
            self.installed_waker = None;
        }
    }

    pub fn drop_receiver(&mut self) -> (dropped: Option<T>)
        requires
            old(self).well_formed(),
            old(self).state() != ReceiverPollState::Done,
            old(self).state() != ReceiverPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).state() == ReceiverPollState::Dropped,
            final(self).closed(),
            final(self).value().is_none(),
            dropped == old(self).value(),
        no_unwind
    {
        self.closed = true;
        let dropped = self.value.take();
        self.installed_waker = None;
        self.pending_waker = None;
        self.state = ReceiverPollState::Dropped;
        dropped
    }
}

pub fn verify_send_during_receiver_registration(value: u64, waker: u64)
{
    let mut core = ReceiverPollCore::new();
    let first = core.begin_poll(waker);
    match first {
        None => {},
        Some(_) => unreached(),
    }
    let wake = core.send(value);
    assert(wake == Ok(None));
    let ready = core.finish_registration();
    assert(ready == ReceiverPollResult::Value(value));
}

pub fn verify_send_wakes_registered_receiver(value: u64, waker: u64)
{
    let mut core = ReceiverPollCore::new();
    let first = core.begin_poll(waker);
    match first {
        None => {},
        Some(_) => unreached(),
    }
    let pending = core.finish_registration();
    match pending {
        ReceiverPollResult::Pending => {},
        ReceiverPollResult::Value(_) | ReceiverPollResult::Closed => unreached(),
    }
    let wake = core.send(value);
    assert(wake == Ok(Some(waker)));
    let ready = core.begin_poll(waker);
    assert(ready == Some(ReceiverPollResult::Value(value)));
}

pub fn verify_receiver_waker_replacement(first: u64, second: u64)
{
    let mut core = ReceiverPollCore::<u64>::new();
    let start = core.begin_poll(first);
    match start {
        None => {},
        Some(_) => unreached(),
    }
    let first_pending = core.finish_registration();
    match first_pending {
        ReceiverPollResult::Pending => {},
        ReceiverPollResult::Value(_) | ReceiverPollResult::Closed => unreached(),
    }
    let replace = core.begin_poll(second);
    match replace {
        None => {},
        Some(_) => unreached(),
    }
    let second_pending = core.finish_registration();
    match second_pending {
        ReceiverPollResult::Pending => {},
        ReceiverPollResult::Value(_) | ReceiverPollResult::Closed => unreached(),
    }
    assert(core.installed_waker() == Some(second));
}

} // verus!
