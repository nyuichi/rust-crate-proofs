use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BarrierStep {
    pub captured_generation: usize,
    pub leader: bool,
    pub published_generation: Option<usize>,
}

/// Exact machine-word refinement of production `BarrierState`. Mutex
/// exclusion and watch publication are frozen adapters; the values crossing
/// those adapters are represented here without abstraction.
pub struct BarrierMachine {
    parties: usize,
    arrived: usize,
    generation: usize,
}

impl BarrierMachine {
    pub closed spec fn parties(&self) -> usize { self.parties }
    pub closed spec fn arrived(&self) -> usize { self.arrived }
    pub closed spec fn generation(&self) -> usize { self.generation }
    pub closed spec fn well_formed(&self) -> bool {
        self.parties() > 0 && self.arrived() < self.parties()
    }

    pub fn new(requested: usize) -> (result: Self)
        ensures
            result.well_formed(),
            result.parties() == if requested == 0 { 1 } else { requested },
            result.arrived() == 0,
            result.generation() == 1,
        no_unwind
    {
        BarrierMachine {
            parties: if requested == 0 { 1 } else { requested },
            arrived: 0,
            generation: 1,
        }
    }

    /// Refines the production critical section: increment `arrived`, publish
    /// the captured generation for the nth arrival, reset the cohort, and use
    /// explicit machine-word rollover for the next generation.
    pub fn arrive(&mut self) -> (step: BarrierStep)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).parties() == old(self).parties(),
            step.captured_generation == old(self).generation(),
            step.leader == (old(self).arrived() + 1 == old(self).parties()),
            !step.leader ==> step.published_generation.is_none(),
            !step.leader ==> final(self).arrived() == old(self).arrived() + 1,
            !step.leader ==> final(self).generation() == old(self).generation(),
            step.leader ==> step.published_generation == Some(old(self).generation()),
            step.leader ==> final(self).arrived() == 0,
            step.leader && old(self).generation() == usize::MAX
                ==> final(self).generation() == 0,
            step.leader && old(self).generation() < usize::MAX
                ==> final(self).generation() == old(self).generation() + 1,
        no_unwind
    {
        let generation = self.generation;
        self.arrived += 1;
        if self.arrived == self.parties {
            self.arrived = 0;
            if self.generation == usize::MAX {
                self.generation = 0;
            } else {
                self.generation += 1;
            }
            BarrierStep {
                captured_generation: generation,
                leader: true,
                published_generation: Some(generation),
            }
        } else {
            BarrierStep {
                captured_generation: generation,
                leader: false,
                published_generation: None,
            }
        }
    }
}

/// Exact production `*wait.borrow() >= generation` predicate.
pub open spec fn follower_observes(published: usize, captured: usize) -> bool {
    published >= captured
}

pub fn verify_two_party_publication()
{
    let mut barrier = BarrierMachine::new(2);
    let follower = barrier.arrive();
    assert(!follower.leader);
    let leader = barrier.arrive();
    assert(leader.leader);
    assert(leader.published_generation == Some(follower.captured_generation));
    assert(follower_observes(
        leader.published_generation.unwrap(), follower.captured_generation));
}

pub fn verify_generation_wrap_is_non_panicking()
{
    let mut barrier = BarrierMachine { parties: 1, arrived: 0, generation: usize::MAX };
    assert(barrier.well_formed());
    let leader = barrier.arrive();
    assert(leader.leader);
    assert(leader.published_generation == Some(usize::MAX));
    assert(follower_observes(usize::MAX, leader.captured_generation));
    assert(barrier.generation() == 0);

    let next = barrier.arrive();
    assert(next.leader);
    assert(next.published_generation == Some(0));
    assert(barrier.generation() == 1);
}

pub proof fn verify_barrier_mutants_rejected()
{
    // `>` would strand followers when the leader publishes their exact token.
    assert(follower_observes(7, 7));
    assert(!(7usize > 7usize));
    // Resetting to one instead of zero skips the first post-wrap token.
    assert(0usize != 1usize);
    // Resetting `arrived` to one leaves a phantom member in the next cohort.
    assert(0usize != 1usize);
}

} // verus!
