use crate::publication::PublishedOnce;
use vstd::pervasive::unreached;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// SetOnce-specific projection of the `Notified` lifecycle. Pin, Context,
/// Waker, and the intrusive waiter list remain inside the poll-surface adapter.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WaiterState {
    Idle,
    Created,
    Registered,
    Notified,
    Cancelled,
}

pub struct WaitProtocol<T> {
    cell: PublishedOnce<T>,
    waiter: WaiterState,
}

impl<T> WaitProtocol<T> {
    pub closed spec fn contents(&self) -> MemContents<T> {
        self.cell.contents()
    }

    pub closed spec fn waiter_state(&self) -> WaiterState {
        self.waiter
    }

    pub closed spec fn well_formed(&self) -> bool {
        self.cell.well_formed()
            && (self.waiter == WaiterState::Notified ==> self.contents().is_init())
    }

    pub fn empty() -> (result: Self)
        ensures
            result.well_formed(),
            result.contents() == MemContents::Uninit,
            result.waiter_state() == WaiterState::Idle,
    {
        WaitProtocol { cell: PublishedOnce::empty(), waiter: WaiterState::Idle }
    }

    pub fn get<'a>(&'a self) -> (result: Option<&'a T>)
        requires
            self.well_formed(),
        ensures
            match result {
                Some(value) => self.contents() == MemContents::Init(*value),
                None => self.contents() == MemContents::Uninit,
            },
        no_unwind
    {
        self.cell.get()
    }

    /// Create the notification future after an unsuccessful Acquire `get`.
    pub fn create_waiter(&mut self)
        requires
            old(self).well_formed(),
            old(self).waiter_state() == WaiterState::Idle
                || old(self).waiter_state() == WaiterState::Cancelled,
        ensures
            final(self).well_formed(),
            final(self).contents() == old(self).contents(),
            final(self).waiter_state() == WaiterState::Created,
        no_unwind
    {
        self.waiter = WaiterState::Created;
    }

    /// The registration part of `Notified::poll`. Returning true represents a
    /// notification already recorded for this waiter.
    pub fn poll_notified(&mut self) -> (ready: bool)
        requires
            old(self).well_formed(),
            old(self).waiter_state() == WaiterState::Created
                || old(self).waiter_state() == WaiterState::Registered
                || old(self).waiter_state() == WaiterState::Notified,
        ensures
            final(self).well_formed(),
            final(self).contents() == old(self).contents(),
            ready == (old(self).waiter_state() == WaiterState::Notified),
            ready ==> final(self).waiter_state() == WaiterState::Notified,
            !ready ==> final(self).waiter_state() == WaiterState::Registered,
        no_unwind
    {
        match self.waiter {
            WaiterState::Notified => true,
            WaiterState::Created | WaiterState::Registered => {
                self.waiter = WaiterState::Registered;
                false
            },
            WaiterState::Idle | WaiterState::Cancelled => unreached(),
        }
    }

    /// Relaxed flag check following waiter registration. A Ready result only
    /// requests an outer-loop retry and does not itself expose the value.
    pub fn poll_waiter(&mut self) -> (ready: bool)
        requires
            old(self).well_formed(),
            old(self).waiter_state() == WaiterState::Created
                || old(self).waiter_state() == WaiterState::Registered
                || old(self).waiter_state() == WaiterState::Notified,
        ensures
            final(self).well_formed(),
            final(self).contents() == old(self).contents(),
            ready == (old(self).contents().is_init()
                || old(self).waiter_state() == WaiterState::Notified),
            ready ==> final(self).contents().is_init(),
            !ready ==> final(self).waiter_state() == WaiterState::Registered,
        no_unwind
    {
        let notified = self.poll_notified();
        let published = self.cell.relaxed_is_set();
        notified || published
    }

    /// Publish and notify all waiters. A future which exists but has not yet
    /// registered may miss this notification; its following relaxed flag check
    /// still observes readiness and makes the outer loop retry Acquire `get`.
    pub fn publish_and_notify(&mut self, value: T)
        requires
            old(self).well_formed(),
            old(self).contents() == MemContents::Uninit,
        ensures
            final(self).well_formed(),
            final(self).contents() == MemContents::Init(value),
            old(self).waiter_state() == WaiterState::Registered
                ==> final(self).waiter_state() == WaiterState::Notified,
            old(self).waiter_state() != WaiterState::Registered
                ==> final(self).waiter_state() == old(self).waiter_state(),
        no_unwind
    {
        let old_waiter = self.waiter;
        self.cell.publish(value);
        match old_waiter {
            WaiterState::Registered => self.waiter = WaiterState::Notified,
            WaiterState::Idle
            | WaiterState::Created
            | WaiterState::Notified
            | WaiterState::Cancelled => self.waiter = old_waiter,
        }
    }

    /// Dropping a pending wait future removes only its waiter registration.
    pub fn cancel(&mut self)
        requires
            old(self).well_formed(),
            old(self).waiter_state() == WaiterState::Created
                || old(self).waiter_state() == WaiterState::Registered
                || old(self).waiter_state() == WaiterState::Notified,
        ensures
            final(self).well_formed(),
            final(self).contents() == old(self).contents(),
            final(self).waiter_state() == WaiterState::Cancelled,
        no_unwind
    {
        self.waiter = WaiterState::Cancelled;
    }
}

/// Publication between the initial failed get and waiter registration cannot
/// be lost: the relaxed readiness check sends control back to Acquire get.
pub fn verify_publish_before_registration(value: u64)
{
    let mut protocol = WaitProtocol::empty();
    let initial = protocol.get();
    assert(initial.is_none());
    protocol.create_waiter();
    protocol.publish_and_notify(value);
    let ready = protocol.poll_waiter();
    assert(ready);
    let observed = protocol.get().unwrap();
    assert(*observed == value);
}

/// Publication after registration produces a notification and an exact value.
pub fn verify_publish_after_registration(value: u64)
{
    let mut protocol = WaitProtocol::empty();
    protocol.create_waiter();
    let pending = protocol.poll_waiter();
    assert(!pending);
    protocol.publish_and_notify(value);
    let ready = protocol.poll_waiter();
    assert(ready);
    let observed = protocol.get().unwrap();
    assert(*observed == value);
}

/// Cancellation preserves the cell and a fresh wait can still observe a later
/// publication.
pub fn verify_cancel_then_rewait(value: u64)
{
    let mut protocol = WaitProtocol::empty();
    protocol.create_waiter();
    let pending = protocol.poll_waiter();
    assert(!pending);
    protocol.cancel();
    assert(protocol.contents() == MemContents::Uninit);
    protocol.create_waiter();
    protocol.publish_and_notify(value);
    let ready = protocol.poll_waiter();
    assert(ready);
    let observed = protocol.get().unwrap();
    assert(*observed == value);
}

} // verus!
