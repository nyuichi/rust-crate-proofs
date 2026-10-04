use crate::oneshot_state::OneshotBits;
use vstd::cell::pcell::{PCell, PointsTo};
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

/// Physical proof view of production `UnsafeCell<Option<T>>`.
pub struct OneshotValue<T> {
    cell: PCell<Option<T>>,
    permission: Tracked<PointsTo<Option<T>>>,
}

impl<T> OneshotValue<T> {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.permission@.id() == self.cell.id()
    }

    pub closed spec fn contents(&self) -> Option<T> {
        *self.permission@.value()
    }

    pub fn empty() -> (result: Self)
        ensures
            result.contents().is_none(),
    {
        let (cell, permission) = PCell::new(None);
        OneshotValue { cell, permission }
    }

    pub fn store(&mut self, value: T)
        requires
            old(self).contents().is_none(),
        ensures
            final(self).contents() == Some(value),
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        let previous = self.cell.replace(Tracked(self.permission.borrow_mut()), Some(value));
        assert(previous.is_none());
    }

    pub fn take(&mut self) -> (result: Option<T>)
        ensures
            result == old(self).contents(),
            final(self).contents().is_none(),
        no_unwind
    {
        proof { use_type_invariant(&*self); }
        self.cell.replace(Tracked(self.permission.borrow_mut()), None)
    }

    pub fn is_empty(&self) -> (result: bool)
        ensures result == self.contents().is_none(),
        no_unwind
    {
        proof { use_type_invariant(self); }
        self.cell.borrow(Tracked(self.permission.borrow())).is_none()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TryRecvErrorModel {
    Empty,
    Closed,
}

/// Sequential ownership refinement for production send, close, try_recv, and
/// both endpoint destructors. Concurrent linearization is added by the atomic
/// state-machine layer; this model fixes the exact value outcomes.
pub struct OneshotModel<T> {
    bits: OneshotBits,
    value: OneshotValue<T>,
    sender_active: bool,
    receiver_active: bool,
}

impl<T> OneshotModel<T> {
    pub closed spec fn value(&self) -> Option<T> { self.value.contents() }
    pub closed spec fn sender_active(&self) -> bool { self.sender_active }
    pub closed spec fn receiver_active(&self) -> bool { self.receiver_active }
    pub closed spec fn complete(&self) -> bool { self.bits.value_sent() }
    pub closed spec fn closed(&self) -> bool { self.bits.closed() }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.value().is_some() ==> self.complete() && !self.sender_active()
        &&& self.sender_active() ==> !self.complete() && self.value().is_none()
        &&& !self.receiver_active() ==> self.closed()
        &&& !self.sender_active() && !self.closed() ==> self.complete()
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.sender_active(),
            result.receiver_active(),
            !result.complete(),
            !result.closed(),
            result.value().is_none(),
    {
        OneshotModel {
            bits: OneshotBits::new(),
            value: OneshotValue::empty(),
            sender_active: true,
            receiver_active: true,
        }
    }

    pub fn send(&mut self, value: T) -> (result: Result<(), T>)
        requires
            old(self).well_formed(),
            old(self).sender_active(),
        ensures
            final(self).well_formed(),
            !final(self).sender_active(),
            final(self).receiver_active() == old(self).receiver_active(),
            old(self).closed() ==> result == Err(value),
            old(self).closed() ==> final(self).value().is_none(),
            !old(self).closed() ==> result == Ok(()),
            !old(self).closed() ==> final(self).value() == Some(value),
            !old(self).closed() ==> final(self).complete(),
        no_unwind
    {
        self.value.store(value);
        let completed = self.bits.set_complete();
        self.sender_active = false;
        if completed {
            Ok(())
        } else {
            let returned = self.value.take();
            match returned {
                Some(returned) => Err(returned),
                None => unreached(),
            }
        }
    }

    pub fn sender_drop(&mut self)
        requires
            old(self).well_formed(),
            old(self).sender_active(),
        ensures
            final(self).well_formed(),
            !final(self).sender_active(),
            final(self).receiver_active() == old(self).receiver_active(),
            final(self).value().is_none(),
            !old(self).closed() ==> final(self).complete(),
        no_unwind
    {
        self.bits.set_complete();
        self.sender_active = false;
    }

    pub fn close(&mut self)
        requires
            old(self).well_formed(),
            old(self).receiver_active(),
        ensures
            final(self).well_formed(),
            final(self).closed(),
            final(self).sender_active() == old(self).sender_active(),
            final(self).receiver_active(),
            final(self).value() == old(self).value(),
        no_unwind
    {
        self.bits.set_closed();
    }

    pub fn try_recv(&mut self) -> (result: Result<T, TryRecvErrorModel>)
        requires
            old(self).well_formed(),
            old(self).receiver_active(),
        ensures
            final(self).well_formed(),
            final(self).sender_active() == old(self).sender_active(),
            match result {
                Ok(value) => {
                    old(self).value() == Some(value)
                        && !final(self).receiver_active()
                        && final(self).value().is_none()
                },
                Err(TryRecvErrorModel::Empty) => {
                    !old(self).complete()
                        && !old(self).closed()
                        && final(self).receiver_active()
                        && final(self).value() == old(self).value()
                },
                Err(TryRecvErrorModel::Closed) => {
                    (old(self).complete() || old(self).closed())
                        && old(self).value().is_none()
                        && !final(self).receiver_active()
                },
            },
        no_unwind
    {
        if self.bits.is_complete() {
            let value = self.value.take();
            self.bits.set_closed();
            self.receiver_active = false;
            match value {
                Some(value) => Ok(value),
                None => Err(TryRecvErrorModel::Closed),
            }
        } else if self.bits.is_closed() {
            self.receiver_active = false;
            Err(TryRecvErrorModel::Closed)
        } else {
            Err(TryRecvErrorModel::Empty)
        }
    }

    pub fn receiver_drop(&mut self) -> (dropped: Option<T>)
        requires
            old(self).well_formed(),
            old(self).receiver_active(),
        ensures
            final(self).well_formed(),
            !final(self).receiver_active(),
            final(self).sender_active() == old(self).sender_active(),
            final(self).closed(),
            dropped == old(self).value(),
            final(self).value().is_none(),
        no_unwind
    {
        self.bits.set_closed();
        let dropped = self.value.take();
        self.receiver_active = false;
        dropped
    }
}

pub fn verify_send_receive_roundtrip(value: u64)
{
    let mut channel = OneshotModel::new();
    let sent = channel.send(value);
    assert(sent == Ok(()));
    let received = channel.try_recv();
    assert(received == Ok(value));
}

pub fn verify_close_returns_value(value: u64)
{
    let mut channel = OneshotModel::new();
    channel.close();
    let rejected = channel.send(value);
    assert(rejected == Err(value));
}

pub fn verify_sender_drop_reports_closed()
{
    let mut channel = OneshotModel::<u64>::new();
    channel.sender_drop();
    let received = channel.try_recv();
    assert(received == Err(TryRecvErrorModel::Closed));
}

pub fn verify_receiver_drop_takes_sent_value(value: u64)
{
    let mut channel = OneshotModel::new();
    let sent = channel.send(value);
    assert(sent == Ok(()));
    let dropped = channel.receiver_drop();
    assert(dropped == Some(value));
}

} // verus!
