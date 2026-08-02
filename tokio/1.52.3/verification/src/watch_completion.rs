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

/// Owned phase witness returned by a successful lock-held publication. Caller
/// proofs use it to record the generation before modeling the distinct
/// post-unlock `BigNotify::notify_waiters` call. This ordinary exec value does
/// not by itself enforce fanout at the Rust type level.
pub struct WatchNotification {
    generation: usize,
}

impl WatchNotification {
    pub closed spec fn generation(&self) -> usize { self.generation }

    pub fn complete(self) -> (generation: usize)
        ensures generation == self.generation(),
        no_unwind
    {
        self.generation
    }
}

/// RwLock-independent exact-value view of the lock-held phase of every watch
/// update form. The caller supplies the arbitrary post-closure value and
/// outcome. False-returning and panicking closures may therefore mutate the
/// value, but neither advances the generation nor creates a notification
/// phase witness.
pub struct WatchUpdate<T> {
    value: T,
    state: EncodedWatchState,
}

impl<T> WatchUpdate<T> {
    pub closed spec fn value(&self) -> T { self.value }
    pub closed spec fn generation(&self) -> usize { self.state.generation() }
    pub closed spec fn closed(&self) -> bool { self.state.closed() }

    pub closed spec fn well_formed(&self) -> bool {
        self.state.well_formed()
    }

    pub fn new(value: T) -> (result: Self)
        ensures
            result.well_formed(),
            result.value() == value,
            result.generation() == 0,
            !result.closed(),
        no_unwind
    {
        WatchUpdate { value, state: EncodedWatchState::new() }
    }

    /// Phase 1: mutate under the production write lock and, on a true closure
    /// result, advance the encoded version before unlocking. Notification is
    /// deliberately absent from this method.
    pub fn apply_update_under_lock(&mut self, value: T, kind: WatchUpdateKind)
        -> (result: (T, Option<WatchNotification>))
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            result.0 == old(self).value(),
            final(self).value() == value,
            kind == WatchUpdateKind::Modified
                && old(self).generation() < usize::MAX / 2 ==>
                final(self).generation() == old(self).generation() + 1,
            kind == WatchUpdateKind::Modified
                && old(self).generation() == usize::MAX / 2 ==>
                final(self).generation() == 0,
            kind != WatchUpdateKind::Modified ==>
                final(self).generation() == old(self).generation(),
            kind == WatchUpdateKind::Modified ==> result.1.is_some(),
            kind == WatchUpdateKind::Modified ==>
                result.1.unwrap().generation() == final(self).generation(),
            kind != WatchUpdateKind::Modified ==> result.1.is_none(),
            final(self).closed() == old(self).closed(),
        no_unwind
    {
        let mut replacement = WatchUpdate {
            value,
            state: EncodedWatchState {
                generation: self.state.generation,
                closed: self.state.closed,
            },
        };
        core::mem::swap(&mut self.value, &mut replacement.value);
        let notification = match kind {
            WatchUpdateKind::Modified => {
                self.state.advance();
                Some(WatchNotification { generation: self.state.generation })
            },
            WatchUpdateKind::Unmodified | WatchUpdateKind::Panicked => None,
        };
        (replacement.value, notification)
    }

    pub fn close(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).closed(),
            final(self).value() == old(self).value(),
            final(self).generation() == old(self).generation(),
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

pub fn verify_watch_modified_creates_post_unlock_phase(first: u64, second: u64)
{
    let mut channel = WatchUpdate::new(first);
    let (previous, notification) = channel.apply_update_under_lock(
        second, WatchUpdateKind::Modified);
    assert(previous == first);
    assert(notification.is_some());
    let published = notification.unwrap().complete();
    assert(published == channel.generation());
    let observed = channel.snapshot();
    assert(observed == second);
}

pub fn verify_watch_unmodified_and_panic_do_not_notify(first: u64, changed: u64)
{
    let mut channel = WatchUpdate::new(first);
    let (previous, unmodified_notification) = channel.apply_update_under_lock(
        changed, WatchUpdateKind::Unmodified);
    assert(previous == first);
    assert(channel.generation() == 0);
    assert(unmodified_notification.is_none());
    let (after_silent_change, panic_notification) = channel.apply_update_under_lock(
        first, WatchUpdateKind::Panicked);
    assert(after_silent_change == changed);
    assert(channel.generation() == 0);
    assert(panic_notification.is_none());
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
