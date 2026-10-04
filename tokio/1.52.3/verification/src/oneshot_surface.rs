use crate::blocking_recv_refinement::{
    blocking_recv_composed, BlockingRecvExecution, CurrentParkerCell,
};
use crate::broadcast_surface::{BroadcastCorePoll, BroadcastPublicPoll};
use crate::coop_refinement::BudgetValue;
use crate::defer_refinement::SchedulerContextAccess;
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OneshotCorePoll { Pending, Value(u64), Closed }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OneshotPublicPoll { Pending, Value(u64), Closed }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TraceLeaf { Ready, Pending, Panicked }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReceiverFuturePoll {
    Returned(OneshotPublicPoll),
    ReadyRepollPanic,
    TracePanic,
}

pub open spec fn public_of(core: OneshotCorePoll) -> OneshotPublicPoll {
    match core {
        OneshotCorePoll::Pending => OneshotPublicPoll::Pending,
        OneshotCorePoll::Value(value) => OneshotPublicPoll::Value(value),
        OneshotCorePoll::Closed => OneshotPublicPoll::Closed,
    }
}

/// Production `Receiver::poll` surface above the frozen Pin/Poll/Context and
/// arbitrary tracing callback mechanics. It proves trace-before-coop ordering,
/// budget restoration on Pending, progress commitment on Ready, exact removal
/// of the outer `Option<Arc<Inner<T>>>`, and the subsequent repoll panic.
pub struct ReceiverFutureSurface {
    inner_active: bool,
    budget: BudgetValue,
    core_polls: u64,
    progress_commits: u64,
}

impl ReceiverFutureSurface {
    pub closed spec fn inner_active(&self) -> bool { self.inner_active }
    pub closed spec fn budget(&self) -> BudgetValue { self.budget }
    pub closed spec fn core_polls(&self) -> u64 { self.core_polls }
    pub closed spec fn progress_commits(&self) -> u64 { self.progress_commits }

    pub fn new(budget: BudgetValue) -> (result: Self)
        ensures result.inner_active(), result.budget() == budget,
            result.core_polls() == 0, result.progress_commits() == 0,
        no_unwind
    {
        ReceiverFutureSurface {
            inner_active: true,
            budget,
            core_polls: 0,
            progress_commits: 0,
        }
    }

    pub fn poll(
        &mut self,
        trace: TraceLeaf,
        core: OneshotCorePoll,
    ) -> (result: ReceiverFuturePoll)
        requires old(self).core_polls() < u64::MAX,
            old(self).progress_commits() < u64::MAX,
        ensures
            !old(self).inner_active() ==>
                result == ReceiverFuturePoll::ReadyRepollPanic
                    && final(self).inner_active() == old(self).inner_active()
                    && final(self).budget() == old(self).budget()
                    && final(self).core_polls() == old(self).core_polls()
                    && final(self).progress_commits() == old(self).progress_commits(),
            old(self).inner_active() && trace == TraceLeaf::Pending ==>
                result == ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending)
                    && final(self).inner_active()
                    && final(self).budget() == old(self).budget()
                    && final(self).core_polls() == old(self).core_polls()
                    && final(self).progress_commits() == old(self).progress_commits(),
            old(self).inner_active() && trace == TraceLeaf::Panicked ==>
                result == ReceiverFuturePoll::TracePanic
                    && final(self).inner_active()
                    && final(self).budget() == old(self).budget()
                    && final(self).core_polls() == old(self).core_polls()
                    && final(self).progress_commits() == old(self).progress_commits(),
            old(self).inner_active() && trace == TraceLeaf::Ready
                && old(self).budget() == BudgetValue::Constrained(0) ==>
                result == ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending)
                    && final(self).inner_active()
                    && final(self).budget() == old(self).budget()
                    && final(self).core_polls() == old(self).core_polls()
                    && final(self).progress_commits() == old(self).progress_commits(),
            old(self).inner_active() && trace == TraceLeaf::Ready
                && old(self).budget().has_remaining() ==>
                final(self).core_polls() == old(self).core_polls() + 1
                    && result == ReceiverFuturePoll::Returned(public_of(core))
                    && (core == OneshotCorePoll::Pending ==> final(self).inner_active())
                    && (core != OneshotCorePoll::Pending ==> !final(self).inner_active())
                    && (core == OneshotCorePoll::Pending ==>
                        final(self).budget() == old(self).budget()
                            && final(self).progress_commits() == old(self).progress_commits())
                    && (core != OneshotCorePoll::Pending ==>
                        final(self).progress_commits() == old(self).progress_commits() + 1),
        no_unwind
    {
        if !self.inner_active {
            return ReceiverFuturePoll::ReadyRepollPanic;
        }
        match trace {
            TraceLeaf::Pending => {
                return ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending);
            },
            TraceLeaf::Panicked => return ReceiverFuturePoll::TracePanic,
            TraceLeaf::Ready => {},
        }
        let previous_budget = self.budget;
        match self.budget {
            BudgetValue::Constrained(0) => {
                ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending)
            },
            BudgetValue::Constrained(value) => {
                self.budget = BudgetValue::Constrained(value - 1);
                self.core_polls += 1;
                match core {
                    OneshotCorePoll::Pending => {
                        self.budget = previous_budget;
                        ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending)
                    },
                    OneshotCorePoll::Value(value) => {
                        self.progress_commits += 1;
                        self.inner_active = false;
                        ReceiverFuturePoll::Returned(OneshotPublicPoll::Value(value))
                    },
                    OneshotCorePoll::Closed => {
                        self.progress_commits += 1;
                        self.inner_active = false;
                        ReceiverFuturePoll::Returned(OneshotPublicPoll::Closed)
                    },
                }
            },
            BudgetValue::Unconstrained => {
                self.core_polls += 1;
                match core {
                    OneshotCorePoll::Pending => {
                        ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending)
                    },
                    OneshotCorePoll::Value(value) => {
                        self.progress_commits += 1;
                        self.inner_active = false;
                        ReceiverFuturePoll::Returned(OneshotPublicPoll::Value(value))
                    },
                    OneshotCorePoll::Closed => {
                        self.progress_commits += 1;
                        self.inner_active = false;
                        ReceiverFuturePoll::Returned(OneshotPublicPoll::Closed)
                    },
                }
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OneshotBlockingResult {
    ReturnedValue(u64),
    ReturnedClosed,
    RuntimePanic,
    ParkerTlsPanic,
}

/// Instantiates the already proved `future::block_on` runtime/TLS/parker path
/// with oneshot's two terminal outcomes. Arbitrary finite Pending repetition
/// is handled by `blocking_recv_composed`; scheduler liveness stays frozen.
pub fn blocking_recv(
    context: &mut SchedulerContextAccess,
    parker: &mut CurrentParkerCell,
    rt_enabled: bool,
    pending_polls: u64,
    terminal: OneshotPublicPoll,
) -> (result: OneshotBlockingResult)
    requires old(context).well_formed(), old(parker).well_formed(),
        terminal != OneshotPublicPoll::Pending,
    ensures
        final(context).well_formed(), final(parker).well_formed(),
        final(context).tls_accessible() == old(context).tls_accessible(),
        final(context).entry() == old(context).entry(),
        final(context).scoped() == old(context).scoped(),
        final(context).current() == old(context).current(),
    no_unwind
{
    let core = match terminal {
        OneshotPublicPoll::Value(value) => BroadcastCorePoll::Value(value),
        OneshotPublicPoll::Closed => BroadcastCorePoll::Closed,
        OneshotPublicPoll::Pending => unreached(),
    };
    let report = blocking_recv_composed(context, parker, rt_enabled, pending_polls, core);
    match report.into_execution() {
        BlockingRecvExecution::Returned(BroadcastPublicPoll::Value(value)) => {
            OneshotBlockingResult::ReturnedValue(value)
        },
        BlockingRecvExecution::Returned(BroadcastPublicPoll::Closed) => {
            OneshotBlockingResult::ReturnedClosed
        },
        BlockingRecvExecution::RuntimePanic => OneshotBlockingResult::RuntimePanic,
        BlockingRecvExecution::ParkerTlsPanic => OneshotBlockingResult::ParkerTlsPanic,
        BlockingRecvExecution::Returned(BroadcastPublicPoll::Pending)
            | BlockingRecvExecution::Returned(BroadcastPublicPoll::Lagged(_)) => unreached(),
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ErrorView { RecvClosed, TryEmpty, TryClosed }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DisplayView { ChannelClosed, ChannelEmpty }

pub fn clone_error(error: ErrorView) -> (result: ErrorView)
    ensures result == error,
    no_unwind
{
    error
}

pub fn display_view(error: ErrorView) -> (result: DisplayView)
    ensures result == match error {
        ErrorView::RecvClosed | ErrorView::TryClosed => DisplayView::ChannelClosed,
        ErrorView::TryEmpty => DisplayView::ChannelEmpty,
    },
    no_unwind
{
    match error {
        ErrorView::RecvClosed | ErrorView::TryClosed => DisplayView::ChannelClosed,
        ErrorView::TryEmpty => DisplayView::ChannelEmpty,
    }
}

pub fn verify_ready_then_repoll_panics(value: u64)
{
    let mut future = ReceiverFutureSurface::new(BudgetValue::Constrained(2));
    let ready = future.poll(TraceLeaf::Ready, OneshotCorePoll::Value(value));
    assert(ready == ReceiverFuturePoll::Returned(OneshotPublicPoll::Value(value)));
    assert(!future.inner_active());
    let repoll = future.poll(TraceLeaf::Ready, OneshotCorePoll::Closed);
    assert(repoll == ReceiverFuturePoll::ReadyRepollPanic);
}

pub fn verify_pending_restores_budget()
{
    let mut future = ReceiverFutureSurface::new(BudgetValue::Constrained(2));
    let pending = future.poll(TraceLeaf::Ready, OneshotCorePoll::Pending);
    assert(pending == ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending));
    assert(future.budget() == BudgetValue::Constrained(2));
    assert(future.inner_active());
}

pub fn verify_trace_precedes_coop()
{
    let mut future = ReceiverFutureSurface::new(BudgetValue::Constrained(2));
    let pending = future.poll(TraceLeaf::Pending, OneshotCorePoll::Value(7));
    assert(pending == ReceiverFuturePoll::Returned(OneshotPublicPoll::Pending));
    assert(future.core_polls() == 0);
    assert(future.budget() == BudgetValue::Constrained(2));
    let panicked = future.poll(TraceLeaf::Panicked, OneshotCorePoll::Value(7));
    assert(panicked == ReceiverFuturePoll::TracePanic);
    assert(future.core_polls() == 0);
    assert(future.inner_active());
}

} // verus!
