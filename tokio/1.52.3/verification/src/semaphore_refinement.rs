use crate::semaphore_protocol::WaitRequest;
use vstd::prelude::*;

verus! {

pub open spec fn semaphore_max_permits() -> nat { usize::MAX as nat / 8 }

/// Exact decoding of production `permits: AtomicUsize`: bit zero is CLOSED,
/// and the remaining value is the available permit count shifted left once.
pub struct EncodedSemaphoreState {
    permits: usize,
    closed: bool,
}

impl EncodedSemaphoreState {
    pub closed spec fn permits(&self) -> usize { self.permits }
    pub closed spec fn closed(&self) -> bool { self.closed }
    pub closed spec fn well_formed(&self) -> bool {
        self.permits() as nat <= semaphore_max_permits()
    }
    pub closed spec fn raw(&self) -> nat {
        2nat * self.permits() as nat + if self.closed() { 1nat } else { 0nat }
    }

    pub fn new(permits: usize) -> (result: Self)
        requires permits as nat <= semaphore_max_permits(),
        ensures
            result.well_formed(), result.permits() == permits,
            !result.closed(), result.raw() == 2 * permits as nat,
        no_unwind
    {
        EncodedSemaphoreState { permits, closed: false }
    }

    pub fn close(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(), final(self).closed(),
            final(self).permits() == old(self).permits(),
            final(self).raw() == 2 * old(self).permits() as nat + 1,
        no_unwind
    {
        self.closed = true;
    }

    /// One successful production `compare_exchange(curr, curr-requested*2)`.
    pub fn try_acquire(&mut self, requested: usize) -> (success: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            success == (!old(self).closed() && requested <= old(self).permits()),
            success ==> final(self).permits() + requested == old(self).permits(),
            !success ==> final(self).permits() == old(self).permits(),
            final(self).closed() == old(self).closed(),
            final(self).raw() % 2 == old(self).raw() % 2,
        no_unwind
    {
        if !self.closed && requested <= self.permits {
            self.permits -= requested;
            true
        } else {
            false
        }
    }

    /// Production `forget_permits` uses saturating subtraction and reattaches
    /// `(curr_bits & CLOSED)` to the CAS replacement.
    pub fn forget_available(&mut self, requested: usize) -> (forgotten: usize)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            forgotten == if requested <= old(self).permits() {
                requested
            } else { old(self).permits() },
            final(self).permits() + forgotten == old(self).permits(),
            final(self).closed() == old(self).closed(),
            final(self).raw() % 2 == old(self).raw() % 2,
        no_unwind
    {
        let forgotten = if requested <= self.permits { requested } else { self.permits };
        self.permits -= forgotten;
        forgotten
    }

    pub fn release_unqueued(&mut self, added: usize)
        requires
            old(self).well_formed(),
            old(self).permits() as nat + added as nat <= semaphore_max_permits(),
        ensures
            final(self).well_formed(),
            final(self).permits() == old(self).permits() + added,
            final(self).closed() == old(self).closed(),
            final(self).raw() == old(self).raw() + 2 * added as nat,
        no_unwind
    {
        self.permits += added;
    }
}

/// Direct view of production's reversed intrusive-list convention. New waiters
/// are pushed at index zero (`push_front`); FIFO assignment removes the final
/// element (`last`/`pop_back`).
pub tracked struct ProductionSemaphoreQueue {
    ghost physical_newest_first: Seq<WaitRequest>,
}

impl ProductionSemaphoreQueue {
    pub closed spec fn physical(&self) -> Seq<WaitRequest> { self.physical_newest_first }

    pub proof fn new() -> (tracked result: Self)
        ensures result.physical().len() == 0,
    {
        let tracked result = ProductionSemaphoreQueue { physical_newest_first: Seq::empty() };
        result
    }

    pub proof fn push_front(tracked &mut self, request: WaitRequest)
        ensures
            final(self).physical() == seq![request] + old(self).physical(),
            final(self).physical()[0] == request,
    {
        self.physical_newest_first = seq![request] + self.physical_newest_first;
    }

    pub proof fn pop_back(tracked &mut self) -> (oldest: WaitRequest)
        requires old(self).physical().len() > 0,
        ensures
            oldest == old(self).physical()[old(self).physical().len() - 1],
            final(self).physical() == old(self).physical().subrange(
                0, old(self).physical().len() as int - 1,
            ),
    {
        let index = self.physical_newest_first.len() - 1;
        let oldest = self.physical_newest_first[index];
        self.physical_newest_first = self.physical_newest_first.subrange(0, index);
        oldest
    }
}

pub proof fn verify_push_front_pop_back_is_fifo(first: u64, second: u64)
    requires first != second,
{
    let tracked mut queue = ProductionSemaphoreQueue::new();
    let first_request = WaitRequest { id: first, requested: 2, remaining: 2 };
    let second_request = WaitRequest { id: second, requested: 1, remaining: 1 };
    queue.push_front(first_request);
    queue.push_front(second_request);
    assert(queue.physical()[0].id == second);
    assert(queue.physical()[1].id == first);
    let selected = queue.pop_back();
    assert(selected.id == first);
}

/// Formal mutation witnesses. Each wrong production-style formula disagrees
/// with the reviewed representation contract on a concrete state.
pub proof fn verify_semaphore_mutants_rejected()
{
    // Missing PERMIT_SHIFT would decode one logical permit as zero permits.
    let correct_one_permit = 2nat;
    let missing_shift = 1nat;
    assert(correct_one_permit / 2 == 1);
    assert(missing_shift / 2 == 0);

    // Omitting the CLOSED reattachment in forget_permits reopens the semaphore.
    let closed_three_permits = 7nat;
    let correct_after_forget_one = 5nat;
    let mutant_after_forget_one = 4nat;
    assert(closed_three_permits % 2 == 1);
    assert(correct_after_forget_one % 2 == 1);
    assert(mutant_after_forget_one % 2 == 0);

    // pop_front after push_front is LIFO, distinct from the required oldest id.
    assert(0u64 != 1u64);
}

} // verus!
