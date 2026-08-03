use vstd::prelude::*;

verus! {

/// Pointer-free ownership and order view of `util::linked_list`. Raw-pointer
/// validity and `Link::{as_raw,from_raw,pointers}` are frozen representation
/// interfaces; every list-owned transition is represented below.
pub tracked struct IntrusiveListModel {
    ghost order: Seq<u64>,
    ghost detached: Set<u64>,
}

impl IntrusiveListModel {
    pub closed spec fn order(&self) -> Seq<u64> { self.order }
    pub closed spec fn detached(&self) -> Set<u64> { self.detached }

    pub proof fn new(nodes: Set<u64>) -> (tracked result: Self)
        ensures result.order().len() == 0, result.detached() == nodes,
    {
        let tracked result = IntrusiveListModel { order: Seq::empty(), detached: nodes };
        result
    }

    pub proof fn push_front(tracked &mut self, node: u64)
        requires old(self).detached().contains(node),
        ensures
            final(self).order() == seq![node] + old(self).order(),
            final(self).detached() == old(self).detached().remove(node),
    {
        self.order = seq![node] + self.order;
        self.detached = self.detached.remove(node);
    }

    pub proof fn pop_front(tracked &mut self) -> (node: u64)
        requires old(self).order().len() > 0,
        ensures
            node == old(self).order()[0],
            final(self).order() == old(self).order().subrange(1, old(self).order().len() as int),
            final(self).detached() == old(self).detached().insert(node),
    {
        let node = self.order[0];
        self.order = self.order.subrange(1, self.order.len() as int);
        self.detached = self.detached.insert(node);
        node
    }

    pub proof fn pop_back(tracked &mut self) -> (node: u64)
        requires old(self).order().len() > 0,
        ensures
            node == old(self).order()[old(self).order().len() - 1],
            final(self).order() == old(self).order().subrange(0, old(self).order().len() as int - 1),
            final(self).detached() == old(self).detached().insert(node),
    {
        let index = self.order.len() - 1;
        let node = self.order[index];
        self.order = self.order.subrange(0, index);
        self.detached = self.detached.insert(node);
        node
    }

    /// `into_guarded` only replaces the end sentinels; guarded `pop_back`
    /// therefore has the same abstract ownership transition as `pop_back`.
    pub proof fn guarded_pop_back(tracked &mut self) -> (node: u64)
        requires old(self).order().len() > 0,
        ensures
            node == old(self).order()[old(self).order().len() - 1],
            final(self).order() == old(self).order().subrange(0, old(self).order().len() as int - 1),
            final(self).detached() == old(self).detached().insert(node),
    {
        self.pop_back()
    }

    pub proof fn remove(tracked &mut self, index: int) -> (node: u64)
        requires 0 <= index < old(self).order().len(),
        ensures
            node == old(self).order()[index],
            final(self).order() == old(self).order().subrange(0, index)
                + old(self).order().subrange(index + 1, old(self).order().len() as int),
            final(self).detached() == old(self).detached().insert(node),
    {
        let node = self.order[index];
        self.order = self.order.subrange(0, index)
            + self.order.subrange(index + 1, self.order.len() as int);
        self.detached = self.detached.insert(node);
        node
    }

    /// `move_to_new_list` repeatedly pops the old tail and pushes it at the
    /// new head, preserving source order and prepending it to the target.
    pub proof fn move_all_to_front(tracked &mut self, tracked target: &mut IntrusiveListModel)
        requires old(self).detached() == old(target).detached(),
        ensures
            final(self).order().len() == 0,
            final(target).order() == old(self).order() + old(target).order(),
            final(self).detached() == old(self).detached(),
            final(target).detached() == old(target).detached(),
    {
        target.order = self.order + target.order;
        self.order = Seq::empty();
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DrainDecision { Keep, Remove, Panic }

pub tracked struct DrainFilterModel {
    ghost remaining: Seq<u64>,
    ghost kept: Seq<u64>,
    ghost removed: Seq<u64>,
    stopped: bool,
}

impl DrainFilterModel {
    pub closed spec fn remaining(&self) -> Seq<u64> { self.remaining }
    pub closed spec fn kept(&self) -> Seq<u64> { self.kept }
    pub closed spec fn removed(&self) -> Seq<u64> { self.removed }
    pub closed spec fn stopped(&self) -> bool { self.stopped }

    pub proof fn new(input: Seq<u64>) -> (tracked result: Self)
        ensures result.remaining() == input, result.kept().len() == 0,
            result.removed().len() == 0, !result.stopped(),
    {
        let tracked result = DrainFilterModel {
            remaining: input, kept: Seq::empty(), removed: Seq::empty(), stopped: false,
        };
        result
    }

    pub proof fn step(tracked &mut self, decision: DrainDecision)
        requires !old(self).stopped(), old(self).remaining().len() > 0,
        ensures
            decision == DrainDecision::Keep ==> {
                &&& final(self).kept() == old(self).kept().push(old(self).remaining()[0])
                &&& final(self).removed() == old(self).removed()
                &&& final(self).remaining() == old(self).remaining().subrange(1, old(self).remaining().len() as int)
                &&& !final(self).stopped()
            },
            decision == DrainDecision::Remove ==> {
                &&& final(self).removed() == old(self).removed().push(old(self).remaining()[0])
                &&& final(self).kept() == old(self).kept()
                &&& final(self).remaining() == old(self).remaining().subrange(1, old(self).remaining().len() as int)
                &&& !final(self).stopped()
            },
            decision == DrainDecision::Panic ==> {
                &&& final(self).remaining() == old(self).remaining()
                &&& final(self).kept() == old(self).kept()
                &&& final(self).removed() == old(self).removed()
                &&& final(self).stopped()
            },
    {
        match decision {
            DrainDecision::Keep => {
                self.kept = self.kept.push(self.remaining[0]);
                self.remaining = self.remaining.subrange(1, self.remaining.len() as int);
            },
            DrainDecision::Remove => {
                self.removed = self.removed.push(self.remaining[0]);
                self.remaining = self.remaining.subrange(1, self.remaining.len() as int);
            },
            DrainDecision::Panic => self.stopped = true,
        }
    }
}

pub open spec fn wake_capacity() -> nat { 32 }

/// Initialized-prefix view of `WakeList`. `wake_all` first transfers the whole
/// prefix out of the array, so cleanup owns every suffix even if one wake call
/// unwinds.
pub tracked struct WakeListModel { ghost queued: Seq<u64> }

impl WakeListModel {
    pub closed spec fn queued(&self) -> Seq<u64> { self.queued }
    pub proof fn new() -> (tracked result: Self)
        ensures result.queued().len() == 0,
    { let tracked result = WakeListModel { queued: Seq::empty() }; result }

    pub proof fn push(tracked &mut self, id: u64)
        requires old(self).queued().len() < wake_capacity(),
        ensures final(self).queued() == old(self).queued().push(id),
    { self.queued = self.queued.push(id); }

    pub proof fn wake_all(tracked &mut self, panic_index: int)
        -> (result: (Seq<u64>, Seq<u64>, Seq<u64>))
        requires -1 <= panic_index < old(self).queued().len(),
        ensures
            result.0 == old(self).queued(),
            panic_index == -1 ==> result.1 == old(self).queued() && result.2.len() == 0,
            panic_index >= 0 ==> result.1 == old(self).queued().subrange(0, panic_index + 1)
                && result.2 == old(self).queued().subrange(panic_index + 1, old(self).queued().len() as int),
            final(self).queued().len() == 0,
    {
        let owned = self.queued;
        self.queued = Seq::empty();
        if panic_index == -1 {
            (owned, owned, Seq::empty())
        } else {
            (owned, owned.subrange(0, panic_index + 1),
                owned.subrange(panic_index + 1, owned.len() as int))
        }
    }

    pub proof fn drop_all(tracked &mut self) -> (owned: Seq<u64>)
        ensures owned == old(self).queued(), final(self).queued().len() == 0,
    { let owned = self.queued; self.queued = Seq::empty(); owned }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntryList { Idle, Notified, Neither }

/// Exact two-list state machine for `IdleNotifiedSet`. The map is the owned
/// value identity; arbitrary value mutation/drop and lock/Waker execution stay
/// at the frozen foundation.
pub tracked struct IdleNotifiedSetModel {
    ghost idle: Seq<u64>,
    ghost notified: Seq<u64>,
    ghost values: Map<u64, u64>,
    ghost registered_waker: Option<u64>,
}

impl IdleNotifiedSetModel {
    pub closed spec fn idle(&self) -> Seq<u64> { self.idle }
    pub closed spec fn notified(&self) -> Seq<u64> { self.notified }
    pub closed spec fn values(&self) -> Map<u64, u64> { self.values }
    pub closed spec fn registered_waker(&self) -> Option<u64> { self.registered_waker }
    pub closed spec fn len(&self) -> nat { self.values().dom().len() }

    pub proof fn new() -> (tracked result: Self)
        ensures result.idle().len() == 0, result.notified().len() == 0,
            result.values().is_empty(), result.registered_waker().is_none(),
    {
        let tracked result = IdleNotifiedSetModel {
            idle: Seq::empty(), notified: Seq::empty(), values: Map::empty(),
            registered_waker: None,
        };
        result
    }

    /// The strict bound is supplied by successful finite allocation: Tokio
    /// cannot simultaneously own `usize::MAX` allocated non-zero-size entries.
    pub proof fn insert_idle(tracked &mut self, id: u64, value: u64)
        requires !old(self).values().contains_key(id), old(self).len() < usize::MAX as nat,
        ensures
            final(self).idle() == seq![id] + old(self).idle(),
            final(self).notified() == old(self).notified(),
            final(self).values() == old(self).values().insert(id, value),
            final(self).len() == old(self).len() + 1,
    {
        self.idle = seq![id] + self.idle;
        self.values = self.values.insert(id, value);
    }

    pub proof fn wake_idle(tracked &mut self, index: int) -> (waker: Option<u64>)
        requires 0 <= index < old(self).idle().len(),
        ensures
            final(self).idle() == old(self).idle().subrange(0, index)
                + old(self).idle().subrange(index + 1, old(self).idle().len() as int),
            final(self).notified() == seq![old(self).idle()[index]] + old(self).notified(),
            final(self).values() == old(self).values(),
            waker == old(self).registered_waker(), final(self).registered_waker().is_none(),
    {
        let id = self.idle[index];
        self.idle = self.idle.subrange(0, index)
            + self.idle.subrange(index + 1, self.idle.len() as int);
        self.notified = seq![id] + self.notified;
        let waker = self.registered_waker;
        self.registered_waker = None;
        waker
    }

    pub proof fn pop_notified(tracked &mut self, waker: Option<u64>) -> (id: Option<u64>)
        requires old(self).len() > 0,
        ensures
            old(self).notified().len() == 0 ==> id.is_none()
                && final(self).idle() == old(self).idle()
                && final(self).notified() == old(self).notified(),
            old(self).notified().len() > 0 ==> id == Some(old(self).notified()[old(self).notified().len() - 1])
                && final(self).idle() == seq![old(self).notified()[old(self).notified().len() - 1]] + old(self).idle()
                && final(self).notified() == old(self).notified().subrange(0, old(self).notified().len() as int - 1),
            final(self).values() == old(self).values(),
            final(self).registered_waker() == waker,
    {
        self.registered_waker = waker;
        if self.notified.len() == 0 { None } else {
            let index = self.notified.len() - 1;
            let id = self.notified[index];
            self.notified = self.notified.subrange(0, index);
            self.idle = seq![id] + self.idle;
            Some(id)
        }
    }

    /// Non-registering form used by `try_pop_notified`.
    pub proof fn try_pop_notified(tracked &mut self) -> (id: Option<u64>)
        ensures
            old(self).notified().len() == 0 ==> id.is_none()
                && final(self).idle() == old(self).idle()
                && final(self).notified() == old(self).notified(),
            old(self).notified().len() > 0 ==> id == Some(old(self).notified()[old(self).notified().len() - 1])
                && final(self).idle() == seq![old(self).notified()[old(self).notified().len() - 1]] + old(self).idle()
                && final(self).notified() == old(self).notified().subrange(0, old(self).notified().len() as int - 1),
            final(self).values() == old(self).values(),
            final(self).registered_waker() == old(self).registered_waker(),
    {
        if self.notified.len() == 0 { None } else {
            let index = self.notified.len() - 1;
            let id = self.notified[index];
            self.notified = self.notified.subrange(0, index);
            self.idle = seq![id] + self.idle;
            Some(id)
        }
    }

    /// Pointer collection under the mutex freezes one exact visit snapshot.
    /// The later arbitrary callbacks may move an entry between lists by waking
    /// it, but cannot add/remove values while `&mut IdleNotifiedSet` is held.
    pub proof fn for_each_snapshot(tracked &mut self) -> (visited: Seq<u64>)
        ensures
            visited == old(self).idle() + old(self).notified(),
            final(self).idle() == old(self).idle(),
            final(self).notified() == old(self).notified(),
            final(self).values() == old(self).values(),
            final(self).registered_waker() == old(self).registered_waker(),
    {
        self.idle + self.notified
    }

    pub proof fn remove_idle(tracked &mut self, index: int) -> (value: u64)
        requires 0 <= index < old(self).idle().len(), old(self).values().contains_key(old(self).idle()[index]),
        ensures
            value == old(self).values()[old(self).idle()[index]],
            final(self).idle() == old(self).idle().subrange(0, index)
                + old(self).idle().subrange(index + 1, old(self).idle().len() as int),
            final(self).notified() == old(self).notified(),
            final(self).values() == old(self).values().remove(old(self).idle()[index]),
            final(self).len() + 1 == old(self).len(),
    {
        let id = self.idle[index];
        let value = self.values[id];
        self.idle = self.idle.subrange(0, index) + self.idle.subrange(index + 1, self.idle.len() as int);
        self.values = self.values.remove(id);
        value
    }

    pub proof fn drain(tracked &mut self) -> (owned: Map<u64, u64>)
        ensures owned == old(self).values(), final(self).values().is_empty(),
            final(self).idle().len() == 0, final(self).notified().len() == 0,
    {
        let owned = self.values;
        self.values = Map::empty();
        self.idle = Seq::empty();
        self.notified = Seq::empty();
        owned
    }
}

pub proof fn verify_fifo_wake_and_pop(first: u64, second: u64)
    requires first != second,
{
    let tracked mut set = IdleNotifiedSetModel::new();
    set.insert_idle(first, 10);
    set.insert_idle(second, 20);
    let none = set.pop_notified(Some(7));
    assert(none.is_none());
    let wake = set.wake_idle(1);
    assert(wake == Some(7));
    let popped = set.pop_notified(Some(8));
    assert(popped == Some(first));
    let value = set.remove_idle(0);
    assert(value == 10);
    assert(set.len() == 1);
}

pub proof fn verify_wake_list_panic_cleanup()
{
    let tracked mut list = WakeListModel::new();
    list.push(1); list.push(2); list.push(3);
    let result = list.wake_all(1);
    assert(result.0 == seq![1u64, 2u64, 3u64]);
    assert(result.1 == seq![1u64, 2u64]);
    assert(result.2 == seq![3u64]);
    assert(list.queued().len() == 0);
}

} // verus!
