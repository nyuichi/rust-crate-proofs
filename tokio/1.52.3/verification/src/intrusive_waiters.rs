use vstd::prelude::*;

verus! {

/// Global ownership view of Notify's two intrusive waiter lists. A node is in
/// exactly one of `detached`, `main`, or the guarded broadcast list. The raw
/// pointer links are a representation adapter; membership transfer is proved
/// here.
pub tracked struct IntrusiveWaiters {
    ghost detached: Set<u64>,
    ghost main: Set<u64>,
    ghost broadcast: Set<u64>,
    ghost notified_one: Set<u64>,
    ghost notified_all: Set<u64>,
    ghost epoch: nat,
}

impl IntrusiveWaiters {
    pub closed spec fn detached(&self) -> Set<u64> { self.detached }
    pub closed spec fn main_list(&self) -> Set<u64> { self.main }
    pub closed spec fn broadcast(&self) -> Set<u64> { self.broadcast }
    pub closed spec fn notified_one(&self) -> Set<u64> { self.notified_one }
    pub closed spec fn notified_all(&self) -> Set<u64> { self.notified_all }
    pub closed spec fn epoch(&self) -> nat { self.epoch }

    pub closed spec fn nodes(&self) -> Set<u64> {
        self.detached.union(self.main).union(self.broadcast)
    }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.detached.disjoint(self.main)
        &&& self.detached.disjoint(self.broadcast)
        &&& self.main.disjoint(self.broadcast)
        &&& self.notified_one.disjoint(self.notified_all)
        &&& self.notified_one.subset_of(self.detached)
        &&& self.notified_all.subset_of(self.detached)
    }

    pub proof fn new(nodes: Set<u64>) -> (tracked result: Self)
        ensures
            result.well_formed(),
            result.nodes() == nodes,
            result.detached() == nodes,
            result.main_list().is_empty(),
            result.broadcast().is_empty(),
            result.notified_one().is_empty(),
            result.notified_all().is_empty(),
            result.epoch() == 0,
    {
        let tracked result = IntrusiveWaiters {
            detached: nodes,
            main: Set::empty(),
            broadcast: Set::empty(),
            notified_one: Set::empty(),
            notified_all: Set::empty(),
            epoch: 0,
        };
        result
    }

    pub proof fn register(tracked &mut self, node: u64)
        requires
            old(self).well_formed(),
            old(self).detached().contains(node),
            !old(self).notified_one().contains(node),
            !old(self).notified_all().contains(node),
        ensures
            final(self).well_formed(),
            final(self).nodes() == old(self).nodes(),
            final(self).detached() == old(self).detached().remove(node),
            final(self).main_list() == old(self).main_list().insert(node),
            final(self).broadcast() == old(self).broadcast(),
            final(self).notified_one() == old(self).notified_one(),
            final(self).notified_all() == old(self).notified_all(),
            final(self).main_list().contains(node),
            !final(self).detached().contains(node),
            final(self).epoch() == old(self).epoch(),
    {
        self.detached = self.detached.remove(node);
        self.main = self.main.insert(node);
    }

    /// Production `notify_waiters` increments the generation while holding the
    /// lock and transfers every main-list node to its guarded local list.
    pub proof fn begin_broadcast(tracked &mut self)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).nodes() == old(self).nodes(),
            final(self).main_list().is_empty(),
            final(self).broadcast() == old(self).broadcast().union(old(self).main_list()),
            final(self).epoch() == old(self).epoch() + 1,
    {
        self.broadcast = self.broadcast.union(self.main);
        self.main = Set::empty();
        self.epoch = self.epoch + 1;
    }

    /// Unlink before publishing `Notification::All`, matching the production
    /// ordering that makes the node exclusively accessible to its future.
    pub proof fn finish_broadcast_node(tracked &mut self, node: u64)
        requires
            old(self).well_formed(),
            old(self).broadcast().contains(node),
        ensures
            final(self).well_formed(),
            final(self).nodes() == old(self).nodes(),
            final(self).broadcast() == old(self).broadcast().remove(node),
            final(self).detached() == old(self).detached().insert(node),
            final(self).main_list() == old(self).main_list(),
            final(self).notified_one() == old(self).notified_one(),
            final(self).notified_all() == old(self).notified_all().insert(node),
            final(self).detached().contains(node),
            final(self).notified_all().contains(node),
            !final(self).broadcast().contains(node),
            final(self).epoch() == old(self).epoch(),
    {
        self.broadcast = self.broadcast.remove(node);
        self.detached = self.detached.insert(node);
        self.notified_all = self.notified_all.insert(node);
    }

    pub proof fn notify_one(tracked &mut self, node: u64)
        requires
            old(self).well_formed(),
            old(self).main_list().contains(node),
        ensures
            final(self).well_formed(),
            final(self).nodes() == old(self).nodes(),
            final(self).detached().contains(node),
            final(self).notified_one().contains(node),
            !final(self).main_list().contains(node),
            final(self).epoch() == old(self).epoch(),
    {
        self.main = self.main.remove(node);
        self.detached = self.detached.insert(node);
        self.notified_one = self.notified_one.insert(node);
    }

    /// Poll consumes the marker only after the notifier has detached the node.
    pub proof fn consume_notification(tracked &mut self, node: u64)
        requires
            old(self).well_formed(),
            old(self).detached().contains(node),
            old(self).notified_one().contains(node)
                || old(self).notified_all().contains(node),
        ensures
            final(self).well_formed(),
            final(self).nodes() == old(self).nodes(),
            final(self).detached().contains(node),
            !final(self).notified_one().contains(node),
            !final(self).notified_all().contains(node),
            final(self).epoch() == old(self).epoch(),
    {
        self.notified_one = self.notified_one.remove(node);
        self.notified_all = self.notified_all.remove(node);
    }

    /// Cancellation unlinks a main-list node. A node in the guarded broadcast
    /// list stays owned by that list until its destructor finishes the detach.
    pub proof fn cancel_main(tracked &mut self, node: u64)
        requires
            old(self).well_formed(),
            old(self).main_list().contains(node),
        ensures
            final(self).well_formed(),
            final(self).nodes() == old(self).nodes(),
            final(self).detached().contains(node),
            !final(self).main_list().contains(node),
            final(self).epoch() == old(self).epoch(),
    {
        self.main = self.main.remove(node);
        self.detached = self.detached.insert(node);
    }
}

pub proof fn verify_global_waiter_ownership(first: u64, second: u64)
    requires
        first != second,
{
    let nodes = Set::empty().insert(first).insert(second);
    let tracked mut list = IntrusiveWaiters::new(nodes);
    assert(nodes.contains(first));
    assert(nodes.contains(second));
    assert(!list.notified_one().contains(first));
    assert(!list.notified_all().contains(first));
    list.register(first);
    assert(list.detached().contains(second));
    assert(!list.notified_one().contains(second));
    assert(!list.notified_all().contains(second));
    list.register(second);
    assert(list.main_list().contains(first));
    assert(list.main_list().contains(second));

    list.begin_broadcast();
    assert(list.main_list().is_empty());
    assert(list.broadcast().contains(first));
    assert(list.broadcast().contains(second));

    list.finish_broadcast_node(first);
    list.finish_broadcast_node(second);
    assert(list.broadcast().is_empty());
    assert(list.notified_all().contains(first));
    assert(list.notified_all().contains(second));
    assert(list.nodes() == nodes);
}

} // verus!
