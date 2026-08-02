use crate::atomic_waker_refinement::{AtomicWakerMachine, RegisterBranch};
use crate::mpsc_protocol::{MpscCapacity, QueueRead, ReserveResult};
use crate::mpsc_refinement::{RecvEnvironment, UnboundedAccounting};
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TryRecvPhase { InitialPop, NeedWake, NeedRegister, Recheck, NeedPark, Done }

#[derive(PartialEq, Eq)]
pub enum TryRecvOutcome<T> { Value(T), Empty, Disconnected, Continue }

pub open spec fn terminal_try_recv_outcome<T>(
    read: QueueRead<T>,
    env: RecvEnvironment,
) -> TryRecvOutcome<T> {
    match read {
        QueueRead::Value(value) => TryRecvOutcome::Value(value),
        QueueRead::Closed => TryRecvOutcome::Disconnected,
        QueueRead::Empty => if env.receiver_closed() && env.semaphore_idle() {
            TryRecvOutcome::Disconnected
        } else {
            TryRecvOutcome::Empty
        },
        QueueRead::Busy => TryRecvOutcome::Continue,
    }
}

/// Below-saturation finite-prefix safety projection of production `Chan::try_recv`.
///
/// Every modeled Busy recheck requires a completed park-waker registration and
/// leads to exactly one park before another registration, while the `usize`
/// proof-trace counters remain below saturation. Waker execution is a frozen
/// boundary. Scheduler liveness is also an allowed foundation, but this
/// refinement does not instantiate a liveness premise sufficient to prove this
/// production loop terminates. Tokio-specific CachedParkThread TLS/Condvar/state
/// mechanics and production loop termination therefore remain unproved.
pub struct MpscTryRecvRefinement {
    phase: TryRecvPhase,
    wake_calls: usize,
    previous_woken: Option<u64>,
    park_waker: Option<u64>,
    registrations: usize,
    parks: usize,
    permits_returned: usize,
}

impl MpscTryRecvRefinement {
    pub closed spec fn phase(&self) -> TryRecvPhase { self.phase }
    pub closed spec fn wake_calls(&self) -> usize { self.wake_calls }
    pub closed spec fn previous_woken(&self) -> Option<u64> { self.previous_woken }
    pub closed spec fn park_waker(&self) -> Option<u64> { self.park_waker }
    pub closed spec fn registrations(&self) -> usize { self.registrations }
    pub closed spec fn parks(&self) -> usize { self.parks }
    pub closed spec fn permits_returned(&self) -> usize { self.permits_returned }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.parks() <= self.registrations()
        &&& self.registrations() <= self.parks() + 1
        &&& (self.phase() == TryRecvPhase::Recheck
            || self.phase() == TryRecvPhase::NeedPark ==>
                self.registrations() == self.parks() + 1
                    && self.park_waker().is_some())
        &&& (self.phase() == TryRecvPhase::NeedRegister ==>
                self.registrations() == self.parks()
                    && self.park_waker().is_some())
        &&& (self.phase() == TryRecvPhase::InitialPop
            || self.phase() == TryRecvPhase::NeedWake ==>
                self.wake_calls() == 0
                    && self.registrations() == 0
                    && self.parks() == 0
                    && self.park_waker().is_none())
        &&& (self.phase() != TryRecvPhase::Done ==> self.permits_returned() == 0)
        &&& self.permits_returned() <= 1
        &&& self.wake_calls() <= 1
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.phase() == TryRecvPhase::InitialPop,
            result.wake_calls() == 0, result.previous_woken().is_none(),
            result.park_waker().is_none(), result.registrations() == 0,
            result.parks() == 0, result.permits_returned() == 0,
        no_unwind
    {
        MpscTryRecvRefinement {
            phase: TryRecvPhase::InitialPop,
            wake_calls: 0,
            previous_woken: None,
            park_waker: None,
            registrations: 0,
            parks: 0,
            permits_returned: 0,
        }
    }

    pub fn initial_observe<T>(&mut self, read: QueueRead<T>, env: &RecvEnvironment)
        -> (result: TryRecvOutcome<T>)
        requires old(self).well_formed(), old(self).phase() == TryRecvPhase::InitialPop,
            env.well_formed(), read == QueueRead::Closed ==> env.tx_closed_at_head(),
        ensures final(self).well_formed(),
            result == terminal_try_recv_outcome(read, *env),
            read == QueueRead::Busy ==>
                final(self).phase() == TryRecvPhase::NeedWake,
            read != QueueRead::Busy ==> final(self).phase() == TryRecvPhase::Done,
            matches!(read, QueueRead::Value(_)) ==>
                final(self).permits_returned() == 1,
            !matches!(read, QueueRead::Value(_)) ==>
                final(self).permits_returned() == 0,
            final(self).wake_calls() == 0,
            final(self).registrations() == 0,
            final(self).parks() == 0,
        no_unwind
    {
        match read {
            QueueRead::Value(value) => {
                self.phase = TryRecvPhase::Done;
                self.permits_returned = 1;
                TryRecvOutcome::Value(value)
            },
            QueueRead::Closed => {
                self.phase = TryRecvPhase::Done;
                TryRecvOutcome::Disconnected
            },
            QueueRead::Empty => {
                self.phase = TryRecvPhase::Done;
                if env.receiver_closed_and_idle() {
                    TryRecvOutcome::Disconnected
                } else {
                    TryRecvOutcome::Empty
                }
            },
            QueueRead::Busy => {
                self.phase = TryRecvPhase::NeedWake;
                TryRecvOutcome::Continue
            },
        }
    }

    /// The one wake before CachedParkThread creation. `woken` is supplied by
    /// the already-proved S09 AtomicWaker transition.
    pub fn wake_previous_and_prepare(&mut self, woken: Option<u64>, park_waker: u64)
        requires old(self).well_formed(), old(self).phase() == TryRecvPhase::NeedWake,
        ensures final(self).well_formed(),
            final(self).phase() == TryRecvPhase::NeedRegister,
            final(self).wake_calls() == 1,
            final(self).previous_woken() == woken,
            final(self).park_waker() == Some(park_waker),
            final(self).registrations() == 0, final(self).parks() == 0,
            final(self).permits_returned() == 0,
        no_unwind
    {
        self.wake_calls = 1;
        self.previous_woken = woken;
        self.park_waker = Some(park_waker);
        self.phase = TryRecvPhase::NeedRegister;
    }

    /// Abstract protocol bookkeeping. Only the exact two-Busy witness below
    /// directly composes these registrations with the S09 AtomicWaker machine.
    pub fn register_iteration(&mut self)
        requires old(self).well_formed(), old(self).phase() == TryRecvPhase::NeedRegister,
            old(self).registrations() < usize::MAX,
        ensures final(self).well_formed(), final(self).phase() == TryRecvPhase::Recheck,
            final(self).registrations() == old(self).registrations() + 1,
            final(self).parks() == old(self).parks(),
            final(self).park_waker() == old(self).park_waker(),
            final(self).wake_calls() == old(self).wake_calls(),
            final(self).previous_woken() == old(self).previous_woken(),
            final(self).permits_returned() == 0,
        no_unwind
    {
        self.registrations += 1;
        self.phase = TryRecvPhase::Recheck;
    }

    pub fn recheck<T>(&mut self, read: QueueRead<T>, env: &RecvEnvironment)
        -> (result: TryRecvOutcome<T>)
        requires old(self).well_formed(), old(self).phase() == TryRecvPhase::Recheck,
            env.well_formed(), read == QueueRead::Closed ==> env.tx_closed_at_head(),
        ensures final(self).well_formed(),
            result == terminal_try_recv_outcome(read, *env),
            read == QueueRead::Busy ==> final(self).phase() == TryRecvPhase::NeedPark,
            read != QueueRead::Busy ==> final(self).phase() == TryRecvPhase::Done,
            matches!(read, QueueRead::Value(_)) ==>
                final(self).permits_returned() == 1,
            !matches!(read, QueueRead::Value(_)) ==>
                final(self).permits_returned() == 0,
            final(self).registrations() == old(self).registrations(),
            final(self).parks() == old(self).parks(),
            final(self).park_waker() == old(self).park_waker(),
            final(self).wake_calls() == old(self).wake_calls(),
            final(self).previous_woken() == old(self).previous_woken(),
        no_unwind
    {
        match read {
            QueueRead::Value(value) => {
                self.phase = TryRecvPhase::Done;
                self.permits_returned = 1;
                TryRecvOutcome::Value(value)
            },
            QueueRead::Closed => {
                self.phase = TryRecvPhase::Done;
                TryRecvOutcome::Disconnected
            },
            QueueRead::Empty => {
                self.phase = TryRecvPhase::Done;
                if env.receiver_closed_and_idle() {
                    TryRecvOutcome::Disconnected
                } else {
                    TryRecvOutcome::Empty
                }
            },
            QueueRead::Busy => {
                self.phase = TryRecvPhase::NeedPark;
                TryRecvOutcome::Continue
            },
        }
    }

    pub fn park_once(&mut self)
        requires old(self).well_formed(), old(self).phase() == TryRecvPhase::NeedPark,
            old(self).parks() < usize::MAX,
        ensures final(self).well_formed(),
            final(self).phase() == TryRecvPhase::NeedRegister,
            final(self).parks() == old(self).parks() + 1,
            final(self).registrations() == old(self).registrations(),
            final(self).park_waker() == old(self).park_waker(),
            final(self).wake_calls() == old(self).wake_calls(),
            final(self).previous_woken() == old(self).previous_woken(),
            final(self).permits_returned() == 0,
        no_unwind
    {
        self.parks += 1;
        self.phase = TryRecvPhase::NeedRegister;
    }
}

pub fn verify_try_recv_initial_value_has_no_busy_side_effect(value: u64)
{
    let open = RecvEnvironment::new(false, false, false);
    let mut recv = MpscTryRecvRefinement::new();
    let result = recv.initial_observe(QueueRead::Value(value), &open);
    assert(result == TryRecvOutcome::Value(value));
    assert(recv.permits_returned() == 1);
    assert(recv.wake_calls() == 0 && recv.registrations() == 0 && recv.parks() == 0);
}

pub fn verify_try_recv_two_finite_busy_iterations(value: u64, old_waker: u64, park_id: u64)
{
    let mut waker = AtomicWakerMachine::new();
    let old_start = waker.begin_register(old_waker);
    assert(old_start == RegisterBranch::Locked);
    let old_finish = waker.finish_register(true);
    assert(!old_finish.resumes_panic);

    let open = RecvEnvironment::new(false, false, false);
    let mut recv = MpscTryRecvRefinement::new();
    let initial = recv.initial_observe::<u64>(QueueRead::Busy, &open);
    assert(initial == TryRecvOutcome::Continue);
    let displaced = waker.take_waker();
    assert(displaced == Some(old_waker));
    recv.wake_previous_and_prepare(displaced, park_id);

    let first_register = waker.begin_register(park_id);
    assert(first_register == RegisterBranch::Locked);
    let first_finish = waker.finish_register(true);
    assert(!first_finish.resumes_panic);
    recv.register_iteration();
    let first_recheck = recv.recheck::<u64>(QueueRead::Busy, &open);
    assert(first_recheck == TryRecvOutcome::Continue);
    recv.park_once();

    let second_register = waker.begin_register(park_id);
    assert(second_register == RegisterBranch::Locked);
    let second_finish = waker.finish_register(true);
    assert(!second_finish.resumes_panic);
    recv.register_iteration();
    let second_recheck = recv.recheck(QueueRead::Value(value), &open);
    assert(second_recheck == TryRecvOutcome::Value(value));
    assert(recv.registrations() == 2 && recv.parks() == 1);
    assert(recv.permits_returned() == 1 && recv.wake_calls() == 1);
    assert(waker.slot() == Some(park_id));
}

pub fn verify_try_recv_busy_value_restores_bounded_capacity(value: u64, park_id: u64)
{
    let mut capacity = MpscCapacity::new(1);
    let reserved = capacity.reserve();
    assert(reserved == ReserveResult::Permit);
    capacity.commit_permit();

    let open = RecvEnvironment::new(false, false, false);
    let mut recv = MpscTryRecvRefinement::new();
    let initial = recv.initial_observe::<u64>(QueueRead::Busy, &open);
    assert(initial == TryRecvOutcome::Continue);
    recv.wake_previous_and_prepare(None, park_id);
    recv.register_iteration();
    let result = recv.recheck(QueueRead::Value(value), &open);
    assert(result == TryRecvOutcome::Value(value));
    let received = capacity.receive();
    assert(received && capacity.available() == 1 && capacity.queued() == 0);
}

pub fn verify_try_recv_busy_value_decrements_unbounded_once(value: u64, park_id: u64)
{
    let mut accounting = UnboundedAccounting::new();
    accounting.publish();
    let open = RecvEnvironment::new(false, false, false);
    let mut recv = MpscTryRecvRefinement::new();
    let initial = recv.initial_observe::<u64>(QueueRead::Busy, &open);
    assert(initial == TryRecvOutcome::Continue);
    recv.wake_previous_and_prepare(None, park_id);
    recv.register_iteration();
    let result = recv.recheck(QueueRead::Value(value), &open);
    assert(result == TryRecvOutcome::Value(value));
    accounting.receive_value();
    assert(accounting.encoded() == 0 && accounting.messages() == 0);
}

pub fn verify_try_recv_empty_priorities_after_busy(park_id: u64)
{
    let reserved = RecvEnvironment::new(false, true, false);
    let mut pending_close = MpscTryRecvRefinement::new();
    let initial = pending_close.initial_observe::<u64>(QueueRead::Busy, &reserved);
    assert(initial == TryRecvOutcome::Continue);
    pending_close.wake_previous_and_prepare(None, park_id);
    pending_close.register_iteration();
    let still_empty = pending_close.recheck::<u64>(QueueRead::Empty, &reserved);
    assert(still_empty == TryRecvOutcome::Empty);

    let idle = RecvEnvironment::new(true, true, false);
    let mut closed = MpscTryRecvRefinement::new();
    let closed_result = closed.initial_observe::<u64>(QueueRead::Empty, &idle);
    assert(closed_result == TryRecvOutcome::Disconnected);
}

} // verus!
