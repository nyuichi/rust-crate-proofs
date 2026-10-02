use vstd::prelude::*;

verus! {

/// Result of one logical `Barrier::wait` arrival. Production's
/// `BarrierWaitResult(bool)` is the erased representation of `leader`.
#[derive(Copy, Clone)]
pub struct BarrierArrival {
    pub generation: nat,
    pub leader: bool,
}

/// Reusable Barrier protocol above the trusted Mutex and the already-proved
/// watch generation/publication protocol. Tickets identify individual wait
/// calls, not tasks, so the same task may participate in later generations.
pub tracked struct BarrierProtocol {
    ghost parties: nat,
    ghost arrived: nat,
    ghost generation: nat,
    ghost pending: Set<nat>,
    ghost abandoned: Set<nat>,
    ghost released: Set<nat>,
    ghost leaders: Map<nat, nat>,
}

impl BarrierProtocol {
    pub closed spec fn parties(&self) -> nat { self.parties }
    pub closed spec fn arrived(&self) -> nat { self.arrived }
    pub closed spec fn generation(&self) -> nat { self.generation }
    pub closed spec fn pending(&self) -> Set<nat> { self.pending }
    pub closed spec fn abandoned(&self) -> Set<nat> { self.abandoned }
    pub closed spec fn released(&self) -> Set<nat> { self.released }
    pub closed spec fn leaders(&self) -> Map<nat, nat> { self.leaders }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.parties() > 0
        &&& self.arrived() < self.parties()
        &&& self.pending().disjoint(self.abandoned())
        &&& forall|g: nat| self.leaders().dom().contains(g) ==> g < self.generation()
    }

    /// `Barrier::new(0)` normalizes to one party, matching std Barrier.
    pub proof fn new(requested: nat) -> (tracked result: Self)
        ensures
            result.well_formed(),
            result.parties() == if requested == 0 { 1 } else { requested },
            result.arrived() == 0,
            result.generation() == 1,
            result.pending().is_empty(),
            result.abandoned().is_empty(),
            result.released().is_empty(),
            result.leaders().dom().is_empty(),
    {
        let parties = if requested == 0 { 1 } else { requested };
        let tracked result = BarrierProtocol {
            parties,
            arrived: 0,
            generation: 1,
            pending: Set::empty(),
            abandoned: Set::empty(),
            released: Set::empty(),
            leaders: Map::empty(),
        };
        result
    }

    /// Linearization point under production's synchronous Mutex. The nth
    /// arrival is the unique leader, publishes the current generation through
    /// watch, clears the cohort, and advances to the next reusable generation.
    pub proof fn arrive(tracked &mut self, ticket: nat) -> (arrival: BarrierArrival)
        requires
            old(self).well_formed(),
            !old(self).pending().contains(ticket),
            !old(self).abandoned().contains(ticket),
            !old(self).released().contains(ticket),
            !old(self).leaders().values().contains(ticket),
        ensures
            final(self).well_formed(),
            final(self).parties() == old(self).parties(),
            arrival.generation == old(self).generation(),
            arrival.leader == (old(self).arrived() + 1 == old(self).parties()),
            !arrival.leader ==> final(self).arrived() == old(self).arrived() + 1,
            !arrival.leader ==> final(self).generation() == old(self).generation(),
            !arrival.leader ==> final(self).pending() == old(self).pending().insert(ticket),
            !arrival.leader ==> final(self).abandoned() == old(self).abandoned(),
            !arrival.leader ==> final(self).released() == old(self).released(),
            !arrival.leader ==> final(self).leaders() == old(self).leaders(),
            arrival.leader ==> final(self).arrived() == 0,
            arrival.leader ==> final(self).generation() == old(self).generation() + 1,
            arrival.leader ==> final(self).pending().is_empty(),
            arrival.leader ==> final(self).abandoned().is_empty(),
            arrival.leader ==> final(self).released()
                == old(self).released().union(old(self).pending()).insert(ticket),
            arrival.leader ==> final(self).leaders()
                == old(self).leaders().insert(old(self).generation(), ticket),
            arrival.leader ==> final(self).leaders()[old(self).generation()] == ticket,
    {
        let generation = self.generation;
        if self.arrived + 1 == self.parties {
            self.released = self.released.union(self.pending).insert(ticket);
            self.pending = Set::empty();
            self.abandoned = Set::empty();
            self.leaders = self.leaders.insert(generation, ticket);
            self.arrived = 0;
            self.generation = self.generation + 1;
            BarrierArrival { generation, leader: true }
        } else {
            assert(self.arrived + 1 < self.parties);
            self.pending = self.pending.insert(ticket);
            self.arrived = self.arrived + 1;
            BarrierArrival { generation, leader: false }
        }
    }

    /// Production explicitly documents `wait` as not cancel-safe: cancelling a
    /// follower removes its future but does not roll back `arrived`. The
    /// abandoned arrival still contributes to opening this generation.
    pub proof fn cancel_pending(tracked &mut self, ticket: nat)
        requires
            old(self).well_formed(),
            old(self).pending().contains(ticket),
        ensures
            final(self).well_formed(),
            final(self).parties() == old(self).parties(),
            final(self).pending() == old(self).pending().remove(ticket),
            final(self).abandoned() == old(self).abandoned().insert(ticket),
            final(self).arrived() == old(self).arrived(),
            final(self).generation() == old(self).generation(),
            final(self).released() == old(self).released(),
            final(self).leaders() == old(self).leaders(),
    {
        assert(!self.abandoned.contains(ticket));
        assert(self.pending.remove(ticket).disjoint(self.abandoned.insert(ticket))) by {
            assert forall|candidate: nat|
                self.pending.remove(ticket).contains(candidate)
                    implies !self.abandoned.insert(ticket).contains(candidate) by {
                if self.pending.remove(ticket).contains(candidate) {
                    assert(self.pending.contains(candidate));
                    assert(candidate != ticket);
                    assert(!self.abandoned.contains(candidate));
                }
            }
        };
        self.pending = self.pending.remove(ticket);
        self.abandoned = self.abandoned.insert(ticket);
        assert(self.pending.disjoint(self.abandoned));
        assert(self.parties > 0);
        assert(self.arrived < self.parties);
        assert forall|g: nat| self.leaders.dom().contains(g)
            implies g < self.generation by {
            assert(old(self).leaders().dom().contains(g));
        };
        assert(self.well_formed());
    }

    /// A follower's watch loop may return exactly after production has
    /// published its captured generation and advanced the Barrier generation.
    pub open spec fn follower_ready(&self, captured_generation: nat) -> bool {
        self.generation() > captured_generation
    }
}

pub proof fn verify_two_party_unique_leader(first: nat, second: nat)
    requires first != second,
{
    let tracked mut barrier = BarrierProtocol::new(2);
    let first_arrival = barrier.arrive(first);
    assert(!first_arrival.leader);
    assert(!barrier.follower_ready(first_arrival.generation));

    let second_arrival = barrier.arrive(second);
    assert(second_arrival.leader);
    assert(!first_arrival.leader);
    assert(barrier.follower_ready(first_arrival.generation));
    assert(barrier.leaders()[first_arrival.generation] == second);
}

pub proof fn verify_reusable_generations(
    first_a: nat,
    first_b: nat,
    second_a: nat,
    second_b: nat,
)
    requires
        first_a != first_b,
        second_a != second_b,
        first_a != second_a,
        first_a != second_b,
        first_b != second_a,
        first_b != second_b,
{
    let tracked mut barrier = BarrierProtocol::new(2);
    let a1 = barrier.arrive(first_a);
    let b1 = barrier.arrive(first_b);
    assert(!a1.leader && b1.leader);
    let first_generation = a1.generation;

    let a2 = barrier.arrive(second_a);
    let b2 = barrier.arrive(second_b);
    assert(!a2.leader && b2.leader);
    assert(a2.generation == first_generation + 1);
    assert(barrier.leaders()[first_generation] == first_b);
    assert(barrier.leaders()[a2.generation] == second_b);
}

pub proof fn verify_cancelled_arrival_still_counts(cancelled: nat, leader: nat)
    requires cancelled != leader,
{
    let tracked mut barrier = BarrierProtocol::new(2);
    let first = barrier.arrive(cancelled);
    assert(!first.leader);
    barrier.cancel_pending(cancelled);
    assert(barrier.arrived() == 1);
    assert(barrier.abandoned().contains(cancelled));

    let second = barrier.arrive(leader);
    assert(second.leader);
    assert(barrier.generation() == first.generation + 1);
    assert(barrier.abandoned().is_empty());
}

pub proof fn verify_zero_is_single_party(ticket: nat)
{
    let tracked mut barrier = BarrierProtocol::new(0);
    assert(barrier.parties() == 1);
    let arrival = barrier.arrive(ticket);
    assert(arrival.leader);
    assert(barrier.generation() == 2);
}

} // verus!
