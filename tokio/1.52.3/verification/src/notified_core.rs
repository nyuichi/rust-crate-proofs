use vstd::prelude::*;

verus! {

/// Proof view of the production `Notified` future's private state.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FutureState {
    Init,
    Registering,
    Waiting,
    Done,
    Dropped,
}

/// A waiter can be in the main intrusive list or in the guarded secondary
/// list used while `notify_waiters` wakes a batch outside the mutex.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ListLocation {
    Detached,
    Main,
    Broadcast,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Notification {
    None,
    One,
    All,
}

/// Body-proved state machine for the non-pointer portion of `Notified::poll`,
/// `notify_waiters`, and `Notified::drop`.
///
/// `epoch` is Tokio's `num_notify_waiters_calls`; `snapshot` is the value saved
/// when the future was created. Wakers are represented by stable numeric ids,
/// leaving clone/wake/drop execution at the poll-surface boundary.
pub struct NotifiedCore {
    state: FutureState,
    location: ListLocation,
    notification: Notification,
    epoch: u64,
    snapshot: u64,
    waker: Option<u64>,
}

impl NotifiedCore {
    pub closed spec fn state(&self) -> FutureState { self.state }
    pub closed spec fn location(&self) -> ListLocation { self.location }
    pub closed spec fn notification(&self) -> Notification { self.notification }
    pub closed spec fn epoch(&self) -> u64 { self.epoch }
    pub closed spec fn snapshot(&self) -> u64 { self.snapshot }
    pub closed spec fn waker(&self) -> Option<u64> { self.waker }

    pub closed spec fn well_formed(&self) -> bool {
        match self.state {
            FutureState::Init => {
                self.location == ListLocation::Detached
                    && self.notification == Notification::None
                    && self.waker.is_none()
            },
            FutureState::Registering => {
                self.location == ListLocation::Detached
                    && self.notification == Notification::None
                    && self.waker.is_some()
            },
            FutureState::Waiting => {
                (match self.notification {
                    Notification::None => {
                        self.location == ListLocation::Main
                            || self.location == ListLocation::Broadcast
                    },
                    Notification::One | Notification::All => {
                        self.location == ListLocation::Detached
                    },
                }) && self.waker.is_some()
            },
            FutureState::Done | FutureState::Dropped => {
                self.location == ListLocation::Detached
                    && self.notification == Notification::None
                    && self.waker.is_none()
            },
        }
    }

    pub fn new(epoch: u64) -> (result: Self)
        ensures
            result.well_formed(),
            result.state() == FutureState::Init,
            result.epoch() == epoch,
            result.snapshot() == epoch,
        no_unwind
    {
        NotifiedCore {
            state: FutureState::Init,
            location: ListLocation::Detached,
            notification: Notification::None,
            epoch,
            snapshot: epoch,
            waker: None,
        }
    }

    /// Optimistic part of polling an Init future, before taking the list lock.
    pub fn start_poll(&mut self, waker: u64) -> (ready: bool)
        requires
            old(self).well_formed(),
            old(self).state() == FutureState::Init,
        ensures
            final(self).well_formed(),
            final(self).epoch() == old(self).epoch(),
            final(self).snapshot() == old(self).snapshot(),
            ready == (old(self).epoch() != old(self).snapshot()),
            ready ==> final(self).state() == FutureState::Done,
            !ready ==> final(self).state() == FutureState::Registering,
        no_unwind
    {
        if self.epoch != self.snapshot {
            self.state = FutureState::Done;
            true
        } else {
            self.state = FutureState::Registering;
            self.waker = Some(waker);
            false
        }
    }

    /// Locked recheck followed by insertion into the main waiter list.
    pub fn finish_registration(&mut self) -> (ready: bool)
        requires
            old(self).well_formed(),
            old(self).state() == FutureState::Registering,
        ensures
            final(self).well_formed(),
            final(self).epoch() == old(self).epoch(),
            final(self).snapshot() == old(self).snapshot(),
            ready == (old(self).epoch() != old(self).snapshot()),
            ready ==> final(self).state() == FutureState::Done,
            !ready ==> final(self).state() == FutureState::Waiting,
            !ready ==> final(self).location() == ListLocation::Main,
        no_unwind
    {
        if self.epoch != self.snapshot {
            self.state = FutureState::Done;
            self.waker = None;
            true
        } else {
            self.state = FutureState::Waiting;
            self.location = ListLocation::Main;
            false
        }
    }

    /// First half of production `notify_waiters`: increment the generation and
    /// move a registered node under the guarded broadcast list.
    pub fn begin_broadcast(&mut self)
        requires
            old(self).well_formed(),
            old(self).epoch() < u64::MAX,
        ensures
            final(self).well_formed(),
            final(self).epoch() == old(self).epoch() + 1,
            final(self).snapshot() == old(self).snapshot(),
            final(self).state() == old(self).state(),
            final(self).notification() == old(self).notification(),
            final(self).waker() == old(self).waker(),
            final(self).location() == if old(self).state() == FutureState::Waiting
                && old(self).location() == ListLocation::Main {
                    ListLocation::Broadcast
                } else {
                    old(self).location()
                },
        no_unwind
    {
        self.epoch = self.epoch + 1;
        match self.state {
            FutureState::Waiting => match self.location {
                ListLocation::Main => self.location = ListLocation::Broadcast,
                ListLocation::Detached | ListLocation::Broadcast => {},
            },
            FutureState::Init
            | FutureState::Registering
            | FutureState::Done
            | FutureState::Dropped => {},
        }
    }

    /// Finish unlinking the guarded waiter and publish its broadcast marker.
    pub fn finish_broadcast(&mut self)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).epoch() == old(self).epoch(),
            final(self).snapshot() == old(self).snapshot(),
            final(self).state() == old(self).state(),
            final(self).waker() == old(self).waker(),
            old(self).location() == ListLocation::Broadcast
                ==> final(self).location() == ListLocation::Detached,
            old(self).location() == ListLocation::Broadcast
                ==> final(self).notification() == Notification::All,
            old(self).location() != ListLocation::Broadcast
                ==> final(self).location() == old(self).location(),
            old(self).location() != ListLocation::Broadcast
                ==> final(self).notification() == old(self).notification(),
        no_unwind
    {
        match self.location {
            ListLocation::Broadcast => {
                self.location = ListLocation::Detached;
                self.notification = Notification::All;
            },
            ListLocation::Detached | ListLocation::Main => {},
        }
    }

    pub fn notify_one(&mut self)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).epoch() == old(self).epoch(),
            final(self).snapshot() == old(self).snapshot(),
            final(self).state() == old(self).state(),
            final(self).waker() == old(self).waker(),
            old(self).state() == FutureState::Waiting
                && old(self).location() == ListLocation::Main
                ==> final(self).notification() == Notification::One,
        no_unwind
    {
        match self.state {
            FutureState::Waiting => match self.location {
                ListLocation::Main => {
                    self.location = ListLocation::Detached;
                    self.notification = Notification::One;
                },
                ListLocation::Detached | ListLocation::Broadcast => {},
            },
            FutureState::Init
            | FutureState::Registering
            | FutureState::Done
            | FutureState::Dropped => {},
        }
    }

    /// Poll a waiter which has completed registration. Epoch mismatch covers
    /// the interval in which the waiter is owned by the guarded broadcast list.
    pub fn poll_waiting(&mut self, waker: u64) -> (ready: bool)
        requires
            old(self).well_formed(),
            old(self).state() == FutureState::Waiting,
        ensures
            final(self).well_formed(),
            final(self).epoch() == old(self).epoch(),
            ready == (old(self).notification() != Notification::None
                || old(self).epoch() != old(self).snapshot()),
            ready ==> final(self).state() == FutureState::Done,
            !ready ==> final(self).state() == FutureState::Waiting,
            !ready ==> final(self).waker() == Some(waker),
            !ready ==> final(self).notification() == Notification::None,
            !ready ==> final(self).location() == old(self).location(),
        no_unwind
    {
        let notified = match self.notification {
            Notification::None => false,
            Notification::One | Notification::All => true,
        };
        if notified || self.epoch != self.snapshot {
            self.location = ListLocation::Detached;
            self.notification = Notification::None;
            self.waker = None;
            self.state = FutureState::Done;
            true
        } else {
            self.waker = Some(waker);
            false
        }
    }

    /// Dropping a pending waiter unlinks it. An unconsumed notify-one permit is
    /// returned to the caller so production can forward it to another waiter.
    pub fn cancel(&mut self) -> (forward_one: bool)
        requires
            old(self).well_formed(),
            old(self).state() != FutureState::Dropped,
        ensures
            final(self).well_formed(),
            final(self).state() == FutureState::Dropped,
            forward_one == (old(self).notification() == Notification::One),
        no_unwind
    {
        let forward_one = match self.notification {
            Notification::One => true,
            Notification::None | Notification::All => false,
        };
        self.location = ListLocation::Detached;
        self.notification = Notification::None;
        self.waker = None;
        self.state = FutureState::Dropped;
        forward_one
    }
}

pub fn verify_broadcast_before_first_poll(epoch: u64)
    requires epoch < u64::MAX,
{
    let mut waiter = NotifiedCore::new(epoch);
    waiter.begin_broadcast();
    waiter.finish_broadcast();
    let ready = waiter.start_poll(1);
    assert(ready);
    assert(waiter.state() == FutureState::Done);
}

pub fn verify_broadcast_during_registration(epoch: u64)
    requires epoch < u64::MAX,
{
    let mut waiter = NotifiedCore::new(epoch);
    let initial = waiter.start_poll(1);
    assert(!initial);
    waiter.begin_broadcast();
    waiter.finish_broadcast();
    let ready = waiter.finish_registration();
    assert(ready);
}

pub fn verify_broadcast_after_registration(epoch: u64)
    requires epoch < u64::MAX,
{
    let mut waiter = NotifiedCore::new(epoch);
    let started = waiter.start_poll(1);
    assert(!started);
    let registered = waiter.finish_registration();
    assert(!registered);
    waiter.begin_broadcast();
    assert(waiter.location() == ListLocation::Broadcast);
    let ready_during_broadcast = waiter.poll_waiting(1);
    assert(ready_during_broadcast);
    waiter.finish_broadcast();
    assert(waiter.state() == FutureState::Done);
}

pub fn verify_waker_replacement_and_cancel(epoch: u64)
{
    let mut waiter = NotifiedCore::new(epoch);
    let started = waiter.start_poll(1);
    assert(!started);
    let registered = waiter.finish_registration();
    assert(!registered);
    let repolled = waiter.poll_waiting(2);
    assert(!repolled);
    assert(waiter.waker() == Some(2));
    let forward = waiter.cancel();
    assert(!forward);
    assert(waiter.location() == ListLocation::Detached);
}

pub fn verify_cancel_forwards_notify_one(epoch: u64)
{
    let mut waiter = NotifiedCore::new(epoch);
    let started = waiter.start_poll(1);
    assert(!started);
    let registered = waiter.finish_registration();
    assert(!registered);
    waiter.notify_one();
    let forward = waiter.cancel();
    assert(forward);
}

} // verus!
