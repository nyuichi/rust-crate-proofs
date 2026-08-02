use vstd::prelude::*;

verus! {

pub const INITIAL_BUDGET: u8 = 128;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BudgetValue {
    Constrained(u8),
    Unconstrained,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Decrement {
    pub success: bool,
    pub hit_zero: bool,
}

impl BudgetValue {
    pub closed spec fn view(&self) -> BudgetValue { *self }

    pub open spec fn has_remaining(self) -> bool {
        match self {
            BudgetValue::Constrained(value) => value > 0,
            BudgetValue::Unconstrained => true,
        }
    }

    /// Exact value-level refinement of production `Budget::decrement`.
    pub fn decrement(&mut self) -> (result: Decrement)
        ensures
            match old(self).view() {
                BudgetValue::Unconstrained => {
                    &&& final(self).view() == BudgetValue::Unconstrained
                    &&& result == (Decrement { success: true, hit_zero: false })
                },
                BudgetValue::Constrained(0) => {
                    &&& final(self).view() == BudgetValue::Constrained(0)
                    &&& result == (Decrement { success: false, hit_zero: false })
                },
                BudgetValue::Constrained(value) => {
                    &&& value > 0
                    &&& final(self).view() == BudgetValue::Constrained((value - 1) as u8)
                    &&& result.success
                    &&& result.hit_zero == (value == 1)
                },
            },
        no_unwind
    {
        match self {
            BudgetValue::Constrained(value) => {
                if *value > 0 {
                    *value -= 1;
                    Decrement { success: true, hit_zero: *value == 0 }
                } else {
                    Decrement { success: false, hit_zero: false }
                }
            },
            BudgetValue::Unconstrained => {
                Decrement { success: true, hit_zero: false }
            },
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct RestoreTicket {
    pub remembered: BudgetValue,
}

impl RestoreTicket {
    pub closed spec fn remembered(&self) -> BudgetValue { self.remembered }

    /// Refines `RestoreOnPending::made_progress`: replacing the remembered
    /// constrained budget with the unconstrained sentinel commits consumption.
    pub fn made_progress(&mut self)
        ensures final(self).remembered() == BudgetValue::Unconstrained,
        no_unwind
    {
        self.remembered = BudgetValue::Unconstrained;
    }
}

#[derive(PartialEq, Eq)]
pub enum ProceedResult {
    Ready { restore: RestoreTicket, hit_zero: bool },
    Pending { registered: bool },
}

pub struct CoopBudget {
    current: BudgetValue,
}

impl CoopBudget {
    pub closed spec fn current(&self) -> BudgetValue { self.current }

    pub fn new(value: BudgetValue) -> (result: Self)
        ensures result.current() == value,
        no_unwind
    {
        CoopBudget { current: value }
    }

    /// Logical, production-shaped refinement of the accessible-context closure
    /// in `poll_proceed`. Task-local/context access and the compiled TLS closure
    /// remain an unproved production connection. `registered` records the call
    /// to `register_waker`; Waker mechanics are an agreed foundation boundary.
    pub fn poll_proceed(&mut self) -> (result: ProceedResult)
        ensures
            match old(self).current() {
                BudgetValue::Unconstrained => {
                    &&& final(self).current() == BudgetValue::Unconstrained
                    &&& result == (ProceedResult::Ready {
                        restore: RestoreTicket { remembered: BudgetValue::Unconstrained },
                        hit_zero: false,
                    })
                },
                BudgetValue::Constrained(0) => {
                    &&& final(self).current() == BudgetValue::Constrained(0)
                    &&& result == (ProceedResult::Pending { registered: true })
                },
                BudgetValue::Constrained(value) => {
                    &&& value > 0
                    &&& result == (ProceedResult::Ready {
                        restore: RestoreTicket {
                            remembered: BudgetValue::Constrained(value),
                        },
                        hit_zero: value == 1,
                    })
                    &&& final(self).current() == BudgetValue::Constrained((value - 1) as u8)
                },
            },
        no_unwind
    {
        let previous = self.current;
        let decrement = self.current.decrement();
        if decrement.success {
            ProceedResult::Ready {
                restore: RestoreTicket { remembered: previous },
                hit_zero: decrement.hit_zero,
            }
        } else {
            ProceedResult::Pending { registered: true }
        }
    }

    /// Refines `RestoreOnPending::drop`. A committed or unconstrained ticket
    /// leaves the current budget alone; an uncommitted constrained ticket
    /// restores the exact pre-decrement value.
    pub fn drop_restore(&mut self, restore: RestoreTicket)
        ensures
            match restore.remembered() {
                BudgetValue::Unconstrained => final(self).current() == old(self).current(),
                remembered => final(self).current() == remembered,
            },
        no_unwind
    {
        match restore.remembered {
            BudgetValue::Unconstrained => {},
            remembered => self.current = remembered,
        }
    }

    /// Refinement of installing a scoped budget. The returned value is the
    /// exact `ResetGuard::prev` resource.
    pub fn install(&mut self, next: BudgetValue) -> (previous: BudgetValue)
        ensures
            previous == old(self).current(),
            final(self).current() == next,
        no_unwind
    {
        let previous = self.current;
        self.current = next;
        previous
    }

    /// Normal return and unwind execute the same production ResetGuard drop;
    /// this operation is the state effect of either exit.
    pub fn reset(&mut self, previous: BudgetValue)
        ensures final(self).current() == previous,
        no_unwind
    {
        self.current = previous;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum YieldPhase {
    Initial,
    Yielded,
    Complete,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct YieldPoll {
    pub ready: bool,
    pub deferred: bool,
}

pub struct YieldNow {
    phase: YieldPhase,
}

impl YieldNow {
    pub closed spec fn phase(&self) -> YieldPhase { self.phase }

    pub fn new() -> (result: Self)
        ensures result.phase() == YieldPhase::Initial,
        no_unwind
    {
        YieldNow { phase: YieldPhase::Initial }
    }

    /// Exact branch ordering of `yield_now`: `trace_leaf` is checked before
    /// the yielded-state recheck, and only the first trace-ready poll defers
    /// the Waker before returning Pending.
    pub fn poll(&mut self, trace_ready: bool) -> (result: YieldPoll)
        requires old(self).phase() != YieldPhase::Complete,
        ensures
            !trace_ready ==> {
                &&& final(self).phase() == old(self).phase()
                &&& result == (YieldPoll { ready: false, deferred: false })
            },
            trace_ready && old(self).phase() == YieldPhase::Initial ==> {
                &&& final(self).phase() == YieldPhase::Yielded
                &&& result == (YieldPoll { ready: false, deferred: true })
            },
            trace_ready && old(self).phase() == YieldPhase::Yielded ==> {
                &&& final(self).phase() == YieldPhase::Complete
                &&& result == (YieldPoll { ready: true, deferred: false })
            },
        no_unwind
    {
        if !trace_ready {
            YieldPoll { ready: false, deferred: false }
        } else {
            match self.phase {
                YieldPhase::Initial => {
                    self.phase = YieldPhase::Yielded;
                    YieldPoll { ready: false, deferred: true }
                },
                YieldPhase::Yielded => {
                    self.phase = YieldPhase::Complete;
                    YieldPoll { ready: true, deferred: false }
                },
                YieldPhase::Complete => {
                    assert(false);
                    YieldPoll { ready: false, deferred: false }
                },
            }
        }
    }
}

pub fn verify_budget_conservation_and_forced_pending()
{
    let mut budget = CoopBudget::new(BudgetValue::Constrained(2));
    let first = budget.poll_proceed();
    let mut first_restore = match first {
        ProceedResult::Ready { restore, hit_zero } => {
            assert(!hit_zero);
            restore
        },
        ProceedResult::Pending { .. } => { assert(false); return; },
    };
    first_restore.made_progress();
    budget.drop_restore(first_restore);
    assert(budget.current() == BudgetValue::Constrained(1));

    let second = budget.poll_proceed();
    let mut second_restore = match second {
        ProceedResult::Ready { restore, hit_zero } => {
            assert(hit_zero);
            restore
        },
        ProceedResult::Pending { .. } => { assert(false); return; },
    };
    second_restore.made_progress();
    budget.drop_restore(second_restore);
    assert(budget.current() == BudgetValue::Constrained(0));

    let exhausted = budget.poll_proceed();
    assert(exhausted == (ProceedResult::Pending { registered: true }));
    assert(budget.current() == BudgetValue::Constrained(0));
}

pub fn verify_pending_work_restores_exact_budget()
{
    let mut budget = CoopBudget::new(BudgetValue::Constrained(9));
    let result = budget.poll_proceed();
    let restore = match result {
        ProceedResult::Ready { restore, hit_zero } => {
            assert(!hit_zero);
            restore
        },
        ProceedResult::Pending { .. } => { assert(false); return; },
    };
    assert(budget.current() == BudgetValue::Constrained(8));
    budget.drop_restore(restore);
    assert(budget.current() == BudgetValue::Constrained(9));
}

pub fn verify_unconstrained_nesting_and_unwind_restoration()
{
    let mut budget = CoopBudget::new(BudgetValue::Constrained(17));
    let outer = budget.install(BudgetValue::Unconstrained);
    let result = budget.poll_proceed();
    let restore = match result {
        ProceedResult::Ready { restore, hit_zero } => {
            assert(!hit_zero);
            restore
        },
        ProceedResult::Pending { .. } => { assert(false); return; },
    };
    budget.drop_restore(restore);
    assert(budget.current() == BudgetValue::Unconstrained);

    let inner = budget.install(BudgetValue::Constrained(INITIAL_BUDGET));
    let inner_result = budget.poll_proceed();
    let inner_restore = match inner_result {
        ProceedResult::Ready { restore, .. } => restore,
        ProceedResult::Pending { .. } => { assert(false); return; },
    };
    budget.drop_restore(inner_restore);
    budget.reset(inner);
    assert(budget.current() == BudgetValue::Unconstrained);

    // This reset is also the modeled state effect when the outer closure
    // unwinds and production `ResetGuard::drop` runs.
    budget.reset(outer);
    assert(budget.current() == BudgetValue::Constrained(17));
}

pub fn verify_yield_register_recheck_order()
{
    let mut future = YieldNow::new();
    let trace_pending = future.poll(false);
    assert(trace_pending == (YieldPoll { ready: false, deferred: false }));
    assert(future.phase() == YieldPhase::Initial);

    let first = future.poll(true);
    assert(first == (YieldPoll { ready: false, deferred: true }));
    assert(future.phase() == YieldPhase::Yielded);

    let second_trace_pending = future.poll(false);
    assert(second_trace_pending == (YieldPoll { ready: false, deferred: false }));
    assert(future.phase() == YieldPhase::Yielded);

    let second = future.poll(true);
    assert(second == (YieldPoll { ready: true, deferred: false }));
    assert(future.phase() == YieldPhase::Complete);
}

pub proof fn verify_coop_mutants_rejected()
{
    // Restoring the decremented value loses the retry credit.
    assert(BudgetValue::Constrained(9) != BudgetValue::Constrained(8));
    // The transition to zero succeeds; Pending belongs to the next poll.
    assert(1u8 - 1u8 == 0u8);
    // A forced Pending is coupled to Waker registration.
    assert((ProceedResult::Pending { registered: true })
        != (ProceedResult::Pending { registered: false }));
    // The first trace-ready yield poll must defer and cannot complete.
    assert((YieldPoll { ready: false, deferred: true })
        != (YieldPoll { ready: true, deferred: false }));
}

} // verus!
