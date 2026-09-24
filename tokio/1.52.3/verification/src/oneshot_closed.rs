use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ClosedPollState {
    Idle,
    Registering,
    Waiting,
    Done,
    Dropped,
}

/// Projection of `Sender::poll_closed` and the TX_TASK_SET slot. Waker ids
/// model identity/replacement while actual clone, wake, and destructor calls
/// remain at the standard poll boundary.
pub struct SenderClosedCore {
    state: ClosedPollState,
    receiver_closed: bool,
    send_complete: bool,
    installed_waker: Option<u64>,
    pending_waker: Option<u64>,
}

impl SenderClosedCore {
    pub closed spec fn state(&self) -> ClosedPollState { self.state }
    pub closed spec fn receiver_closed(&self) -> bool { self.receiver_closed }
    pub closed spec fn send_complete(&self) -> bool { self.send_complete }
    pub closed spec fn installed_waker(&self) -> Option<u64> { self.installed_waker }
    pub closed spec fn pending_waker(&self) -> Option<u64> { self.pending_waker }

    pub closed spec fn well_formed(&self) -> bool {
        match self.state() {
            ClosedPollState::Idle => {
                self.installed_waker().is_none() && self.pending_waker().is_none()
            },
            ClosedPollState::Registering => self.pending_waker().is_some(),
            ClosedPollState::Waiting => {
                self.pending_waker().is_none()
                    && ((!self.receiver_closed() && !self.send_complete())
                        ==> self.installed_waker().is_some())
            },
            ClosedPollState::Done | ClosedPollState::Dropped => {
                self.installed_waker().is_none() && self.pending_waker().is_none()
            },
        }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.state() == ClosedPollState::Idle,
            !result.receiver_closed(),
            !result.send_complete(),
    {
        SenderClosedCore {
            state: ClosedPollState::Idle,
            receiver_closed: false,
            send_complete: false,
            installed_waker: None,
            pending_waker: None,
        }
    }

    /// `Some(true)` is Ready; `None` continues through task-bit publication.
    pub fn begin_poll_closed(&mut self, waker: u64) -> (result: Option<bool>)
        requires
            old(self).well_formed(),
            old(self).state() == ClosedPollState::Idle
                || old(self).state() == ClosedPollState::Waiting,
        ensures
            final(self).well_formed(),
            final(self).receiver_closed() == old(self).receiver_closed(),
            final(self).send_complete() == old(self).send_complete(),
            match result {
                Some(ready) => {
                    ready
                        && old(self).receiver_closed()
                        && final(self).state() == ClosedPollState::Done
                },
                None => {
                    !old(self).receiver_closed()
                        && final(self).state() == ClosedPollState::Registering
                        && final(self).installed_waker().is_none()
                        && final(self).pending_waker() == Some(waker)
                },
            },
        no_unwind
    {
        if self.receiver_closed {
            self.installed_waker = None;
            self.pending_waker = None;
            self.state = ClosedPollState::Done;
            Some(true)
        } else {
            self.installed_waker = None;
            self.pending_waker = Some(waker);
            self.state = ClosedPollState::Registering;
            None
        }
    }

    pub fn finish_registration(&mut self) -> (ready: bool)
        requires
            old(self).well_formed(),
            old(self).state() == ClosedPollState::Registering,
        ensures
            final(self).well_formed(),
            final(self).receiver_closed() == old(self).receiver_closed(),
            final(self).send_complete() == old(self).send_complete(),
            ready == old(self).receiver_closed(),
            ready ==> final(self).state() == ClosedPollState::Done,
            !ready ==> final(self).state() == ClosedPollState::Waiting,
            !ready ==> final(self).installed_waker() == old(self).pending_waker(),
        no_unwind
    {
        if self.receiver_closed {
            self.installed_waker = None;
            self.pending_waker = None;
            self.state = ClosedPollState::Done;
            true
        } else {
            self.installed_waker = self.pending_waker.take();
            self.state = ClosedPollState::Waiting;
            false
        }
    }

    /// Close linearizes before wake. No wake is needed after a successful send
    /// has consumed the Sender endpoint.
    pub fn receiver_close(&mut self) -> (wake: Option<u64>)
        requires
            old(self).well_formed(),
            old(self).state() != ClosedPollState::Done,
            old(self).state() != ClosedPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).receiver_closed(),
            final(self).send_complete() == old(self).send_complete(),
            final(self).state() == old(self).state(),
            final(self).installed_waker() == old(self).installed_waker(),
            final(self).pending_waker() == old(self).pending_waker(),
            wake == if old(self).send_complete() {
                None
            } else {
                old(self).installed_waker()
            },
        no_unwind
    {
        self.receiver_closed = true;
        if self.send_complete {
            None
        } else {
            self.installed_waker
        }
    }

    pub fn complete_send(&mut self)
        requires
            old(self).well_formed(),
            !old(self).send_complete(),
            old(self).state() != ClosedPollState::Done,
            old(self).state() != ClosedPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).send_complete(),
            final(self).receiver_closed() == old(self).receiver_closed(),
        no_unwind
    {
        self.send_complete = true;
    }

    pub fn drop_sender(&mut self)
        requires
            old(self).well_formed(),
            old(self).state() != ClosedPollState::Done,
            old(self).state() != ClosedPollState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).state() == ClosedPollState::Dropped,
            final(self).installed_waker().is_none(),
            final(self).pending_waker().is_none(),
        no_unwind
    {
        self.installed_waker = None;
        self.pending_waker = None;
        self.state = ClosedPollState::Dropped;
    }
}

pub fn verify_close_during_tx_registration(waker: u64)
{
    let mut core = SenderClosedCore::new();
    let first = core.begin_poll_closed(waker);
    match first {
        None => {},
        Some(_) => unreached(),
    }
    let wake = core.receiver_close();
    assert(wake.is_none());
    let ready = core.finish_registration();
    assert(ready);
}

pub fn verify_close_wakes_registered_sender(waker: u64)
{
    let mut core = SenderClosedCore::new();
    let first = core.begin_poll_closed(waker);
    match first {
        None => {},
        Some(_) => unreached(),
    }
    let pending = core.finish_registration();
    assert(!pending);
    let wake = core.receiver_close();
    assert(wake == Some(waker));
}

pub fn verify_tx_waker_replacement(first: u64, second: u64)
{
    let mut core = SenderClosedCore::new();
    let initial = core.begin_poll_closed(first);
    match initial {
        None => {},
        Some(_) => unreached(),
    }
    let pending = core.finish_registration();
    assert(!pending);
    let replace = core.begin_poll_closed(second);
    match replace {
        None => {},
        Some(_) => unreached(),
    }
    let pending_again = core.finish_registration();
    assert(!pending_again);
    assert(core.installed_waker() == Some(second));
}

} // verus!
