#[cfg(verus_keep_ghost)]
use crate::broadcast_surface::public_of;
use crate::broadcast_surface::{BroadcastCorePoll, BroadcastPublicPoll};
use crate::coop_refinement::BudgetValue;
use crate::coop_tls_refinement::{BudgetTlsState, ThreadLocalBudgetCell};
use crate::defer_refinement::{
    BlockingRegionResult, DeferOutcome, DeferQueue, RegisterWakerState, RegisteredBroadcastPoll,
    RuntimeEntry, SchedulerContextAccess,
};
use crate::vstd_ext::thread_local::{LocalState, ThreadLocalCell, TryAccess};
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

/// Tokio body mapping for the distinct `runtime::park::CURRENT_PARKER` key.
/// The owned wrapper supplies one non-Clone current-thread model instance;
/// key tag 3 distinguishes it from the shared CONTEXT/budget tag 2 in this
/// composed proof. Global TLS registry freshness is not claimed.
pub struct CurrentParkerCell {
    tls: ThreadLocalCell<u8>,
    last_access: Option<TryAccess>,
    access_attempts: Ghost<int>,
    callback_runs: Ghost<int>,
}

impl CurrentParkerCell {
    pub closed spec fn accessible(&self) -> bool {
        matches!(self.tls.state(), LocalState::Accessible(_))
    }
    pub closed spec fn last_access(&self) -> Option<TryAccess> { self.last_access }
    pub closed spec fn access_attempts(&self) -> int { self.access_attempts@ }
    pub closed spec fn callback_runs(&self) -> int { self.callback_runs@ }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.tls.thread() == 0
        &&& self.tls.key() == 3
        &&& !matches!(self.tls.state(), LocalState::Borrowed)
    }

    pub fn new_for_thread() -> (result: Self)
        ensures result.well_formed(), result.accessible(),
            result.last_access().is_none(), result.access_attempts() == 0,
            result.callback_runs() == 0,
        no_unwind
    {
        CurrentParkerCell {
            tls: ThreadLocalCell::new(0, 3, 0),
            last_access: None,
            access_attempts: Ghost(0),
            callback_runs: Ghost(0),
        }
    }

    pub fn teardown(&mut self)
        requires old(self).well_formed(), old(self).accessible(),
        ensures final(self).well_formed(), !final(self).accessible(),
            final(self).last_access().is_none(),
            final(self).access_attempts() == old(self).access_attempts(),
            final(self).callback_runs() == old(self).callback_runs(),
        no_unwind
    {
        self.tls.teardown(0);
        self.last_access = None;
    }

    /// Body mapping of `CURRENT_PARKER.try_with(|inner| f(inner))`. The
    /// callback executes exactly once when accessible and zero times during
    /// TLS teardown; the wrapper state is otherwise unchanged.
    pub fn with_current(&mut self) -> (result: CurrentParkerResult)
        requires old(self).well_formed(),
        ensures final(self).well_formed(),
            final(self).accessible() == old(self).accessible(),
            final(self).access_attempts() == old(self).access_attempts() + 1,
            old(self).accessible() ==>
                result == CurrentParkerResult::Callback
                    && final(self).last_access() == Some(TryAccess::ClosureOnce)
                    && final(self).callback_runs() == old(self).callback_runs() + 1,
            !old(self).accessible() ==>
                result == CurrentParkerResult::AccessError
                    && final(self).last_access()
                        == Some(TryAccess::ClosureZeroTeardown)
                    && final(self).callback_runs() == old(self).callback_runs(),
        no_unwind
    {
        let session = self.tls.try_with_begin(0);
        let value = session.get();
        self.access_attempts = Ghost(self.access_attempts@ + 1);
        let result = if value.is_some() {
            self.last_access = Some(TryAccess::ClosureOnce);
            self.callback_runs = Ghost(self.callback_runs@ + 1);
            CurrentParkerResult::Callback
        } else {
            self.last_access = Some(TryAccess::ClosureZeroTeardown);
            CurrentParkerResult::AccessError
        };
        self.tls.try_with_end(session);
        result
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CurrentParkerResult {
    Callback,
    AccessError,
}

/// The three observable exits of production `future::block_on(recv())`.
/// Parker waker/park mechanics and eventual scheduling remain in the frozen
/// foundation; all Tokio TLS routing and broadcast-result mapping are proved.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockingRecvExecution {
    Returned(BroadcastPublicPoll),
    RuntimePanic,
    ParkerTlsPanic,
}

pub struct BlockingRecvReport {
    result: BlockingRecvExecution,
    poll_calls: Ghost<int>,
    park_calls: Ghost<int>,
    budget_tls_closure_runs: Ghost<int>,
}

impl BlockingRecvReport {
    pub closed spec fn result(&self) -> BlockingRecvExecution { self.result }
    pub closed spec fn poll_calls(&self) -> int { self.poll_calls@ }
    pub closed spec fn park_calls(&self) -> int { self.park_calls@ }
    pub closed spec fn budget_tls_closure_runs(&self) -> int {
        self.budget_tls_closure_runs@
    }
}

pub open spec fn core_is_pending(core: BroadcastCorePoll) -> bool {
    matches!(core, BroadcastCorePoll::Pending | BroadcastCorePoll::Empty)
}

/// Recv's two internal retry outcomes are observationally identical at the
/// registered/cooperative boundary: both publish Pending, restore the same
/// budget, preserve context frames, and are followed by exactly one park.
pub proof fn pending_core_normalization()
    ensures core_is_pending(BroadcastCorePoll::Pending),
        core_is_pending(BroadcastCorePoll::Empty),
        public_of(BroadcastCorePoll::Pending) == BroadcastPublicPoll::Pending,
        public_of(BroadcastCorePoll::Empty) == BroadcastPublicPoll::Pending,
{
}

pub open spec fn blocking_result_is_terminal(result: BlockingRecvExecution) -> bool {
    match result {
        BlockingRecvExecution::Returned(BroadcastPublicPoll::Pending) => false,
        _ => true,
    }
}

/// Production-shaped composition for any finite number of Pending/park cycles
/// followed by one terminal poll. Each async poll is executed through the
/// already proved RegisteredBroadcastPoll path with an unconstrained blocking
/// budget; channel outcomes are therefore not supplied as an unconnected
/// public-result oracle.
pub fn blocking_recv_composed(
    context: &mut SchedulerContextAccess,
    parker: &mut CurrentParkerCell,
    rt_enabled: bool,
    pending_polls: u64,
    terminal_core: BroadcastCorePoll,
) -> (report: BlockingRecvReport)
    requires old(context).well_formed(), old(parker).well_formed(),
        !core_is_pending(terminal_core),
    ensures final(context).well_formed(), final(parker).well_formed(),
        final(context).tls_accessible() == old(context).tls_accessible(),
        final(context).entry() == old(context).entry(),
        final(context).scoped() == old(context).scoped(),
        final(context).current() == old(context).current(),
        rt_enabled ==> final(context).callback_runs() == 1,
        rt_enabled && old(context).tls_accessible() ==>
            final(context).tls_closure_runs() == 1,
        rt_enabled && !old(context).tls_accessible() ==>
            final(context).tls_closure_runs() == 0,
        !rt_enabled ==>
            final(context).callback_runs() == old(context).callback_runs()
                && final(context).tls_closure_runs() == old(context).tls_closure_runs(),
        rt_enabled && old(context).tls_accessible()
                && old(context).entry() == RuntimeEntry::Entered ==>
            report.result() == BlockingRecvExecution::RuntimePanic
                && report.poll_calls() == 0 && report.park_calls() == 0
                && report.budget_tls_closure_runs() == 0
                && final(parker).access_attempts() == old(parker).access_attempts()
                && final(parker).callback_runs() == old(parker).callback_runs(),
        ((!rt_enabled || !old(context).tls_accessible()
                || old(context).entry() == RuntimeEntry::NotEntered)
                && !old(parker).accessible()) ==>
            report.result() == BlockingRecvExecution::ParkerTlsPanic
                && report.poll_calls() == 0 && report.park_calls() == 0
                && report.budget_tls_closure_runs() == 0
                && final(parker).access_attempts() == old(parker).access_attempts() + 1
                && final(parker).callback_runs() == old(parker).callback_runs(),
        ((!rt_enabled || !old(context).tls_accessible()
                || old(context).entry() == RuntimeEntry::NotEntered)
                && old(parker).accessible()) ==>
            report.result() == BlockingRecvExecution::Returned(public_of(terminal_core))
                && report.poll_calls() == pending_polls as int + 1
                && report.park_calls() == pending_polls as int
                && report.budget_tls_closure_runs() == if old(context).tls_accessible() {
                    pending_polls as int + 1
                } else {
                    0
                }
                && final(parker).access_attempts()
                    == old(parker).access_attempts() + pending_polls as int + 1
                && final(parker).callback_runs()
                    == old(parker).callback_runs() + pending_polls as int + 1,
        blocking_result_is_terminal(report.result()),
    no_unwind
{
    let initial_attempts = parker.access_attempts;
    let initial_callbacks = parker.callback_runs;
    let initial_context_accessible = Ghost(context.tls_accessible());
    let initial_context_entry = Ghost(context.entry());
    let initial_context_scoped = Ghost(context.scoped());
    let initial_context_current = Ghost(context.current());
    let initial_context_tls_runs = Ghost(context.tls_closure_runs());
    let initial_context_callback_runs = Ghost(context.callback_runs());
    if rt_enabled {
        let entered = context.try_enter_blocking_region();
        match entered {
            BlockingRegionResult::RejectedRuntime => {
                return BlockingRecvReport {
                    result: BlockingRecvExecution::RuntimePanic,
                    poll_calls: Ghost(0),
                    park_calls: Ghost(0),
                    budget_tls_closure_runs: Ghost(0),
                };
            },
            BlockingRegionResult::Allowed(_) => {},
        }
    }

    // CachedParkThread::block_on obtains its Waker before the first poll.
    match parker.with_current() {
        CurrentParkerResult::AccessError => {
            return BlockingRecvReport {
                result: BlockingRecvExecution::ParkerTlsPanic,
                poll_calls: Ghost(0),
                park_calls: Ghost(0),
                budget_tls_closure_runs: Ghost(0),
            };
        },
        CurrentParkerResult::Callback => {},
    }

    let context_accessible = context.is_tls_accessible();
    let mut budget = ThreadLocalBudgetCell::new_linked_to_context(context_accessible);
    let mut poll = RegisteredBroadcastPoll::new();
    let mut registration = RegisterWakerState::new();
    let mut queue = DeferQueue::new();
    let mut remaining = pending_polls;
    let mut poll_calls = Ghost(0int);
    let mut park_calls = Ghost(0int);
    let mut budget_tls_closure_runs = Ghost(0int);
    while remaining > 0
        invariant context.well_formed(), parker.well_formed(), parker.accessible(),
            context.tls_accessible() == initial_context_accessible@,
            context.entry() == initial_context_entry@,
            context.scoped() == initial_context_scoped@,
            context.current() == initial_context_current@,
            rt_enabled ==> context.callback_runs() == 1,
            rt_enabled && initial_context_accessible@ ==>
                context.tls_closure_runs() == 1,
            rt_enabled && !initial_context_accessible@ ==>
                context.tls_closure_runs() == 0,
            !rt_enabled ==>
                context.callback_runs() == initial_context_callback_runs@
                    && context.tls_closure_runs() == initial_context_tls_runs@,
            budget.well_formed(), budget.task_polls_allowed(),
            budget.state()
                    == BudgetTlsState::Accessible(BudgetValue::Unconstrained)
                || budget.state() == BudgetTlsState::TornDown,
            poll_calls@ == (pending_polls - remaining) as int,
            park_calls@ == (pending_polls - remaining) as int,
            budget_tls_closure_runs@ == if context_accessible {
                poll_calls@
            } else {
                0
            },
            parker.access_attempts() == initial_attempts@ + park_calls@ + 1,
            parker.callback_runs() == initial_callbacks@ + park_calls@ + 1,
        decreases remaining,
    {
        proof { pending_core_normalization(); }
        let pending = poll.poll(
            &mut budget,
            context,
            &mut registration,
            &mut queue,
            0,
            rt_enabled,
            false,
            DeferOutcome::Queued,
            BroadcastCorePoll::Empty,
        );
        match pending {
            crate::defer_refinement::BroadcastContextPoll::Public(
                BroadcastPublicPoll::Pending) => {},
            _ => unreached(),
        }
        poll_calls = Ghost(poll_calls@ + 1);
        if context_accessible {
            budget_tls_closure_runs = Ghost(budget_tls_closure_runs@ + 1);
        }

        // Condvar/Unpark behavior and liveness are frozen; the Tokio-specific
        // CURRENT_PARKER access and unwrap failure are represented exactly.
        match parker.with_current() {
            CurrentParkerResult::AccessError => unreached(),
            CurrentParkerResult::Callback => {},
        }
        park_calls = Ghost(park_calls@ + 1);
        remaining = remaining - 1;
    }

    assert(!core_is_pending(terminal_core));
    match terminal_core {
        BroadcastCorePoll::Pending | BroadcastCorePoll::Empty => unreached(),
        BroadcastCorePoll::Value(_)
        | BroadcastCorePoll::Closed
        | BroadcastCorePoll::Lagged(_) => {},
    }
    assert(!matches!(budget.state(),
        BudgetTlsState::Accessible(BudgetValue::Constrained(0))));

    let terminal = poll.poll(
        &mut budget,
        context,
        &mut registration,
        &mut queue,
        0,
        rt_enabled,
        false,
        DeferOutcome::Queued,
        terminal_core,
    );
    poll_calls = Ghost(poll_calls@ + 1);
    if context_accessible {
        budget_tls_closure_runs = Ghost(budget_tls_closure_runs@ + 1);
    }
    let result = match terminal {
        crate::defer_refinement::BroadcastContextPoll::Public(value) => {
            BlockingRecvExecution::Returned(value)
        },
        crate::defer_refinement::BroadcastContextPoll::PanickedDuringRegistration => unreached(),
    };
    BlockingRecvReport { result, poll_calls, park_calls, budget_tls_closure_runs }
}

pub fn verify_blocking_recv_routes()
{
    let mut outside = SchedulerContextAccess::new();
    let mut parker = CurrentParkerCell::new_for_thread();
    let result = blocking_recv_composed(
        &mut outside,
        &mut parker,
        true,
        4,
        BroadcastCorePoll::Value(9),
    );
    assert(result.result()
        == BlockingRecvExecution::Returned(BroadcastPublicPoll::Value(9)));
    assert(result.poll_calls() == 5);
    assert(result.park_calls() == 4);

    let mut inside = SchedulerContextAccess::new();
    inside.enter_runtime();
    let mut other_parker = CurrentParkerCell::new_for_thread();
    let rejected = blocking_recv_composed(
        &mut inside,
        &mut other_parker,
        true,
        0,
        BroadcastCorePoll::Closed,
    );
    assert(rejected.result() == BlockingRecvExecution::RuntimePanic);
}

} // verus!
