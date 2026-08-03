use crate::coop_refinement::BudgetValue;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MpscCorePoll { Pending, Value(u64), Closed, Many(usize) }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MpscPublicPoll { Pending, Value(u64), Closed, Many(usize) }

pub open spec fn public_of(core: MpscCorePoll) -> MpscPublicPoll {
    match core {
        MpscCorePoll::Pending => MpscPublicPoll::Pending,
        MpscCorePoll::Value(value) => MpscPublicPoll::Value(value),
        MpscCorePoll::Closed => MpscPublicPoll::Closed,
        MpscCorePoll::Many(count) => MpscPublicPoll::Many(count),
    }
}

/// Public recv, poll_recv, recv_many, and poll_recv_many all pass through the
/// same cooperative gate. Pending restores the acquired budget; every Ready
/// result consumes one unit. Raw queue state is framed by
/// `MpscRecvGateSnapshot` before this mapping.
pub struct MpscCooperativeSurface {
    budget: BudgetValue,
    core_polled: bool,
}

impl MpscCooperativeSurface {
    pub closed spec fn budget(&self) -> BudgetValue { self.budget }
    pub closed spec fn core_polled(&self) -> bool { self.core_polled }
    pub fn new(budget: BudgetValue) -> (result: Self)
        ensures result.budget() == budget, !result.core_polled(),
        no_unwind
    {
        MpscCooperativeSurface { budget, core_polled: false }
    }

    pub fn poll(&mut self, core: MpscCorePoll) -> (result: MpscPublicPoll)
        ensures old(self).budget() == BudgetValue::Constrained(0) ==>
                result == MpscPublicPoll::Pending && !final(self).core_polled(),
            old(self).budget() != BudgetValue::Constrained(0) ==>
                result == public_of(core) && final(self).core_polled(),
            old(self).budget() == BudgetValue::Unconstrained ==>
                final(self).budget() == BudgetValue::Unconstrained,
            match old(self).budget() {
                BudgetValue::Constrained(value) => value > 0 ==>
                    if core == MpscCorePoll::Pending {
                        final(self).budget() == BudgetValue::Constrained(value)
                    } else {
                        final(self).budget() == BudgetValue::Constrained((value - 1) as u8)
                    },
                BudgetValue::Unconstrained => true,
            },
        no_unwind
    {
        let previous = self.budget;
        match self.budget {
            BudgetValue::Constrained(0) => {
                self.core_polled = false;
                MpscPublicPoll::Pending
            },
            BudgetValue::Constrained(value) => {
                self.budget = BudgetValue::Constrained(value - 1);
                self.core_polled = true;
                match core {
                    MpscCorePoll::Pending => {
                        self.budget = previous;
                        MpscPublicPoll::Pending
                    },
                    MpscCorePoll::Value(value) => MpscPublicPoll::Value(value),
                    MpscCorePoll::Closed => MpscPublicPoll::Closed,
                    MpscCorePoll::Many(count) => MpscPublicPoll::Many(count),
                }
            },
            BudgetValue::Unconstrained => {
                self.core_polled = true;
                match core {
                    MpscCorePoll::Pending => MpscPublicPoll::Pending,
                    MpscCorePoll::Value(value) => MpscPublicPoll::Value(value),
                    MpscCorePoll::Closed => MpscPublicPoll::Closed,
                    MpscCorePoll::Many(count) => MpscPublicPoll::Many(count),
                }
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockingContext { OutsideRuntime, InsideRuntime }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockingResult { Returned(MpscPublicPoll), RuntimePanic }

/// Both `rt` and non-`rt` public blocking wrappers forward the exact async
/// result outside a runtime. The `rt` form first rejects an entered runtime;
/// CachedParkThread/TLS/Condvar and scheduler liveness are the frozen generic
/// block_on foundation already connected by `blocking_recv_refinement`.
pub fn blocking_surface(
    context: BlockingContext,
    async_result: MpscPublicPoll,
) -> (result: BlockingResult)
    ensures context == BlockingContext::InsideRuntime ==>
            result == BlockingResult::RuntimePanic,
        context == BlockingContext::OutsideRuntime ==>
            result == BlockingResult::Returned(async_result),
    no_unwind
{
    match context {
        BlockingContext::InsideRuntime => BlockingResult::RuntimePanic,
        BlockingContext::OutsideRuntime => BlockingResult::Returned(async_result),
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MpscErrorView {
    SendClosed,
    TrySendFull,
    TrySendClosed,
    TryRecvEmpty,
    TryRecvDisconnected,
    SendTimeout,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MpscDisplayView { Closed, Full, Empty, Disconnected, Timeout }

pub fn error_display(error: MpscErrorView) -> (result: MpscDisplayView)
    ensures result == match error {
        MpscErrorView::SendClosed | MpscErrorView::TrySendClosed => MpscDisplayView::Closed,
        MpscErrorView::TrySendFull => MpscDisplayView::Full,
        MpscErrorView::TryRecvEmpty => MpscDisplayView::Empty,
        MpscErrorView::TryRecvDisconnected => MpscDisplayView::Disconnected,
        MpscErrorView::SendTimeout => MpscDisplayView::Timeout,
    },
    no_unwind
{
    match error {
        MpscErrorView::SendClosed | MpscErrorView::TrySendClosed => MpscDisplayView::Closed,
        MpscErrorView::TrySendFull => MpscDisplayView::Full,
        MpscErrorView::TryRecvEmpty => MpscDisplayView::Empty,
        MpscErrorView::TryRecvDisconnected => MpscDisplayView::Disconnected,
        MpscErrorView::SendTimeout => MpscDisplayView::Timeout,
    }
}

pub fn into_inner<T>(value: T) -> (result: T)
    ensures result == value,
    no_unwind
{
    value
}

pub fn same_channel(left: u64, right: u64) -> (result: bool)
    ensures result == (left == right),
    no_unwind
{
    left == right
}

pub fn verify_mpsc_public_surface(value: u64)
{
    let mut pending = MpscCooperativeSurface::new(BudgetValue::Constrained(2));
    let pending_result = pending.poll(MpscCorePoll::Pending);
    assert(pending_result == MpscPublicPoll::Pending);
    assert(pending.budget() == BudgetValue::Constrained(2));
    let mut ready = MpscCooperativeSurface::new(BudgetValue::Constrained(2));
    let ready_result = ready.poll(MpscCorePoll::Value(value));
    assert(ready_result == MpscPublicPoll::Value(value));
    assert(ready.budget() == BudgetValue::Constrained(1));
    let inner = into_inner(value);
    assert(inner == value);
    let same = same_channel(7, 7);
    let different = same_channel(7, 8);
    assert(same && !different);
    let outside = blocking_surface(
        BlockingContext::OutsideRuntime, MpscPublicPoll::Many(2));
    assert(outside == BlockingResult::Returned(MpscPublicPoll::Many(2)));
    let inside = blocking_surface(
        BlockingContext::InsideRuntime, MpscPublicPoll::Closed);
    assert(inside == BlockingResult::RuntimePanic);
}

} // verus!
