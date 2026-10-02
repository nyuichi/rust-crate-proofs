use vstd::prelude::*;

verus! {

pub open spec fn ref_one() -> nat { 64 }
pub open spec fn initial_raw() -> nat { 3 * ref_one() + 8 + 4 }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ToRunning { Success, Cancelled, Failed, Dealloc }
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ToIdle { Ok, OkNotified, OkDealloc, Cancelled }
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NotifyByVal { DoNothing, Submit, Dealloc }
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NotifyByRef { DoNothing, Submit }

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct JoinDrop {
    pub drop_waker: bool,
    pub drop_output: bool,
}

/// Field-wise refinement of the packed word in `runtime/task/state.rs`.
/// Atomic modification order is frozen; Tokio's flag/refcount arithmetic and
/// every branch result are executable body proofs here.
pub struct TaskStateModel {
    running: bool,
    complete: bool,
    notified: bool,
    join_interest: bool,
    join_waker: bool,
    cancelled: bool,
    refs: usize,
}

impl TaskStateModel {
    pub closed spec fn running(&self) -> bool { self.running }
    pub closed spec fn complete(&self) -> bool { self.complete }
    pub closed spec fn notified(&self) -> bool { self.notified }
    pub closed spec fn join_interest(&self) -> bool { self.join_interest }
    pub closed spec fn join_waker(&self) -> bool { self.join_waker }
    pub closed spec fn cancelled(&self) -> bool { self.cancelled }
    pub closed spec fn refs(&self) -> usize { self.refs }
    pub closed spec fn idle(&self) -> bool { !self.running() && !self.complete() }
    pub closed spec fn raw(&self) -> nat {
        self.refs() as nat * ref_one()
            + if self.running() { 1nat } else { 0nat }
            + if self.complete() { 2nat } else { 0nat }
            + if self.notified() { 4nat } else { 0nat }
            + if self.join_interest() { 8nat } else { 0nat }
            + if self.join_waker() { 16nat } else { 0nat }
            + if self.cancelled() { 32nat } else { 0nat }
    }
    pub closed spec fn lifecycle_valid(&self) -> bool {
        !(self.running() && self.complete())
    }

    pub fn new() -> (result: Self)
        ensures
            result.lifecycle_valid(), result.refs() == 3,
            result.idle(), result.notified(), result.join_interest(),
            !result.join_waker(), !result.cancelled(),
            result.raw() == initial_raw(), initial_raw() == 204,
        no_unwind
    {
        TaskStateModel {
            running: false, complete: false, notified: true,
            join_interest: true, join_waker: false, cancelled: false, refs: 3,
        }
    }

    pub fn transition_to_running(&mut self) -> (action: ToRunning)
        requires old(self).lifecycle_valid(), old(self).notified(), old(self).refs() > 0,
            (old(self).running() ==> old(self).refs() > 1),
        ensures
            final(self).lifecycle_valid(),
            old(self).idle() ==> {
                &&& final(self).running() && !final(self).complete()
                &&& !final(self).notified()
                &&& final(self).refs() == old(self).refs()
                &&& action == if old(self).cancelled() { ToRunning::Cancelled } else { ToRunning::Success }
            },
            !old(self).idle() ==> {
                &&& final(self).refs() + 1 == old(self).refs()
                &&& action == if old(self).refs() == 1 { ToRunning::Dealloc } else { ToRunning::Failed }
            },
        no_unwind
    {
        if self.running || self.complete {
            self.refs -= 1;
            if self.refs == 0 { ToRunning::Dealloc } else { ToRunning::Failed }
        } else {
            self.running = true;
            self.notified = false;
            if self.cancelled { ToRunning::Cancelled } else { ToRunning::Success }
        }
    }

    pub fn transition_to_idle(&mut self) -> (action: ToIdle)
        requires old(self).lifecycle_valid(), old(self).running(), old(self).refs() > 0,
            old(self).notified() ==> old(self).refs() < usize::MAX,
        ensures
            final(self).lifecycle_valid(),
            old(self).cancelled() ==> action == ToIdle::Cancelled
                && final(self).raw() == old(self).raw(),
            !old(self).cancelled() && old(self).notified() ==> {
                &&& action == ToIdle::OkNotified
                &&& final(self).idle() && final(self).notified()
                &&& final(self).refs() == old(self).refs() + 1
            },
            !old(self).cancelled() && !old(self).notified() ==> {
                &&& final(self).idle() && !final(self).notified()
                &&& final(self).refs() + 1 == old(self).refs()
                &&& action == if old(self).refs() == 1 { ToIdle::OkDealloc } else { ToIdle::Ok }
            },
        no_unwind
    {
        if self.cancelled {
            ToIdle::Cancelled
        } else {
            self.running = false;
            if self.notified {
                self.refs += 1;
                ToIdle::OkNotified
            } else {
                self.refs -= 1;
                if self.refs == 0 { ToIdle::OkDealloc } else { ToIdle::Ok }
            }
        }
    }

    pub fn transition_to_complete(&mut self)
        requires old(self).lifecycle_valid(), old(self).running(), !old(self).complete(),
        ensures final(self).lifecycle_valid(), !final(self).running(), final(self).complete(),
            final(self).refs() == old(self).refs(),
            final(self).notified() == old(self).notified(),
        no_unwind
    {
        self.running = false;
        self.complete = true;
    }

    pub fn transition_to_terminal(&mut self, count: usize) -> (dealloc: bool)
        requires old(self).complete(), count <= old(self).refs(),
        ensures final(self).refs() + count == old(self).refs(),
            dealloc == (final(self).refs() == 0), final(self).complete(),
        no_unwind
    {
        self.refs -= count;
        self.refs == 0
    }

    pub fn notify_by_val(&mut self) -> (action: NotifyByVal)
        requires old(self).lifecycle_valid(), old(self).refs() > 0,
            old(self).running() ==> old(self).refs() > 1,
            old(self).idle() && !old(self).notified() ==> old(self).refs() < usize::MAX,
        ensures
            final(self).lifecycle_valid(), final(self).complete() == old(self).complete(),
            old(self).running() ==> action == NotifyByVal::DoNothing
                && final(self).notified() && final(self).refs() + 1 == old(self).refs(),
            !old(self).running() && (old(self).complete() || old(self).notified()) ==>
                final(self).refs() + 1 == old(self).refs()
                && action == if old(self).refs() == 1 { NotifyByVal::Dealloc } else { NotifyByVal::DoNothing },
            old(self).idle() && !old(self).notified() ==> action == NotifyByVal::Submit
                && final(self).notified() && final(self).refs() == old(self).refs() + 1,
        no_unwind
    {
        if self.running {
            self.notified = true;
            self.refs -= 1;
            NotifyByVal::DoNothing
        } else if self.complete || self.notified {
            self.refs -= 1;
            if self.refs == 0 { NotifyByVal::Dealloc } else { NotifyByVal::DoNothing }
        } else {
            self.notified = true;
            self.refs += 1;
            NotifyByVal::Submit
        }
    }

    pub fn notify_by_ref(&mut self) -> (action: NotifyByRef)
        requires old(self).lifecycle_valid(),
            old(self).idle() && !old(self).notified() ==> old(self).refs() < usize::MAX,
        ensures
            final(self).lifecycle_valid(), final(self).refs() >= old(self).refs(),
            old(self).complete() || old(self).notified() ==> action == NotifyByRef::DoNothing
                && final(self).raw() == old(self).raw(),
            old(self).running() && !old(self).notified() ==> action == NotifyByRef::DoNothing
                && final(self).notified() && final(self).refs() == old(self).refs(),
            old(self).idle() && !old(self).notified() ==> action == NotifyByRef::Submit
                && final(self).notified() && final(self).refs() == old(self).refs() + 1,
        no_unwind
    {
        if self.complete || self.notified {
            NotifyByRef::DoNothing
        } else if self.running {
            self.notified = true;
            NotifyByRef::DoNothing
        } else {
            self.notified = true;
            self.refs += 1;
            NotifyByRef::Submit
        }
    }

    pub fn notify_and_cancel(&mut self) -> (submit: bool)
        requires old(self).lifecycle_valid(),
            old(self).idle() && !old(self).notified() ==> old(self).refs() < usize::MAX,
        ensures
            final(self).lifecycle_valid(),
            old(self).cancelled() || old(self).complete() ==> !submit && final(self).raw() == old(self).raw(),
            !old(self).cancelled() && !old(self).complete() ==> final(self).cancelled(),
            !old(self).cancelled() && old(self).running() ==> !submit && final(self).notified(),
            !old(self).cancelled() && old(self).idle() ==> submit == !old(self).notified(),
            submit ==> final(self).refs() == old(self).refs() + 1 && final(self).notified(),
        no_unwind
    {
        if self.cancelled || self.complete {
            false
        } else if self.running {
            self.notified = true;
            self.cancelled = true;
            false
        } else {
            self.cancelled = true;
            if !self.notified {
                self.notified = true;
                self.refs += 1;
                true
            } else { false }
        }
    }

    pub fn shutdown(&mut self) -> (locked: bool)
        requires old(self).lifecycle_valid(),
        ensures final(self).lifecycle_valid(), final(self).cancelled(),
            locked == old(self).idle(), locked ==> final(self).running(),
            !locked ==> final(self).running() == old(self).running(),
        no_unwind
    {
        let locked = !self.running && !self.complete;
        if locked { self.running = true; }
        self.cancelled = true;
        locked
    }

    pub fn drop_join_handle(&mut self) -> (result: JoinDrop)
        requires old(self).join_interest(),
        ensures !final(self).join_interest(),
            result.drop_output == old(self).complete(),
            !old(self).complete() ==> !final(self).join_waker(),
            result.drop_waker == !final(self).join_waker(),
            final(self).refs() == old(self).refs(),
        no_unwind
    {
        self.join_interest = false;
        let drop_output = self.complete;
        if !self.complete { self.join_waker = false; }
        JoinDrop { drop_waker: !self.join_waker, drop_output }
    }

    pub fn set_join_waker(&mut self) -> (success: bool)
        requires old(self).join_interest(), !old(self).join_waker(),
        ensures success == !old(self).complete(),
            success ==> final(self).join_waker(),
            !success ==> final(self).raw() == old(self).raw(),
        no_unwind
    {
        if self.complete { false } else { self.join_waker = true; true }
    }

    pub fn unset_join_waker(&mut self) -> (success: bool)
        requires old(self).join_interest(),
            !old(self).complete() ==> old(self).join_waker(),
        ensures success == !old(self).complete(),
            success ==> !final(self).join_waker(),
            !success ==> final(self).raw() == old(self).raw(),
        no_unwind
    {
        if self.complete { false } else { self.join_waker = false; true }
    }

    pub fn ref_inc(&mut self)
        requires old(self).refs() < usize::MAX,
        ensures final(self).refs() == old(self).refs() + 1,
        no_unwind
    { self.refs += 1; }

    pub fn ref_dec(&mut self) -> (dealloc: bool)
        requires old(self).refs() > 0,
        ensures final(self).refs() + 1 == old(self).refs(), dealloc == (final(self).refs() == 0),
        no_unwind
    { self.refs -= 1; self.refs == 0 }
}

} // verus!
