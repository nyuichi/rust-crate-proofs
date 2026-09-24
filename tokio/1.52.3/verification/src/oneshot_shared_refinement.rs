use crate::vstd_ext::shared_pcell::{SharedPCell, SharedPCellPermission};
use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlotPermissionPhase {
    Sender,
    Staged,
    Receiver,
    InnerDrop,
    Released,
}

/// Linear capability that connects Tokio's oneshot state role to the exact
/// vstd permission for `UnsafeCell<Option<T>>`.
pub tracked struct OneshotSlotPermission<T> {
    pub tracked cell: SharedPCellPermission<Option<T>>,
    ghost phase: SlotPermissionPhase,
}

impl<T> OneshotSlotPermission<T> {
    pub closed spec fn cell_id(&self) -> vstd::cell::CellId { self.cell.id() }
    pub closed spec fn value(&self) -> Option<T> { self.cell.value() }
    pub closed spec fn phase(&self) -> SlotPermissionPhase { self.phase }
}

/// Body-proved shared-reference view of production `OneshotValue<T>`.  The
/// permission is deliberately not stored in this object: the state protocol
/// transfers it from Sender to the atomic invariant and then to Receiver.
pub struct SharedOneshotValue<T> {
    cell: SharedPCell<Option<T>>,
}

impl<T> SharedOneshotValue<T> {
    pub closed spec fn id(&self) -> vstd::cell::CellId { self.cell.id() }

    pub fn empty() -> (result: (Self, Tracked<OneshotSlotPermission<T>>))
        ensures
            result.1@.cell_id() == result.0.id(),
            result.1@.phase() == SlotPermissionPhase::Sender,
            result.1@.value().is_none(),
        no_unwind
    {
        let (cell, Tracked(permission)) = SharedPCell::new(None);
        (
            SharedOneshotValue { cell },
            Tracked(OneshotSlotPermission {
                cell: permission,
                phase: SlotPermissionPhase::Sender,
            }),
        )
    }

    /// Exact `Inner::value.store(value)` shape: the cell is accessed through
    /// `&self`, while the unique Sender capability supplies the permission.
    pub fn sender_store(
        &self,
        Tracked(permission): Tracked<&mut OneshotSlotPermission<T>>,
        value: T,
    )
        requires
            old(permission).cell_id() == self.id(),
            old(permission).phase() == SlotPermissionPhase::Sender,
            old(permission).value().is_none(),
        ensures
            final(permission).cell_id() == self.id(),
            final(permission).phase() == SlotPermissionPhase::Staged,
            final(permission).value() == Some(value),
        no_unwind
    {
        let previous = self.cell.replace(Tracked(&mut permission.cell), Some(value));
        assert(previous.is_none());
        proof { permission.phase = SlotPermissionPhase::Staged; }
    }

    /// Successful AcqRel completion CAS deposits the slot permission for the
    /// receiver. The atomic memory ordering is the frozen primitive adapter;
    /// the capability transfer and payload preservation are proved here.
    pub proof fn publish_after_complete(
        tracked permission: &mut OneshotSlotPermission<T>,
    )
        requires
            old(permission).phase() == SlotPermissionPhase::Staged,
            old(permission).value().is_some(),
        ensures
            final(permission).phase() == SlotPermissionPhase::Receiver,
            final(permission).cell_id() == old(permission).cell_id(),
            final(permission).value() == old(permission).value(),
    {
        permission.phase = SlotPermissionPhase::Receiver;
    }

    /// If CLOSED wins the completion race, Sender still owns the staged
    /// permission and retrieves precisely its original value through `&self`.
    pub fn sender_take_after_closed(
        &self,
        Tracked(permission): Tracked<&mut OneshotSlotPermission<T>>,
    ) -> (result: T)
        requires
            old(permission).cell_id() == self.id(),
            old(permission).phase() == SlotPermissionPhase::Staged,
            old(permission).value().is_some(),
        ensures
            old(permission).value() == Some(result),
            final(permission).cell_id() == self.id(),
            final(permission).phase() == SlotPermissionPhase::Released,
            final(permission).value().is_none(),
        no_unwind
    {
        let previous = self.cell.replace(Tracked(&mut permission.cell), None);
        proof { permission.phase = SlotPermissionPhase::Released; }
        match previous {
            Some(value) => value,
            None => unreached(),
        }
    }

    /// Acquire observation of VALUE_SENT makes this Receiver capability
    /// available. `has_value` is a shared read and cannot consume it.
    pub fn receiver_has_value(
        &self,
        Tracked(permission): Tracked<&OneshotSlotPermission<T>>,
    ) -> (result: bool)
        requires
            permission.cell_id() == self.id(),
            permission.phase() == SlotPermissionPhase::Receiver,
        ensures
            result == permission.value().is_some(),
            permission.phase() == SlotPermissionPhase::Receiver,
        no_unwind
    {
        self.cell.borrow(Tracked(&permission.cell)).is_some()
    }

    pub fn receiver_take(
        &self,
        Tracked(permission): Tracked<&mut OneshotSlotPermission<T>>,
    ) -> (result: Option<T>)
        requires
            old(permission).cell_id() == self.id(),
            old(permission).phase() == SlotPermissionPhase::Receiver,
        ensures
            result == old(permission).value(),
            final(permission).cell_id() == self.id(),
            final(permission).phase() == SlotPermissionPhase::Released,
            final(permission).value().is_none(),
        no_unwind
    {
        let result = self.cell.replace(Tracked(&mut permission.cell), None);
        proof { permission.phase = SlotPermissionPhase::Released; }
        result
    }

    pub proof fn transfer_to_inner_drop(
        tracked permission: &mut OneshotSlotPermission<T>,
    )
        requires
            old(permission).phase() == SlotPermissionPhase::Sender,
            old(permission).value().is_none(),
        ensures
            final(permission).phase() == SlotPermissionPhase::InnerDrop,
            final(permission).cell_id() == old(permission).cell_id(),
            final(permission).value().is_none(),
    {
        permission.phase = SlotPermissionPhase::InnerDrop;
    }

    pub fn inner_drop_take(
        &self,
        Tracked(permission): Tracked<&mut OneshotSlotPermission<T>>,
    ) -> (result: Option<T>)
        requires
            old(permission).cell_id() == self.id(),
            old(permission).phase() == SlotPermissionPhase::InnerDrop,
        ensures
            result == old(permission).value(),
            final(permission).phase() == SlotPermissionPhase::Released,
            final(permission).value().is_none(),
        no_unwind
    {
        let result = self.cell.replace(Tracked(&mut permission.cell), None);
        proof { permission.phase = SlotPermissionPhase::Released; }
        result
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaskSlotPhase { Vacant, Writing, Published, Detached, Released }

/// Exact transition-in-progress ownership for either RX_TASK_SET or
/// TX_TASK_SET. A Waker id is used because Waker clone/wake/drop execution is
/// frozen; slot ownership, bit publication, replacement, and terminal cleanup
/// are not.
pub tracked struct TaskSlotPermission {
    ghost phase: TaskSlotPhase,
    ghost slot: Option<u64>,
    ghost task_bit: bool,
}

impl TaskSlotPermission {
    pub closed spec fn phase(&self) -> TaskSlotPhase { self.phase }
    pub closed spec fn slot(&self) -> Option<u64> { self.slot }
    pub closed spec fn task_bit(&self) -> bool { self.task_bit }
    pub closed spec fn well_formed(&self) -> bool {
        &&& self.task_bit() == (self.phase() == TaskSlotPhase::Published)
        &&& self.slot().is_some() == matches!(self.phase(),
            TaskSlotPhase::Writing | TaskSlotPhase::Published | TaskSlotPhase::Detached)
    }

    pub proof fn vacant() -> (tracked result: Self)
        ensures result.well_formed(), result.phase() == TaskSlotPhase::Vacant,
            result.slot().is_none(), !result.task_bit(),
    {
        TaskSlotPermission { phase: TaskSlotPhase::Vacant, slot: None, task_bit: false }
    }

    /// Write happens while the task bit is clear, so a concurrent terminal
    /// operation cannot read or wake a partially initialized Waker.
    pub proof fn begin_write(tracked &mut self, waker: u64)
        requires old(self).well_formed(), old(self).phase() == TaskSlotPhase::Vacant,
        ensures final(self).well_formed(), final(self).phase() == TaskSlotPhase::Writing,
            final(self).slot() == Some(waker), !final(self).task_bit(),
    {
        self.phase = TaskSlotPhase::Writing;
        self.slot = Some(waker);
    }

    pub proof fn publish_bit(tracked &mut self)
        requires old(self).well_formed(), old(self).phase() == TaskSlotPhase::Writing,
        ensures final(self).well_formed(), final(self).phase() == TaskSlotPhase::Published,
            final(self).slot() == old(self).slot(), final(self).task_bit(),
    {
        self.phase = TaskSlotPhase::Published;
        self.task_bit = true;
    }

    /// AcqRel bit clear linearizes before reading/taking the Waker slot.
    pub proof fn unset_and_detach(tracked &mut self) -> (waker: u64)
        requires old(self).well_formed(), old(self).phase() == TaskSlotPhase::Published,
        ensures final(self).well_formed(), final(self).phase() == TaskSlotPhase::Detached,
            old(self).slot() == Some(waker), final(self).slot() == Some(waker),
            !final(self).task_bit(),
    {
        self.task_bit = false;
        self.phase = TaskSlotPhase::Detached;
        match self.slot {
            Some(waker) => waker,
            None => proof_from_false(),
        }
    }

    pub proof fn finish_drop(tracked &mut self, reuse: bool)
        requires old(self).well_formed(), old(self).phase() == TaskSlotPhase::Detached,
        ensures final(self).well_formed(), final(self).slot().is_none(),
            !final(self).task_bit(),
            reuse ==> final(self).phase() == TaskSlotPhase::Vacant,
            !reuse ==> final(self).phase() == TaskSlotPhase::Released,
    {
        self.slot = None;
        self.phase = if reuse { TaskSlotPhase::Vacant } else { TaskSlotPhase::Released };
    }
}

pub proof fn verify_registration_terminal_race(first: u64, replacement: u64)
{
    let tracked mut task = TaskSlotPermission::vacant();
    task.begin_write(first);
    assert(!task.task_bit());
    task.publish_bit();
    let removed = task.unset_and_detach();
    assert(removed == first);
    task.finish_drop(true);
    task.begin_write(replacement);
    // A terminal state observed by the post-publication recheck either sees
    // no bit here, or removes the fully initialized slot after publication.
    task.publish_bit();
    let terminal = task.unset_and_detach();
    assert(terminal == replacement);
    task.finish_drop(false);
    assert(task.phase() == TaskSlotPhase::Released);
}

pub fn verify_shared_slot_send_receive(value: u64)
{
    let (slot, Tracked(mut permission)) = SharedOneshotValue::empty();
    slot.sender_store(Tracked(&mut permission), value);
    proof { SharedOneshotValue::publish_after_complete(&mut permission); }
    let present = slot.receiver_has_value(Tracked(&permission));
    assert(present);
    let received = slot.receiver_take(Tracked(&mut permission));
    assert(received == Some(value));
}

pub fn verify_shared_slot_closed_return(value: u64)
{
    let (slot, Tracked(mut permission)) = SharedOneshotValue::empty();
    slot.sender_store(Tracked(&mut permission), value);
    let returned = slot.sender_take_after_closed(Tracked(&mut permission));
    assert(returned == value);
}

} // verus!
