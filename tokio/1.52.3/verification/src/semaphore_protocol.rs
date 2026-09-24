use vstd::prelude::*;

verus! {

/// Conservation model for `batch_semaphore`. The atomic counter, mutex, and
/// intrusive links are trusted representation adapters; all Tokio-level permit
/// transfers must preserve this equation.
pub struct SemaphoreAccounting {
    total: u64,
    available: u64,
    held: u64,
    escrowed: u64,
    forgotten: u64,
    closed: bool,
}

impl SemaphoreAccounting {
    pub closed spec fn total(&self) -> u64 { self.total }
    pub closed spec fn available(&self) -> u64 { self.available }
    pub closed spec fn held(&self) -> u64 { self.held }
    pub closed spec fn escrowed(&self) -> u64 { self.escrowed }
    pub closed spec fn forgotten(&self) -> u64 { self.forgotten }
    pub closed spec fn closed(&self) -> bool { self.closed }

    pub closed spec fn well_formed(&self) -> bool {
        self.available() + self.held() + self.escrowed() + self.forgotten()
            == self.total()
    }

    pub fn new(permits: u64) -> (result: Self)
        ensures
            result.well_formed(),
            result.total() == permits,
            result.available() == permits,
            result.held() == 0,
            result.escrowed() == 0,
            result.forgotten() == 0,
            !result.closed(),
        no_unwind
    {
        SemaphoreAccounting {
            total: permits,
            available: permits,
            held: 0,
            escrowed: 0,
            forgotten: 0,
            closed: false,
        }
    }

    /// Models both `try_acquire_many` and the uncontended poll fast path.
    pub fn try_acquire(&mut self, requested: u64) -> (success: bool)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            success == (!old(self).closed() && requested <= old(self).available()),
            success ==> final(self).available() + requested == old(self).available(),
            success ==> final(self).held() == old(self).held() + requested,
            !success ==> final(self).available() == old(self).available(),
            !success ==> final(self).held() == old(self).held(),
            final(self).escrowed() == old(self).escrowed(),
            final(self).forgotten() == old(self).forgotten(),
            final(self).total() == old(self).total(),
            final(self).closed() == old(self).closed(),
        no_unwind
    {
        if !self.closed && requested <= self.available {
            self.available -= requested;
            self.held += requested;
            true
        } else {
            false
        }
    }

    /// Moves immediately available permits into a queued waiter's escrow.
    pub fn escrow_available(&mut self, requested: u64) -> (assigned: u64)
        requires
            old(self).well_formed(),
            !old(self).closed(),
        ensures
            final(self).well_formed(),
            assigned == if requested <= old(self).available() {
                requested
            } else {
                old(self).available()
            },
            final(self).available() + assigned == old(self).available(),
            final(self).escrowed() == old(self).escrowed() + assigned,
            final(self).held() == old(self).held(),
            final(self).forgotten() == old(self).forgotten(),
            final(self).total() == old(self).total(),
        no_unwind
    {
        let assigned = if requested <= self.available { requested } else { self.available };
        self.available -= assigned;
        self.escrowed += assigned;
        assigned
    }

    /// Completion transfers a waiter's entire request from queue escrow to a
    /// public borrowed or owned permit.
    pub fn complete_waiter(&mut self, requested: u64)
        requires
            old(self).well_formed(),
            requested <= old(self).escrowed(),
        ensures
            final(self).well_formed(),
            final(self).escrowed() + requested == old(self).escrowed(),
            final(self).held() == old(self).held() + requested,
            final(self).available() == old(self).available(),
            final(self).forgotten() == old(self).forgotten(),
            final(self).total() == old(self).total(),
        no_unwind
    {
        self.escrowed -= requested;
        self.held += requested;
    }

    /// Dropping a queued future returns every partially assigned permit.
    pub fn cancel_waiter(&mut self, assigned: u64)
        requires
            old(self).well_formed(),
            assigned <= old(self).escrowed(),
        ensures
            final(self).well_formed(),
            final(self).escrowed() + assigned == old(self).escrowed(),
            final(self).available() == old(self).available() + assigned,
            final(self).held() == old(self).held(),
            final(self).forgotten() == old(self).forgotten(),
            final(self).total() == old(self).total(),
        no_unwind
    {
        self.escrowed -= assigned;
        self.available += assigned;
    }

    pub fn add_permits(&mut self, added: u64)
        requires
            old(self).well_formed(),
            old(self).total() <= u64::MAX - added,
            old(self).available() <= u64::MAX - added,
        ensures
            final(self).well_formed(),
            final(self).total() == old(self).total() + added,
            final(self).available() == old(self).available() + added,
            final(self).held() == old(self).held(),
            final(self).escrowed() == old(self).escrowed(),
            final(self).forgotten() == old(self).forgotten(),
        no_unwind
    {
        self.total += added;
        self.available += added;
    }

    pub fn drop_permit(&mut self, permits: u64)
        requires
            old(self).well_formed(),
            permits <= old(self).held(),
            old(self).available() <= u64::MAX - permits,
        ensures
            final(self).well_formed(),
            final(self).held() + permits == old(self).held(),
            final(self).available() == old(self).available() + permits,
            final(self).escrowed() == old(self).escrowed(),
            final(self).forgotten() == old(self).forgotten(),
            final(self).total() == old(self).total(),
        no_unwind
    {
        self.held -= permits;
        self.available += permits;
    }

    pub fn forget_held(&mut self, permits: u64)
        requires
            old(self).well_formed(),
            permits <= old(self).held(),
            old(self).forgotten() <= u64::MAX - permits,
        ensures
            final(self).well_formed(),
            final(self).held() + permits == old(self).held(),
            final(self).forgotten() == old(self).forgotten() + permits,
            final(self).available() == old(self).available(),
            final(self).escrowed() == old(self).escrowed(),
            final(self).total() == old(self).total(),
        no_unwind
    {
        self.held -= permits;
        self.forgotten += permits;
    }

    /// `forget_permits` removes at most the currently available count.
    pub fn forget_available(&mut self, requested: u64) -> (forgotten_now: u64)
        requires
            old(self).well_formed(),
            old(self).forgotten() <= u64::MAX - old(self).available(),
        ensures
            final(self).well_formed(),
            forgotten_now == if requested <= old(self).available() {
                requested
            } else {
                old(self).available()
            },
            final(self).available() + forgotten_now == old(self).available(),
            final(self).forgotten() == old(self).forgotten() + forgotten_now,
            final(self).held() == old(self).held(),
            final(self).escrowed() == old(self).escrowed(),
            final(self).total() == old(self).total(),
        no_unwind
    {
        let forgotten_now = if requested <= self.available { requested } else { self.available };
        self.available -= forgotten_now;
        self.forgotten += forgotten_now;
        forgotten_now
    }

    pub fn close(&mut self)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).closed(),
            final(self).total() == old(self).total(),
            final(self).available() == old(self).available(),
            final(self).held() == old(self).held(),
            final(self).escrowed() == old(self).escrowed(),
            final(self).forgotten() == old(self).forgotten(),
        no_unwind
    {
        self.closed = true;
    }
}

#[derive(Copy, Clone)]
pub struct WaitRequest {
    pub id: u64,
    pub requested: nat,
    pub remaining: nat,
}

/// Logical view of the intrusive FIFO. `queue[0]` corresponds to production
/// `waiters.queue.last()`, since production pushes at the front and pops back.
pub tracked struct SemaphoreFifo {
    ghost queue: Seq<WaitRequest>,
    ghost closed: bool,
}

impl SemaphoreFifo {
    pub closed spec fn queue(&self) -> Seq<WaitRequest> { self.queue }
    pub closed spec fn closed(&self) -> bool { self.closed }

    pub proof fn new() -> (tracked result: Self)
        ensures result.queue().len() == 0, !result.closed(),
    {
        let tracked result = SemaphoreFifo { queue: Seq::empty(), closed: false };
        result
    }

    pub proof fn enqueue(tracked &mut self, id: u64, requested: nat)
        requires
            !old(self).closed(),
            requested > 0,
            forall|i: int| 0 <= i < old(self).queue().len()
                ==> old(self).queue()[i].id != id,
        ensures
            final(self).queue() == old(self).queue().push(WaitRequest {
                id,
                requested,
                remaining: requested,
            }),
            final(self).closed() == old(self).closed(),
    {
        self.queue = self.queue.push(WaitRequest { id, requested, remaining: requested });
    }

    /// Assign only to the oldest waiter. A later small request cannot bypass a
    /// partially satisfied large request at the head.
    pub proof fn assign_front(tracked &mut self, permits: nat) -> (completed: bool)
        requires
            !old(self).closed(),
            old(self).queue().len() > 0,
            old(self).queue()[0].remaining > 0,
        ensures
            completed == (permits >= old(self).queue()[0].remaining),
            completed ==> final(self).queue()
                == old(self).queue().subrange(1, old(self).queue().len() as int),
            !completed ==> final(self).queue().len() == old(self).queue().len(),
            !completed ==> final(self).queue()[0].id == old(self).queue()[0].id,
            !completed ==> final(self).queue()[0].requested == old(self).queue()[0].requested,
            !completed ==> final(self).queue()[0].remaining
                == old(self).queue()[0].remaining - permits,
            !completed ==> forall|i: int| 1 <= i < final(self).queue().len()
                ==> final(self).queue()[i] == old(self).queue()[i],
            final(self).closed() == old(self).closed(),
    {
        let front = self.queue[0];
        if permits >= front.remaining {
            self.queue = self.queue.subrange(1, self.queue.len() as int);
            true
        } else {
            assert(permits < front.remaining);
            let updated = WaitRequest {
                id: front.id,
                requested: front.requested,
                remaining: (front.remaining - permits) as nat,
            };
            self.queue = self.queue.update(0, updated);
            false
        }
    }

    /// Dropping any queued acquire unlinks exactly that node and reports the
    /// partial allocation which production `Acquire::drop` redistributes.
    pub proof fn cancel(tracked &mut self, index: int) -> (assigned: nat)
        requires
            0 <= index < old(self).queue().len(),
            old(self).queue()[index].remaining <= old(self).queue()[index].requested,
        ensures
            assigned == old(self).queue()[index].requested
                - old(self).queue()[index].remaining,
            final(self).queue() == old(self).queue().subrange(0, index)
                + old(self).queue().subrange(index + 1, old(self).queue().len() as int),
            final(self).closed() == old(self).closed(),
    {
        let assigned = (self.queue[index].requested - self.queue[index].remaining) as nat;
        self.queue = self.queue.subrange(0, index)
            + self.queue.subrange(index + 1, self.queue.len() as int);
        assigned
    }

    /// Closing detaches every waiter. Waker execution itself is trusted, while
    /// the Tokio-specific fact that no waiter remains queued is proved here.
    pub proof fn close(tracked &mut self) -> (detached: Seq<WaitRequest>)
        requires !old(self).closed(),
        ensures
            detached == old(self).queue(),
            final(self).queue().len() == 0,
            final(self).closed(),
    {
        let detached = self.queue;
        self.queue = Seq::empty();
        self.closed = true;
        detached
    }
}

/// Public `SemaphorePermit` and `OwnedSemaphorePermit` use the same numerical
/// lifecycle. Arc identity and borrowed lifetimes are representation adapters.
pub struct PermitValue {
    permits: u64,
}

impl PermitValue {
    pub closed spec fn permits(&self) -> u64 { self.permits }

    pub fn new(permits: u64) -> (result: Self)
        ensures result.permits() == permits,
        no_unwind
    {
        PermitValue { permits }
    }

    pub fn split(&mut self, n: u64) -> (result: Option<Self>)
        ensures
            result.is_some() == (n <= old(self).permits()),
            match result {
                Some(part) => final(self).permits() + part.permits() == old(self).permits()
                    && part.permits() == n,
                None => final(self).permits() == old(self).permits(),
            },
        no_unwind
    {
        if n <= self.permits {
            self.permits -= n;
            Some(PermitValue { permits: n })
        } else {
            None
        }
    }

    pub fn merge(&mut self, other: Self)
        requires old(self).permits() <= u64::MAX - other.permits(),
        ensures final(self).permits() == old(self).permits() + other.permits(),
        no_unwind
    {
        self.permits += other.permits;
    }

    pub fn forget(&mut self) -> (forgotten: u64)
        ensures
            forgotten == old(self).permits(),
            final(self).permits() == 0,
        no_unwind
    {
        let forgotten = self.permits;
        self.permits = 0;
        forgotten
    }

    pub fn drop_value(&mut self) -> (released: u64)
        ensures
            released == old(self).permits(),
            final(self).permits() == 0,
        no_unwind
    {
        let released = self.permits;
        self.permits = 0;
        released
    }
}

pub fn verify_semaphore_conservation(initial: u64, requested: u64)
    requires requested <= initial,
{
    let mut accounting = SemaphoreAccounting::new(initial);
    let acquired = accounting.try_acquire(requested);
    assert(acquired);
    let mut permit = PermitValue::new(requested);
    let released = permit.drop_value();
    accounting.drop_permit(released);
    assert(accounting.available() == initial);
    assert(accounting.held() == 0);
    assert(accounting.well_formed());
}

pub proof fn verify_fifo_head_blocking(first: u64, second: u64)
    requires first != second,
{
    let tracked mut fifo = SemaphoreFifo::new();
    fifo.enqueue(first, 5);
    fifo.enqueue(second, 1);
    let completed = fifo.assign_front(1);
    assert(!completed);
    assert(fifo.queue()[0].id == first);
    assert(fifo.queue()[0].remaining == 4);
    assert(fifo.queue()[1].id == second);
}

pub proof fn verify_cancel_returns_partial(waiter: u64)
{
    let tracked mut fifo = SemaphoreFifo::new();
    fifo.enqueue(waiter, 5);
    let completed = fifo.assign_front(2);
    assert(!completed);
    let assigned = fifo.cancel(0);
    assert(assigned == 2);
    assert(fifo.queue().len() == 0);
}

} // verus!
