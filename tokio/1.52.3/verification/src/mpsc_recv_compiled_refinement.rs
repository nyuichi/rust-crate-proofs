use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GateResult { Pending, Passed }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GatePhase { Trace, Coop, Body, ReturnedPending }

/// Snapshot carried across production `trace_leaf` and `poll_proceed`. Those
/// gates may return Pending and arrange a wake, but they do not own mpsc raw
/// queue, receiver buffer, or semaphore state. This makes that full frame
/// explicit rather than preserving only the Waker/progress bookkeeping.
pub struct MpscRecvGateSnapshot<T> {
    phase: GatePhase,
    queue_head: nat,
    queue_tail: nat,
    buffer: Ghost<Seq<T>>,
    available: usize,
    queued: usize,
    waker: u64,
}

impl<T> MpscRecvGateSnapshot<T> {
    pub closed spec fn phase(&self) -> GatePhase { self.phase }
    pub closed spec fn queue_head(&self) -> nat { self.queue_head }
    pub closed spec fn queue_tail(&self) -> nat { self.queue_tail }
    pub closed spec fn buffer(&self) -> Seq<T> { self.buffer@ }
    pub closed spec fn available(&self) -> usize { self.available }
    pub closed spec fn queued(&self) -> usize { self.queued }
    pub closed spec fn waker(&self) -> u64 { self.waker }
    pub closed spec fn state_frame(&self) -> (nat, nat, Seq<T>, usize, usize) {
        (self.queue_head(), self.queue_tail(), self.buffer(),
            self.available(), self.queued())
    }
    pub closed spec fn well_formed(&self) -> bool {
        self.queue_head() <= self.queue_tail()
    }

    pub fn new(
        queue_head: nat,
        queue_tail: nat,
        Ghost(buffer): Ghost<Seq<T>>,
        available: usize,
        queued: usize,
        waker: u64,
    ) -> (result: Self)
        requires queue_head <= queue_tail,
        ensures result.well_formed(), result.phase() == GatePhase::Trace,
            result.state_frame() == (queue_head, queue_tail, buffer, available, queued),
            result.waker() == waker,
        no_unwind
    {
        MpscRecvGateSnapshot {
            phase: GatePhase::Trace, queue_head, queue_tail,
            buffer: Ghost(buffer), available, queued, waker,
        }
    }

    pub fn trace(&mut self, result: GateResult) -> (passed: bool)
        requires old(self).well_formed(), old(self).phase() == GatePhase::Trace,
        ensures final(self).well_formed(),
            final(self).state_frame() == old(self).state_frame(),
            final(self).waker() == old(self).waker(),
            passed == (result == GateResult::Passed),
            passed ==> final(self).phase() == GatePhase::Coop,
            !passed ==> final(self).phase() == GatePhase::ReturnedPending,
        no_unwind
    {
        match result {
            GateResult::Passed => {
                self.phase = GatePhase::Coop;
                true
            },
            GateResult::Pending => {
                self.phase = GatePhase::ReturnedPending;
                false
            },
        }
    }

    pub fn coop(&mut self, result: GateResult) -> (passed: bool)
        requires old(self).well_formed(), old(self).phase() == GatePhase::Coop,
        ensures final(self).well_formed(),
            final(self).state_frame() == old(self).state_frame(),
            final(self).waker() == old(self).waker(),
            passed == (result == GateResult::Passed),
            passed ==> final(self).phase() == GatePhase::Body,
            !passed ==> final(self).phase() == GatePhase::ReturnedPending,
        no_unwind
    {
        match result {
            GateResult::Passed => {
                self.phase = GatePhase::Body;
                true
            },
            GateResult::Pending => {
                self.phase = GatePhase::ReturnedPending;
                false
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CompiledManyPhase { Receiving, HavePopped, Returned, Unwound }

/// Exact channel-specific state held by production `RecvManyGuard` around
/// `list.pop`, `record_pop`, `Vec::push`, normal bulk release, and guard Drop.
/// Vec allocation and arbitrary `T::drop` remain frozen execution boundaries;
/// both possible control-flow outcomes are proved here without requiring spare
/// Vec capacity.
pub struct CompiledRecvMany<T> {
    limit: usize,
    initial_prefix: Ghost<Seq<T>>,
    appended: Ghost<Seq<T>>,
    current: Ghost<Option<T>>,
    pending: usize,
    accounting_returned: usize,
    phase: CompiledManyPhase,
}

impl<T> CompiledRecvMany<T> {
    pub closed spec fn limit(&self) -> usize { self.limit }
    pub closed spec fn initial_prefix(&self) -> Seq<T> { self.initial_prefix@ }
    pub closed spec fn appended(&self) -> Seq<T> { self.appended@ }
    pub closed spec fn current(&self) -> Option<T> { self.current@ }
    pub closed spec fn pending(&self) -> usize { self.pending }
    pub closed spec fn accounting_returned(&self) -> usize {
        self.accounting_returned
    }
    pub closed spec fn phase(&self) -> CompiledManyPhase { self.phase }
    pub open spec fn buffer(&self) -> Seq<T> {
        self.initial_prefix().add(self.appended())
    }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.initial_prefix().len() <= usize::MAX
        &&& self.appended().len() <= self.limit()
        &&& (self.phase() == CompiledManyPhase::HavePopped
            <==> self.current().is_some())
        &&& (self.phase() == CompiledManyPhase::Receiving
            ==> self.current().is_none() && self.pending() < self.limit()
                && self.pending() as nat == self.appended().len()
                && self.accounting_returned() == 0)
        &&& (self.phase() == CompiledManyPhase::HavePopped
            ==> self.pending() as nat == self.appended().len() + 1
                && self.pending() <= self.limit()
                && self.accounting_returned() == 0)
        &&& (self.phase() == CompiledManyPhase::Returned
            ==> self.current().is_none() && self.pending() == 0
                && self.accounting_returned() as nat == self.appended().len())
        &&& (self.phase() == CompiledManyPhase::Unwound
            ==> self.current().is_none() && self.pending() == 0
                && self.accounting_returned() as nat == self.appended().len() + 1)
    }

    pub fn new(limit: usize, Ghost(prefix): Ghost<Seq<T>>) -> (result: Self)
        requires prefix.len() <= usize::MAX, limit > 0,
        ensures result.well_formed(), result.limit() == limit,
            result.initial_prefix() == prefix, result.appended().len() == 0,
            result.buffer() == prefix, result.pending() == 0,
            result.accounting_returned() == 0,
            result.phase() == CompiledManyPhase::Receiving,
        no_unwind
    {
        CompiledRecvMany {
            limit, initial_prefix: Ghost(prefix), appended: Ghost(Seq::empty()),
            current: Ghost(None), pending: 0, accounting_returned: 0,
            phase: CompiledManyPhase::Receiving,
        }
    }

    /// `list.pop` has moved a value from the raw slot and `record_pop` runs
    /// before `Vec::push`. The guard now owns exactly one additional unit of
    /// accounting even if the subsequent push unwinds.
    pub fn record_raw_pop(&mut self, Ghost(value): Ghost<T>)
        requires old(self).well_formed(),
            old(self).phase() == CompiledManyPhase::Receiving,
        ensures final(self).well_formed(),
            final(self).limit() == old(self).limit(),
            final(self).phase() == CompiledManyPhase::HavePopped,
            final(self).current() == Some(value),
            final(self).pending() == old(self).pending() + 1,
            final(self).buffer() == old(self).buffer(),
            final(self).appended() == old(self).appended(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).accounting_returned() == old(self).accounting_returned(),
        no_unwind
    {
        self.current = Ghost(Some(value));
        self.pending += 1;
        self.phase = CompiledManyPhase::HavePopped;
    }

    /// Successful compiled `Vec::push`: the immutable caller prefix is
    /// retained, regardless of whether Vec had to allocate and move storage.
    pub fn push_succeeded(&mut self)
        requires old(self).well_formed(),
            old(self).phase() == CompiledManyPhase::HavePopped,
        ensures final(self).well_formed(),
            final(self).limit() == old(self).limit(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).appended() == old(self).appended().push(old(self).current().unwrap()),
            final(self).buffer() == old(self).buffer().push(old(self).current().unwrap()),
            final(self).current().is_none(),
            old(self).pending() < old(self).limit() ==>
                final(self).pending() == old(self).pending()
                    && final(self).accounting_returned() == 0
                    && final(self).phase() == CompiledManyPhase::Receiving,
            old(self).pending() == old(self).limit() ==>
                final(self).phase() == CompiledManyPhase::Returned
                    && final(self).accounting_returned() == old(self).pending()
                    && final(self).pending() == 0,
        no_unwind
    {
        let ghost value = self.current@.unwrap();
        self.appended = Ghost(self.appended@.push(value));
        self.current = Ghost(None);
        if self.pending == self.limit {
            self.accounting_returned = self.pending;
            self.pending = 0;
            self.phase = CompiledManyPhase::Returned;
        } else {
            self.phase = CompiledManyPhase::Receiving;
        }
    }

    /// `Vec::push` unwound. Stack unwinding drops the current value and then
    /// `RecvManyGuard::drop` bulk-returns the exact pending count once. The
    /// caller prefix and all earlier successful appends are unchanged.
    pub fn push_unwound_and_guard_dropped(&mut self) -> (returned: usize)
        requires old(self).well_formed(),
            old(self).phase() == CompiledManyPhase::HavePopped,
        ensures final(self).well_formed(), returned == old(self).pending(),
            final(self).phase() == CompiledManyPhase::Unwound,
            final(self).pending() == 0,
            final(self).accounting_returned() == returned,
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).appended() == old(self).appended(),
            final(self).buffer() == old(self).buffer(),
            final(self).current().is_none(),
        no_unwind
    {
        let returned = self.pending;
        self.current = Ghost(None);
        self.pending = 0;
        self.accounting_returned = returned;
        self.phase = CompiledManyPhase::Unwound;
        returned
    }

    pub fn finish_short_batch(&mut self) -> (returned: usize)
        requires old(self).well_formed(),
            old(self).phase() == CompiledManyPhase::Receiving,
            old(self).pending() > 0,
        ensures final(self).well_formed(), returned == old(self).pending(),
            final(self).phase() == CompiledManyPhase::Returned,
            final(self).pending() == 0,
            final(self).accounting_returned() == returned,
            final(self).buffer() == old(self).buffer(),
            final(self).initial_prefix() == old(self).initial_prefix(),
        no_unwind
    {
        let returned = self.pending;
        self.pending = 0;
        self.accounting_returned = returned;
        self.phase = CompiledManyPhase::Returned;
        returned
    }
}

pub fn verify_gates_preserve_full_receive_snapshot(
    head: nat, tail: nat, available: usize, queued: usize, waker: u64,
    Ghost(buffer): Ghost<Seq<u64>>,
)
    requires head <= tail,
{
    let mut gates: MpscRecvGateSnapshot<u64> = MpscRecvGateSnapshot::new(
        head, tail, Ghost(buffer), available, queued, waker);
    let ghost before = gates.state_frame();
    if gates.trace(GateResult::Passed) {
        gates.coop(GateResult::Passed);
    }
    assert(gates.state_frame() == before);
}

pub fn verify_recv_many_growth_success_without_spare_capacity(
    value: u64, Ghost(prefix): Ghost<Seq<u64>>,
)
    requires prefix.len() <= usize::MAX,
{
    let mut many = CompiledRecvMany::new(2, Ghost(prefix));
    many.record_raw_pop(Ghost(value));
    assert(many.pending() == 1);
    assert(many.pending() < many.limit());
    many.push_succeeded();
    assert(many.buffer() == prefix.push(value));
    assert(many.phase() == CompiledManyPhase::Receiving);
}

pub fn verify_recv_many_growth_unwind_returns_exactly_once(
    first: u64, second: u64, Ghost(prefix): Ghost<Seq<u64>>,
)
    requires prefix.len() <= usize::MAX,
{
    let mut many = CompiledRecvMany::new(3, Ghost(prefix));
    many.record_raw_pop(Ghost(first));
    assert(many.pending() == 1);
    assert(many.pending() < many.limit());
    many.push_succeeded();
    assert(many.phase() == CompiledManyPhase::Receiving);
    many.record_raw_pop(Ghost(second));
    let returned = many.push_unwound_and_guard_dropped();
    assert(returned == 2);
    assert(many.buffer() == prefix.push(first));
    assert(many.accounting_returned() == 2);
}

} // verus!
