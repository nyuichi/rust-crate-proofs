use crate::coop_refinement::BudgetValue;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchAsyncCore {
    Pending,
    Changed,
    Closed,
    PredicateMatched,
    PredicatePanicked,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchAsyncPublic {
    Pending,
    Changed,
    Closed,
    PredicateMatched,
    PredicatePanicked,
}

pub open spec fn watch_public_of(core: WatchAsyncCore) -> WatchAsyncPublic {
    match core {
        WatchAsyncCore::Pending => WatchAsyncPublic::Pending,
        WatchAsyncCore::Changed => WatchAsyncPublic::Changed,
        WatchAsyncCore::Closed => WatchAsyncPublic::Closed,
        WatchAsyncCore::PredicateMatched => WatchAsyncPublic::PredicateMatched,
        WatchAsyncCore::PredicatePanicked => WatchAsyncPublic::PredicatePanicked,
    }
}

/// Shared `cooperative(...)` projection used by watch `changed`, `wait_for`,
/// and `Sender::closed`.  T01 proves the generic budget/TLS implementation;
/// this body proves the watch-specific public result mapping and rollback on
/// Pending.
pub struct WatchCooperativeSurface {
    budget: BudgetValue,
    core_polled: bool,
}

impl WatchCooperativeSurface {
    pub closed spec fn budget(&self) -> BudgetValue { self.budget }
    pub closed spec fn core_polled(&self) -> bool { self.core_polled }

    pub fn new(budget: BudgetValue) -> (result: Self)
        ensures result.budget() == budget, !result.core_polled(),
        no_unwind
    {
        WatchCooperativeSurface { budget, core_polled: false }
    }

    pub fn poll(&mut self, core: WatchAsyncCore) -> (result: WatchAsyncPublic)
        ensures
            old(self).budget() == BudgetValue::Constrained(0) ==>
                result == WatchAsyncPublic::Pending && !final(self).core_polled(),
            old(self).budget() != BudgetValue::Constrained(0) ==>
                result == watch_public_of(core) && final(self).core_polled(),
            old(self).budget() == BudgetValue::Unconstrained ==>
                final(self).budget() == BudgetValue::Unconstrained,
            match old(self).budget() {
                BudgetValue::Constrained(value) => (value > 0) ==>
                    (if core == WatchAsyncCore::Pending {
                        final(self).budget() == BudgetValue::Constrained(value)
                    } else {
                        final(self).budget() == BudgetValue::Constrained((value - 1) as u8)
                    }),
                BudgetValue::Unconstrained => true,
            },
        no_unwind
    {
        let previous = self.budget;
        match self.budget {
            BudgetValue::Constrained(0) => {
                self.core_polled = false;
                WatchAsyncPublic::Pending
            },
            BudgetValue::Constrained(value) => {
                self.budget = BudgetValue::Constrained(value - 1);
                self.core_polled = true;
                match core {
                    WatchAsyncCore::Pending => {
                        self.budget = previous;
                        WatchAsyncPublic::Pending
                    },
                    WatchAsyncCore::Changed => WatchAsyncPublic::Changed,
                    WatchAsyncCore::Closed => WatchAsyncPublic::Closed,
                    WatchAsyncCore::PredicateMatched => WatchAsyncPublic::PredicateMatched,
                    WatchAsyncCore::PredicatePanicked => WatchAsyncPublic::PredicatePanicked,
                }
            },
            BudgetValue::Unconstrained => {
                self.core_polled = true;
                match core {
                    WatchAsyncCore::Pending => WatchAsyncPublic::Pending,
                    WatchAsyncCore::Changed => WatchAsyncPublic::Changed,
                    WatchAsyncCore::Closed => WatchAsyncPublic::Closed,
                    WatchAsyncCore::PredicateMatched => WatchAsyncPublic::PredicateMatched,
                    WatchAsyncCore::PredicatePanicked => WatchAsyncPublic::PredicatePanicked,
                }
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchDisplayView {
    ChannelClosed,
}

pub fn send_error_display<T>(value: T) -> (result: (WatchDisplayView, T))
    ensures result.0 == WatchDisplayView::ChannelClosed, result.1 == value,
    no_unwind
{
    (WatchDisplayView::ChannelClosed, value)
}

pub fn recv_error_display() -> (result: WatchDisplayView)
    ensures result == WatchDisplayView::ChannelClosed,
    no_unwind
{
    WatchDisplayView::ChannelClosed
}

pub fn clone_send_error<T: Copy>(value: T) -> (result: T)
    ensures result == value,
    no_unwind
{
    value
}

pub struct DefaultSenderView<T> {
    value: T,
    senders: usize,
    receivers: usize,
}

impl<T> DefaultSenderView<T> {
    pub closed spec fn value(&self) -> T { self.value }
    pub closed spec fn senders(&self) -> usize { self.senders }
    pub closed spec fn receivers(&self) -> usize { self.receivers }

    pub fn from_default(value: T) -> (result: Self)
        ensures result.value() == value, result.senders() == 1,
            result.receivers() == 0,
        no_unwind
    {
        DefaultSenderView { value, senders: 1, receivers: 0 }
    }
}

pub fn same_channel(left_identity: u64, right_identity: u64) -> (result: bool)
    ensures result == (left_identity == right_identity),
    no_unwind
{
    left_identity == right_identity
}

pub fn verify_watch_surface_paths(value: u64)
{
    let mut pending = WatchCooperativeSurface::new(BudgetValue::Constrained(2));
    let pending_result = pending.poll(WatchAsyncCore::Pending);
    assert(pending_result == WatchAsyncPublic::Pending);
    assert(pending.budget() == BudgetValue::Constrained(2));

    let mut ready = WatchCooperativeSurface::new(BudgetValue::Constrained(2));
    let ready_result = ready.poll(WatchAsyncCore::Changed);
    assert(ready_result == WatchAsyncPublic::Changed);
    assert(ready.budget() == BudgetValue::Constrained(1));

    let (display, preserved) = send_error_display(value);
    assert(display == WatchDisplayView::ChannelClosed);
    assert(preserved == value);
    let same = same_channel(7, 7);
    let different = same_channel(7, 8);
    assert(same);
    assert(!different);
    let default = DefaultSenderView::from_default(value);
    assert(default.value() == value);
    assert(default.senders() == 1);
    assert(default.receivers() == 0);
}

} // verus!
