use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NotifyStrategy {
    Fifo,
    Lifo,
}

/// Global, ordered protocol for production `Notify`. Existing `NotifiedCore`
/// proves each future's poll lifecycle, while this model proves how all public
/// operations select and conserve notifications across futures.
pub tracked struct NotifyProtocol {
    ghost queue: Seq<u64>,
    ghost stored_permit: bool,
    ghost epoch: nat,
    ghost notified_fifo: Set<u64>,
    ghost notified_lifo: Set<u64>,
    ghost notified_all: Set<u64>,
}

impl NotifyProtocol {
    pub closed spec fn queue(&self) -> Seq<u64> { self.queue }
    pub closed spec fn stored_permit(&self) -> bool { self.stored_permit }
    pub closed spec fn epoch(&self) -> nat { self.epoch }
    pub closed spec fn notified_fifo(&self) -> Set<u64> { self.notified_fifo }
    pub closed spec fn notified_lifo(&self) -> Set<u64> { self.notified_lifo }
    pub closed spec fn notified_all(&self) -> Set<u64> { self.notified_all }

    pub proof fn new() -> (tracked result: Self)
        ensures
            result.queue().len() == 0,
            !result.stored_permit(),
            result.epoch() == 0,
            result.notified_fifo().is_empty(),
            result.notified_lifo().is_empty(),
            result.notified_all().is_empty(),
    {
        let tracked result = NotifyProtocol {
            queue: Seq::empty(),
            stored_permit: false,
            epoch: 0,
            notified_fifo: Set::empty(),
            notified_lifo: Set::empty(),
            notified_all: Set::empty(),
        };
        result
    }

    /// Models `enable` or the first poll after the future captured `snapshot`.
    /// A broadcast generation wins before a stored notify-one permit, matching
    /// production's check order.
    pub proof fn enable(tracked &mut self, id: u64, snapshot: nat) -> (ready: bool)
        requires
            forall|i: int| 0 <= i < old(self).queue().len()
                ==> old(self).queue()[i] != id,
            !old(self).notified_fifo().contains(id),
            !old(self).notified_lifo().contains(id),
            !old(self).notified_all().contains(id),
        ensures
            ready == (snapshot != old(self).epoch() || old(self).stored_permit()),
            snapshot != old(self).epoch() ==> final(self).notified_all().contains(id),
            snapshot != old(self).epoch()
                ==> final(self).stored_permit() == old(self).stored_permit(),
            snapshot == old(self).epoch() && old(self).stored_permit()
                ==> !final(self).stored_permit(),
            !ready ==> final(self).queue() == old(self).queue().push(id),
            ready ==> final(self).queue() == old(self).queue(),
            final(self).epoch() == old(self).epoch(),
            final(self).notified_fifo() == old(self).notified_fifo(),
            final(self).notified_lifo() == old(self).notified_lifo(),
            final(self).notified_all() == if snapshot != old(self).epoch() {
                old(self).notified_all().insert(id)
            } else {
                old(self).notified_all()
            },
            final(self).stored_permit() == if snapshot != old(self).epoch() {
                old(self).stored_permit()
            } else {
                false
            },
    {
        if snapshot != self.epoch {
            self.notified_all = self.notified_all.insert(id);
            true
        } else if self.stored_permit {
            self.stored_permit = false;
            true
        } else {
            self.queue = self.queue.push(id);
            false
        }
    }

    /// `notify_one` is FIFO and `notify_last` is LIFO. If no task is waiting,
    /// repeated calls coalesce into the single stored permit.
    pub proof fn notify(tracked &mut self, strategy: NotifyStrategy) -> (selected: Option<u64>)
        ensures
            old(self).queue().len() == 0 ==> selected.is_none(),
            old(self).queue().len() == 0 ==> final(self).stored_permit(),
            old(self).queue().len() == 0 ==> final(self).queue() == old(self).queue(),
            old(self).queue().len() > 0 && strategy == NotifyStrategy::Fifo
                ==> selected == Some(old(self).queue()[0]),
            old(self).queue().len() > 0 && strategy == NotifyStrategy::Fifo
                ==> final(self).queue()
                    == old(self).queue().subrange(1, old(self).queue().len() as int),
            old(self).queue().len() > 0 && strategy == NotifyStrategy::Lifo
                ==> selected == Some(old(self).queue()[old(self).queue().len() - 1]),
            old(self).queue().len() > 0 && strategy == NotifyStrategy::Lifo
                ==> final(self).queue()
                    == old(self).queue().subrange(0, old(self).queue().len() as int - 1),
            selected.is_some() ==> !final(self).stored_permit(),
            selected.is_some() && strategy == NotifyStrategy::Fifo
                ==> final(self).notified_fifo().contains(selected.unwrap()),
            selected.is_some() && strategy == NotifyStrategy::Lifo
                ==> final(self).notified_lifo().contains(selected.unwrap()),
            final(self).notified_fifo() == if selected.is_some()
                && strategy == NotifyStrategy::Fifo {
                    old(self).notified_fifo().insert(selected.unwrap())
                } else {
                    old(self).notified_fifo()
                },
            final(self).notified_lifo() == if selected.is_some()
                && strategy == NotifyStrategy::Lifo {
                    old(self).notified_lifo().insert(selected.unwrap())
                } else {
                    old(self).notified_lifo()
                },
            final(self).epoch() == old(self).epoch(),
            final(self).notified_all() == old(self).notified_all(),
    {
        if self.queue.len() == 0 {
            self.stored_permit = true;
            None
        } else {
            self.stored_permit = false;
            match strategy {
                NotifyStrategy::Fifo => {
                    let id = self.queue[0];
                    self.queue = self.queue.subrange(1, self.queue.len() as int);
                    self.notified_fifo = self.notified_fifo.insert(id);
                    Some(id)
                },
                NotifyStrategy::Lifo => {
                    let index = self.queue.len() - 1;
                    let id = self.queue[index];
                    self.queue = self.queue.subrange(0, index);
                    self.notified_lifo = self.notified_lifo.insert(id);
                    Some(id)
                },
            }
        }
    }

    /// Broadcast is atomic at the protocol level: its epoch advances and the
    /// whole pre-existing queue becomes ready. A stored one-permit is neither
    /// created nor consumed by `notify_waiters`.
    pub proof fn notify_waiters(tracked &mut self) -> (notified: Seq<u64>)
        ensures
            notified == old(self).queue(),
            final(self).queue().len() == 0,
            final(self).epoch() == old(self).epoch() + 1,
            final(self).stored_permit() == old(self).stored_permit(),
            final(self).notified_all()
                == old(self).notified_all().union(old(self).queue().to_set()),
            final(self).notified_fifo() == old(self).notified_fifo(),
            final(self).notified_lifo() == old(self).notified_lifo(),
    {
        let notified = self.queue;
        self.notified_all = self.notified_all.union(self.queue.to_set());
        self.queue = Seq::empty();
        self.epoch = self.epoch + 1;
        notified
    }

    pub proof fn cancel_waiting(tracked &mut self, index: int) -> (id: u64)
        requires 0 <= index < old(self).queue().len(),
        ensures
            id == old(self).queue()[index],
            final(self).queue() == old(self).queue().subrange(0, index)
                + old(self).queue().subrange(index + 1, old(self).queue().len() as int),
            final(self).stored_permit() == old(self).stored_permit(),
            final(self).epoch() == old(self).epoch(),
    {
        let id = self.queue[index];
        self.queue = self.queue.subrange(0, index)
            + self.queue.subrange(index + 1, self.queue.len() as int);
        id
    }

    /// If a selected future is dropped before consuming `Notification::One`,
    /// production forwards using the original FIFO/LIFO strategy.
    pub proof fn cancel_notified_one(
        tracked &mut self,
        id: u64,
        strategy: NotifyStrategy,
    ) -> (forwarded: Option<u64>)
        requires
            strategy == NotifyStrategy::Fifo
                ==> old(self).notified_fifo().contains(id),
            strategy == NotifyStrategy::Lifo
                ==> old(self).notified_lifo().contains(id),
            old(self).queue().len() > 0
                ==> old(self).queue()[0] != id,
            old(self).queue().len() > 0
                ==> old(self).queue()[old(self).queue().len() - 1] != id,
        ensures
            old(self).queue().len() == 0 ==> forwarded.is_none(),
            old(self).queue().len() == 0 ==> final(self).stored_permit(),
            old(self).queue().len() > 0 && strategy == NotifyStrategy::Fifo
                ==> forwarded == Some(old(self).queue()[0]),
            old(self).queue().len() > 0 && strategy == NotifyStrategy::Lifo
                ==> forwarded == Some(old(self).queue()[old(self).queue().len() - 1]),
            !final(self).notified_fifo().contains(id),
            !final(self).notified_lifo().contains(id),
            final(self).epoch() == old(self).epoch(),
    {
        self.notified_fifo = self.notified_fifo.remove(id);
        self.notified_lifo = self.notified_lifo.remove(id);
        if self.queue.len() == 0 {
            self.stored_permit = true;
            None
        } else {
            self.stored_permit = false;
            match strategy {
                NotifyStrategy::Fifo => {
                    let next = self.queue[0];
                    self.queue = self.queue.subrange(1, self.queue.len() as int);
                    self.notified_fifo = self.notified_fifo.insert(next);
                    Some(next)
                },
                NotifyStrategy::Lifo => {
                    let index = self.queue.len() - 1;
                    let next = self.queue[index];
                    self.queue = self.queue.subrange(0, index);
                    self.notified_lifo = self.notified_lifo.insert(next);
                    Some(next)
                },
            }
        }
    }
}

pub proof fn verify_notify_one_coalesces()
{
    let tracked mut notify = NotifyProtocol::new();
    let first = notify.notify(NotifyStrategy::Fifo);
    let second = notify.notify(NotifyStrategy::Fifo);
    assert(first.is_none());
    assert(second.is_none());
    assert(notify.stored_permit());
    let ready = notify.enable(1, notify.epoch());
    assert(ready);
    assert(!notify.stored_permit());
}

pub proof fn verify_fifo_lifo_selection(first: u64, second: u64)
    requires first != second,
{
    let tracked mut fifo = NotifyProtocol::new();
    let first_pending = fifo.enable(first, 0);
    let second_pending = fifo.enable(second, 0);
    assert(!first_pending && !second_pending);
    let selected_first = fifo.notify(NotifyStrategy::Fifo);
    assert(selected_first == Some(first));

    let tracked mut lifo = NotifyProtocol::new();
    let first_pending = lifo.enable(first, 0);
    let second_pending = lifo.enable(second, 0);
    assert(!first_pending && !second_pending);
    let selected_last = lifo.notify(NotifyStrategy::Lifo);
    assert(selected_last == Some(second));
}

pub proof fn verify_cancel_forwards_same_strategy(first: u64, second: u64)
    requires first != second,
{
    let tracked mut notify = NotifyProtocol::new();
    let first_pending = notify.enable(first, 0);
    let second_pending = notify.enable(second, 0);
    assert(!first_pending);
    assert(!second_pending);
    let selected = notify.notify(NotifyStrategy::Fifo);
    assert(selected == Some(first));
    let forwarded = notify.cancel_notified_one(first, NotifyStrategy::Fifo);
    assert(forwarded == Some(second));
}

pub proof fn verify_broadcast_precedes_stored_permit(id: u64)
{
    let tracked mut notify = NotifyProtocol::new();
    let stored = notify.notify(NotifyStrategy::Fifo);
    assert(stored.is_none());
    let snapshot = notify.epoch();
    notify.notify_waiters();
    let ready = notify.enable(id, snapshot);
    assert(ready);
    assert(notify.stored_permit());
}

} // verus!
