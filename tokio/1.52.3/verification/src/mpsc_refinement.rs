use crate::mpsc_protocol::{MpscCapacity, QueueRead, ReserveResult};
#[cfg(verus_keep_ghost)]
use vstd::arithmetic::power::pow;
#[cfg(verus_keep_ghost)]
use vstd::arithmetic::power2::pow2;
#[cfg(verus_keep_ghost)]
use vstd::bits::lemma_usize_shl_is_mul;
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

/// Modeled bookkeeping visible around `trace_leaf` and `poll_proceed`. If
/// either gate returns Pending, this projection is unchanged. The raw queue
/// and semaphore are deliberately outside this type and are not proved here.
pub struct PollSurfaceState {
    waker: Option<u64>,
    permits_returned: usize,
    made_progress: bool,
}

/// Exact arithmetic projection of the unbounded-channel semaphore word:
/// `(message_count << 1) | receiver_closed`.
pub struct UnboundedAccounting {
    encoded: usize,
    messages: usize,
    receiver_closed: bool,
}

impl UnboundedAccounting {
    pub closed spec fn encoded(&self) -> usize { self.encoded }
    pub closed spec fn messages(&self) -> usize { self.messages }
    pub closed spec fn receiver_closed(&self) -> bool { self.receiver_closed }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.messages() <= usize::MAX / 2
        &&& (!self.receiver_closed() ==>
            self.encoded() == self.messages() * 2)
        &&& (self.receiver_closed() ==>
            self.encoded() == self.messages() * 2 + 1)
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.encoded() == 0,
            result.messages() == 0, !result.receiver_closed(),
        no_unwind
    {
        UnboundedAccounting { encoded: 0, messages: 0, receiver_closed: false }
    }

    pub fn publish(&mut self)
        requires old(self).well_formed(), !old(self).receiver_closed(),
            old(self).messages() < usize::MAX / 2,
        ensures final(self).well_formed(),
            final(self).encoded() == old(self).encoded() + 2,
            final(self).messages() == old(self).messages() + 1,
            !final(self).receiver_closed(),
        no_unwind
    {
        self.encoded += 2;
        self.messages += 1;
    }

    pub fn close_receiver(&mut self)
        requires old(self).well_formed(),
        ensures final(self).well_formed(), final(self).receiver_closed(),
            final(self).messages() == old(self).messages(),
            old(self).receiver_closed() ==>
                final(self).encoded() == old(self).encoded(),
            !old(self).receiver_closed() ==>
                final(self).encoded() == old(self).encoded() + 1,
        no_unwind
    {
        if !self.receiver_closed {
            self.encoded += 1;
            self.receiver_closed = true;
        }
    }

    /// Models `unbounded::Semaphore::add_permit` after a Value pop. The
    /// positive-count precondition is the production abort guard.
    pub fn receive_value(&mut self)
        requires old(self).well_formed(), old(self).messages() > 0,
        ensures final(self).well_formed(),
            final(self).encoded() + 2 == old(self).encoded(),
            final(self).messages() + 1 == old(self).messages(),
            final(self).receiver_closed() == old(self).receiver_closed(),
        no_unwind
    {
        self.encoded -= 2;
        self.messages -= 1;
    }

    /// Models one production unbounded `fetch_sub(number_added << 1)` after a
    /// preallocated `recv_many` batch. The count bound is also the production
    /// abort guard and makes the shift/subtraction exact.
    pub fn receive_many(&mut self, count: usize)
        requires old(self).well_formed(), count > 0,
            count <= old(self).messages(),
            count * 2 <= usize::MAX,
        ensures final(self).well_formed(),
            count << 1 == count * 2,
            final(self).encoded() + (count << 1) == old(self).encoded(),
            final(self).encoded() + count * 2 == old(self).encoded(),
            final(self).messages() + count == old(self).messages(),
            final(self).receiver_closed() == old(self).receiver_closed(),
        no_unwind
    {
        let delta = count << 1;
        proof {
            reveal(pow);
            assert(pow2(1) == 2) by (compute_only);
            lemma_usize_shl_is_mul(count, 1);
            assert(delta == count * 2);
        }
        self.encoded -= delta;
        self.messages -= count;
    }

    pub fn finish_without_value(&mut self) -> (ready_closed: bool)
        requires old(self).well_formed(),
        ensures final(self).well_formed(), *final(self) == *old(self),
            ready_closed == (old(self).receiver_closed() && old(self).messages() == 0),
        no_unwind
    {
        self.receiver_closed && self.messages == 0
    }
}

impl PollSurfaceState {
    pub closed spec fn waker(&self) -> Option<u64> { self.waker }
    pub closed spec fn permits_returned(&self) -> usize { self.permits_returned }
    pub closed spec fn made_progress(&self) -> bool { self.made_progress }
    pub fn new(waker: Option<u64>) -> (result: Self)
        ensures result.waker() == waker, result.permits_returned() == 0,
            !result.made_progress(),
        no_unwind
    {
        PollSurfaceState { waker, permits_returned: 0, made_progress: false }
    }

    pub fn pass_trace_and_coop(&mut self, trace_ready: bool, coop_ready: bool)
        -> (passed: bool)
        ensures passed == (trace_ready && coop_ready),
            final(self).waker() == old(self).waker(),
            final(self).permits_returned() == old(self).permits_returned(),
            final(self).made_progress() == old(self).made_progress(),
        no_unwind
    {
        trace_ready && coop_ready
    }
}

/// Stable facts read by `Chan::recv` after a list pop. TX_CLOSED can reach the
/// list head only after all preceding sends and outstanding permits are gone.
pub struct RecvEnvironment {
    semaphore_idle: bool,
    receiver_closed: bool,
    tx_closed_at_head: bool,
}

impl RecvEnvironment {
    pub closed spec fn semaphore_idle(&self) -> bool { self.semaphore_idle }
    pub closed spec fn receiver_closed(&self) -> bool { self.receiver_closed }
    pub closed spec fn tx_closed_at_head(&self) -> bool { self.tx_closed_at_head }
    pub closed spec fn well_formed(&self) -> bool {
        self.tx_closed_at_head() ==> self.semaphore_idle()
    }

    pub fn new(semaphore_idle: bool, receiver_closed: bool, tx_closed_at_head: bool)
        -> (result: Self)
        requires tx_closed_at_head ==> semaphore_idle,
        ensures result.well_formed(),
            result.semaphore_idle() == semaphore_idle,
            result.receiver_closed() == receiver_closed,
            result.tx_closed_at_head() == tx_closed_at_head,
        no_unwind
    {
        RecvEnvironment { semaphore_idle, receiver_closed, tx_closed_at_head }
    }

    pub fn receiver_closed_and_idle(&self) -> (result: bool)
        requires self.well_formed(),
        ensures result == (self.receiver_closed() && self.semaphore_idle()),
        no_unwind
    {
        self.receiver_closed && self.semaphore_idle
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RecvPhase { FirstPop, NeedRegister, Recheck, Done }

#[derive(PartialEq, Eq)]
pub enum RecvPoll<T> { ReadyValue(T), ReadyClosed, NeedRegister, Pending }

pub open spec fn first_pop_corresponds<T>(
    read: QueueRead<T>,
    result: RecvPoll<T>,
) -> bool {
    match (read, result) {
        (QueueRead::Value(input), RecvPoll::ReadyValue(output)) => input == output,
        (QueueRead::Closed, RecvPoll::ReadyClosed) => true,
        (QueueRead::Empty, RecvPoll::NeedRegister) => true,
        (QueueRead::Busy, RecvPoll::NeedRegister) => true,
        _ => false,
    }
}

pub open spec fn second_pop_corresponds<T>(
    read: QueueRead<T>,
    result: RecvPoll<T>,
    env: RecvEnvironment,
) -> bool {
    match (read, result) {
        (QueueRead::Value(input), RecvPoll::ReadyValue(output)) => input == output,
        (QueueRead::Closed, RecvPoll::ReadyClosed) => true,
        (QueueRead::Empty, RecvPoll::ReadyClosed) =>
            env.receiver_closed() && env.semaphore_idle(),
        (QueueRead::Busy, RecvPoll::ReadyClosed) =>
            env.receiver_closed() && env.semaphore_idle(),
        (QueueRead::Empty, RecvPoll::Pending) =>
            !(env.receiver_closed() && env.semaphore_idle()),
        (QueueRead::Busy, RecvPoll::Pending) =>
            !(env.receiver_closed() && env.semaphore_idle()),
        _ => false,
    }
}

/// Logical control projection matching `Chan::recv`'s branch order after the
/// trace/cooperative gates. Its QueueRead input is not a raw-list refinement.
/// QueueRead::Empty and QueueRead::Busy both represent production list::pop's
/// `None`; the distinction is retained only for mutation witnesses.
pub struct MpscRecvRefinement {
    phase: RecvPhase,
    waker: Option<u64>,
    permits_returned: usize,
    made_progress: bool,
}

impl MpscRecvRefinement {
    pub closed spec fn phase(&self) -> RecvPhase { self.phase }
    pub closed spec fn waker(&self) -> Option<u64> { self.waker }
    pub closed spec fn permits_returned(&self) -> usize { self.permits_returned }
    pub closed spec fn made_progress(&self) -> bool { self.made_progress }
    pub closed spec fn well_formed(&self) -> bool {
        &&& (self.phase() != RecvPhase::Done ==>
            self.permits_returned() == 0 && !self.made_progress())
        &&& (self.phase() == RecvPhase::Recheck ==> self.waker().is_some())
    }

    pub fn new(previous_waker: Option<u64>) -> (result: Self)
        ensures result.well_formed(), result.phase() == RecvPhase::FirstPop,
            result.waker() == previous_waker,
            result.permits_returned() == 0,
            !result.made_progress(),
        no_unwind
    {
        MpscRecvRefinement {
            phase: RecvPhase::FirstPop,
            waker: previous_waker,
            permits_returned: 0,
            made_progress: false,
        }
    }

    pub fn first_pop<T>(&mut self, read: QueueRead<T>, env: &RecvEnvironment)
        -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::FirstPop,
            env.well_formed(),
            read == QueueRead::Closed ==> env.tx_closed_at_head(),
        ensures final(self).well_formed(),
            final(self).phase() == if matches!(result, RecvPoll::NeedRegister) {
                RecvPhase::NeedRegister
            } else { RecvPhase::Done },
            first_pop_corresponds(read, result),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        match read {
            QueueRead::Value(value) => {
                self.phase = RecvPhase::Done;
                self.permits_returned = 1;
                self.made_progress = true;
                RecvPoll::ReadyValue(value)
            },
            QueueRead::Closed => {
                self.phase = RecvPhase::Done;
                self.made_progress = true;
                RecvPoll::ReadyClosed
            },
            QueueRead::Empty | QueueRead::Busy => {
                self.phase = RecvPhase::NeedRegister;
                RecvPoll::NeedRegister
            },
        }
    }

    pub fn first_value<T>(&mut self, value: T) -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::FirstPop,
        ensures final(self).well_formed(), result == RecvPoll::ReadyValue(value),
            final(self).phase() == RecvPhase::Done,
            final(self).permits_returned() == 1,
            final(self).made_progress(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.phase = RecvPhase::Done;
        self.permits_returned = 1;
        self.made_progress = true;
        RecvPoll::ReadyValue(value)
    }

    pub fn first_closed<T>(&mut self, env: &RecvEnvironment) -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::FirstPop,
            env.well_formed(), env.tx_closed_at_head(),
        ensures final(self).well_formed(), result == RecvPoll::ReadyClosed,
            final(self).phase() == RecvPhase::Done,
            final(self).permits_returned() == 0,
            final(self).made_progress(), env.semaphore_idle(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.phase = RecvPhase::Done;
        self.made_progress = true;
        RecvPoll::ReadyClosed
    }

    pub fn first_none<T>(&mut self, _busy: bool) -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::FirstPop,
        ensures final(self).well_formed(), result == RecvPoll::NeedRegister,
            final(self).phase() == RecvPhase::NeedRegister,
            final(self).permits_returned() == 0,
            !final(self).made_progress(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.phase = RecvPhase::NeedRegister;
        RecvPoll::NeedRegister
    }

    pub fn register_latest(&mut self, waker: u64)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::NeedRegister,
        ensures final(self).well_formed(), final(self).phase() == RecvPhase::Recheck,
            final(self).waker() == Some(waker),
            final(self).permits_returned() == old(self).permits_returned(),
            final(self).made_progress() == old(self).made_progress(),
        no_unwind
    {
        self.waker = Some(waker);
        self.phase = RecvPhase::Recheck;
    }

    pub fn second_pop<T>(&mut self, read: QueueRead<T>, env: &RecvEnvironment)
        -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::Recheck,
            env.well_formed(),
            read == QueueRead::Closed ==> env.tx_closed_at_head(),
        ensures final(self).well_formed(), final(self).phase() == RecvPhase::Done,
            final(self).waker() == old(self).waker(),
            second_pop_corresponds(read, result, *env),
        no_unwind
    {
        self.phase = RecvPhase::Done;
        match read {
            QueueRead::Value(value) => {
                self.permits_returned = 1;
                self.made_progress = true;
                RecvPoll::ReadyValue(value)
            },
            QueueRead::Closed => {
                self.made_progress = true;
                RecvPoll::ReadyClosed
            },
            QueueRead::Empty | QueueRead::Busy => {
                if env.receiver_closed && env.semaphore_idle {
                    self.made_progress = true;
                    RecvPoll::ReadyClosed
                } else {
                    RecvPoll::Pending
                }
            },
        }
    }

    pub fn second_value<T>(&mut self, value: T) -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::Recheck,
        ensures final(self).well_formed(), result == RecvPoll::ReadyValue(value),
            final(self).phase() == RecvPhase::Done,
            final(self).permits_returned() == 1,
            final(self).made_progress(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.phase = RecvPhase::Done;
        self.permits_returned = 1;
        self.made_progress = true;
        RecvPoll::ReadyValue(value)
    }

    pub fn second_closed<T>(&mut self, env: &RecvEnvironment) -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::Recheck,
            env.well_formed(), env.tx_closed_at_head(),
        ensures final(self).well_formed(), result == RecvPoll::ReadyClosed,
            final(self).phase() == RecvPhase::Done,
            final(self).permits_returned() == 0,
            final(self).made_progress(), env.semaphore_idle(),
            final(self).waker() == old(self).waker(),
        no_unwind
    {
        self.phase = RecvPhase::Done;
        self.made_progress = true;
        RecvPoll::ReadyClosed
    }

    pub fn second_none<T>(&mut self, env: &RecvEnvironment, _busy: bool)
        -> (result: RecvPoll<T>)
        requires old(self).well_formed(), old(self).phase() == RecvPhase::Recheck,
            env.well_formed(),
        ensures final(self).well_formed(), final(self).phase() == RecvPhase::Done,
            final(self).waker() == old(self).waker(),
            env.receiver_closed() && env.semaphore_idle() ==>
                result == RecvPoll::ReadyClosed && final(self).made_progress(),
            !(env.receiver_closed() && env.semaphore_idle()) ==>
                result == RecvPoll::Pending && !final(self).made_progress()
                    && final(self).waker().is_some(),
            final(self).permits_returned() == 0,
        no_unwind
    {
        self.phase = RecvPhase::Done;
        if env.receiver_closed && env.semaphore_idle {
            self.made_progress = true;
            RecvPoll::ReadyClosed
        } else {
            RecvPoll::Pending
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ManyPhase { FirstPass, NeedRegister, Recheck, Done }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ManyPoll { Continue, NeedRegister, Ready(usize), Pending }

/// `recv_many` observes TX_CLOSED before it bulk-returns the permits for values
/// already appended in the same call. Unlike the single-recv environment,
/// TX_CLOSED therefore does not imply semaphore idle at observation time.
pub struct RecvManyEnvironment {
    semaphore_idle: bool,
    receiver_closed: bool,
    tx_closed_at_head: bool,
}

impl RecvManyEnvironment {
    pub closed spec fn semaphore_idle(&self) -> bool { self.semaphore_idle }
    pub closed spec fn receiver_closed(&self) -> bool { self.receiver_closed }
    pub closed spec fn tx_closed_at_head(&self) -> bool { self.tx_closed_at_head }
    pub closed spec fn well_formed(&self) -> bool { true }

    pub fn new(semaphore_idle: bool, receiver_closed: bool, tx_closed_at_head: bool)
        -> (result: Self)
        ensures result.well_formed(),
            result.semaphore_idle() == semaphore_idle,
            result.receiver_closed() == receiver_closed,
            result.tx_closed_at_head() == tx_closed_at_head,
        no_unwind
    {
        RecvManyEnvironment { semaphore_idle, receiver_closed, tx_closed_at_head }
    }
}

pub open spec fn many_observation_result<T>(
    phase: ManyPhase,
    added: usize,
    limit: usize,
    read: QueueRead<T>,
    env: RecvManyEnvironment,
) -> ManyPoll {
    match read {
        QueueRead::Value(_) =>
            if added + 1 == limit { ManyPoll::Ready((added + 1) as usize) }
            else { ManyPoll::Continue },
        QueueRead::Closed => ManyPoll::Ready(added),
        QueueRead::Empty | QueueRead::Busy => {
            if added > 0 { ManyPoll::Ready(added) }
            else if phase == ManyPhase::FirstPass { ManyPoll::NeedRegister }
            else if env.receiver_closed() && env.semaphore_idle() { ManyPoll::Ready(0) }
            else { ManyPoll::Pending }
        },
    }
}

/// `recv_many` uses `values.len()` as its only progress measure. It is the
/// exact number added after the immutable caller prefix, remaining is
/// `limit - added`, and the same number of permits is returned once on every
/// Ready-with-values path. `buffer_capacity - initial_prefix.len() >= limit`
/// selects the no-grow production branch in this logical refinement. vstd does
/// not expose physical Vec capacity here, so production tests check that
/// precondition and capacity preservation separately. Vec growth, allocator
/// failure, and its panic path remain unproved; no new trusted boundary is used.
pub struct MpscRecvManyRefinement<T> {
    limit: usize,
    buffer_capacity: usize,
    initial_prefix: Ghost<Seq<T>>,
    values: Vec<T>,
    phase: ManyPhase,
    waker: Option<u64>,
    permits_returned: usize,
    accounting_applied: bool,
    made_progress: bool,
}

impl<T> MpscRecvManyRefinement<T> {
    pub closed spec fn limit(&self) -> usize { self.limit }
    pub closed spec fn buffer_capacity(&self) -> usize { self.buffer_capacity }
    pub closed spec fn initial_prefix(&self) -> Seq<T> { self.initial_prefix@ }
    pub closed spec fn values(&self) -> Seq<T> { self.values@ }
    pub open spec fn buffer(&self) -> Seq<T> {
        self.initial_prefix().add(self.values())
    }
    pub closed spec fn added(&self) -> usize { self.values@.len() as usize }
    pub open spec fn remaining(&self) -> int {
        self.limit() as int - self.added() as int
    }
    pub open spec fn spare_capacity(&self) -> int {
        self.buffer_capacity() as int - self.buffer().len()
    }
    pub closed spec fn phase(&self) -> ManyPhase { self.phase }
    pub closed spec fn waker(&self) -> Option<u64> { self.waker }
    pub closed spec fn permits_returned(&self) -> usize { self.permits_returned }
    pub closed spec fn accounting_applied(&self) -> bool { self.accounting_applied }
    pub closed spec fn made_progress(&self) -> bool { self.made_progress }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.initial_prefix().len() <= usize::MAX
        &&& self.initial_prefix().len() + self.limit() <= self.buffer_capacity()
        &&& self.buffer().len() <= self.buffer_capacity()
        &&& self.added() <= self.limit()
        &&& self.remaining() <= self.spare_capacity()
        &&& (self.permits_returned() == 0 ||
            (self.phase() == ManyPhase::Done
                && self.permits_returned() == self.added()
                && self.added() > 0))
        &&& (self.accounting_applied() ==>
            self.phase() == ManyPhase::Done && self.permits_returned() > 0)
        &&& (self.phase() == ManyPhase::Recheck ==> self.waker().is_some())
        &&& (self.phase() != ManyPhase::Done ==>
            self.permits_returned() == 0
                && !self.accounting_applied() && !self.made_progress())
    }

    pub fn new(limit: usize, previous_waker: Option<u64>) -> (result: Self)
        ensures result.well_formed(), result.limit() == limit,
            result.buffer_capacity() == limit,
            result.initial_prefix() == Seq::<T>::empty(),
            result.buffer() == Seq::<T>::empty(),
            result.values().len() == 0, result.added() == 0,
            result.phase() == ManyPhase::FirstPass,
            result.waker() == previous_waker,
            result.permits_returned() == 0, !result.accounting_applied(),
            !result.made_progress(),
    {
        MpscRecvManyRefinement {
            limit,
            buffer_capacity: limit,
            initial_prefix: Ghost(Seq::empty()),
            values: Vec::new(),
            phase: ManyPhase::FirstPass,
            waker: previous_waker,
            permits_returned: 0,
            accounting_applied: false,
            made_progress: false,
        }
    }

    /// Conditional production entry with enough caller-provided spare capacity
    /// for every Value permitted by `limit`. The prefix is ghost-only because
    /// this refinement observes its preservation, not arbitrary `T` cloning.
    pub fn new_preallocated(
        limit: usize,
        buffer_capacity: usize,
        Ghost(initial_prefix): Ghost<Seq<T>>,
        previous_waker: Option<u64>,
    ) -> (result: Self)
        requires initial_prefix.len() <= usize::MAX,
            initial_prefix.len() + limit <= buffer_capacity,
        ensures result.well_formed(), result.limit() == limit,
            result.buffer_capacity() == buffer_capacity,
            result.initial_prefix() == initial_prefix,
            result.buffer() == initial_prefix,
            result.values().len() == 0, result.added() == 0,
            result.phase() == ManyPhase::FirstPass,
            result.waker() == previous_waker,
            result.permits_returned() == 0, !result.accounting_applied(),
            !result.made_progress(),
    {
        MpscRecvManyRefinement {
            limit,
            buffer_capacity,
            initial_prefix: Ghost(initial_prefix),
            values: Vec::new(),
            phase: ManyPhase::FirstPass,
            waker: previous_waker,
            permits_returned: 0,
            accounting_applied: false,
            made_progress: false,
        }
    }

    /// Called only after both outer poll gates passed.
    pub fn start(&mut self) -> (result: ManyPoll)
        requires old(self).well_formed(), old(self).phase() == ManyPhase::FirstPass,
            old(self).added() == 0,
        ensures final(self).well_formed(),
            final(self).accounting_applied() == old(self).accounting_applied(),
            final(self).buffer_capacity() == old(self).buffer_capacity(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).buffer() == old(self).buffer(),
            final(self).values() == old(self).values(),
            final(self).waker() == old(self).waker(),
            final(self).permits_returned() == old(self).permits_returned(),
            old(self).limit() == 0 ==> result == ManyPoll::Ready(0),
            old(self).limit() == 0 ==> final(self).phase() == ManyPhase::Done,
            old(self).limit() == 0 ==> final(self).made_progress(),
            old(self).limit() > 0 ==> result == ManyPoll::Continue,
            old(self).limit() > 0 ==> *final(self) == *old(self),
    {
        if self.limit == 0 {
            self.phase = ManyPhase::Done;
            self.made_progress = true;
            ManyPoll::Ready(0)
        } else {
            ManyPoll::Continue
        }
    }

    pub fn observe(&mut self, read: QueueRead<T>, env: &RecvManyEnvironment)
        -> (result: ManyPoll)
        requires old(self).well_formed(),
            old(self).phase() == ManyPhase::FirstPass
                || old(self).phase() == ManyPhase::Recheck,
            old(self).added() < old(self).limit(),
            env.well_formed(),
            read == QueueRead::Closed ==> env.tx_closed_at_head(),
        ensures final(self).well_formed(),
            final(self).accounting_applied() == old(self).accounting_applied(),
            final(self).buffer_capacity() == old(self).buffer_capacity(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).buffer() == old(self).initial_prefix().add(final(self).values()),
            result == many_observation_result(
                old(self).phase(), old(self).added(), old(self).limit(), read, *env),
            match read {
                QueueRead::Value(value) => {
                    final(self).values() == old(self).values().push(value)
                        && (result == ManyPoll::Continue ==>
                            final(self).added() < final(self).limit())
                        && (result == ManyPoll::Ready(final(self).added()) ==>
                            final(self).added() == final(self).limit()
                                && final(self).permits_returned() == final(self).added()
                                && final(self).made_progress())
                },
                QueueRead::Closed => {
                    result == ManyPoll::Ready(old(self).added())
                        && final(self).permits_returned() == old(self).added()
                        && final(self).made_progress()
                },
                QueueRead::Empty | QueueRead::Busy => {
                    old(self).added() > 0 ==>
                        result == ManyPoll::Ready(old(self).added())
                            && final(self).permits_returned() == old(self).added()
                            && final(self).made_progress()
                            && final(self).waker() == old(self).waker()
                },
            },
            result == ManyPoll::NeedRegister ==>
                old(self).phase() == ManyPhase::FirstPass
                    && old(self).added() == 0
                    && final(self).phase() == ManyPhase::NeedRegister,
            result == ManyPoll::Ready(0) && old(self).added() == 0 ==>
                (read == QueueRead::Closed
                    || (old(self).phase() == ManyPhase::Recheck
                        && matches!(read, QueueRead::Empty | QueueRead::Busy)
                        && env.receiver_closed() && env.semaphore_idle()))
                    && final(self).made_progress(),
    {
        match read {
            QueueRead::Value(value) => {
                self.values.push(value);
                if self.values.len() == self.limit {
                    self.permits_returned = self.values.len();
                    self.made_progress = true;
                    self.phase = ManyPhase::Done;
                    ManyPoll::Ready(self.values.len())
                } else {
                    ManyPoll::Continue
                }
            },
            QueueRead::Closed => {
                self.permits_returned = self.values.len();
                self.made_progress = true;
                self.phase = ManyPhase::Done;
                ManyPoll::Ready(self.values.len())
            },
            QueueRead::Empty | QueueRead::Busy => {
                if self.values.len() > 0 {
                    self.permits_returned = self.values.len();
                    self.made_progress = true;
                    self.phase = ManyPhase::Done;
                    ManyPoll::Ready(self.values.len())
                } else {
                    match self.phase {
                        ManyPhase::FirstPass => {
                            self.phase = ManyPhase::NeedRegister;
                            ManyPoll::NeedRegister
                        },
                        ManyPhase::Recheck => {
                            self.phase = ManyPhase::Done;
                            if env.receiver_closed && env.semaphore_idle {
                                self.made_progress = true;
                                ManyPoll::Ready(0)
                            } else {
                                ManyPoll::Pending
                            }
                        },
                        ManyPhase::NeedRegister | ManyPhase::Done => unreached(),
                    }
                }
            },
        }
    }

    pub fn observe_value(&mut self, value: T) -> (result: ManyPoll)
        requires old(self).well_formed(),
            old(self).phase() == ManyPhase::FirstPass
                || old(self).phase() == ManyPhase::Recheck,
            old(self).added() < old(self).limit(),
        ensures final(self).well_formed(),
            final(self).accounting_applied() == old(self).accounting_applied(),
            final(self).buffer_capacity() == old(self).buffer_capacity(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).buffer() == old(self).buffer().push(value),
            old(self).buffer().len() < old(self).buffer_capacity(),
            final(self).buffer().len() <= final(self).buffer_capacity(),
            final(self).values() == old(self).values().push(value),
            final(self).added() == old(self).added() + 1,
            final(self).limit() == old(self).limit(),
            final(self).waker() == old(self).waker(),
            old(self).added() + 1 == old(self).limit() ==>
                result == ManyPoll::Ready(old(self).limit())
                    && final(self).phase() == ManyPhase::Done
                    && final(self).permits_returned() == old(self).limit()
                    && final(self).made_progress(),
            old(self).added() + 1 < old(self).limit() ==>
                result == ManyPoll::Continue
                    && final(self).phase() == old(self).phase()
                    && final(self).permits_returned() == 0
                    && !final(self).made_progress(),
    {
        self.values.push(value);
        if self.values.len() == self.limit {
            self.permits_returned = self.values.len();
            self.made_progress = true;
            self.phase = ManyPhase::Done;
            ManyPoll::Ready(self.values.len())
        } else {
            ManyPoll::Continue
        }
    }

    pub fn observe_closed(&mut self, env: &RecvManyEnvironment) -> (result: ManyPoll)
        requires old(self).well_formed(),
            old(self).phase() == ManyPhase::FirstPass
                || old(self).phase() == ManyPhase::Recheck,
            old(self).added() < old(self).limit(),
            env.well_formed(), env.tx_closed_at_head(),
        ensures final(self).well_formed(), result == ManyPoll::Ready(old(self).added()),
            final(self).accounting_applied() == old(self).accounting_applied(),
            final(self).buffer_capacity() == old(self).buffer_capacity(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).buffer() == old(self).buffer(),
            final(self).phase() == ManyPhase::Done,
            final(self).limit() == old(self).limit(),
            final(self).values() == old(self).values(),
            final(self).waker() == old(self).waker(),
            final(self).permits_returned() == old(self).added(),
            final(self).made_progress(),
        no_unwind
    {
        self.permits_returned = self.values.len();
        self.made_progress = true;
        self.phase = ManyPhase::Done;
        ManyPoll::Ready(self.values.len())
    }

    pub fn observe_none(&mut self, env: &RecvManyEnvironment, _busy: bool)
        -> (result: ManyPoll)
        requires old(self).well_formed(),
            old(self).phase() == ManyPhase::FirstPass
                || old(self).phase() == ManyPhase::Recheck,
            old(self).added() < old(self).limit(), env.well_formed(),
        ensures final(self).well_formed(),
            final(self).accounting_applied() == old(self).accounting_applied(),
            final(self).buffer_capacity() == old(self).buffer_capacity(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).buffer() == old(self).buffer(),
            final(self).limit() == old(self).limit(),
            final(self).values() == old(self).values(),
            final(self).waker() == old(self).waker(),
            old(self).added() > 0 ==>
                result == ManyPoll::Ready(old(self).added())
                    && final(self).phase() == ManyPhase::Done
                    && final(self).permits_returned() == old(self).added()
                    && final(self).made_progress(),
            old(self).added() == 0 && old(self).phase() == ManyPhase::FirstPass ==>
                result == ManyPoll::NeedRegister
                    && final(self).phase() == ManyPhase::NeedRegister
                    && final(self).permits_returned() == 0
                    && !final(self).made_progress(),
            old(self).added() == 0 && old(self).phase() == ManyPhase::Recheck
                && env.receiver_closed() && env.semaphore_idle() ==>
                result == ManyPoll::Ready(0)
                    && final(self).phase() == ManyPhase::Done
                    && final(self).permits_returned() == 0
                    && final(self).made_progress(),
            old(self).added() == 0 && old(self).phase() == ManyPhase::Recheck
                && !(env.receiver_closed() && env.semaphore_idle()) ==>
                result == ManyPoll::Pending
                    && final(self).phase() == ManyPhase::Done
                    && final(self).permits_returned() == 0
                    && !final(self).made_progress()
                    && final(self).waker().is_some(),
        no_unwind
    {
        if self.values.len() > 0 {
            self.permits_returned = self.values.len();
            self.made_progress = true;
            self.phase = ManyPhase::Done;
            ManyPoll::Ready(self.values.len())
        } else {
            match self.phase {
                ManyPhase::FirstPass => {
                    self.phase = ManyPhase::NeedRegister;
                    ManyPoll::NeedRegister
                },
                ManyPhase::Recheck => {
                    self.phase = ManyPhase::Done;
                    if env.receiver_closed && env.semaphore_idle {
                        self.made_progress = true;
                        ManyPoll::Ready(0)
                    } else {
                        ManyPoll::Pending
                    }
                },
                ManyPhase::NeedRegister | ManyPhase::Done => unreached(),
            }
        }
    }

    pub fn register_latest(&mut self, waker: u64)
        requires old(self).well_formed(), old(self).phase() == ManyPhase::NeedRegister,
            old(self).added() == 0,
        ensures final(self).well_formed(), final(self).phase() == ManyPhase::Recheck,
            final(self).accounting_applied() == old(self).accounting_applied(),
            final(self).buffer_capacity() == old(self).buffer_capacity(),
            final(self).initial_prefix() == old(self).initial_prefix(),
            final(self).buffer() == old(self).buffer(),
            final(self).waker() == Some(waker),
            final(self).limit() == old(self).limit(),
            final(self).added() == old(self).added(),
            final(self).values() == old(self).values(),
            final(self).permits_returned() == old(self).permits_returned(),
            final(self).made_progress() == old(self).made_progress(),
        no_unwind
    {
        self.waker = Some(waker);
        self.phase = ManyPhase::Recheck;
    }

    /// Connects an arbitrary nonempty terminal batch directly to production's
    /// one bounded `add_permits(number_added)` operation.
    pub fn apply_bounded_bulk_return(&mut self, capacity: &mut MpscCapacity)
        requires old(self).well_formed(), old(self).phase() == ManyPhase::Done,
            !old(self).accounting_applied(),
            old(self).permits_returned() > 0,
            old(self).permits_returned() <= u64::MAX,
            old(self).permits_returned() as u64 <= old(capacity).queued(),
            old(capacity).well_formed(),
        ensures final(self).well_formed(), final(self).accounting_applied(),
            final(self).permits_returned() == old(self).permits_returned(),
            final(self).added() == old(self).added(),
            final(capacity).well_formed(),
            final(capacity).queued() + old(self).permits_returned() as u64
                == old(capacity).queued(),
            final(capacity).available()
                == old(capacity).available() + old(self).permits_returned() as u64,
            final(capacity).reserved() == old(capacity).reserved(),
            final(capacity).capacity() == old(capacity).capacity(),
        no_unwind
    {
        capacity.receive_many(self.permits_returned as u64);
        self.accounting_applied = true;
    }

    /// Connects an arbitrary nonempty terminal batch directly to production's
    /// one unbounded `fetch_sub(number_added << 1)` operation.
    pub fn apply_unbounded_bulk_return(&mut self, accounting: &mut UnboundedAccounting)
        requires old(self).well_formed(), old(self).phase() == ManyPhase::Done,
            !old(self).accounting_applied(),
            old(self).permits_returned() > 0,
            old(self).permits_returned() <= old(accounting).messages(),
            old(self).permits_returned() * 2 <= usize::MAX,
            old(accounting).well_formed(),
        ensures final(self).well_formed(), final(self).accounting_applied(),
            final(self).permits_returned() == old(self).permits_returned(),
            final(self).added() == old(self).added(),
            final(accounting).well_formed(),
            old(self).permits_returned() << 1 == old(self).permits_returned() * 2,
            final(accounting).encoded() + (old(self).permits_returned() << 1)
                == old(accounting).encoded(),
            final(accounting).messages() + old(self).permits_returned()
                == old(accounting).messages(),
            final(accounting).receiver_closed() == old(accounting).receiver_closed(),
        no_unwind
    {
        accounting.receive_many(self.permits_returned);
        self.accounting_applied = true;
    }
}

pub fn verify_gate_pending_preserves_channel_state(waker: u64)
{
    let mut surface = PollSurfaceState::new(Some(waker));
    let passed = surface.pass_trace_and_coop(false, true);
    assert(!passed);
    assert(surface.waker() == Some(waker));
    assert(surface.permits_returned() == 0);
    assert(!surface.made_progress());
}

pub fn verify_recv_register_recheck_value(value: u64, waker: u64)
{
    let mut recv = MpscRecvRefinement::new(None);
    let first = recv.first_none::<u64>(false);
    assert(first == RecvPoll::NeedRegister);
    recv.register_latest(waker);
    let second = recv.second_value(value);
    assert(second == RecvPoll::ReadyValue(value));
    assert(recv.waker() == Some(waker));
    assert(recv.permits_returned() == 1);
    assert(recv.made_progress());
}

pub fn verify_recv_first_value_does_not_register(value: u64, old_waker: u64)
{
    let mut recv = MpscRecvRefinement::new(Some(old_waker));
    let first = recv.first_value(value);
    assert(first == RecvPoll::ReadyValue(value));
    assert(recv.waker() == Some(old_waker));
    assert(recv.permits_returned() == 1);
}

pub fn verify_recv_closed_marker_priority(old_waker: u64)
{
    let closed = RecvEnvironment::new(true, false, true);
    let mut recv = MpscRecvRefinement::new(Some(old_waker));
    let first = recv.first_closed::<u64>(&closed);
    assert(first == RecvPoll::ReadyClosed);
    assert(recv.waker() == Some(old_waker));
    assert(recv.made_progress());
}

pub fn verify_receiver_close_with_reserved_permit_stays_pending(waker: u64)
{
    let closed_not_idle = RecvEnvironment::new(false, true, false);
    let mut recv = MpscRecvRefinement::new(None);
    let first = recv.first_none::<u64>(false);
    assert(first == RecvPoll::NeedRegister);
    recv.register_latest(waker);
    let second = recv.second_none::<u64>(&closed_not_idle, false);
    assert(second == RecvPoll::Pending);
    assert(recv.waker() == Some(waker));
    assert(!recv.made_progress());
}

pub fn verify_receiver_close_idle_is_ready(waker: u64)
{
    let closed_idle = RecvEnvironment::new(true, true, false);
    let mut recv = MpscRecvRefinement::new(None);
    recv.first_none::<u64>(false);
    recv.register_latest(waker);
    let second = recv.second_none::<u64>(&closed_idle, true);
    assert(second == RecvPoll::ReadyClosed);
    assert(recv.waker() == Some(waker));
}

pub fn verify_bounded_recv_composes_capacity(value: u64)
{
    let mut capacity = MpscCapacity::new(1);
    let reserved = capacity.reserve();
    assert(reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let mut recv = MpscRecvRefinement::new(None);
    let result = recv.first_value(value);
    assert(result == RecvPoll::ReadyValue(value));
    let received = capacity.receive();
    assert(received);
    assert(capacity.available() == 1);
    assert(recv.permits_returned() == 1);
}

pub fn verify_recv_many_limit_zero_after_gates()
{
    let mut surface = PollSurfaceState::new(None);
    let passed = surface.pass_trace_and_coop(true, true);
    assert(passed);
    let mut many = MpscRecvManyRefinement::<u64>::new(0, None);
    let result = many.start();
    assert(result == ManyPoll::Ready(0));
    assert(many.values().len() == 0);
    assert(many.permits_returned() == 0);
    assert(many.made_progress());
}

pub fn verify_recv_many_exact_logical_oracle_batch(first: u64, second: u64, old_waker: u64)
{
    let open = RecvManyEnvironment::new(false, false, false);
    let mut many = MpscRecvManyRefinement::new(3, Some(old_waker));
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let first_step = many.observe_value(first);
    assert(first_step == ManyPoll::Continue);
    let second_step = many.observe_value(second);
    assert(second_step == ManyPoll::Continue);
    let ready = many.observe_none(&open, false);
    assert(ready == ManyPoll::Ready(2));
    assert(many.values() == seq![first, second]);
    assert(many.permits_returned() == 2);
    assert(many.waker() == Some(old_waker));
}

pub fn verify_recv_many_composes_bounded_batch(first: u64, second: u64)
{
    let mut capacity = MpscCapacity::new(3);
    let first_reserved = capacity.reserve();
    assert(first_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let second_reserved = capacity.reserve();
    assert(second_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    assert(capacity.available() == 1);
    assert(capacity.queued() == 2);

    let open = RecvManyEnvironment::new(false, false, false);
    let mut many = MpscRecvManyRefinement::new(3, None);
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let first_step = many.observe_value(first);
    assert(first_step == ManyPoll::Continue);
    let second_step = many.observe_value(second);
    assert(second_step == ManyPoll::Continue);
    let ready = many.observe_none(&open, false);
    assert(ready == ManyPoll::Ready(2));
    assert(many.permits_returned() == 2);

    let received_first = capacity.receive();
    assert(received_first);
    let received_second = capacity.receive();
    assert(received_second);
    assert(capacity.available() == 3);
    assert(capacity.reserved() == 0);
    assert(capacity.queued() == 0);
}

pub fn verify_reserved_permit_send_then_value_then_terminal(value: u64, waker: u64)
{
    let mut capacity = MpscCapacity::new(1);
    let reserved = capacity.reserve();
    assert(reserved == ReserveResult::Permit);
    capacity.close_receiver();
    let idle_before_send = capacity.drained_and_closed();
    assert(!idle_before_send);
    assert(capacity.reserved() == 1);

    let closed_not_idle = RecvEnvironment::new(false, true, false);
    let mut pending = MpscRecvRefinement::new(None);
    let first = pending.first_none::<u64>(false);
    assert(first == RecvPoll::NeedRegister);
    pending.register_latest(waker);
    let second = pending.second_none::<u64>(&closed_not_idle, false);
    assert(second == RecvPoll::Pending);

    capacity.commit_permit();
    let mut value_poll = MpscRecvRefinement::new(Some(waker));
    let value_result = value_poll.first_value(value);
    assert(value_result == RecvPoll::ReadyValue(value));
    let received = capacity.receive();
    assert(received);
    assert(capacity.queued() == 0 && capacity.reserved() == 0);
    let idle_after_value = capacity.drained_and_closed();
    assert(idle_after_value);

    let closed_idle = RecvEnvironment::new(true, true, false);
    let mut terminal = MpscRecvRefinement::new(Some(waker));
    let terminal_first = terminal.first_none::<u64>(false);
    assert(terminal_first == RecvPoll::NeedRegister);
    terminal.register_latest(waker);
    let terminal_second = terminal.second_none::<u64>(&closed_idle, false);
    assert(terminal_second == RecvPoll::ReadyClosed);
}

pub fn verify_reserved_permit_cancel_then_terminal(waker: u64)
{
    let mut capacity = MpscCapacity::new(1);
    let reserved = capacity.reserve();
    assert(reserved == ReserveResult::Permit);
    capacity.close_receiver();
    let idle_before_cancel = capacity.drained_and_closed();
    assert(!idle_before_cancel);
    assert(capacity.reserved() == 1);

    let closed_not_idle = RecvEnvironment::new(false, true, false);
    let mut pending = MpscRecvRefinement::new(None);
    let first = pending.first_none::<u64>(false);
    assert(first == RecvPoll::NeedRegister);
    pending.register_latest(waker);
    let second = pending.second_none::<u64>(&closed_not_idle, false);
    assert(second == RecvPoll::Pending);

    capacity.cancel_permit();
    assert(capacity.queued() == 0 && capacity.reserved() == 0);
    let idle_after_cancel = capacity.drained_and_closed();
    assert(idle_after_cancel);
    let closed_idle = RecvEnvironment::new(true, true, false);
    let mut terminal = MpscRecvRefinement::new(Some(waker));
    let terminal_first = terminal.first_none::<u64>(false);
    assert(terminal_first == RecvPoll::NeedRegister);
    terminal.register_latest(waker);
    let terminal_second = terminal.second_none::<u64>(&closed_idle, false);
    assert(terminal_second == RecvPoll::ReadyClosed);
}

pub fn verify_unbounded_value_decrements_exactly_once(value: u64)
{
    let mut accounting = UnboundedAccounting::new();
    accounting.publish();
    assert(accounting.encoded() == 2 && accounting.messages() == 1);
    let mut recv = MpscRecvRefinement::new(None);
    let result = recv.first_value(value);
    assert(result == RecvPoll::ReadyValue(value));
    accounting.receive_value();
    assert(accounting.encoded() == 0 && accounting.messages() == 0);
}

pub fn verify_unbounded_closed_value_then_idle(value: u64)
{
    let mut accounting = UnboundedAccounting::new();
    accounting.publish();
    accounting.close_receiver();
    assert(accounting.encoded() == 3 && accounting.messages() == 1);
    let while_nonidle = accounting.finish_without_value();
    assert(!while_nonidle);

    let mut recv = MpscRecvRefinement::new(None);
    let value_result = recv.first_value(value);
    assert(value_result == RecvPoll::ReadyValue(value));
    accounting.receive_value();
    assert(accounting.encoded() == 1 && accounting.messages() == 0);
    let when_idle = accounting.finish_without_value();
    assert(when_idle);
}

pub fn verify_unbounded_pending_preserves_word()
{
    let mut accounting = UnboundedAccounting::new();
    let open = accounting.finish_without_value();
    assert(!open);
    assert(accounting.encoded() == 0);
    accounting.close_receiver();
    let closed = accounting.finish_without_value();
    assert(closed);
    assert(accounting.encoded() == 1);
}

pub fn verify_unbounded_tx_closed_marker_preserves_word()
{
    let mut accounting = UnboundedAccounting::new();
    let tx_closed = RecvEnvironment::new(true, false, true);
    let mut recv = MpscRecvRefinement::new(None);
    let result = recv.first_closed::<u64>(&tx_closed);
    assert(result == RecvPoll::ReadyClosed);
    assert(accounting.encoded() == 0);
    let idle_observation = accounting.finish_without_value();
    assert(!idle_observation);
    assert(accounting.encoded() == 0);
}

pub fn verify_recv_many_recheck_value_uses_latest_waker(value: u64, waker: u64)
{
    let open = RecvManyEnvironment::new(false, false, false);
    let mut many = MpscRecvManyRefinement::new(2, None);
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let first = many.observe_none(&open, true);
    assert(first == ManyPoll::NeedRegister);
    many.register_latest(waker);
    let value_step = many.observe_value(value);
    assert(value_step == ManyPoll::Continue);
    let ready = many.observe_none(&open, false);
    assert(ready == ManyPoll::Ready(1));
    assert(many.values() == seq![value]);
    assert(many.permits_returned() == 1);
    assert(many.waker() == Some(waker));
}

pub fn verify_recv_many_preallocated_bounded_bulk_return(
    first: u64,
    second: u64,
    Ghost(prefix): Ghost<Seq<u64>>,
    buffer_capacity: usize,
)
    requires prefix.len() <= usize::MAX,
        prefix.len() + 3 <= buffer_capacity,
{
    let mut capacity = MpscCapacity::new(3);
    let first_reserved = capacity.reserve();
    assert(first_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let second_reserved = capacity.reserve();
    assert(second_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let held = capacity.reserve();
    assert(held == ReserveResult::Permit);

    let open_with_held_permit = RecvManyEnvironment::new(false, false, false);
    let mut many = MpscRecvManyRefinement::new_preallocated(
        3, buffer_capacity, Ghost(prefix), None,
    );
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let first_step = many.observe_value(first);
    assert(first_step == ManyPoll::Continue);
    let second_step = many.observe_value(second);
    assert(second_step == ManyPoll::Continue);
    let ready = many.observe_none(&open_with_held_permit, false);
    assert(ready == ManyPoll::Ready(2));
    assert(many.initial_prefix() == prefix);
    assert(many.buffer() == prefix.add(seq![first, second]));
    assert(many.buffer_capacity() == buffer_capacity);
    assert(many.permits_returned() == 2);

    many.apply_bounded_bulk_return(&mut capacity);
    assert(capacity.available() == 2);
    assert(capacity.reserved() == 1);
    assert(capacity.queued() == 0);
}

pub fn verify_recv_many_preallocated_limit_bulk_return(
    first: u64,
    second: u64,
    Ghost(prefix): Ghost<Seq<u64>>,
    buffer_capacity: usize,
)
    requires prefix.len() <= usize::MAX,
        prefix.len() + 2 <= buffer_capacity,
{
    let mut capacity = MpscCapacity::new(2);
    let first_reserved = capacity.reserve();
    assert(first_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let second_reserved = capacity.reserve();
    assert(second_reserved == ReserveResult::Permit);
    capacity.commit_permit();

    let mut many = MpscRecvManyRefinement::new_preallocated(
        2, buffer_capacity, Ghost(prefix), None,
    );
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let first_step = many.observe_value(first);
    assert(first_step == ManyPoll::Continue);
    let ready = many.observe_value(second);
    assert(ready == ManyPoll::Ready(2));
    assert(many.initial_prefix() == prefix);
    assert(many.buffer() == prefix.add(seq![first, second]));
    assert(many.permits_returned() == 2);

    many.apply_bounded_bulk_return(&mut capacity);
    assert(capacity.available() == 2);
    assert(capacity.queued() == 0);
}

pub fn verify_recv_many_preallocated_unbounded_exact_decrement(
    first: u64,
    second: u64,
    Ghost(prefix): Ghost<Seq<u64>>,
    buffer_capacity: usize,
)
    requires prefix.len() <= usize::MAX,
        prefix.len() + 3 <= buffer_capacity,
{
    let mut accounting = UnboundedAccounting::new();
    accounting.publish();
    accounting.publish();
    accounting.close_receiver();
    assert(accounting.encoded() == 5);

    let closed_with_values = RecvManyEnvironment::new(false, true, false);
    let mut many = MpscRecvManyRefinement::new_preallocated(
        3, buffer_capacity, Ghost(prefix), None,
    );
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let first_step = many.observe_value(first);
    assert(first_step == ManyPoll::Continue);
    let second_step = many.observe_value(second);
    assert(second_step == ManyPoll::Continue);
    let ready = many.observe_none(&closed_with_values, false);
    assert(ready == ManyPoll::Ready(2));
    assert(many.initial_prefix() == prefix);
    assert(many.buffer() == prefix.add(seq![first, second]));
    assert(many.permits_returned() == 2);

    many.apply_unbounded_bulk_return(&mut accounting);
    assert(accounting.encoded() == 1);
    assert(accounting.messages() == 0);
    assert(accounting.receiver_closed());
}

pub fn verify_recv_many_preallocated_nonempty_closed_bulk_return(
    value: u64,
    Ghost(prefix): Ghost<Seq<u64>>,
    buffer_capacity: usize,
)
    requires prefix.len() <= usize::MAX,
        prefix.len() + 2 <= buffer_capacity,
{
    let mut capacity = MpscCapacity::new(1);
    let reserved = capacity.reserve();
    assert(reserved == ReserveResult::Permit);
    capacity.commit_permit();

    let tx_closed = RecvManyEnvironment::new(false, false, true);
    let mut many = MpscRecvManyRefinement::new_preallocated(
        2, buffer_capacity, Ghost(prefix), None,
    );
    let started = many.start();
    assert(started == ManyPoll::Continue);
    let value_step = many.observe_value(value);
    assert(value_step == ManyPoll::Continue);
    let closed = many.observe_closed(&tx_closed);
    assert(closed == ManyPoll::Ready(1));
    assert(many.initial_prefix() == prefix);
    assert(many.buffer() == prefix.push(value));
    assert(many.permits_returned() == 1);

    many.apply_bounded_bulk_return(&mut capacity);
    assert(capacity.available() == 1);
    assert(capacity.available() == capacity.capacity());
    assert(capacity.reserved() == 0);
    assert(capacity.queued() == 0);
    assert(many.accounting_applied());
}

pub fn verify_recv_many_preallocated_zero_value_priorities(
    Ghost(prefix): Ghost<Seq<u64>>,
    buffer_capacity: usize,
    waker: u64,
)
    requires prefix.len() <= usize::MAX,
        prefix.len() + 1 <= buffer_capacity,
{
    let receiver_closed_with_permit = RecvManyEnvironment::new(false, true, false);
    let mut pending = MpscRecvManyRefinement::<u64>::new_preallocated(
        1, buffer_capacity, Ghost(prefix), None,
    );
    let started = pending.start();
    assert(started == ManyPoll::Continue);
    let first_empty = pending.observe_none(&receiver_closed_with_permit, false);
    assert(first_empty == ManyPoll::NeedRegister);
    pending.register_latest(waker);
    let pending_result = pending.observe_none(&receiver_closed_with_permit, false);
    assert(pending_result == ManyPoll::Pending);
    assert(pending.buffer() == prefix);
    assert(!pending.made_progress());

    let receiver_closed_and_idle = RecvManyEnvironment::new(true, true, false);
    let mut terminal = MpscRecvManyRefinement::<u64>::new_preallocated(
        1, buffer_capacity, Ghost(prefix), None,
    );
    let terminal_started = terminal.start();
    assert(terminal_started == ManyPoll::Continue);
    let terminal_first_empty = terminal.observe_none(&receiver_closed_and_idle, false);
    assert(terminal_first_empty == ManyPoll::NeedRegister);
    terminal.register_latest(waker);
    let terminal_result = terminal.observe_none(&receiver_closed_and_idle, false);
    assert(terminal_result == ManyPoll::Ready(0));
    assert(terminal.buffer() == prefix);
    assert(terminal.made_progress());

    let tx_closed = RecvManyEnvironment::new(true, false, true);
    let mut closed = MpscRecvManyRefinement::<u64>::new_preallocated(
        1, buffer_capacity, Ghost(prefix), None,
    );
    let closed_started = closed.start();
    assert(closed_started == ManyPoll::Continue);
    let closed_result = closed.observe_closed(&tx_closed);
    assert(closed_result == ManyPoll::Ready(0));
    assert(closed.buffer() == prefix);
    assert(closed.made_progress());
}

} // verus!
