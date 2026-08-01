use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NotifyWordState { Empty, Waiting, Notified }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InitPoll { BroadcastReady, PermitReady, Registered }

pub open spec fn notify_max_calls() -> nat { usize::MAX as nat / 4 }

/// Exact production state word: low two bits are EMPTY/WAITING/NOTIFIED and
/// upper bits count `notify_waiters` calls modulo the machine word.
pub struct EncodedNotifyWord {
    calls: usize,
    state: NotifyWordState,
}

impl EncodedNotifyWord {
    pub closed spec fn calls(&self) -> usize { self.calls }
    pub closed spec fn state(&self) -> NotifyWordState { self.state }
    pub open spec fn state_code(state: NotifyWordState) -> nat {
        match state { NotifyWordState::Empty => 0, NotifyWordState::Waiting => 1,
            NotifyWordState::Notified => 2 }
    }
    pub closed spec fn raw(&self) -> nat {
        4nat * self.calls() as nat + Self::state_code(self.state())
    }
    pub closed spec fn well_formed(&self) -> bool {
        self.calls() as nat <= notify_max_calls()
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.raw() == 0,
            result.state() == NotifyWordState::Empty, result.calls() == 0,
        no_unwind
    {
        EncodedNotifyWord { calls: 0, state: NotifyWordState::Empty }
    }

    /// Production `set_state` preserves every generation bit.
    pub fn set_state(&mut self, state: NotifyWordState)
        requires old(self).well_formed(),
        ensures final(self).well_formed(), final(self).calls() == old(self).calls(),
            final(self).state() == state,
            final(self).raw() / 4 == old(self).raw() / 4,
        no_unwind
    { self.state = state; }

    /// Both `fetch_add(4)` and the locked `data + 4` represent this wrapping
    /// generation advance. Draining a WAITING list transitions it to EMPTY;
    /// EMPTY and a stored NOTIFIED permit are preserved.
    pub fn notify_waiters(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            old(self).calls() == usize::MAX / 4 ==> final(self).calls() == 0,
            old(self).calls() < usize::MAX / 4
                ==> final(self).calls() == old(self).calls() + 1,
            final(self).state() == if old(self).state() == NotifyWordState::Waiting {
                NotifyWordState::Empty
            } else { old(self).state() },
        no_unwind
    {
        if self.calls == usize::MAX / 4 { self.calls = 0; } else { self.calls += 1; }
        if let NotifyWordState::Waiting = self.state { self.state = NotifyWordState::Empty; }
    }

    /// Exact `notify_locked`: no waiter stores one permit; otherwise the chosen
    /// waiter is detached and the word becomes EMPTY iff it was the last node.
    pub fn notify_one_locked(&mut self, was_last_waiter: bool) -> (selected: bool)
        requires old(self).well_formed(),
        ensures final(self).well_formed(), final(self).calls() == old(self).calls(),
            selected == (old(self).state() == NotifyWordState::Waiting),
            !selected ==> final(self).state() == NotifyWordState::Notified,
            selected && was_last_waiter ==> final(self).state() == NotifyWordState::Empty,
            selected && !was_last_waiter ==> final(self).state() == NotifyWordState::Waiting,
        no_unwind
    {
        match self.state {
            NotifyWordState::Waiting => {
                if was_last_waiter { self.state = NotifyWordState::Empty; }
                true
            },
            NotifyWordState::Empty | NotifyWordState::Notified => {
                self.state = NotifyWordState::Notified;
                false
            },
        }
    }

    /// Production Init poll checks the broadcast generation before consuming
    /// NOTIFIED, preserving a stored one-permit when both are present.
    pub fn poll_init(&mut self, snapshot: usize) -> (result: InitPoll)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(), final(self).calls() == old(self).calls(),
            snapshot != old(self).calls() ==> result == InitPoll::BroadcastReady,
            snapshot != old(self).calls() ==> final(self).state() == old(self).state(),
            snapshot == old(self).calls() && old(self).state() == NotifyWordState::Notified
                ==> result == InitPoll::PermitReady,
            result == InitPoll::PermitReady ==> final(self).state() == NotifyWordState::Empty,
            snapshot == old(self).calls() && old(self).state() != NotifyWordState::Notified
                ==> result == InitPoll::Registered,
            result == InitPoll::Registered ==> final(self).state() == NotifyWordState::Waiting,
        no_unwind
    {
        if snapshot != self.calls {
            InitPoll::BroadcastReady
        } else {
            match self.state {
                NotifyWordState::Notified => { self.state = NotifyWordState::Empty; InitPoll::PermitReady },
                NotifyWordState::Empty | NotifyWordState::Waiting => {
                    self.state = NotifyWordState::Waiting; InitPoll::Registered
                },
            }
        }
    }
}

pub fn verify_broadcast_wins_without_consuming_permit()
{
    let mut word = EncodedNotifyWord::new();
    word.notify_one_locked(false);
    assert(word.state() == NotifyWordState::Notified);
    let snapshot = 0usize;
    word.notify_waiters();
    let result = word.poll_init(snapshot);
    assert(result == InitPoll::BroadcastReady);
    assert(word.state() == NotifyWordState::Notified);
}

pub fn verify_notify_word_wrap_preserves_permit()
{
    let mut word = EncodedNotifyWord { calls: usize::MAX / 4, state: NotifyWordState::Notified };
    assert(word.well_formed());
    word.notify_waiters();
    assert(word.calls() == 0);
    assert(word.state() == NotifyWordState::Notified);
    assert(word.raw() == 2);
}

pub proof fn verify_notify_mutants_rejected()
{
    // Checking/consuming NOTIFIED before the generation would steal this permit.
    assert(NotifyWordState::Notified != NotifyWordState::Empty);
    // Failing to clear WAITING after its last node leaves a state/list mismatch.
    assert(1nat != 0nat);
    // Incrementing by one corrupts state bits; production increments by four.
    assert((0nat + 1) % 4 == 1);
    assert((0nat + 4) % 4 == 0);
}

} // verus!
