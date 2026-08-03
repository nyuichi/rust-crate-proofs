use crate::watch_completion::WatchUpdateKind;
use vstd::prelude::*;

verus! {

pub open spec fn notify_limit() -> nat { usize::MAX as nat / 4 }
pub open spec fn update_limit() -> nat { (notify_limit() - 1) as nat }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UpdateResult { Published, Unmodified, UserPanic, TerminalPanic }

pub struct WatchUpdateBudget<T> {
    value: T,
    generation: usize,
    fanout_calls: usize,
    lock_held: bool,
    closure_called: bool,
}

impl<T> WatchUpdateBudget<T> {
    pub closed spec fn value(&self) -> T { self.value }
    pub closed spec fn generation(&self) -> usize { self.generation }
    pub closed spec fn fanout_calls(&self) -> usize { self.fanout_calls }
    pub closed spec fn lock_held(&self) -> bool { self.lock_held }
    pub closed spec fn closure_called(&self) -> bool { self.closure_called }
    pub closed spec fn well_formed(&self) -> bool {
        &&& (self.generation() as nat) <= update_limit()
        &&& self.fanout_calls() == self.generation()
        &&& !self.lock_held()
    }

    pub fn apply(&mut self, post_value: T, kind: WatchUpdateKind)
        -> (result: UpdateResult)
        requires old(self).well_formed(),
        ensures final(self).well_formed(), !final(self).lock_held(),
            (old(self).generation() as nat) == update_limit() ==>
                result == UpdateResult::TerminalPanic
                && !final(self).closure_called()
                && final(self).value() == old(self).value()
                && final(self).generation() == old(self).generation()
                && final(self).fanout_calls() == old(self).fanout_calls(),
            (old(self).generation() as nat) < update_limit() ==>
                final(self).closure_called() && final(self).value() == post_value,
            (old(self).generation() as nat) < update_limit()
                && kind == WatchUpdateKind::Modified ==>
                result == UpdateResult::Published
                && final(self).generation() == old(self).generation() + 1
                && final(self).fanout_calls() == old(self).fanout_calls() + 1,
            (old(self).generation() as nat) < update_limit()
                && kind != WatchUpdateKind::Modified ==>
                final(self).generation() == old(self).generation()
                && final(self).fanout_calls() == old(self).fanout_calls(),
        no_unwind
    {
        self.lock_held = true;
        self.closure_called = false;
        if self.generation == usize::MAX / 4 - 1 {
            self.lock_held = false;
            UpdateResult::TerminalPanic
        } else {
            self.closure_called = true;
            self.value = post_value;
            match kind {
                WatchUpdateKind::Modified => {
                    self.generation += 1;
                    self.lock_held = false;
                    self.fanout_calls += 1;
                    UpdateResult::Published
                },
                WatchUpdateKind::Unmodified => {
                    self.lock_held = false;
                    UpdateResult::Unmodified
                },
                WatchUpdateKind::Panicked => {
                    self.lock_held = false;
                    UpdateResult::UserPanic
                },
            }
        }
    }

    pub fn final_sender_drop(&mut self)
        requires old(self).well_formed(),
        ensures final(self).generation() == old(self).generation(),
            final(self).fanout_calls() == old(self).fanout_calls() + 1,
            (final(self).fanout_calls() as nat) <= notify_limit(),
            final(self).value() == old(self).value(), !final(self).lock_held(),
        no_unwind
    {
        self.fanout_calls += 1;
    }
}

pub fn verify_terminal_false_closure_is_not_called(value: u64)
{
    let mut update = WatchUpdateBudget {
        value,
        generation: usize::MAX / 4 - 1,
        fanout_calls: usize::MAX / 4 - 1,
        lock_held: false,
        closure_called: true,
    };
    assert(update.well_formed());
    let result = update.apply(value, WatchUpdateKind::Unmodified);
    assert(result == UpdateResult::TerminalPanic);
    assert(!update.closure_called());
    assert(!update.lock_held());
    assert(update.value() == value);
}

pub fn verify_last_publish_leaves_sender_drop_slot(value: u64)
{
    let mut update = WatchUpdateBudget {
        value,
        generation: usize::MAX / 4 - 2,
        fanout_calls: usize::MAX / 4 - 2,
        lock_held: false,
        closure_called: false,
    };
    assert(update.well_formed());
    let result = update.apply(value, WatchUpdateKind::Modified);
    assert(result == UpdateResult::Published);
    assert((update.fanout_calls() as nat) == update_limit());
    update.final_sender_drop();
    assert((update.fanout_calls() as nat) == notify_limit());
}

} // verus!
