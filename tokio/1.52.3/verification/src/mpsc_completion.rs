use vstd::prelude::*;

verus! {

/// Arbitrary-length logical ownership of production mpsc block cells. Raw
/// pointers and allocation are a common adapter; no slot can be ready,
/// consumed, or reclaimed without the corresponding linear transfer here.
pub tracked struct MpscBlockOwnership {
    ghost claimed: Set<nat>,
    ghost ready: Set<nat>,
    ghost consumed: Set<nat>,
    ghost reclaimed_blocks: Set<nat>,
    ghost next_claim: nat,
    ghost head: nat,
    ghost block_capacity: nat,
    ghost close_at: Option<nat>,
}

impl MpscBlockOwnership {
    pub closed spec fn claimed(&self) -> Set<nat> { self.claimed }
    pub closed spec fn ready(&self) -> Set<nat> { self.ready }
    pub closed spec fn consumed(&self) -> Set<nat> { self.consumed }
    pub closed spec fn reclaimed_blocks(&self) -> Set<nat> { self.reclaimed_blocks }
    pub closed spec fn next_claim(&self) -> nat { self.next_claim }
    pub closed spec fn head(&self) -> nat { self.head }
    pub closed spec fn block_capacity(&self) -> nat { self.block_capacity }
    pub closed spec fn close_at(&self) -> Option<nat> { self.close_at }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.block_capacity() > 0
        &&& self.ready().subset_of(self.claimed())
        &&& self.consumed().subset_of(self.claimed())
        &&& self.ready().disjoint(self.consumed())
        &&& forall|i: nat| self.claimed().contains(i) ==> i < self.next_claim()
        &&& forall|i: nat| self.consumed().contains(i) ==> i < self.head()
        &&& self.head() <= self.next_claim()
        &&& match self.close_at() {
            Some(i) => i == self.next_claim(),
            None => true,
        }
    }

    pub proof fn new(block_capacity: nat) -> (tracked result: Self)
        requires block_capacity > 0,
        ensures
            result.well_formed(),
            result.claimed().is_empty(),
            result.ready().is_empty(),
            result.consumed().is_empty(),
            result.reclaimed_blocks().is_empty(),
            result.next_claim() == 0,
            result.head() == 0,
            result.block_capacity() == block_capacity,
            result.close_at().is_none(),
    {
        let tracked result = MpscBlockOwnership {
            claimed: Set::empty(),
            ready: Set::empty(),
            consumed: Set::empty(),
            reclaimed_blocks: Set::empty(),
            next_claim: 0,
            head: 0,
            block_capacity,
            close_at: None,
        };
        result
    }

    pub proof fn claim(tracked &mut self) -> (index: nat)
        requires
            old(self).well_formed(),
            old(self).close_at().is_none(),
        ensures
            final(self).well_formed(),
            index == old(self).next_claim(),
            final(self).next_claim() == old(self).next_claim() + 1,
            final(self).claimed() == old(self).claimed().insert(index),
            final(self).ready() == old(self).ready(),
            final(self).consumed() == old(self).consumed(),
            final(self).head() == old(self).head(),
            final(self).block_capacity() == old(self).block_capacity(),
            final(self).close_at() == old(self).close_at(),
            final(self).reclaimed_blocks() == old(self).reclaimed_blocks(),
    {
        let index = self.next_claim;
        self.next_claim = self.next_claim + 1;
        self.claimed = self.claimed.insert(index);
        index
    }

    pub proof fn publish(tracked &mut self, index: nat)
        requires
            old(self).well_formed(),
            old(self).claimed().contains(index),
            !old(self).ready().contains(index),
            !old(self).consumed().contains(index),
        ensures
            final(self).well_formed(),
            final(self).ready() == old(self).ready().insert(index),
            final(self).claimed() == old(self).claimed(),
            final(self).consumed() == old(self).consumed(),
            final(self).next_claim() == old(self).next_claim(),
            final(self).head() == old(self).head(),
            final(self).close_at() == old(self).close_at(),
            final(self).block_capacity() == old(self).block_capacity(),
            final(self).reclaimed_blocks() == old(self).reclaimed_blocks(),
    {
        self.ready = self.ready.insert(index);
    }

    pub proof fn pop_ready(tracked &mut self) -> (index: nat)
        requires
            old(self).well_formed(),
            old(self).ready().contains(old(self).head()),
        ensures
            final(self).well_formed(),
            index == old(self).head(),
            final(self).head() == old(self).head() + 1,
            final(self).ready() == old(self).ready().remove(index),
            final(self).consumed() == old(self).consumed().insert(index),
            final(self).claimed() == old(self).claimed(),
            final(self).next_claim() == old(self).next_claim(),
            final(self).close_at() == old(self).close_at(),
            final(self).block_capacity() == old(self).block_capacity(),
            final(self).reclaimed_blocks() == old(self).reclaimed_blocks(),
    {
        let index = self.head;
        self.ready = self.ready.remove(index);
        self.consumed = self.consumed.insert(index);
        self.head = self.head + 1;
        index
    }

    pub proof fn close(tracked &mut self)
        requires
            old(self).well_formed(),
            old(self).close_at().is_none(),
        ensures
            final(self).well_formed(),
            final(self).close_at() == Some(old(self).next_claim()),
            final(self).claimed() == old(self).claimed(),
            final(self).ready() == old(self).ready(),
            final(self).consumed() == old(self).consumed(),
            final(self).head() == old(self).head(),
    {
        self.close_at = Some(self.next_claim);
    }

    pub proof fn reclaim_block(tracked &mut self, block: nat)
        requires
            old(self).well_formed(),
            !old(self).reclaimed_blocks().contains(block),
            forall|offset: nat| offset < old(self).block_capacity() ==>
                #[trigger] old(self).consumed().contains(
                    block * old(self).block_capacity() + offset,
                ),
        ensures
            final(self).well_formed(),
            final(self).reclaimed_blocks() == old(self).reclaimed_blocks().insert(block),
            final(self).claimed() == old(self).claimed(),
            final(self).ready() == old(self).ready(),
            final(self).consumed() == old(self).consumed(),
            final(self).head() == old(self).head(),
    {
        self.reclaimed_blocks = self.reclaimed_blocks.insert(block);
    }
}

/// Linear batch returned by reserve_many/permit iterators. It is independent
/// of semaphore implementation and proves that every acquired permit is
/// committed or returned exactly once.
pub struct PermitBatch {
    acquired: u64,
    remaining: u64,
    committed: u64,
    returned: u64,
}

impl PermitBatch {
    pub closed spec fn acquired(&self) -> u64 { self.acquired }
    pub closed spec fn remaining(&self) -> u64 { self.remaining }
    pub closed spec fn committed(&self) -> u64 { self.committed }
    pub closed spec fn returned(&self) -> u64 { self.returned }

    pub closed spec fn well_formed(&self) -> bool {
        self.remaining() + self.committed() + self.returned() == self.acquired()
    }

    pub fn new(acquired: u64) -> (result: Self)
        ensures
            result.well_formed(),
            result.acquired() == acquired,
            result.remaining() == acquired,
            result.committed() == 0,
            result.returned() == 0,
        no_unwind
    {
        PermitBatch { acquired, remaining: acquired, committed: 0, returned: 0 }
    }

    pub fn send_one(&mut self) -> (sent: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            sent == (old(self).remaining() > 0),
            sent ==> final(self).remaining() + 1 == old(self).remaining(),
            sent ==> final(self).committed() == old(self).committed() + 1,
            !sent ==> final(self).remaining() == old(self).remaining(),
            final(self).returned() == old(self).returned(),
        no_unwind
    {
        if self.remaining > 0 {
            self.remaining -= 1;
            self.committed += 1;
            true
        } else {
            false
        }
    }

    pub fn cancel_remaining(&mut self) -> (returned_now: u64)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            returned_now == old(self).remaining(),
            final(self).remaining() == 0,
            final(self).committed() == old(self).committed(),
            final(self).returned() == old(self).returned() + returned_now,
        no_unwind
    {
        let returned_now = self.remaining;
        self.remaining = 0;
        self.returned += returned_now;
        returned_now
    }
}

/// Strong/weak Sender state, including the no-resurrection rule for upgrade.
pub struct MpscSenderCounts {
    strong: u64,
    weak: u64,
}

impl MpscSenderCounts {
    pub closed spec fn strong(&self) -> u64 { self.strong }
    pub closed spec fn weak(&self) -> u64 { self.weak }

    pub fn new() -> (result: Self)
        ensures result.strong() == 1, result.weak() == 0,
        no_unwind
    {
        MpscSenderCounts { strong: 1, weak: 0 }
    }

    pub fn downgrade(&mut self)
        requires old(self).weak() < u64::MAX,
        ensures final(self).strong() == old(self).strong(),
            final(self).weak() == old(self).weak() + 1,
        no_unwind
    {
        self.weak += 1;
    }

    pub fn drop_strong(&mut self) -> (last: bool)
        requires old(self).strong() > 0,
        ensures final(self).strong() + 1 == old(self).strong(),
            final(self).weak() == old(self).weak(),
            last == (old(self).strong() == 1),
        no_unwind
    {
        self.strong -= 1;
        self.strong == 0
    }

    pub fn upgrade(&mut self) -> (success: bool)
        requires old(self).strong() < u64::MAX,
        ensures
            success == (old(self).strong() > 0),
            success ==> final(self).strong() == old(self).strong() + 1,
            !success ==> final(self).strong() == 0,
            final(self).weak() == old(self).weak(),
        no_unwind
    {
        if self.strong == 0 {
            false
        } else {
            self.strong += 1;
            true
        }
    }
}

pub proof fn verify_mpsc_out_of_order_ready_is_blocked()
{
    let tracked mut queue = MpscBlockOwnership::new(2);
    let first = queue.claim();
    let second = queue.claim();
    queue.publish(second);
    assert(queue.claimed().contains(first));
    assert(!queue.ready().contains(first));
    assert(queue.ready().contains(second));
    queue.publish(first);
    let popped_first = queue.pop_ready();
    assert(popped_first == first);
    let popped_second = queue.pop_ready();
    assert(popped_second == second);
}

pub fn verify_permit_batch_conservation(count: u64)
{
    let mut batch = PermitBatch::new(count);
    if count > 0 {
        let sent = batch.send_one();
        assert(sent);
    }
    batch.cancel_remaining();
    assert(batch.remaining() == 0);
    assert(batch.committed() + batch.returned() == count);
}

pub fn verify_weak_cannot_resurrect()
{
    let mut counts = MpscSenderCounts::new();
    counts.downgrade();
    let last = counts.drop_strong();
    assert(last);
    let upgraded = counts.upgrade();
    assert(!upgraded);
    assert(counts.strong() == 0);
}

} // verus!
