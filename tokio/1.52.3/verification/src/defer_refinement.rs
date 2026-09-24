#[cfg(verus_keep_ghost)]
use crate::broadcast_surface::public_of;
use crate::broadcast_surface::{BroadcastCorePoll, BroadcastPublicPoll};
use crate::coop_refinement::BudgetValue;
#[cfg(verus_keep_ghost)]
use crate::coop_tls_refinement::decremented_budget;
use crate::coop_tls_refinement::{
    tls_poll_proceed, BroadcastTlsPoll, BudgetTlsState, ThreadLocalBudgetCell, TlsProceed,
};
use crate::vstd_ext::thread_local::{
    LocalState, ScopeGuard, ScopedBinding, ThreadLocalCell, TotalCallback, TryAccess,
};
use vstd::prelude::*;

verus! {

/// Body-proved mapping of Tokio's shared `runtime::context::CONTEXT` state.
/// The generic thread-local/Cell mechanics below it are supplied by the
/// repository-local vstd extension; scheduler execution remains out of scope.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RuntimeEntry {
    NotEntered,
    Entered,
}

pub type ContextTlsAccess = TryAccess;

pub struct SchedulerContextAccess {
    tls: ThreadLocalCell<u8>,
    tls_live: bool,
    entry: RuntimeEntry,
    scoped: ScopedBinding<u64>,
    tls_closure_runs: u8,
    callback_runs: u8,
}

impl SchedulerContextAccess {
    pub closed spec fn tls_accessible(&self) -> bool {
        self.tls_live
    }
    pub closed spec fn entry(&self) -> RuntimeEntry { self.entry }
    pub closed spec fn scoped(&self) -> Seq<u64> { self.scoped.stack() }
    pub closed spec fn current(&self) -> Option<u64> { self.scoped.current() }
    pub closed spec fn tls_closure_runs(&self) -> u8 { self.tls_closure_runs }
    pub closed spec fn callback_runs(&self) -> u8 { self.callback_runs }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.tls.thread() == 0
        &&& self.tls.key() == 2
        &&& self.tls_live == matches!(self.tls.state(), LocalState::Accessible(_))
        &&& self.scoped.thread() == 0
        &&& !matches!(self.tls.state(), LocalState::Borrowed)
        &&& (!self.tls_accessible() ==> self.scoped().len() == 0)
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.tls_accessible(), result.entry() == RuntimeEntry::NotEntered,
            result.scoped().len() == 0, result.tls_closure_runs() == 0,
            result.current().is_none(), result.callback_runs() == 0,
        no_unwind
    {
        SchedulerContextAccess {
            tls: ThreadLocalCell::new(0, 2, 0),
            tls_live: true,
            entry: RuntimeEntry::NotEntered,
            scoped: ScopedBinding::new(0),
            tls_closure_runs: 0,
            callback_runs: 0,
        }
    }

    pub fn enter_runtime(&mut self)
        requires old(self).well_formed(), old(self).tls_accessible(), old(self).entry() == RuntimeEntry::NotEntered,
        ensures final(self).well_formed(), final(self).entry() == RuntimeEntry::Entered,
            final(self).tls_accessible() == old(self).tls_accessible(),
            final(self).scoped() == old(self).scoped(),
            final(self).current() == old(self).current(),
        no_unwind
    {
        self.entry = RuntimeEntry::Entered;
    }

    /// Executable projection used only to couple models of fields stored in
    /// the same production CONTEXT. It performs no simulated TLS access.
    pub fn is_tls_accessible(&self) -> (result: bool)
        requires self.well_formed(),
        ensures result == self.tls_accessible(),
        no_unwind
    {
        self.tls_live
    }

    pub fn teardown(&mut self)
        requires old(self).well_formed(), old(self).tls_accessible(), old(self).scoped().len() == 0,
        ensures final(self).well_formed(), !final(self).tls_accessible(),
            final(self).entry() == old(self).entry(),
            final(self).scoped() == old(self).scoped(),
            final(self).current() == old(self).current(),
        no_unwind
    {
        self.tls.teardown(0);
        self.tls_live = false;
    }

    /// Body mapping of `runtime::context::try_enter_blocking_region`. This is
    /// deliberately a method on the same CONTEXT model used by
    /// `with_scheduler`, rather than an independent thread-local key.
    pub fn try_enter_blocking_region(&mut self) -> (result: BlockingRegionResult)
        requires old(self).well_formed(),
        ensures final(self).well_formed(),
            final(self).tls_accessible() == old(self).tls_accessible(),
            final(self).entry() == old(self).entry(),
            final(self).scoped() == old(self).scoped(),
            final(self).current() == old(self).current(),
            final(self).callback_runs() == 1,
            old(self).tls_accessible() && old(self).entry() == RuntimeEntry::Entered ==>
                result == BlockingRegionResult::RejectedRuntime
                    && final(self).tls_closure_runs() == 1,
            old(self).tls_accessible() && old(self).entry() == RuntimeEntry::NotEntered ==>
                result == BlockingRegionResult::Allowed(ContextTlsAccess::ClosureOnce)
                    && final(self).tls_closure_runs() == 1,
            !old(self).tls_accessible() ==>
                result == BlockingRegionResult::Allowed(
                    ContextTlsAccess::ClosureZeroTeardown)
                    && final(self).tls_closure_runs() == 0,
        no_unwind
    {
        let session = self.tls.try_with_begin(0);
        let current = session.get();
        let access = if current.is_some() {
            ContextTlsAccess::ClosureOnce
        } else {
            ContextTlsAccess::ClosureZeroTeardown
        };
        let total = TotalCallback::from_access(access);
        let runs = total.runs();
        self.tls_closure_runs = runs.0;
        self.callback_runs = runs.1;
        let result = match current {
            Some(_) => match self.entry {
                RuntimeEntry::Entered => BlockingRegionResult::RejectedRuntime,
                RuntimeEntry::NotEntered => BlockingRegionResult::Allowed(access),
            },
            None => BlockingRegionResult::Allowed(access),
        };
        self.tls.try_with_end(session);
        result
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockingRegionResult {
    Allowed(ContextTlsAccess),
    RejectedRuntime,
}

pub type SchedulerScopeGuard = ScopeGuard<u64>;

/// Body model of Scoped::set entry. The common primitive establishes that the
/// binding is same-thread and cannot escape the dynamic closure extent.
pub fn scheduler_scope_enter(
    context: &mut SchedulerContextAccess,
    scheduler: u64,
) -> (guard: SchedulerScopeGuard)
    requires old(context).well_formed(), old(context).tls_accessible(),
    ensures final(context).well_formed(), final(context).tls_accessible(),
        final(context).entry() == old(context).entry(),
        final(context).scoped() == old(context).scoped().push(scheduler),
        final(context).current() == Some(scheduler),
        guard.previous() == old(context).scoped(),
        guard.previous_current() == old(context).current(),
    no_unwind
{
    context.scoped.enter(0, scheduler)
}

/// ResetGuard normal return and unwind effect. Supplying nested guards in LIFO
/// order restores the exact prior same-thread binding stack.
pub fn scheduler_scope_drop(
    context: &mut SchedulerContextAccess,
    guard: SchedulerScopeGuard,
)
    requires old(context).well_formed(), old(context).tls_accessible(), old(context).scoped().len() > 0,
        old(context).scoped().drop_last() == guard.previous(),
    ensures final(context).well_formed(), final(context).tls_accessible(),
        final(context).entry() == old(context).entry(),
        final(context).scoped() == guard.previous(),
        final(context).current() == guard.previous_current(),
    no_unwind
{
    context.scoped.exit(0, guard);
}

pub struct WithSchedulerResult {
    /// Non-dereferenceable route tag. The adapter's actual scheduler reference
    /// is valid only during the callback and cannot escape its dynamic extent.
    pub scheduler: Option<u64>,
    pub access: ContextTlsAccess,
}

/// One execution of `with_scheduler`: the CONTEXT closure executes once on
/// successful TLS access and zero times after teardown, while the caller's `f`
/// executes exactly once in either the success or fallback path.
pub fn with_scheduler(
    context: &mut SchedulerContextAccess,
) -> (result: WithSchedulerResult)
    requires old(context).well_formed(),
    ensures final(context).well_formed(),
        final(context).tls_accessible() == old(context).tls_accessible(),
        final(context).entry() == old(context).entry(),
        final(context).scoped() == old(context).scoped(),
        final(context).current() == old(context).current(),
        final(context).callback_runs() == 1,
        old(context).tls_accessible() ==> result.scheduler == match old(context).entry() {
            RuntimeEntry::Entered => old(context).current(),
            RuntimeEntry::NotEntered => None,
        },
        !old(context).tls_accessible() ==> result.scheduler.is_none(),
        old(context).tls_accessible() ==>
            result.access == ContextTlsAccess::ClosureOnce
                && final(context).tls_closure_runs() == 1,
        !old(context).tls_accessible() ==>
            result.access == ContextTlsAccess::ClosureZeroTeardown
                && final(context).tls_closure_runs() == 0,
    no_unwind
{
    let session = context.tls.try_with_begin(0);
    assert(session.original() == old(context).tls.state());
    let access = session.get();
    let access_kind = if access.is_some() {
        TryAccess::ClosureOnce
    } else {
        TryAccess::ClosureZeroTeardown
    };
    let total = TotalCallback::from_access(access_kind);
    let runs = total.runs();
    context.tls_closure_runs = runs.0;
    context.callback_runs = runs.1;
    if access.is_some() {
        let scheduler = match context.entry {
            RuntimeEntry::Entered => context.scoped.get_current(),
            RuntimeEntry::NotEntered => None,
        };
        context.tls.try_with_end(session);
        WithSchedulerResult { scheduler, access: TryAccess::ClosureOnce }
    } else {
        context.tls.try_with_end(session);
        WithSchedulerResult {
            scheduler: None,
            access: TryAccess::ClosureZeroTeardown,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DeferOutcome {
    Deduplicated,
    Queued,
    ClonePanicked,
    AllocationPanicked,
}

pub struct DeferQueue {
    queued: Ghost<Seq<u64>>,
    handed_to_waker: Ghost<Seq<u64>>,
    released_on_drop: Ghost<Seq<u64>>,
}

impl DeferQueue {
    pub closed spec fn queued(&self) -> Seq<u64> { self.queued@ }
    pub closed spec fn handed_to_waker(&self) -> Seq<u64> { self.handed_to_waker@ }
    pub closed spec fn released_on_drop(&self) -> Seq<u64> { self.released_on_drop@ }

    pub fn new() -> (result: Self)
        ensures result.queued().len() == 0, result.handed_to_waker().len() == 0,
            result.released_on_drop().len() == 0,
        no_unwind
    {
        DeferQueue {
            queued: Ghost(Seq::empty()),
            handed_to_waker: Ghost(Seq::empty()),
            released_on_drop: Ghost(Seq::empty()),
        }
    }

    /// Body refinement of Defer::defer. `will_wake` identity is represented by
    /// task id. Clone/allocation panic occurs before Vec publishes a new entry,
    /// so the queue and its ownership remain unchanged.
    pub fn defer(&mut self, task: u64, outcome: DeferOutcome)
        requires outcome == DeferOutcome::Deduplicated ==>
                old(self).queued().len() > 0 && old(self).queued().last() == task,
            outcome == DeferOutcome::Queued ==>
                (old(self).queued().len() == 0 || old(self).queued().last() != task),
            outcome == DeferOutcome::ClonePanicked ==>
                (old(self).queued().len() == 0 || old(self).queued().last() != task),
            outcome == DeferOutcome::AllocationPanicked ==>
                (old(self).queued().len() == 0 || old(self).queued().last() != task),
        ensures outcome == DeferOutcome::Queued ==>
                final(self).queued() == old(self).queued().push(task),
            outcome != DeferOutcome::Queued ==>
                final(self).queued() == old(self).queued(),
            final(self).handed_to_waker() == old(self).handed_to_waker(),
            final(self).released_on_drop() == old(self).released_on_drop(),
        no_unwind
    {
        match outcome {
            DeferOutcome::Queued => self.queued = Ghost(self.queued@.push(task)),
            DeferOutcome::Deduplicated
            | DeferOutcome::ClonePanicked
            | DeferOutcome::AllocationPanicked => {},
        }
    }

    /// `Defer::wake` pops before invoking arbitrary Waker code. Thus normal
    /// return and Waker panic both transfer exactly the popped ownership out of
    /// the queue, with no duplicate or loss inside Defer.
    pub fn pop_and_handoff(&mut self, task: u64)
        requires old(self).queued().len() > 0, old(self).queued().last() == task,
        ensures
            final(self).queued() == old(self).queued().drop_last(),
            final(self).handed_to_waker() == old(self).handed_to_waker().push(task),
            final(self).released_on_drop() == old(self).released_on_drop(),
        no_unwind
    {
        self.queued = Ghost(self.queued@.drop_last());
        self.handed_to_waker = Ghost(self.handed_to_waker@.push(task));
    }

    /// Destruction releases every still-queued Waker token exactly once. The
    /// arbitrary destructor bodies themselves remain in the frozen boundary.
    pub fn drop_remaining(&mut self)
        ensures final(self).queued().len() == 0,
            final(self).handed_to_waker() == old(self).handed_to_waker(),
            final(self).released_on_drop()
                == old(self).released_on_drop().add(old(self).queued()),
        no_unwind
    {
        self.released_on_drop = Ghost(self.released_on_drop@.add(self.queued@));
        self.queued = Ghost(Seq::empty());
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RegisterRoute {
    NoRtImmediate { waker_panicked: bool },
    RtImmediate { waker_panicked: bool },
    RtDeferred(DeferOutcome),
}

pub struct RegisterWakerState {
    immediate_wake_calls: Ghost<Seq<u64>>,
}

impl RegisterWakerState {
    pub closed spec fn immediate_wake_calls(&self) -> Seq<u64> { self.immediate_wake_calls@ }

    pub fn new() -> (result: Self)
        ensures result.immediate_wake_calls().len() == 0,
        no_unwind
    {
        RegisterWakerState { immediate_wake_calls: Ghost(Seq::empty()) }
    }

    /// Connection for cfg(no-rt), rt outside a scheduler, and rt with the
    /// selected scheduler. Immediate `wake_by_ref` and eventual scheduling are
    /// within the agreed Waker/scheduler-liveness foundation; queue ownership is
    /// body-proved above.
    pub fn register(
        &mut self,
        queue: &mut DeferQueue,
        task: u64,
        route: RegisterRoute,
    )
        requires match route {
            RegisterRoute::RtDeferred(outcome) => {
                &&& outcome == DeferOutcome::Deduplicated ==>
                    old(queue).queued().len() > 0 && old(queue).queued().last() == task
                &&& outcome != DeferOutcome::Deduplicated ==>
                    (old(queue).queued().len() == 0 || old(queue).queued().last() != task)
            },
            _ => true,
        },
        ensures final(queue).handed_to_waker() == old(queue).handed_to_waker(),
        final(queue).released_on_drop() == old(queue).released_on_drop(),
        match route {
            RegisterRoute::NoRtImmediate { .. } | RegisterRoute::RtImmediate { .. } => {
                &&& final(self).immediate_wake_calls()
                    == old(self).immediate_wake_calls().push(task)
                &&& final(queue).queued() == old(queue).queued()
            },
            RegisterRoute::RtDeferred(DeferOutcome::Queued) => {
                &&& final(self).immediate_wake_calls() == old(self).immediate_wake_calls()
                &&& final(queue).queued() == old(queue).queued().push(task)
            },
            RegisterRoute::RtDeferred(_) => {
                &&& final(self).immediate_wake_calls() == old(self).immediate_wake_calls()
                &&& final(queue).queued() == old(queue).queued()
            },
        },
    no_unwind
    {
        match route {
            RegisterRoute::NoRtImmediate { .. } | RegisterRoute::RtImmediate { .. } => {
                self.immediate_wake_calls = Ghost(self.immediate_wake_calls@.push(task));
            },
            RegisterRoute::RtDeferred(outcome) => queue.defer(task, outcome),
        }
    }
}

pub open spec fn scheduler_is_selected(context: &SchedulerContextAccess) -> bool {
    context.tls_accessible()
        && context.entry() == RuntimeEntry::Entered
        && context.current().is_some()
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ForcedPendingResult {
    Registered,
    Panicked,
}

/// End-to-end zero-budget orchestration for production poll_proceed and
/// register_waker. The channel core is never polled on this branch. Route is
/// derived from cfg(rt) plus the nonescaping with_scheduler callback result,
/// rather than supplied as an oracle.
pub fn register_after_needs(
    context: &mut SchedulerContextAccess,
    registration: &mut RegisterWakerState,
    queue: &mut DeferQueue,
    task: u64,
    rt_enabled: bool,
    immediate_waker_panics: bool,
    deferred_outcome: DeferOutcome,
) -> (result: ForcedPendingResult)
    requires old(context).well_formed(),
        rt_enabled && scheduler_is_selected(old(context)) ==>
            (deferred_outcome == DeferOutcome::Deduplicated ==>
                old(queue).queued().len() > 0 && old(queue).queued().last() == task),
        rt_enabled && scheduler_is_selected(old(context)) ==>
            (deferred_outcome != DeferOutcome::Deduplicated ==>
                (old(queue).queued().len() == 0 || old(queue).queued().last() != task)),
    ensures final(context).well_formed(),
        final(queue).handed_to_waker() == old(queue).handed_to_waker(),
        final(queue).released_on_drop() == old(queue).released_on_drop(),
        !rt_enabled || !scheduler_is_selected(old(context)) ==>
            final(queue).queued() == old(queue).queued()
                && final(registration).immediate_wake_calls()
                    == old(registration).immediate_wake_calls().push(task),
        rt_enabled && scheduler_is_selected(old(context)) ==>
            final(registration).immediate_wake_calls()
                == old(registration).immediate_wake_calls(),
        rt_enabled && scheduler_is_selected(old(context))
                && deferred_outcome == DeferOutcome::Queued ==>
            final(queue).queued() == old(queue).queued().push(task),
        rt_enabled && scheduler_is_selected(old(context))
                && deferred_outcome != DeferOutcome::Queued ==>
            final(queue).queued() == old(queue).queued(),
        (result == ForcedPendingResult::Panicked) ==
            ((!rt_enabled || !scheduler_is_selected(old(context)))
                && immediate_waker_panics
             || rt_enabled && scheduler_is_selected(old(context))
                && matches!(deferred_outcome,
                    DeferOutcome::ClonePanicked | DeferOutcome::AllocationPanicked)),
    no_unwind
{
    if !rt_enabled {
        registration.register(
            queue,
            task,
            RegisterRoute::NoRtImmediate { waker_panicked: immediate_waker_panics },
        );
        if immediate_waker_panics {
            ForcedPendingResult::Panicked
        } else {
            ForcedPendingResult::Registered
        }
    } else {
        let selected = with_scheduler(context);
        match selected.scheduler {
            None => {
                registration.register(
                    queue,
                    task,
                    RegisterRoute::RtImmediate { waker_panicked: immediate_waker_panics },
                );
                if immediate_waker_panics {
                    ForcedPendingResult::Panicked
                } else {
                    ForcedPendingResult::Registered
                }
            },
            Some(_) => {
                registration.register(
                    queue,
                    task,
                    RegisterRoute::RtDeferred(deferred_outcome),
                );
                if matches!(deferred_outcome,
                        DeferOutcome::ClonePanicked | DeferOutcome::AllocationPanicked) {
                    ForcedPendingResult::Panicked
                } else {
                    ForcedPendingResult::Registered
                }
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BroadcastContextPoll {
    Public(BroadcastPublicPoll),
    PanickedDuringRegistration,
}

/// Top-level standard-S04 recv composition. Unlike the lower-level TLS poll,
/// this wrapper discharges NeedsRegistration on the forced-yield branch before
/// publishing Pending; registration panic exits without polling the channel.
pub struct RegisteredBroadcastPoll {
    inner: BroadcastTlsPoll,
}

impl RegisteredBroadcastPoll {
    pub closed spec fn inner_polled(&self) -> bool { self.inner.inner_polled() }

    pub fn new() -> (result: Self)
        ensures !result.inner_polled(),
        no_unwind
    {
        RegisteredBroadcastPoll { inner: BroadcastTlsPoll::new() }
    }

    pub fn poll(
        &mut self,
        cell: &mut ThreadLocalBudgetCell,
        context: &mut SchedulerContextAccess,
        registration: &mut RegisterWakerState,
        queue: &mut DeferQueue,
        task: u64,
        rt_enabled: bool,
        immediate_waker_panics: bool,
        deferred_outcome: DeferOutcome,
        core: BroadcastCorePoll,
    ) -> (result: BroadcastContextPoll)
        requires old(cell).well_formed(), old(cell).task_polls_allowed(),
            old(context).well_formed(),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0)))
                && rt_enabled && scheduler_is_selected(old(context)) ==>
                    (deferred_outcome == DeferOutcome::Deduplicated ==>
                        old(queue).queued().len() > 0
                            && old(queue).queued().last() == task),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0)))
                && rt_enabled && scheduler_is_selected(old(context)) ==>
                    (deferred_outcome != DeferOutcome::Deduplicated ==>
                        (old(queue).queued().len() == 0
                            || old(queue).queued().last() != task)),
        ensures final(cell).well_formed(),
            final(cell).task_polls_allowed() == old(cell).task_polls_allowed(),
            final(context).well_formed(),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                    !final(self).inner_polled()
                        && final(cell).state() == old(cell).state()
                        && (result == BroadcastContextPoll::PanickedDuringRegistration
                            || result == BroadcastContextPoll::Public(
                                BroadcastPublicPoll::Pending)),
            !matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                    final(self).inner_polled()
                        && result == BroadcastContextPoll::Public(public_of(core)),
            match old(cell).state() {
                BudgetTlsState::Accessible(value) => value != BudgetValue::Constrained(0) ==>
                    if matches!(core, BroadcastCorePoll::Pending | BroadcastCorePoll::Empty) {
                        final(cell).state() == BudgetTlsState::Accessible(value)
                    } else {
                        final(cell).state()
                            == BudgetTlsState::Accessible(decremented_budget(value))
                    },
                BudgetTlsState::TornDown | BudgetTlsState::Borrowed => true,
            },
            old(cell).state() == BudgetTlsState::TornDown ==>
                final(cell).state() == BudgetTlsState::TornDown,
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                    final(queue).handed_to_waker() == old(queue).handed_to_waker()
                        && final(queue).released_on_drop() == old(queue).released_on_drop(),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0)))
                    && (!rt_enabled || !scheduler_is_selected(old(context))) ==>
                        final(registration).immediate_wake_calls()
                            == old(registration).immediate_wake_calls().push(task)
                        && final(queue).queued() == old(queue).queued(),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0)))
                    && rt_enabled && scheduler_is_selected(old(context)) ==>
                        final(registration).immediate_wake_calls()
                            == old(registration).immediate_wake_calls(),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0)))
                    && rt_enabled && scheduler_is_selected(old(context))
                    && deferred_outcome == DeferOutcome::Queued ==>
                        final(queue).queued() == old(queue).queued().push(task),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0)))
                    && rt_enabled && scheduler_is_selected(old(context))
                    && deferred_outcome != DeferOutcome::Queued ==>
                        final(queue).queued() == old(queue).queued(),
            matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                (result == BroadcastContextPoll::PanickedDuringRegistration) ==
                    (((!rt_enabled || !scheduler_is_selected(old(context)))
                        && immediate_waker_panics)
                    || (rt_enabled && scheduler_is_selected(old(context))
                        && matches!(deferred_outcome,
                            DeferOutcome::ClonePanicked
                                | DeferOutcome::AllocationPanicked))),
            !matches!(old(cell).state(),
                BudgetTlsState::Accessible(BudgetValue::Constrained(0))) ==>
                    final(registration).immediate_wake_calls()
                        == old(registration).immediate_wake_calls()
                    && final(queue).queued() == old(queue).queued()
                    && final(queue).handed_to_waker() == old(queue).handed_to_waker()
                    && final(queue).released_on_drop() == old(queue).released_on_drop()
                    && final(context).tls_accessible() == old(context).tls_accessible()
                    && final(context).entry() == old(context).entry()
                    && final(context).scoped() == old(context).scoped()
                    && final(context).current() == old(context).current()
                    && final(context).tls_closure_runs() == old(context).tls_closure_runs()
                    && final(context).callback_runs() == old(context).callback_runs(),
        no_unwind
    {
        let proceed = tls_poll_proceed(cell);
        match proceed {
            TlsProceed::NeedsRegistration => {
                self.inner.reset_not_polled();
                let registered = register_after_needs(
                context,
                registration,
                queue,
                task,
                rt_enabled,
                immediate_waker_panics,
                deferred_outcome,
                );
                match registered {
                    ForcedPendingResult::Registered =>
                        BroadcastContextPoll::Public(BroadcastPublicPoll::Pending),
                    ForcedPendingResult::Panicked =>
                        BroadcastContextPoll::PanickedDuringRegistration,
                }
            },
            TlsProceed::Ready(restore) => {
                let remembered = restore.remembered;
                let result = self.inner.poll_ready(cell, restore, remembered, core);
                BroadcastContextPoll::Public(result)
            },
        }
    }
}

pub fn verify_scheduler_scope_lifo_and_defer_ownership()
{
    let mut context = SchedulerContextAccess::new();
    context.enter_runtime();
    let outer = scheduler_scope_enter(&mut context, 10);
    let inner = scheduler_scope_enter(&mut context, 20);
    assert(context.scoped() == seq![10u64, 20u64]);
    assert(context.scoped().drop_last() == inner.previous());
    let selected = with_scheduler(&mut context);
    assert(selected.scheduler == Some(20));
    scheduler_scope_drop(&mut context, inner);
    assert(context.scoped().last() == 10);

    let mut queue = DeferQueue::new();
    let mut registration = RegisterWakerState::new();
    registration.register(
        &mut queue,
        7,
        RegisterRoute::RtDeferred(DeferOutcome::Queued),
    );
    registration.register(
        &mut queue,
        7,
        RegisterRoute::RtDeferred(DeferOutcome::Deduplicated),
    );
    assert(queue.queued().len() == 1);
    queue.pop_and_handoff(7);
    assert(queue.queued().len() == 0);
    assert(queue.handed_to_waker().len() == 1);
    assert(queue.handed_to_waker()[0] == 7);

    registration.register(
        &mut queue,
        8,
        RegisterRoute::RtDeferred(DeferOutcome::Queued),
    );
    queue.drop_remaining();
    assert(queue.queued().len() == 0);
    assert(queue.released_on_drop().len() == 1);
    assert(queue.released_on_drop()[0] == 8);

    assert(context.scoped().drop_last() == outer.previous());
    scheduler_scope_drop(&mut context, outer);
    assert(context.scoped().len() == 0);
}

} // verus!
