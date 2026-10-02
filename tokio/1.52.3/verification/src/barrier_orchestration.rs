use crate::barrier_refinement::{BarrierMachine, BarrierStep};
use vstd::prelude::*;

verus! {

pub const NOTIFY_LIMIT: usize = usize::MAX / 4;
pub const UPDATE_LIMIT: usize = NOTIFY_LIMIT - 1;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CohortArrival {
    Admitted(BarrierStep),
    TerminalRejected,
}

/// Erased value mapping for production `BarrierWaitResult(bool)`.
pub struct BarrierWaitResultModel {
    leader: bool,
}

impl BarrierWaitResultModel {
    pub closed spec fn view(&self) -> bool { self.leader }

    pub fn from_step(step: BarrierStep) -> (result: Self)
        ensures result.view() == step.leader,
        no_unwind
    {
        BarrierWaitResultModel { leader: step.leader }
    }

    pub fn is_leader(&self) -> (result: bool)
        ensures result == self.view(),
        no_unwind
    {
        self.leader
    }

    pub fn clone_result(&self) -> (result: Self)
        ensures result.view() == self.view(),
        no_unwind
    {
        BarrierWaitResultModel { leader: self.leader }
    }
}

/// Thin composition of the production Barrier critical section with S03's
/// finite watch-update and Receiver-construction budgets. Mutex execution is
/// the frozen adapter; all state transitions around it are represented here.
pub struct BarrierOrchestration {
    machine: BarrierMachine,
    update_generation: usize,
    receiver_reservations: usize,
    reserved_receivers: usize,
}

impl BarrierOrchestration {
    pub closed spec fn parties(&self) -> usize { self.machine.parties() }
    pub closed spec fn arrived(&self) -> usize { self.machine.arrived() }
    pub closed spec fn generation(&self) -> usize { self.machine.generation() }
    pub closed spec fn update_generation(&self) -> usize { self.update_generation }
    pub closed spec fn receiver_reservations(&self) -> usize {
        self.receiver_reservations
    }
    pub closed spec fn reserved_receivers(&self) -> usize { self.reserved_receivers }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.machine.well_formed()
        &&& self.update_generation() <= UPDATE_LIMIT
        &&& self.receiver_reservations() <= NOTIFY_LIMIT
        &&& self.generation() as int == self.update_generation() as int + 1
        &&& self.reserved_receivers()
            == if self.arrived() == 0 {
                0
            } else {
                self.parties() - self.arrived() - 1
            }
        &&& self.arrived() > 0 ==> self.update_generation() < UPDATE_LIMIT
    }

    pub fn new(requested: usize) -> (result: Self)
        ensures
            result.well_formed(),
            result.parties() == if requested == 0 { 1 } else { requested },
            result.arrived() == 0,
            result.update_generation() == 0,
            result.receiver_reservations() == 1,
        no_unwind
    {
        BarrierOrchestration {
            machine: BarrierMachine::new(requested),
            update_generation: 0,
            receiver_reservations: 1,
            reserved_receivers: 0,
        }
    }

    /// Proof fixture for a clean cohort boundary at a reviewed finite-resource
    /// position. Production reaches these states only through earlier complete
    /// cohorts; this constructor is not part of Tokio's runtime surface.
    pub fn boundary_with_resources(
        parties: usize,
        update_generation: usize,
        receiver_reservations: usize,
    ) -> (result: Self)
        requires
            parties > 0,
            update_generation <= UPDATE_LIMIT,
            receiver_reservations <= NOTIFY_LIMIT,
        ensures
            result.well_formed(),
            result.parties() == parties,
            result.arrived() == 0,
            result.update_generation() == update_generation,
            result.receiver_reservations() == receiver_reservations,
        no_unwind
    {
        BarrierOrchestration {
            machine: BarrierMachine::at_boundary(parties, update_generation + 1),
            update_generation,
            receiver_reservations,
            reserved_receivers: 0,
        }
    }

    /// Production-shaped arrival. At a cohort boundary it first claims the
    /// one update and `parties - 1` Receiver credits needed by the whole
    /// cohort. Failure is mutation-free. A follower consumes one preclaimed
    /// Receiver credit before its arrival is committed; the leader consumes
    /// the promised update and returns the exact BarrierMachine step.
    pub fn arrive(&mut self) -> (result: CohortArrival)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).parties() == old(self).parties(),
            result == CohortArrival::TerminalRejected ==>
                final(self).arrived() == old(self).arrived()
                && final(self).generation() == old(self).generation()
                && final(self).update_generation() == old(self).update_generation()
                && final(self).receiver_reservations()
                    == old(self).receiver_reservations()
                && final(self).reserved_receivers() == old(self).reserved_receivers(),
            result == CohortArrival::TerminalRejected ==> old(self).arrived() == 0,
            old(self).arrived() == 0
                && old(self).update_generation() < UPDATE_LIMIT
                && old(self).parties() - 1
                    <= NOTIFY_LIMIT - old(self).receiver_reservations()
                ==> result != CohortArrival::TerminalRejected,
            result != CohortArrival::TerminalRejected ==> match result {
                CohortArrival::Admitted(step) => {
                    &&& step.captured_generation == old(self).generation()
                    &&& step.leader == (old(self).arrived() + 1 == old(self).parties())
                    &&& step.leader ==> step.published_generation
                        == Some(old(self).generation())
                    &&& !step.leader ==> step.published_generation.is_none()
                    &&& step.leader ==> final(self).arrived() == 0
                    &&& !step.leader ==> final(self).arrived() == old(self).arrived() + 1
                    &&& !step.leader ==> final(self).generation() == old(self).generation()
                    &&& step.leader ==> final(self).update_generation()
                        == old(self).update_generation() + 1
                    &&& !step.leader ==> final(self).update_generation()
                        == old(self).update_generation()
                    &&& old(self).arrived() == 0 ==> final(self).receiver_reservations()
                        == old(self).receiver_reservations() + old(self).parties() - 1
                    &&& old(self).arrived() > 0 ==> final(self).receiver_reservations()
                        == old(self).receiver_reservations()
                },
                CohortArrival::TerminalRejected => false,
            },
        no_unwind
    {
        assert(self.well_formed());
        assert(self.machine.well_formed());
        if self.machine.arrived_value() == 0 {
            let parties = self.machine.parties_value();
            assert(parties > 0);
            let receiver_credits = parties - 1;
            let update_exhausted = self.update_generation == UPDATE_LIMIT;
            let receiver_exhausted =
                receiver_credits > NOTIFY_LIMIT - self.receiver_reservations;
            if update_exhausted || receiver_exhausted {
                return CohortArrival::TerminalRejected;
            }
            self.receiver_reservations += receiver_credits;
            self.reserved_receivers = receiver_credits;
        }

        let leader = self.machine.arrived_value() + 1 == self.machine.parties_value();
        if !leader {
            // Matches production: consume and construct the reserved Receiver
            // before committing the corresponding Barrier arrival.
            assert(self.reserved_receivers > 0);
            self.reserved_receivers -= 1;
        }

        let step = self.machine.arrive();
        assert(step.leader == leader);
        if leader {
            assert(self.reserved_receivers == 0);
            self.update_generation += 1;
        }
        CohortArrival::Admitted(step)
    }
}

pub fn verify_zero_and_one_party_surface()
{
    let mut zero = BarrierOrchestration::new(0);
    let zero_result = zero.arrive();
    assert(matches!(zero_result, CohortArrival::Admitted(step) if step.leader));

    let mut one = BarrierOrchestration::new(1);
    let one_result = one.arrive();
    assert(matches!(one_result, CohortArrival::Admitted(step) if step.leader));
}

pub fn verify_three_party_clone_credits_and_publication()
{
    let mut barrier = BarrierOrchestration::new(3);
    let first = barrier.arrive();
    let second = barrier.arrive();
    let third = barrier.arrive();
    assert(matches!(first, CohortArrival::Admitted(step) if !step.leader));
    assert(matches!(second, CohortArrival::Admitted(step) if !step.leader));
    assert(matches!(third, CohortArrival::Admitted(step) if step.leader));
    assert(barrier.reserved_receivers() == 0);
    assert(barrier.receiver_reservations() == 3);
    match (first, third) {
        (CohortArrival::Admitted(follower), CohortArrival::Admitted(leader)) => {
            assert(leader.published_generation == Some(follower.captured_generation));
            assert(crate::barrier_refinement::follower_observes(
                leader.published_generation.unwrap(), follower.captured_generation));
        },
        _ => { assert(false); },
    }
}

pub fn verify_last_three_party_cohort_and_terminal_rejection()
{
    let mut barrier = BarrierOrchestration::boundary_with_resources(
        3,
        UPDATE_LIMIT - 1,
        NOTIFY_LIMIT - 2,
    );
    let first = barrier.arrive();
    let second = barrier.arrive();
    let third = barrier.arrive();
    assert(matches!(first, CohortArrival::Admitted(step) if !step.leader));
    assert(matches!(second, CohortArrival::Admitted(step) if !step.leader));
    assert(matches!(third, CohortArrival::Admitted(step) if step.leader));
    assert(barrier.arrived() == 0);
    assert(barrier.update_generation() == UPDATE_LIMIT);
    assert(barrier.receiver_reservations() == NOTIFY_LIMIT);

    let ghost generation = barrier.generation();
    let rejected = barrier.arrive();
    assert(rejected == CohortArrival::TerminalRejected);
    assert(barrier.arrived() == 0);
    assert(barrier.generation() == generation);
    assert(barrier.update_generation() == UPDATE_LIMIT);
    assert(barrier.receiver_reservations() == NOTIFY_LIMIT);
}

pub fn verify_receiver_shortage_rejects_before_three_party_arrival()
{
    let mut barrier = BarrierOrchestration::boundary_with_resources(
        3,
        UPDATE_LIMIT - 1,
        NOTIFY_LIMIT - 1,
    );
    let ghost generation = barrier.generation();
    let rejected = barrier.arrive();
    assert(rejected == CohortArrival::TerminalRejected);
    assert(barrier.arrived() == 0);
    assert(barrier.generation() == generation);
    assert(barrier.update_generation() == UPDATE_LIMIT - 1);
    assert(barrier.receiver_reservations() == NOTIFY_LIMIT - 1);
}

pub fn verify_public_result_mapping()
{
    let mut barrier = BarrierOrchestration::new(2);
    let follower = barrier.arrive();
    let leader = barrier.arrive();
    match (follower, leader) {
        (CohortArrival::Admitted(follower_step), CohortArrival::Admitted(leader_step)) => {
            let follower_result = BarrierWaitResultModel::from_step(follower_step);
            let leader_result = BarrierWaitResultModel::from_step(leader_step);
            let follower_is_leader = follower_result.is_leader();
            let leader_is_leader = leader_result.is_leader();
            assert(!follower_is_leader);
            assert(leader_is_leader);
            let cloned = leader_result.clone_result();
            let cloned_is_leader = cloned.is_leader();
            assert(cloned_is_leader == leader_is_leader);
        },
        _ => { assert(false); },
    }
}

pub fn verify_terminal_gate_prevents_generation_reuse()
{
    let mut barrier = BarrierOrchestration::boundary_with_resources(
        1,
        UPDATE_LIMIT,
        NOTIFY_LIMIT,
    );
    assert(barrier.generation() as int == UPDATE_LIMIT as int + 1);
    let ghost generation = barrier.generation();
    let rejected = barrier.arrive();
    assert(rejected == CohortArrival::TerminalRejected);
    assert(barrier.generation() == generation);
    assert(barrier.generation() > 0);
}

} // verus!
