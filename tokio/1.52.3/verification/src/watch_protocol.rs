use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchChange {
    Pending,
    Changed,
    Closed,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchWaitState {
    Idle,
    Registering,
    Waiting,
}

/// Channel-wide state. Production encodes `closed` in the low bit and advances
/// the version by two; this proof view uses a logical generation. Integer wrap
/// and the raw atomic representation stay in the ordered-atomic adapter.
pub struct WatchState {
    generation: u64,
    closed: bool,
    senders: u64,
    receivers: u64,
}

impl WatchState {
    pub closed spec fn generation(&self) -> u64 { self.generation }
    pub closed spec fn closed(&self) -> bool { self.closed }
    pub closed spec fn senders(&self) -> u64 { self.senders }
    pub closed spec fn receivers(&self) -> u64 { self.receivers }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.closed() == (self.senders() == 0)
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.generation() == 0,
            result.senders() == 1,
            result.receivers() == 1,
            !result.closed(),
        no_unwind
    {
        WatchState { generation: 0, closed: false, senders: 1, receivers: 1 }
    }

    pub fn clone_sender(&mut self)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
            old(self).senders() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).senders() == old(self).senders() + 1,
            final(self).receivers() == old(self).receivers(),
            final(self).generation() == old(self).generation(),
        no_unwind
    {
        self.senders += 1;
    }

    pub fn clone_receiver(&mut self) -> (seen: u64)
        requires
            old(self).well_formed(),
            old(self).receivers() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).receivers() == old(self).receivers() + 1,
            final(self).senders() == old(self).senders(),
            final(self).generation() == old(self).generation(),
            seen == old(self).generation(),
        no_unwind
    {
        self.receivers += 1;
        self.generation
    }

    pub fn drop_receiver(&mut self) -> (last: bool)
        requires
            old(self).well_formed(),
            old(self).receivers() > 0,
        ensures
            final(self).well_formed(),
            final(self).receivers() + 1 == old(self).receivers(),
            final(self).senders() == old(self).senders(),
            final(self).generation() == old(self).generation(),
            last == (final(self).receivers() == 0),
        no_unwind
    {
        self.receivers -= 1;
        self.receivers == 0
    }

    pub fn drop_sender(&mut self) -> (closed_now: bool)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
        ensures
            final(self).well_formed(),
            final(self).senders() + 1 == old(self).senders(),
            final(self).receivers() == old(self).receivers(),
            final(self).generation() == old(self).generation(),
            closed_now == (old(self).senders() == 1),
            closed_now == final(self).closed(),
        no_unwind
    {
        self.senders -= 1;
        if self.senders == 0 {
            self.closed = true;
            true
        } else {
            false
        }
    }

    /// Linearization point for `send`, `send_replace`, and successful
    /// `send_if_modified`. The caller holds the production write lock.
    pub fn publish(&mut self) -> (notified: bool)
        requires
            old(self).well_formed(),
            old(self).senders() > 0,
            old(self).generation() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).generation() == old(self).generation() + 1,
            final(self).senders() == old(self).senders(),
            final(self).receivers() == old(self).receivers(),
            notified == (old(self).receivers() > 0),
        no_unwind
    {
        self.generation += 1;
        self.receivers > 0
    }

    pub fn send_allowed(&self) -> (result: bool)
        requires self.well_formed(),
        ensures result == (self.receivers() > 0),
        no_unwind
    {
        self.receivers > 0
    }
}

/// Exact-value replacement view of the production RwLock-protected slot.
pub struct WatchValue<T> {
    value: T,
}

impl<T> WatchValue<T> {
    pub closed spec fn value(&self) -> T { self.value }

    pub fn new(value: T) -> (result: Self)
        ensures result.value() == value,
        no_unwind
    {
        WatchValue { value }
    }

    pub fn replace(&mut self, value: T) -> (previous: T)
        ensures
            previous == old(self).value(),
            final(self).value() == value,
        no_unwind
    {
        let mut replacement = WatchValue { value };
        core::mem::swap(self, &mut replacement);
        replacement.value
    }

    pub fn borrow_by_value(&self) -> (result: T)
        where T: Copy
        ensures result == self.value(),
        no_unwind
    {
        self.value
    }
}

/// Per-Receiver state for `changed()`. Notification delivery is abstracted to
/// a boolean generation recheck; the proof keeps the lost-wakeup-sensitive
/// register-then-recheck ordering explicit.
pub struct WatchReceiver {
    seen: u64,
    wait: WatchWaitState,
    registered_generation: Option<u64>,
}

impl WatchReceiver {
    pub closed spec fn seen(&self) -> u64 { self.seen }
    pub closed spec fn wait(&self) -> WatchWaitState { self.wait }
    pub closed spec fn registered_generation(&self) -> Option<u64> {
        self.registered_generation
    }

    pub closed spec fn well_formed(&self) -> bool {
        match self.wait() {
            WatchWaitState::Idle => self.registered_generation().is_none(),
            WatchWaitState::Registering | WatchWaitState::Waiting => {
                self.registered_generation().is_some()
            },
        }
    }

    pub fn new(generation: u64) -> (result: Self)
        ensures
            result.well_formed(),
            result.seen() == generation,
            result.wait() == WatchWaitState::Idle,
        no_unwind
    {
        WatchReceiver { seen: generation, wait: WatchWaitState::Idle,
            registered_generation: None }
    }

    pub fn borrow_and_update(&mut self, generation: u64) -> (changed: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).seen() == generation,
            changed == (old(self).seen() != generation),
            final(self).wait() == old(self).wait(),
        no_unwind
    {
        let changed = self.seen != generation;
        self.seen = generation;
        changed
    }

    pub fn begin_changed(&mut self, generation: u64, closed: bool)
        -> (result: WatchChange)
        requires
            old(self).well_formed(),
            old(self).wait() == WatchWaitState::Idle,
        ensures
            final(self).well_formed(),
            result == WatchChange::Changed ==> final(self).seen() == generation,
            result == WatchChange::Changed ==> old(self).seen() != generation,
            result == WatchChange::Closed ==> closed && old(self).seen() == generation,
            result == WatchChange::Pending ==> !closed && old(self).seen() == generation,
            result == WatchChange::Pending ==> final(self).seen() == old(self).seen(),
            result == WatchChange::Pending ==> final(self).wait() == WatchWaitState::Registering,
            result == WatchChange::Pending ==>
                final(self).registered_generation() == Some(generation),
        no_unwind
    {
        if self.seen != generation {
            self.seen = generation;
            WatchChange::Changed
        } else if closed {
            WatchChange::Closed
        } else {
            self.wait = WatchWaitState::Registering;
            self.registered_generation = Some(generation);
            WatchChange::Pending
        }
    }

    pub fn finish_registration(&mut self, generation: u64, closed: bool)
        -> (result: WatchChange)
        requires
            old(self).well_formed(),
            old(self).wait() == WatchWaitState::Registering,
        ensures
            final(self).well_formed(),
            result == WatchChange::Changed ==> final(self).seen() == generation,
            result == WatchChange::Changed ==> old(self).seen() != generation,
            result == WatchChange::Closed ==> closed && old(self).seen() == generation,
            result == WatchChange::Pending ==> !closed && old(self).seen() == generation,
            result == WatchChange::Pending ==> final(self).wait() == WatchWaitState::Waiting,
            result != WatchChange::Pending ==> final(self).wait() == WatchWaitState::Idle,
        no_unwind
    {
        if self.seen != generation {
            self.seen = generation;
            self.wait = WatchWaitState::Idle;
            self.registered_generation = None;
            WatchChange::Changed
        } else if closed {
            self.wait = WatchWaitState::Idle;
            self.registered_generation = None;
            WatchChange::Closed
        } else {
            self.wait = WatchWaitState::Waiting;
            WatchChange::Pending
        }
    }

    pub fn notified(&mut self)
        requires
            old(self).well_formed(),
            old(self).wait() == WatchWaitState::Waiting,
        ensures
            final(self).well_formed(),
            final(self).seen() == old(self).seen(),
            final(self).wait() == WatchWaitState::Idle,
            final(self).registered_generation().is_none(),
        no_unwind
    {
        self.wait = WatchWaitState::Idle;
        self.registered_generation = None;
    }
}

pub fn verify_watch_roundtrip(first: u64, second: u64)
{
    let mut state = WatchState::new();
    let mut value = WatchValue::new(first);
    let mut receiver = WatchReceiver::new(state.generation);
    let pending = receiver.begin_changed(state.generation, state.closed);
    assert(pending == WatchChange::Pending);
    let previous = value.replace(second);
    assert(previous == first);
    state.publish();
    let changed = receiver.finish_registration(state.generation, state.closed);
    assert(changed == WatchChange::Changed);
    let observed = value.borrow_by_value();
    assert(observed == second);
}

pub fn verify_only_last_sender_closes()
{
    let mut state = WatchState::new();
    state.clone_sender();
    let first_drop = state.drop_sender();
    assert(!first_drop);
    assert(!state.closed());
    let last_drop = state.drop_sender();
    assert(last_drop);
    assert(state.closed());
}

pub fn verify_independent_receiver_versions()
{
    let mut state = WatchState::new();
    let mut first = WatchReceiver::new(state.generation);
    let second_seen = state.clone_receiver();
    let mut second = WatchReceiver::new(second_seen);
    state.publish();
    let first_changed = first.borrow_and_update(state.generation);
    assert(first_changed);
    assert(first.seen() == state.generation());
    assert(second.seen() != state.generation());
    let second_changed = second.begin_changed(state.generation, state.closed);
    assert(second_changed == WatchChange::Changed);
}

} // verus!
