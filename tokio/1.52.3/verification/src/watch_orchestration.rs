use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PredicateOutcome {
    Matched,
    Rejected,
    Panicked,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WaitForStep {
    ReturnValue,
    WaitForChange,
    ReturnClosed,
    ResumePanic,
}

/// One exact lock-held iteration of production `Receiver::wait_for_inner`.
/// `closed_from_wait` is the local flag returned by the preceding
/// `changed_impl`, not a fresh read of the state closed bit.  The arbitrary
/// predicate result is an input: execution of user code is not trusted to
/// establish a Tokio state transition.
pub struct WaitForOrchestration<T> {
    value: T,
    receiver_version: usize,
    channel_version: usize,
    closed_from_wait: bool,
    read_lock_held: bool,
}

impl<T> WaitForOrchestration<T> {
    pub closed spec fn value(&self) -> T { self.value }
    pub closed spec fn receiver_version(&self) -> usize { self.receiver_version }
    pub closed spec fn channel_version(&self) -> usize { self.channel_version }
    pub closed spec fn closed_from_wait(&self) -> bool { self.closed_from_wait }
    pub closed spec fn read_lock_held(&self) -> bool { self.read_lock_held }
    pub closed spec fn has_changed(&self) -> bool {
        self.receiver_version() != self.channel_version()
    }
    pub closed spec fn predicate_is_called(&self) -> bool {
        !self.closed_from_wait() || self.has_changed()
    }

    pub fn new(
        value: T,
        receiver_version: usize,
        channel_version: usize,
        closed_from_wait: bool,
    ) -> (result: Self)
        ensures
            result.value() == value,
            result.receiver_version() == receiver_version,
            result.channel_version() == channel_version,
            result.closed_from_wait() == closed_from_wait,
            !result.read_lock_held(),
        no_unwind
    {
        WaitForOrchestration {
            value,
            receiver_version,
            channel_version,
            closed_from_wait,
            read_lock_held: false,
        }
    }

    /// Models read-lock acquisition, version load, marking the current value
    /// seen, predicate dispatch, and every exit.  A successful result retains
    /// the lock as the returned `Ref`; every other result releases it.  The
    /// panic branch therefore cannot poison the production RwLock.
    pub fn inspect(&mut self, predicate: PredicateOutcome) -> (result: WaitForStep)
        requires !old(self).read_lock_held(),
        ensures
            final(self).value() == old(self).value(),
            final(self).channel_version() == old(self).channel_version(),
            final(self).receiver_version() == old(self).channel_version(),
            final(self).closed_from_wait() == old(self).closed_from_wait(),
            old(self).predicate_is_called() && predicate == PredicateOutcome::Matched
                ==> result == WaitForStep::ReturnValue,
            old(self).predicate_is_called() && predicate == PredicateOutcome::Rejected
                && !old(self).closed_from_wait()
                ==> result == WaitForStep::WaitForChange,
            old(self).predicate_is_called() && predicate == PredicateOutcome::Rejected
                && old(self).closed_from_wait()
                ==> result == WaitForStep::ReturnClosed,
            old(self).predicate_is_called() && predicate == PredicateOutcome::Panicked
                ==> result == WaitForStep::ResumePanic,
            !old(self).predicate_is_called() ==> result == WaitForStep::ReturnClosed,
            final(self).read_lock_held() == (result == WaitForStep::ReturnValue),
        no_unwind
    {
        self.read_lock_held = true;
        let has_changed = self.receiver_version != self.channel_version;
        self.receiver_version = self.channel_version;

        if !self.closed_from_wait || has_changed {
            match predicate {
                PredicateOutcome::Matched => WaitForStep::ReturnValue,
                PredicateOutcome::Rejected => {
                    self.read_lock_held = false;
                    if self.closed_from_wait {
                        WaitForStep::ReturnClosed
                    } else {
                        WaitForStep::WaitForChange
                    }
                },
                PredicateOutcome::Panicked => {
                    self.read_lock_held = false;
                    WaitForStep::ResumePanic
                },
            }
        } else {
            self.read_lock_held = false;
            WaitForStep::ReturnClosed
        }
    }

    pub fn release_returned_ref(&mut self)
        requires old(self).read_lock_held(),
        ensures
            !final(self).read_lock_held(),
            final(self).value() == old(self).value(),
            final(self).receiver_version() == old(self).receiver_version(),
            final(self).channel_version() == old(self).channel_version(),
        no_unwind
    {
        self.read_lock_held = false;
    }
}

pub fn verify_wait_for_initial_match(value: u64, version: usize)
{
    let mut wait = WaitForOrchestration::new(value, version, version, false);
    let result = wait.inspect(PredicateOutcome::Matched);
    assert(result == WaitForStep::ReturnValue);
    assert(wait.read_lock_held());
    wait.release_returned_ref();
}

pub fn verify_wait_for_rejection_then_wait(value: u64, version: usize)
{
    let mut wait = WaitForOrchestration::new(value, version, version, false);
    let result = wait.inspect(PredicateOutcome::Rejected);
    assert(result == WaitForStep::WaitForChange);
    assert(!wait.read_lock_held());
}

pub fn verify_wait_for_closed_final_change_is_tested(
    value: u64,
    old_version: usize,
    current_version: usize,
)
    requires old_version != current_version,
{
    let mut wait = WaitForOrchestration::new(
        value, old_version, current_version, true);
    let result = wait.inspect(PredicateOutcome::Matched);
    assert(result == WaitForStep::ReturnValue);
    assert(wait.receiver_version() == current_version);
}

pub fn verify_wait_for_closed_seen_value_skips_predicate(value: u64, version: usize)
{
    let mut wait = WaitForOrchestration::new(value, version, version, true);
    let result = wait.inspect(PredicateOutcome::Matched);
    assert(result == WaitForStep::ReturnClosed);
    assert(!wait.read_lock_held());
}

pub fn verify_wait_for_predicate_panic_preserves_observation(
    value: u64,
    old_version: usize,
    current_version: usize,
)
{
    let mut wait = WaitForOrchestration::new(
        value, old_version, current_version, false);
    let result = wait.inspect(PredicateOutcome::Panicked);
    assert(result == WaitForStep::ResumePanic);
    assert(wait.value() == value);
    assert(wait.receiver_version() == current_version);
    assert(!wait.read_lock_held());
}

} // verus!
