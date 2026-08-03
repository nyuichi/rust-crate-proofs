#[cfg(verus_keep_ghost)]
use crate::broadcast_surface::public_of;
use crate::broadcast_surface::{BroadcastCorePoll, BroadcastPublicPoll};
use crate::coop_refinement::BudgetValue;
use crate::vstd_ext::thread_local::{LocalState, ThreadLocalCell, TotalCallback, TryAccess};
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

/// Tokio budget mapping over the shared repository-local TLS/Cell primitive.
pub type BudgetTlsState = LocalState<BudgetValue>;
pub type TlsAccess = TryAccess;

pub struct ThreadLocalBudgetCell {
    tls: ThreadLocalCell<BudgetValue>,
    task_polls_allowed: bool,
    last_access: Option<TlsAccess>,
}

impl ThreadLocalBudgetCell {
    pub closed spec fn state(&self) -> BudgetTlsState { self.tls.state() }
    pub closed spec fn task_polls_allowed(&self) -> bool { self.task_polls_allowed }
    pub closed spec fn last_access(&self) -> Option<TlsAccess> { self.last_access }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.tls.thread() == 0
        // Budget is a Cell field inside the same production CONTEXT TLS key
        // used by scheduler and blocking-region access.
        &&& self.tls.key() == 2
        &&& !matches!(self.state(), BudgetTlsState::Borrowed)
    }

    pub fn new_for_thread() -> (result: Self)
        ensures result.well_formed(),
            result.state() == BudgetTlsState::Accessible(BudgetValue::Unconstrained),
            result.task_polls_allowed(), result.last_access().is_none(),
        no_unwind
    {
        ThreadLocalBudgetCell {
            tls: ThreadLocalCell::new(0, 2, BudgetValue::Unconstrained),
            task_polls_allowed: true,
            last_access: None,
        }
    }

    /// Projection of the budget Cell from an already modeled CONTEXT TLS
    /// instance. `context_accessible` is obtained from the owning context
    /// wrapper, so scheduler, blocking-region, and budget teardown agree.
    pub fn new_linked_to_context(context_accessible: bool) -> (result: Self)
        ensures result.well_formed(), result.task_polls_allowed(),
            context_accessible ==>
                result.state()
                    == BudgetTlsState::Accessible(BudgetValue::Unconstrained),
            !context_accessible ==> result.state() == BudgetTlsState::TornDown,
            result.last_access().is_none(),
        no_unwind
    {
        let mut result = ThreadLocalBudgetCell::new_for_thread();
        if !context_accessible {
            result.teardown();
        }
        result
    }

    pub fn teardown(&mut self)
        requires old(self).well_formed(), old(self).task_polls_allowed(),
            matches!(old(self).state(), BudgetTlsState::Accessible(_)),
        ensures final(self).well_formed(),
            final(self).state() == BudgetTlsState::TornDown,
            final(self).task_polls_allowed() == old(self).task_polls_allowed(),
            final(self).last_access().is_none(),
        no_unwind
    {
        self.tls.teardown(0);
        self.last_access = None;
    }

}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TlsRestoreTicket {
    pub remembered: BudgetValue,
}

impl TlsRestoreTicket {
    pub closed spec fn remembered(&self) -> BudgetValue { self.remembered }

    pub fn made_progress(&mut self)
        ensures final(self).remembered() == BudgetValue::Unconstrained,
        no_unwind
    {
        self.remembered = BudgetValue::Unconstrained;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TlsProceed {
    Ready(TlsRestoreTicket),
    NeedsRegistration,
}

pub open spec fn proceed_is_ready(result: &TlsProceed) -> bool {
    matches!(result, TlsProceed::Ready(_))
}

pub open spec fn proceed_needs_registration(result: &TlsProceed) -> bool {
    matches!(result, TlsProceed::NeedsRegistration)
}

pub open spec fn proceed_remembered(result: &TlsProceed) -> Option<BudgetValue> {
    match result {
        TlsProceed::Ready(ticket) => Some(ticket.remembered()),
        TlsProceed::NeedsRegistration => None,
    }
}

pub open spec fn decremented_budget(value: BudgetValue) -> BudgetValue {
    match value {
        BudgetValue::Constrained(0) => BudgetValue::Constrained(0),
        BudgetValue::Constrained(n) => BudgetValue::Constrained((n - 1) as u8),
        BudgetValue::Unconstrained => BudgetValue::Unconstrained,
    }
}

/// Exact compiled closure and inaccessible-TLS fallback of poll_proceed.
pub fn tls_poll_proceed(cell: &mut ThreadLocalBudgetCell) -> (result: TlsProceed)
    requires old(cell).well_formed(),
    ensures final(cell).well_formed(),
        final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
        old(cell).state() == BudgetTlsState::TornDown ==>
            final(cell).state() == BudgetTlsState::TornDown
                && final(cell).last_access() == Some(TlsAccess::ClosureZeroTeardown)
                && proceed_remembered(&result) == Some(BudgetValue::Unconstrained),
        old(cell).state() == BudgetTlsState::TornDown ==> proceed_is_ready(&result),
        old(cell).state() == BudgetTlsState::Accessible(BudgetValue::Constrained(0)) ==>
            final(cell).last_access() == Some(TlsAccess::ClosureOnce),
        result == TlsProceed::NeedsRegistration ==>
            old(cell).state() == BudgetTlsState::Accessible(BudgetValue::Constrained(0)),
        old(cell).task_polls_allowed() ==> match old(cell).state() {
            BudgetTlsState::Accessible(BudgetValue::Constrained(0)) => {
                &&& final(cell).state()
                    == BudgetTlsState::Accessible(BudgetValue::Constrained(0))
                &&& final(cell).last_access() == Some(TlsAccess::ClosureOnce)
                &&& result == TlsProceed::NeedsRegistration
            }
            BudgetTlsState::Accessible(value) => {
                &&& value.has_remaining()
                &&& final(cell).state()
                    == BudgetTlsState::Accessible(decremented_budget(value))
                &&& final(cell).last_access() == Some(TlsAccess::ClosureOnce)
                &&& result == TlsProceed::Ready(TlsRestoreTicket { remembered: value })
            },
            BudgetTlsState::TornDown => {
                &&& final(cell).state() == BudgetTlsState::TornDown
                &&& final(cell).last_access()
                    == Some(TlsAccess::ClosureZeroTeardown)
                &&& result == TlsProceed::Ready(TlsRestoreTicket {
                    remembered: BudgetValue::Unconstrained,
                })
            },
            BudgetTlsState::Borrowed => false,
        },
    no_unwind
{
    let mut session = cell.tls.try_with_begin(0);
    assert(session.original() == old(cell).state());
    let initial = session.get();
    match initial {
        None => {
            assert(old(cell).state() == BudgetTlsState::TornDown);
            assert(!(old(cell).state() != BudgetTlsState::TornDown));
            cell.last_access = Some(TlsAccess::ClosureZeroTeardown);
            cell.tls.try_with_end(session);
            TlsProceed::Ready(TlsRestoreTicket { remembered: BudgetValue::Unconstrained })
        },
        Some(mut budget) => {
            assert(session.access() == TlsAccess::ClosureOnce);
            assert(old(cell).state() == BudgetTlsState::Accessible(budget));
            cell.last_access = Some(TlsAccess::ClosureOnce);
            let previous = budget;
            match previous {
                BudgetValue::Unconstrained => {
                    assert(budget == BudgetValue::Unconstrained);
                    session.set(budget);
                    cell.tls.try_with_end(session);
                    TlsProceed::Ready(TlsRestoreTicket { remembered: previous })
                },
                BudgetValue::Constrained(value) => {
                    assert(budget == BudgetValue::Constrained(value));
                    if value == 0 {
                        // Production register_waker is connected separately in
                        // defer_refinement; this result carries that obligation.
                        cell.tls.try_with_end(session);
                        TlsProceed::NeedsRegistration
                    } else {
                        budget = BudgetValue::Constrained((value - 1) as u8);
                        session.set(budget);
                        cell.tls.try_with_end(session);
                        TlsProceed::Ready(TlsRestoreTicket { remembered: previous })
                    }
                },
            }
        },
    }
}

/// Drop effect for RestoreOnPending. Arbitrary unwind invokes the same body.
pub fn tls_drop_restore(cell: &mut ThreadLocalBudgetCell, ticket: TlsRestoreTicket)
    requires old(cell).well_formed(),
    ensures final(cell).well_formed(),
        final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
        match ticket.remembered() {
            BudgetValue::Unconstrained => {
                final(cell).state() == old(cell).state()
                    && final(cell).last_access().is_none()
            },
            remembered => match old(cell).state() {
                BudgetTlsState::Accessible(_) => {
                    final(cell).state() == BudgetTlsState::Accessible(remembered)
                        && final(cell).last_access() == Some(TlsAccess::ClosureOnce)
                },
                BudgetTlsState::TornDown => {
                    final(cell).state() == BudgetTlsState::TornDown
                        && final(cell).last_access()
                            == Some(TlsAccess::ClosureZeroTeardown)
                },
                BudgetTlsState::Borrowed => false,
            },
        },
    no_unwind
{
    cell.last_access = None;
    match ticket.remembered {
        BudgetValue::Unconstrained => {},
        remembered => {
            let mut session = cell.tls.try_with_begin(0);
            assert(session.original() == old(cell).state());
            let current = session.get();
            match current {
                Some(_) => {
                    assert(session.access() == TlsAccess::ClosureOnce);
                    session.set(remembered);
                    cell.last_access = Some(TlsAccess::ClosureOnce);
                },
                None => {
                    cell.last_access = Some(TlsAccess::ClosureZeroTeardown);
                },
            }
            cell.tls.try_with_end(session);
        },
    }
}

pub struct TlsResetGuard {
    previous: BudgetValue,
}

pub enum TlsBudgetInstall {
    Guard(TlsResetGuard),
    NoGuardTeardown,
}

pub struct WithBudgetEntryResult {
    install: TlsBudgetInstall,
    tls_closure_runs: u8,
    body_runs: u8,
}

impl WithBudgetEntryResult {
    pub closed spec fn has_guard(&self) -> bool {
        matches!(self.install, TlsBudgetInstall::Guard(_))
    }
    pub closed spec fn previous(&self) -> Option<BudgetValue> {
        match self.install {
            TlsBudgetInstall::Guard(ref guard) => Some(guard.previous()),
            TlsBudgetInstall::NoGuardTeardown => None,
        }
    }
    pub closed spec fn tls_closure_runs(&self) -> u8 { self.tls_closure_runs }
    pub closed spec fn body_runs(&self) -> u8 { self.body_runs }

    pub fn is_teardown(&self) -> (result: bool)
        ensures result == !self.has_guard(),
        no_unwind
    {
        matches!(self.install, TlsBudgetInstall::NoGuardTeardown)
    }
}

/// Exact entry behavior of `with_budget`: an accessible TLS cell installs the
/// budget and returns a ResetGuard, while failed TLS access runs no TLS closure
/// and creates no guard. In both cases the caller-owned body runs exactly once
/// after this entry step; that body fact is represented by the witness below.
pub fn tls_with_budget_enter(
    cell: &mut ThreadLocalBudgetCell,
    next: BudgetValue,
) -> (result: WithBudgetEntryResult)
    requires old(cell).well_formed(),
    ensures final(cell).well_formed(),
        final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
        result.body_runs() == 1,
        match old(cell).state() {
            BudgetTlsState::Accessible(previous) =>
                final(cell).state() == BudgetTlsState::Accessible(next)
                    && final(cell).last_access() == Some(TlsAccess::ClosureOnce)
                    && result.has_guard()
                    && result.previous() == Some(previous)
                    && result.tls_closure_runs() == 1,
            BudgetTlsState::TornDown =>
                final(cell).state() == BudgetTlsState::TornDown
                    && final(cell).last_access() == Some(TlsAccess::ClosureZeroTeardown)
                    && !result.has_guard()
                    && result.previous().is_none()
                    && result.tls_closure_runs() == 0,
            BudgetTlsState::Borrowed => false,
        },
    no_unwind
{
    let mut session = cell.tls.try_with_begin(0);
    assert(session.original() == old(cell).state());
    let current = session.get();
    let access = if current.is_some() {
        TlsAccess::ClosureOnce
    } else {
        TlsAccess::ClosureZeroTeardown
    };
    let total = TotalCallback::from_access(access);
    let runs = total.runs();
    let install = match current {
        Some(previous) => {
            assert(session.access() == TlsAccess::ClosureOnce);
            session.set(next);
            cell.last_access = Some(TlsAccess::ClosureOnce);
            cell.tls.try_with_end(session);
            TlsBudgetInstall::Guard(TlsResetGuard { previous })
        },
        None => {
            cell.last_access = Some(TlsAccess::ClosureZeroTeardown);
            cell.tls.try_with_end(session);
            TlsBudgetInstall::NoGuardTeardown
        },
    };
    WithBudgetEntryResult {
        install,
        tls_closure_runs: runs.0,
        body_runs: runs.1,
    }
}

impl TlsResetGuard {
    pub closed spec fn previous(&self) -> BudgetValue { self.previous }
}

pub fn tls_install_budget(
    cell: &mut ThreadLocalBudgetCell,
    next: BudgetValue,
) -> (guard: TlsResetGuard)
    requires old(cell).well_formed(), old(cell).task_polls_allowed(),
        matches!(old(cell).state(), BudgetTlsState::Accessible(_)),
    ensures final(cell).well_formed(),
        final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
        final(cell).state() == BudgetTlsState::Accessible(next),
        final(cell).last_access() == Some(TlsAccess::ClosureOnce),
        guard.previous() == match old(cell).state() {
            BudgetTlsState::Accessible(value) => value,
            BudgetTlsState::TornDown => BudgetValue::Unconstrained,
            BudgetTlsState::Borrowed => BudgetValue::Unconstrained,
        },
    no_unwind
{
    let mut session = cell.tls.try_with_begin(0);
    assert(session.original() == old(cell).state());
    let current = session.get();
    let previous = match current {
        Some(value) => value,
        None => unreached(),
    };
    assert(session.access() == TlsAccess::ClosureOnce);
    session.set(next);
    cell.last_access = Some(TlsAccess::ClosureOnce);
    cell.tls.try_with_end(session);
    TlsResetGuard { previous }
}

/// ResetGuard Drop on normal return and unwind. Nested guards must be supplied
/// in LIFO order; the witness below proves exact restoration.
pub fn tls_drop_reset(cell: &mut ThreadLocalBudgetCell, guard: TlsResetGuard)
    requires old(cell).well_formed(),
    ensures final(cell).well_formed(),
        final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
        matches!(old(cell).state(), BudgetTlsState::Accessible(_)) ==>
            final(cell).state() == BudgetTlsState::Accessible(guard.previous()),
        matches!(old(cell).state(), BudgetTlsState::Accessible(_)) ==>
            final(cell).last_access() == Some(TlsAccess::ClosureOnce),
        old(cell).state() == BudgetTlsState::TornDown ==>
            final(cell).state() == BudgetTlsState::TornDown,
    no_unwind
{
    let mut session = cell.tls.try_with_begin(0);
    assert(session.original() == old(cell).state());
    let current = session.get();
    match current {
        Some(_) => {
            assert(session.access() == TlsAccess::ClosureOnce);
            session.set(guard.previous);
            cell.last_access = Some(TlsAccess::ClosureOnce);
        },
        None => {
            cell.last_access = Some(TlsAccess::ClosureZeroTeardown);
        },
    }
    cell.tls.try_with_end(session);
}

/// Broadcast's standard `sync` path has cfg_not_taskdump trace_leaf, whose
/// compiled body is unconditional Ready and has no state effect. The unstable
/// taskdump callback path is intentionally not included in this contract.
pub fn broadcast_trace_leaf_standard() -> (ready: bool)
    ensures ready,
    no_unwind
{
    true
}

pub struct BroadcastTlsPoll {
    inner_polled: bool,
}

impl BroadcastTlsPoll {
    pub closed spec fn inner_polled(&self) -> bool { self.inner_polled }

    pub fn new() -> (result: Self)
        ensures !result.inner_polled(),
        no_unwind
    {
        BroadcastTlsPoll { inner_polled: false }
    }

    pub fn reset_not_polled(&mut self)
        ensures !final(self).inner_polled(),
        no_unwind
    {
        self.inner_polled = false;
    }

    pub fn poll_ready(
        &mut self,
        cell: &mut ThreadLocalBudgetCell,
        mut restore: TlsRestoreTicket,
        remembered: BudgetValue,
        core: BroadcastCorePoll,
    ) -> (result: BroadcastPublicPoll)
        requires old(cell).well_formed(), old(cell).task_polls_allowed(),
            restore.remembered == remembered,
            old(cell).state() == BudgetTlsState::Accessible(decremented_budget(remembered))
                || (old(cell).state() == BudgetTlsState::TornDown
                    && remembered == BudgetValue::Unconstrained),
        ensures final(cell).well_formed(),
            final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
            final(self).inner_polled(), result == public_of(core),
            old(cell).state() == BudgetTlsState::TornDown ==>
                final(cell).state() == BudgetTlsState::TornDown,
            old(cell).state() != BudgetTlsState::TornDown
                && matches!(core, BroadcastCorePoll::Pending | BroadcastCorePoll::Empty) ==>
                    final(cell).state() == BudgetTlsState::Accessible(remembered),
            old(cell).state() != BudgetTlsState::TornDown
                && !matches!(core, BroadcastCorePoll::Pending | BroadcastCorePoll::Empty) ==>
                    final(cell).state()
                        == BudgetTlsState::Accessible(decremented_budget(remembered)),
        no_unwind
    {
        let trace_ready = broadcast_trace_leaf_standard();
        assert(trace_ready);
        self.inner_polled = true;
        let result = match core {
            BroadcastCorePoll::Pending | BroadcastCorePoll::Empty => BroadcastPublicPoll::Pending,
            BroadcastCorePoll::Value(value) => BroadcastPublicPoll::Value(value),
            BroadcastCorePoll::Closed => BroadcastPublicPoll::Closed,
            BroadcastCorePoll::Lagged(amount) => BroadcastPublicPoll::Lagged(amount),
        };
        if !matches!(core, BroadcastCorePoll::Pending | BroadcastCorePoll::Empty) {
            restore.made_progress();
        }
        tls_drop_restore(cell, restore);
        result
    }

    pub fn poll(
        &mut self,
        cell: &mut ThreadLocalBudgetCell,
        core: BroadcastCorePoll,
    ) -> (result: BroadcastPublicPoll)
        requires old(cell).well_formed(), old(cell).task_polls_allowed(),
        ensures final(cell).well_formed(),
            final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
            matches!(old(cell).state(), BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                result == BroadcastPublicPoll::Pending && !final(self).inner_polled()
                    && final(cell).state() == old(cell).state(),
            !matches!(old(cell).state(), BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                final(self).inner_polled() && result == public_of(core),
            match old(cell).state() {
                BudgetTlsState::Accessible(value) => {
                    if value == BudgetValue::Constrained(0) {
                        final(cell).state() == old(cell).state()
                    } else if matches!(core, BroadcastCorePoll::Pending | BroadcastCorePoll::Empty) {
                        final(cell).state() == old(cell).state()
                    } else {
                        final(cell).state() == BudgetTlsState::Accessible(decremented_budget(value))
                    }
                },
                BudgetTlsState::TornDown => {
                    &&& final(cell).state() == BudgetTlsState::TornDown
                    &&& final(self).inner_polled()
                    &&& result == public_of(core)
                },
                BudgetTlsState::Borrowed => false,
            },
        no_unwind
    {
        let proceed = tls_poll_proceed(cell);
        match proceed {
            TlsProceed::NeedsRegistration => {
                self.reset_not_polled();
                BroadcastPublicPoll::Pending
            },
            TlsProceed::Ready(restore) => {
                let remembered = restore.remembered;
                self.poll_ready(cell, restore, remembered, core)
            },
        }
    }
}

pub fn verify_tls_nesting_and_broadcast_poll()
{
    let mut cell = ThreadLocalBudgetCell::new_for_thread();
    let outer = tls_install_budget(&mut cell, BudgetValue::Constrained(2));
    let inner = tls_install_budget(&mut cell, BudgetValue::Unconstrained);
    tls_drop_reset(&mut cell, inner);
    assert(cell.state() == BudgetTlsState::Accessible(BudgetValue::Constrained(2)));

    let mut poll = BroadcastTlsPoll::new();
    let value = poll.poll(&mut cell, BroadcastCorePoll::Value(11));
    assert(value == BroadcastPublicPoll::Value(11));
    assert(poll.inner_polled());
    assert(cell.state() == BudgetTlsState::Accessible(BudgetValue::Constrained(1)));
    tls_drop_reset(&mut cell, outer);
    assert(cell.state() == BudgetTlsState::Accessible(BudgetValue::Unconstrained));
}

pub fn verify_tls_teardown_runs_no_closure()
{
    let mut cell = ThreadLocalBudgetCell::new_for_thread();
    cell.teardown();
    let result = tls_poll_proceed(&mut cell);
    assert(matches!(result, TlsProceed::Ready(_)));
    assert(cell.last_access() == Some(TlsAccess::ClosureZeroTeardown));
    assert(cell.task_polls_allowed());
}

pub fn verify_with_budget_teardown_runs_body_without_guard()
{
    let mut cell = ThreadLocalBudgetCell::new_for_thread();
    cell.teardown();
    let install = tls_with_budget_enter(&mut cell, BudgetValue::Constrained(8));
    let is_teardown = install.is_teardown();
    assert(is_teardown);
    assert(install.body_runs() == 1);
    assert(install.tls_closure_runs() == 0);
    assert(cell.state() == BudgetTlsState::TornDown);
    assert(cell.last_access() == Some(TlsAccess::ClosureZeroTeardown));
}

} // verus!
