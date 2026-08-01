use crate::watch_protocol::{WatchChange, WatchReceiver};
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchUpdateKind {
    Modified,
    Unmodified,
    Panicked,
}

/// Exact logical view of production AtomicState. `generation` represents the
/// even version bits and `closed` the low bit. The branch in `advance` is the
/// wrapping `fetch_add(2)` case, proved without relying on machine overflow.
pub struct EncodedWatchState {
    generation: usize,
    closed: bool,
}

impl EncodedWatchState {
    pub open spec fn max_generation() -> nat { usize::MAX as nat / 2 }
    pub closed spec fn generation(&self) -> usize { self.generation }
    pub closed spec fn closed(&self) -> bool { self.closed }

    pub closed spec fn well_formed(&self) -> bool {
        self.generation() as nat <= Self::max_generation()
    }

    pub closed spec fn raw(&self) -> nat {
        2nat * self.generation() as nat + if self.closed() { 1nat } else { 0nat }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.generation() == 0,
            !result.closed(),
            result.raw() == 0,
        no_unwind
    {
        EncodedWatchState { generation: 0, closed: false }
    }

    pub fn advance(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).closed() == old(self).closed(),
            old(self).generation() == usize::MAX / 2 ==>
                final(self).generation() == 0,
            old(self).generation() < usize::MAX / 2 ==>
                final(self).generation() == old(self).generation() + 1,
            final(self).raw() % 2 == old(self).raw() % 2,
        no_unwind
    {
        if self.generation == usize::MAX / 2 {
            self.generation = 0;
        } else {
            self.generation += 1;
        }
    }

    pub fn close(&mut self) -> (already_closed: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            already_closed == old(self).closed(),
            final(self).closed(),
            final(self).generation() == old(self).generation(),
            final(self).raw() == 2 * old(self).generation() as nat + 1,
        no_unwind
    {
        let previous = self.closed;
        self.closed = true;
        previous
    }
}

/// RwLock-independent exact-value view of all watch update forms. A user
/// closure may mutate before returning false or panicking. Both outcomes expose
/// the new value but do not advance the generation and do not notify.
pub struct WatchUpdate<T> {
    value: T,
    state: EncodedWatchState,
    notified_generation: usize,
}

impl<T> WatchUpdate<T> {
    pub closed spec fn value(&self) -> T { self.value }
    pub closed spec fn generation(&self) -> usize { self.state.generation() }
    pub closed spec fn closed(&self) -> bool { self.state.closed() }
    pub closed spec fn notified_generation(&self) -> usize { self.notified_generation }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.state.well_formed()
        &&& self.notified_generation() <= self.generation()
    }

    pub fn new(value: T) -> (result: Self)
        ensures
            result.well_formed(),
            result.value() == value,
            result.generation() == 0,
            result.notified_generation() == 0,
            !result.closed(),
        no_unwind
    {
        WatchUpdate { value, state: EncodedWatchState::new(), notified_generation: 0 }
    }

    /// Models the closure result after releasing neither the value write lock
    /// nor any notification. `previous` makes ownership conservation explicit.
    pub fn apply_update(&mut self, value: T, kind: WatchUpdateKind) -> (previous: T)
        requires
            old(self).well_formed(),
            old(self).generation() < usize::MAX / 2,
        ensures
            final(self).well_formed(),
            previous == old(self).value(),
            final(self).value() == value,
            kind == WatchUpdateKind::Modified ==>
                final(self).generation() == old(self).generation() + 1,
            kind == WatchUpdateKind::Modified ==>
                final(self).notified_generation() == final(self).generation(),
            kind != WatchUpdateKind::Modified ==>
                final(self).generation() == old(self).generation(),
            kind != WatchUpdateKind::Modified ==>
                final(self).notified_generation() == old(self).notified_generation(),
            final(self).closed() == old(self).closed(),
        no_unwind
    {
        let mut replacement = WatchUpdate {
            value,
            state: EncodedWatchState {
                generation: self.state.generation,
                closed: self.state.closed,
            },
            notified_generation: self.notified_generation,
        };
        core::mem::swap(&mut self.value, &mut replacement.value);
        match kind {
            WatchUpdateKind::Modified => {
                let previous_generation = self.state.generation;
                self.state.advance();
                assert(self.state.generation == previous_generation + 1);
                // Production releases the write lock after advancing the state and
                // only then calls notify_waiters. This assignment is that linear
                // notification phase in the proof view.
                self.notified_generation = self.state.generation;
            },
            WatchUpdateKind::Unmodified | WatchUpdateKind::Panicked => {},
        }
        replacement.value
    }

    pub fn close(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).closed(),
            final(self).value() == old(self).value(),
            final(self).generation() == old(self).generation(),
            final(self).notified_generation() == old(self).notified_generation(),
        no_unwind
    {
        self.state.close();
    }

    pub fn snapshot(&self) -> (result: T)
        where T: Copy
        requires self.well_formed(),
        ensures result == self.value(),
        no_unwind
    {
        self.value
    }
}

pub fn verify_watch_modified_reaches_registered_receiver(first: u64, second: u64)
{
    let mut channel = WatchUpdate::new(first);
    let mut receiver = WatchReceiver::new(0);
    let initial = receiver.begin_changed(0, false);
    assert(initial == WatchChange::Pending);
    let previous = channel.apply_update(second, WatchUpdateKind::Modified);
    assert(previous == first);
    let completed = receiver.finish_registration(
        channel.state.generation as u64,
        channel.state.closed,
    );
    assert(completed == WatchChange::Changed);
    let observed = channel.snapshot();
    assert(observed == second);
    assert(channel.notified_generation() == channel.generation());
}

pub fn verify_watch_unmodified_and_panic_do_not_notify(first: u64, changed: u64)
{
    let mut channel = WatchUpdate::new(first);
    let previous = channel.apply_update(changed, WatchUpdateKind::Unmodified);
    assert(previous == first);
    assert(channel.generation() == 0);
    assert(channel.notified_generation() == 0);
    let after_silent_change = channel.apply_update(first, WatchUpdateKind::Panicked);
    assert(after_silent_change == changed);
    assert(channel.generation() == 0);
    assert(channel.notified_generation() == 0);
}

pub fn verify_watch_wrapping_preserves_closed_bit()
{
    let mut state = EncodedWatchState {
        generation: (usize::MAX / 2),
        closed: true,
    };
    assert(state.well_formed());
    state.advance();
    assert(state.generation() == 0);
    assert(state.closed());
    assert(state.raw() == 1);
}

} // verus!
