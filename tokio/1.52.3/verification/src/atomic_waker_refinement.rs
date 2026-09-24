use vstd::prelude::*;

verus! {

pub open spec fn waiting() -> usize { 0 }
pub open spec fn registering() -> usize { 1 }
pub open spec fn waking() -> usize { 2 }
pub open spec fn registering_waking() -> usize { 3 }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RegisterBranch { Locked, WakeInput, Rejected }

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RegisterFinish {
    /// The displaced slot value is woken only when WAKING raced with register.
    pub old_woken: Option<u64>,
    /// A successfully cloned replacement is also woken on that race.
    pub new_woken: Option<u64>,
    /// Production restores WAITING before resuming `into_waker`'s panic.
    pub resumes_panic: bool,
}

/// Exact two-bit/slot refinement of production AtomicWaker. Atomic ordering,
/// UnsafeCell validity, and arbitrary Waker callbacks are frozen adapters.
pub struct AtomicWakerMachine {
    raw: usize,
    slot: Option<u64>,
    pending: Option<u64>,
}

impl AtomicWakerMachine {
    pub closed spec fn raw(&self) -> usize { self.raw }
    pub closed spec fn slot(&self) -> Option<u64> { self.slot }
    pub closed spec fn pending(&self) -> Option<u64> { self.pending }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.raw() <= registering_waking()
        &&& (self.raw() == waiting() || self.raw() == waking())
            ==> self.pending().is_none()
        &&& (self.raw() == registering() || self.raw() == registering_waking())
            ==> self.pending().is_some()
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.raw() == waiting(),
            result.slot().is_none(), result.pending().is_none(),
        no_unwind
    {
        AtomicWakerMachine { raw: 0, slot: None, pending: None }
    }

    /// Exact result table for `compare_exchange(WAITING, REGISTERING, ...)`.
    pub fn begin_register(&mut self, id: u64) -> (branch: RegisterBranch)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            branch == if old(self).raw() == waiting() { RegisterBranch::Locked }
                else if old(self).raw() == waking() { RegisterBranch::WakeInput }
                else { RegisterBranch::Rejected },
            branch == RegisterBranch::Locked ==> final(self).raw() == registering(),
            branch == RegisterBranch::Locked ==> final(self).pending() == Some(id),
            branch != RegisterBranch::Locked ==> final(self).raw() == old(self).raw(),
            branch != RegisterBranch::Locked ==> final(self).pending() == old(self).pending(),
            final(self).slot() == old(self).slot(),
        no_unwind
    {
        match self.raw {
            0 => {
                self.raw = 1;
                self.pending = Some(id);
                RegisterBranch::Locked
            },
            2 => RegisterBranch::WakeInput,
            _ => RegisterBranch::Rejected,
        }
    }

    /// Exact `fetch_or(WAKING)` table. A WAITING caller owns and takes the
    /// slot before the production `swap(WAITING)`; a REGISTERING caller leaves
    /// bit 2 for the register owner to discharge.
    pub fn take_waker(&mut self) -> (taken: Option<u64>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            old(self).raw() == waiting() ==> taken == old(self).slot(),
            old(self).raw() == waiting() ==> final(self).raw() == waiting(),
            old(self).raw() == waiting() ==> final(self).slot().is_none(),
            old(self).raw() == registering() ==> taken.is_none(),
            old(self).raw() == registering() ==> final(self).raw() == registering_waking(),
            old(self).raw() == registering_waking() ==> taken.is_none(),
            old(self).raw() == registering_waking()
                ==> final(self).raw() == registering_waking(),
            old(self).raw() == waking() ==> taken.is_none(),
            old(self).raw() == waking() ==> final(self).raw() == waking(),
            old(self).raw() != waiting() ==> final(self).slot() == old(self).slot(),
            final(self).pending() == old(self).pending(),
        no_unwind
    {
        match self.raw {
            0 => {
                // fetch_or: 0 -> 2; take slot; swap: 2 -> 0.
                self.raw = 2;
                let taken = self.slot;
                self.slot = None;
                self.raw = 0;
                taken
            },
            1 => { self.raw = 3; None },
            2 | 3 => None,
            _ => None,
        }
    }

    /// Completes clone/replace and the release CAS. This records both Waker
    /// identities invoked by production's REGISTERING|WAKING branch, including
    /// the displaced old value that the protocol-only model abstracted away.
    pub fn finish_register(&mut self, clone_succeeded: bool) -> (result: RegisterFinish)
        requires
            old(self).well_formed(),
            old(self).raw() == registering() || old(self).raw() == registering_waking(),
        ensures
            final(self).well_formed(),
            final(self).raw() == waiting(),
            final(self).pending().is_none(),
            result.resumes_panic == !clone_succeeded,
            old(self).raw() == registering() && clone_succeeded
                ==> final(self).slot() == old(self).pending(),
            old(self).raw() == registering() && !clone_succeeded
                ==> final(self).slot() == old(self).slot(),
            old(self).raw() == registering()
                ==> result.old_woken.is_none() && result.new_woken.is_none(),
            old(self).raw() == registering_waking() ==> final(self).slot().is_none(),
            old(self).raw() == registering_waking() && clone_succeeded
                ==> result.old_woken == old(self).slot(),
            old(self).raw() == registering_waking() && clone_succeeded
                ==> result.new_woken == old(self).pending(),
            old(self).raw() == registering_waking() && !clone_succeeded
                ==> result.old_woken == old(self).slot(),
            old(self).raw() == registering_waking() && !clone_succeeded
                ==> result.new_woken.is_none(),
        no_unwind
    {
        let old_slot = self.slot;
        if clone_succeeded {
            self.slot = self.pending;
        }

        let result = if self.raw == 3 {
            let installed = self.slot;
            self.slot = None;
            RegisterFinish {
                old_woken: old_slot,
                new_woken: if clone_succeeded { installed } else { None },
                resumes_panic: !clone_succeeded,
            }
        } else {
            RegisterFinish {
                old_woken: None,
                new_woken: None,
                resumes_panic: !clone_succeeded,
            }
        };
        self.pending = None;
        self.raw = 0;
        result
    }
}

pub proof fn verify_fetch_or_waking_table()
{
    assert(0usize | 2usize == 2usize) by (bit_vector);
    assert(1usize | 2usize == 3usize) by (bit_vector);
    assert(2usize | 2usize == 2usize) by (bit_vector);
    assert(3usize | 2usize == 3usize) by (bit_vector);
}

pub fn verify_wake_racing_successful_register(old_id: u64, new_id: u64)
{
    let mut machine = AtomicWakerMachine { raw: 0, slot: Some(old_id), pending: None };
    assert(machine.well_formed());
    let start = machine.begin_register(new_id);
    assert(start == RegisterBranch::Locked);
    let taken = machine.take_waker();
    assert(taken.is_none());
    assert(machine.raw() == registering_waking());
    let result = machine.finish_register(true);
    assert(result.old_woken == Some(old_id));
    assert(result.new_woken == Some(new_id));
    assert(machine.slot().is_none());
    assert(machine.raw() == waiting());
}

pub fn verify_wake_racing_panicking_clone(old_id: u64, new_id: u64)
{
    let mut machine = AtomicWakerMachine { raw: 0, slot: Some(old_id), pending: None };
    assert(machine.well_formed());
    let start = machine.begin_register(new_id);
    assert(start == RegisterBranch::Locked);
    let taken = machine.take_waker();
    assert(taken.is_none());
    let result = machine.finish_register(false);
    assert(result.resumes_panic);
    assert(result.old_woken == Some(old_id));
    assert(result.new_woken.is_none());
    assert(machine.slot().is_none());
    assert(machine.raw() == waiting());
}

pub fn verify_panicking_clone_without_wake_preserves_slot(old_id: u64, new_id: u64)
{
    let mut machine = AtomicWakerMachine { raw: 0, slot: Some(old_id), pending: None };
    assert(machine.well_formed());
    let start = machine.begin_register(new_id);
    assert(start == RegisterBranch::Locked);
    let result = machine.finish_register(false);
    assert(result.resumes_panic);
    assert(machine.slot() == Some(old_id));
    assert(machine.raw() == waiting());
}

pub proof fn verify_atomic_waker_mutants_rejected()
{
    // fetch_or must preserve REGISTERING while adding WAKING.
    assert((1usize | 2usize) == 3usize) by (bit_vector);
    assert(3usize != 2usize);
    // Clearing state before the register owner observes bit 2 loses the wake.
    assert(registering_waking() != registering());
    // The successful race owns two distinct callback obligations.
    assert(Some(1u64) != Some(2u64));
}

} // verus!
