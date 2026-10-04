use vstd::prelude::*;

verus! {

/// Exact logical interpretation of Tokio's two state bits.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AtomicWakerState {
    Waiting,
    Registering,
    RegisteringWaking,
    Waking,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RegisterStart {
    /// WAITING -> REGISTERING; caller owns the slot critical section.
    Locked,
    /// Registration raced with WAKING, so the input waker is woken directly.
    WakeInput,
    /// Another register owns the slot; this competing registration is dropped.
    Rejected,
}

/// Non-atomic proof state for Tokio's `AtomicWaker`. Raw atomic orderings and
/// UnsafeCell are frozen foundation adapters. The state-bit protocol and exact
/// ownership of the optional logical waker are proved here.
pub struct AtomicWakerProtocol {
    state: AtomicWakerState,
    slot: Option<u64>,
    pending_register: Option<u64>,
    waking_value: Option<u64>,
}

impl AtomicWakerProtocol {
    pub closed spec fn state(&self) -> AtomicWakerState { self.state }
    pub closed spec fn slot(&self) -> Option<u64> { self.slot }
    pub closed spec fn pending_register(&self) -> Option<u64> { self.pending_register }
    pub closed spec fn waking_value(&self) -> Option<u64> { self.waking_value }

    pub closed spec fn well_formed(&self) -> bool {
        match self.state() {
            AtomicWakerState::Waiting => {
                self.pending_register().is_none() && self.waking_value().is_none()
            },
            AtomicWakerState::Registering | AtomicWakerState::RegisteringWaking => {
                self.pending_register().is_some() && self.waking_value().is_none()
            },
            AtomicWakerState::Waking => {
                self.pending_register().is_none()
            },
        }
    }

    pub fn new() -> (result: Self)
        ensures
            result.well_formed(),
            result.state() == AtomicWakerState::Waiting,
            result.slot().is_none(),
        no_unwind
    {
        AtomicWakerProtocol {
            state: AtomicWakerState::Waiting,
            slot: None,
            pending_register: None,
            waking_value: None,
        }
    }

    pub fn begin_register(&mut self, id: u64) -> (outcome: RegisterStart)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            outcome == match old(self).state() {
                AtomicWakerState::Waiting => RegisterStart::Locked,
                AtomicWakerState::Waking => RegisterStart::WakeInput,
                AtomicWakerState::Registering
                | AtomicWakerState::RegisteringWaking => RegisterStart::Rejected,
            },
            outcome == RegisterStart::Locked
                ==> final(self).state() == AtomicWakerState::Registering,
            outcome == RegisterStart::Locked
                ==> final(self).pending_register() == Some(id),
            outcome == RegisterStart::WakeInput
                ==> final(self).state() == AtomicWakerState::Waking,
            outcome == RegisterStart::Rejected
                ==> final(self).state() == old(self).state(),
            final(self).slot() == old(self).slot(),
            final(self).waking_value() == old(self).waking_value(),
        no_unwind
    {
        match self.state {
            AtomicWakerState::Waiting => {
                self.state = AtomicWakerState::Registering;
                self.pending_register = Some(id);
                RegisterStart::Locked
            },
            AtomicWakerState::Waking => {
                RegisterStart::WakeInput
            },
            AtomicWakerState::Registering | AtomicWakerState::RegisteringWaking => {
                RegisterStart::Rejected
            },
        }
    }

    /// Models the `fetch_or(WAKING)` linearization point.
    pub fn begin_wake(&mut self) -> (owns_wake_lock: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            owns_wake_lock == (old(self).state() == AtomicWakerState::Waiting),
            old(self).state() == AtomicWakerState::Waiting
                ==> final(self).state() == AtomicWakerState::Waking,
            old(self).state() == AtomicWakerState::Waiting
                ==> final(self).waking_value() == old(self).slot(),
            old(self).state() == AtomicWakerState::Waiting
                ==> final(self).slot().is_none(),
            old(self).state() == AtomicWakerState::Registering
                ==> final(self).state() == AtomicWakerState::RegisteringWaking,
            old(self).state() == AtomicWakerState::RegisteringWaking
                ==> final(self).state() == AtomicWakerState::RegisteringWaking,
            old(self).state() == AtomicWakerState::Waking
                ==> final(self).state() == AtomicWakerState::Waking,
            final(self).pending_register() == old(self).pending_register(),
            old(self).state() != AtomicWakerState::Waiting
                ==> final(self).slot() == old(self).slot(),
            old(self).state() != AtomicWakerState::Waiting
                ==> final(self).waking_value() == old(self).waking_value(),
        no_unwind
    {
        match self.state {
            AtomicWakerState::Waiting => {
                self.state = AtomicWakerState::Waking;
                self.waking_value = self.slot;
                self.slot = None;
                true
            },
            AtomicWakerState::Registering => {
                self.state = AtomicWakerState::RegisteringWaking;
                false
            },
            AtomicWakerState::RegisteringWaking | AtomicWakerState::Waking => false,
        }
    }

    pub fn finish_wake(&mut self) -> (woken: Option<u64>)
        requires
            old(self).well_formed(),
            old(self).state() == AtomicWakerState::Waking,
        ensures
            final(self).well_formed(),
            woken == old(self).waking_value(),
            final(self).state() == AtomicWakerState::Waiting,
            final(self).waking_value().is_none(),
            final(self).slot() == old(self).slot(),
        no_unwind
    {
        let woken = self.waking_value;
        self.waking_value = None;
        self.state = AtomicWakerState::Waiting;
        woken
    }

    /// Finishes the registering critical section. `clone_succeeded == false`
    /// is the `WakerRef::into_waker` unwind path: the old slot is preserved.
    /// If WAKING arrived meanwhile, whichever value is in the slot is consumed
    /// exactly once before the state returns to WAITING.
    pub fn finish_register(&mut self, clone_succeeded: bool) -> (woken: Option<u64>)
        requires
            old(self).well_formed(),
            old(self).state() == AtomicWakerState::Registering
                || old(self).state() == AtomicWakerState::RegisteringWaking,
        ensures
            final(self).well_formed(),
            final(self).state() == AtomicWakerState::Waiting,
            final(self).pending_register().is_none(),
            old(self).state() == AtomicWakerState::Registering && clone_succeeded
                ==> final(self).slot() == old(self).pending_register(),
            old(self).state() == AtomicWakerState::Registering && !clone_succeeded
                ==> final(self).slot() == old(self).slot(),
            old(self).state() == AtomicWakerState::RegisteringWaking
                ==> final(self).slot().is_none(),
            old(self).state() == AtomicWakerState::RegisteringWaking && clone_succeeded
                ==> woken == old(self).pending_register(),
            old(self).state() == AtomicWakerState::RegisteringWaking && !clone_succeeded
                ==> woken == old(self).slot(),
            old(self).state() == AtomicWakerState::Registering ==> woken.is_none(),
        no_unwind
    {
        let pending = self.pending_register;
        let wake_pending = match self.state {
            AtomicWakerState::RegisteringWaking => true,
            AtomicWakerState::Registering => false,
            AtomicWakerState::Waiting | AtomicWakerState::Waking => false,
        };
        if clone_succeeded {
            self.slot = pending;
        }

        let woken = if wake_pending {
            let value = self.slot;
            self.slot = None;
            value
        } else {
            None
        };

        self.pending_register = None;
        self.state = AtomicWakerState::Waiting;
        woken
    }
}

pub fn verify_registered_value_is_taken(id: u64)
{
    let mut protocol = AtomicWakerProtocol::new();
    let start = protocol.begin_register(id);
    assert(start == RegisterStart::Locked);
    let concurrent = protocol.finish_register(true);
    assert(concurrent.is_none());
    assert(protocol.slot() == Some(id));
    let owns = protocol.begin_wake();
    assert(owns);
    let woken = protocol.finish_wake();
    assert(woken == Some(id));
    assert(protocol.slot().is_none());
}

pub fn verify_wake_during_register_is_not_lost(id: u64)
{
    let mut protocol = AtomicWakerProtocol::new();
    let start = protocol.begin_register(id);
    assert(start == RegisterStart::Locked);
    let owns = protocol.begin_wake();
    assert(!owns);
    assert(protocol.state() == AtomicWakerState::RegisteringWaking);
    let woken = protocol.finish_register(true);
    assert(woken == Some(id));
    assert(protocol.state() == AtomicWakerState::Waiting);
}

pub fn verify_register_during_wake_self_wakes(old_id: u64, new_id: u64)
{
    let mut protocol = AtomicWakerProtocol::new();
    let registered = protocol.begin_register(old_id);
    assert(registered == RegisterStart::Locked);
    let stored = protocol.finish_register(true);
    assert(stored.is_none());
    let wake_started = protocol.begin_wake();
    assert(wake_started);
    let outcome = protocol.begin_register(new_id);
    assert(outcome == RegisterStart::WakeInput);
    let old_woken = protocol.finish_wake();
    assert(old_woken == Some(old_id));
}

pub fn verify_panicking_clone_restores_waiting(old_id: u64, new_id: u64)
{
    let mut protocol = AtomicWakerProtocol::new();
    let old_registered = protocol.begin_register(old_id);
    assert(old_registered == RegisterStart::Locked);
    let old_stored = protocol.finish_register(true);
    assert(old_stored.is_none());
    let new_registered = protocol.begin_register(new_id);
    assert(new_registered == RegisterStart::Locked);
    let woken = protocol.finish_register(false);
    assert(woken.is_none());
    assert(protocol.state() == AtomicWakerState::Waiting);
    assert(protocol.slot() == Some(old_id));
}

} // verus!
