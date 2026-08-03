use crate::coop_refinement::BudgetValue;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastCorePoll {
    Pending,
    Value(u64),
    Empty,
    Closed,
    Lagged(u64),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastPublicPoll {
    Pending,
    Value(u64),
    Closed,
    Lagged(u64),
}

pub open spec fn public_of(core: BroadcastCorePoll) -> BroadcastPublicPoll {
    match core {
        BroadcastCorePoll::Pending | BroadcastCorePoll::Empty => BroadcastPublicPoll::Pending,
        BroadcastCorePoll::Value(value) => BroadcastPublicPoll::Value(value),
        BroadcastCorePoll::Closed => BroadcastPublicPoll::Closed,
        BroadcastCorePoll::Lagged(amount) => BroadcastPublicPoll::Lagged(amount),
    }
}

/// Production-shaped local composition of cooperative gating with Recv::poll.
/// `core_polled` makes the exhausted-budget non-consumption guarantee explicit.
/// The compiled TLS access used to obtain this budget remains the named T01
/// dependency rather than being hidden as a precomputed core oracle.
pub struct BroadcastRecvCoop {
    budget: BudgetValue,
    core_polled: bool,
}

impl BroadcastRecvCoop {
    pub closed spec fn budget(&self) -> BudgetValue { self.budget }
    pub closed spec fn core_polled(&self) -> bool { self.core_polled }

    pub fn new(budget: BudgetValue) -> (result: Self)
        ensures result.budget() == budget, !result.core_polled(),
        no_unwind
    {
        BroadcastRecvCoop { budget, core_polled: false }
    }

    pub fn poll(&mut self, core: BroadcastCorePoll) -> (result: BroadcastPublicPoll)
        ensures
            old(self).budget() == BudgetValue::Constrained(0) ==>
                result == BroadcastPublicPoll::Pending && !final(self).core_polled(),
            old(self).budget() != BudgetValue::Constrained(0) ==>
                final(self).core_polled(),
            old(self).budget() != BudgetValue::Constrained(0) ==>
                result == public_of(core),
            old(self).budget() == BudgetValue::Unconstrained ==>
                final(self).budget() == BudgetValue::Unconstrained,
            match old(self).budget() {
                BudgetValue::Constrained(value) => value > 0 ==>
                    if public_of(core) == BroadcastPublicPoll::Pending {
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
                BroadcastPublicPoll::Pending
            },
            BudgetValue::Constrained(value) => {
                self.budget = BudgetValue::Constrained(value - 1);
                self.core_polled = true;
                match core {
                    BroadcastCorePoll::Pending | BroadcastCorePoll::Empty => {
                        self.budget = previous;
                        BroadcastPublicPoll::Pending
                    },
                    BroadcastCorePoll::Value(value) => BroadcastPublicPoll::Value(value),
                    BroadcastCorePoll::Closed => BroadcastPublicPoll::Closed,
                    BroadcastCorePoll::Lagged(amount) => BroadcastPublicPoll::Lagged(amount),
                }
            },
            BudgetValue::Unconstrained => {
                self.core_polled = true;
                match core {
                    BroadcastCorePoll::Pending | BroadcastCorePoll::Empty => {
                        BroadcastPublicPoll::Pending
                    },
                    BroadcastCorePoll::Value(value) => BroadcastPublicPoll::Value(value),
                    BroadcastCorePoll::Closed => BroadcastPublicPoll::Closed,
                    BroadcastCorePoll::Lagged(amount) => BroadcastPublicPoll::Lagged(amount),
                }
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ClosedPoll {
    PendingRegistered,
    Ready,
}

/// Path-specific projection of `Sender::closed`: construct notification first,
/// then recheck `tail.closed` under the tail lock. A concurrent last Receiver
/// drop therefore either wins the recheck or changes the notification
/// generation observed by the subsequent poll.
pub fn poll_sender_closed(tail_closed: bool, notification_changed: bool)
    -> (result: ClosedPoll)
    ensures
        tail_closed ==> result == ClosedPoll::Ready,
        !tail_closed && notification_changed ==> result == ClosedPoll::Ready,
        !tail_closed && !notification_changed ==> result == ClosedPoll::PendingRegistered,
    no_unwind
{
    if tail_closed || notification_changed {
        ClosedPoll::Ready
    } else {
        ClosedPoll::PendingRegistered
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockingContext {
    OutsideRuntime,
    InsideRuntime,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockingRecvResult {
    Returned(BroadcastPublicPoll),
    RuntimePanic,
}

/// Public blocking surface: production `block_on` rejects a runtime context;
/// outside one it forwards the exact async recv result. Parking/scheduler
/// liveness is not claimed here.
pub fn blocking_recv_surface(
    context: BlockingContext,
    async_result: BroadcastPublicPoll,
) -> (result: BlockingRecvResult)
    ensures
        context == BlockingContext::InsideRuntime
            ==> result == BlockingRecvResult::RuntimePanic,
        context == BlockingContext::OutsideRuntime
            ==> result == BlockingRecvResult::Returned(async_result),
    no_unwind
{
    match context {
        BlockingContext::InsideRuntime => BlockingRecvResult::RuntimePanic,
        BlockingContext::OutsideRuntime => BlockingRecvResult::Returned(async_result),
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastErrorView {
    SendClosed,
    RecvClosed,
    RecvLagged(u64),
    TryEmpty,
    TryClosed,
    TryLagged(u64),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DisplayView {
    ChannelClosed,
    ChannelEmpty,
    ChannelLagged(u64),
}

pub fn display_view(error: BroadcastErrorView) -> (result: DisplayView)
    ensures
        result == match error {
            BroadcastErrorView::SendClosed | BroadcastErrorView::RecvClosed
                | BroadcastErrorView::TryClosed => DisplayView::ChannelClosed,
            BroadcastErrorView::TryEmpty => DisplayView::ChannelEmpty,
            BroadcastErrorView::RecvLagged(amount)
                | BroadcastErrorView::TryLagged(amount) => DisplayView::ChannelLagged(amount),
        },
    no_unwind
{
    match error {
        BroadcastErrorView::SendClosed | BroadcastErrorView::RecvClosed
            | BroadcastErrorView::TryClosed => DisplayView::ChannelClosed,
        BroadcastErrorView::TryEmpty => DisplayView::ChannelEmpty,
        BroadcastErrorView::RecvLagged(amount) | BroadcastErrorView::TryLagged(amount) => {
            DisplayView::ChannelLagged(amount)
        },
    }
}

pub fn verify_public_recv_budget_paths()
{
    let mut ready_budget = BroadcastRecvCoop::new(BudgetValue::Constrained(2));
    let value = ready_budget.poll(BroadcastCorePoll::Value(9));
    assert(value == BroadcastPublicPoll::Value(9));
    assert(ready_budget.budget() == BudgetValue::Constrained(1));
    assert(ready_budget.core_polled());

    let mut pending_budget = BroadcastRecvCoop::new(BudgetValue::Constrained(2));
    let pending = pending_budget.poll(BroadcastCorePoll::Empty);
    assert(pending == BroadcastPublicPoll::Pending);
    assert(pending_budget.budget() == BudgetValue::Constrained(2));
    assert(pending_budget.core_polled());

    let mut exhausted = BroadcastRecvCoop::new(BudgetValue::Constrained(0));
    let forced = exhausted.poll(BroadcastCorePoll::Value(9));
    assert(forced == BroadcastPublicPoll::Pending);
    assert(!exhausted.core_polled());
}

} // verus!
