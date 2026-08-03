use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaskStage {
    Future(u64),
    Output(u64),
    Cancelled(u64),
    Panicked(u64),
    Empty,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HarnessLife { Idle, Running, Complete, Deallocated }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FuturePoll { Pending, PendingAndWake, Ready(u64), Panic(u64) }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HarnessPoll { Idle, Resubmit, Complete, AlreadyDone }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum JoinRead { Pending, Output(u64), Cancelled(u64), Panicked(u64) }

/// Ownership-level composition of `RawTask::new`, `Harness::poll_inner`, and
/// `Harness::complete`. Pin/Poll/Waker and arbitrary Future execution are
/// frozen inputs; Tokio's stage and reference-token movements are proved.
pub struct TaskHarnessModel {
    stage: TaskStage,
    life: HarnessLife,
    notified: bool,
    scheduler_owned: bool,
    join_owned: bool,
    execution_ref: bool,
    queue_ref: bool,
}

impl TaskHarnessModel {
    pub closed spec fn stage(&self) -> TaskStage { self.stage }
    pub closed spec fn life(&self) -> HarnessLife { self.life }
    pub closed spec fn notified(&self) -> bool { self.notified }
    pub closed spec fn scheduler_owned(&self) -> bool { self.scheduler_owned }
    pub closed spec fn join_owned(&self) -> bool { self.join_owned }
    pub closed spec fn execution_ref(&self) -> bool { self.execution_ref }
    pub closed spec fn queue_ref(&self) -> bool { self.queue_ref }
    pub closed spec fn live_refs(&self) -> nat {
        (if self.scheduler_owned() { 1nat } else { 0nat })
        + (if self.join_owned() { 1nat } else { 0nat })
        + (if self.execution_ref() { 1nat } else { 0nat })
        + (if self.queue_ref() { 1nat } else { 0nat })
    }
    pub closed spec fn well_formed(&self) -> bool {
        &&& (self.life() == HarnessLife::Complete ==> !matches!(self.stage(), TaskStage::Future(_)))
        &&& (self.life() == HarnessLife::Complete && self.join_owned() ==>
            matches!(self.stage(), TaskStage::Output(_) | TaskStage::Cancelled(_) | TaskStage::Panicked(_)))
        &&& (self.life() == HarnessLife::Deallocated ==> self.live_refs() == 0)
        &&& (self.queue_ref() ==> self.notified() && self.life() == HarnessLife::Idle)
        &&& (self.life() == HarnessLife::Idle && self.notified() ==> self.queue_ref())
        &&& (self.execution_ref() ==> self.life() == HarnessLife::Running)
    }

    pub fn spawn(future_id: u64) -> (result: Self)
        ensures result.well_formed(), result.stage() == TaskStage::Future(future_id),
            result.life() == HarnessLife::Idle, result.notified(),
            result.scheduler_owned(), result.join_owned(), result.queue_ref(),
            !result.execution_ref(), result.live_refs() == 3,
        no_unwind
    {
        TaskHarnessModel {
            stage: TaskStage::Future(future_id), life: HarnessLife::Idle,
            notified: true, scheduler_owned: true, join_owned: true,
            execution_ref: false, queue_ref: true,
        }
    }

    pub fn begin_poll(&mut self) -> (started: bool)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            started == (old(self).life() == HarnessLife::Idle && old(self).notified()),
            started ==> final(self).life() == HarnessLife::Running
                && final(self).execution_ref() && !final(self).queue_ref()
                && !final(self).notified() && final(self).live_refs() == old(self).live_refs(),
            final(self).stage() == old(self).stage(),
            final(self).scheduler_owned() == old(self).scheduler_owned(),
            final(self).join_owned() == old(self).join_owned(),
        no_unwind
    {
        if matches!(self.life, HarnessLife::Idle) && self.notified {
            self.life = HarnessLife::Running;
            self.notified = false;
            self.queue_ref = false;
            self.execution_ref = true;
            true
        } else { false }
    }

    pub fn poll_normal(&mut self, step: FuturePoll) -> (result: HarnessPoll)
        requires old(self).well_formed(), old(self).life() == HarnessLife::Running,
            old(self).execution_ref(), matches!(old(self).stage(), TaskStage::Future(_)),
            !matches!(step, FuturePoll::Panic(_)),
        ensures
            final(self).well_formed(),
            match step {
                FuturePoll::Pending => result == HarnessPoll::Idle
                    && final(self).life() == HarnessLife::Idle
                    && final(self).stage() == old(self).stage()
                    && !final(self).execution_ref() && !final(self).queue_ref(),
                FuturePoll::PendingAndWake => result == HarnessPoll::Resubmit
                    && final(self).life() == HarnessLife::Idle && final(self).notified()
                    && final(self).queue_ref() && !final(self).execution_ref()
                    && final(self).stage() == old(self).stage(),
                FuturePoll::Ready(output) => result == HarnessPoll::Complete
                    && final(self).life() == HarnessLife::Complete
                    && !final(self).execution_ref() && !final(self).queue_ref()
                    && final(self).stage() == TaskStage::Output(output),
                FuturePoll::Panic(_) => true,
            },
            final(self).scheduler_owned() == old(self).scheduler_owned(),
            final(self).join_owned() == old(self).join_owned(),
        no_unwind
    {
        match step {
            FuturePoll::Pending => {
                self.life = HarnessLife::Idle;
                self.execution_ref = false;
                self.notified = false;
                HarnessPoll::Idle
            },
            FuturePoll::PendingAndWake => {
                self.life = HarnessLife::Idle;
                self.execution_ref = false;
                self.notified = true;
                self.queue_ref = true;
                HarnessPoll::Resubmit
            },
            FuturePoll::Ready(output) => {
                self.stage = TaskStage::Output(output);
                self.life = HarnessLife::Complete;
                self.execution_ref = false;
                HarnessPoll::Complete
            },
            FuturePoll::Panic(_) => HarnessPoll::AlreadyDone,
        }
    }

    /// Composition of remote abort, cancelled transition-to-running,
    /// `cancel_task`, and completion. The future identity is consumed and the
    /// exact cancellation result becomes the stage owner.
    pub fn abort_and_poll(&mut self, error_id: u64) -> (completed: bool)
        requires old(self).well_formed(), old(self).life() == HarnessLife::Idle,
            matches!(old(self).stage(), TaskStage::Future(_)),
        ensures final(self).well_formed(), completed,
            final(self).life() == HarnessLife::Complete,
            final(self).stage() == TaskStage::Cancelled(error_id),
            !final(self).queue_ref(), !final(self).execution_ref(),
            final(self).scheduler_owned() == old(self).scheduler_owned(),
            final(self).join_owned() == old(self).join_owned(),
        no_unwind
    {
        self.stage = TaskStage::Cancelled(error_id);
        self.life = HarnessLife::Complete;
        self.notified = true;
        self.queue_ref = false;
        self.execution_ref = false;
        true
    }

    /// JoinHandle is the unique output reader after COMPLETE. Pending keeps
    /// ownership; Ready moves the exact terminal value and its reference out.
    pub fn join_read(&mut self) -> (result: JoinRead)
        requires old(self).well_formed(), old(self).join_owned(),
        ensures
            old(self).life() != HarnessLife::Complete ==> result == JoinRead::Pending
                && final(self).stage() == old(self).stage() && final(self).join_owned(),
            old(self).life() == HarnessLife::Complete ==> {
                &&& !final(self).join_owned() && final(self).stage() == TaskStage::Empty
                &&& match old(self).stage() {
                    TaskStage::Output(value) => result == JoinRead::Output(value),
                    TaskStage::Cancelled(id) => result == JoinRead::Cancelled(id),
                    TaskStage::Panicked(id) => result == JoinRead::Panicked(id),
                    _ => false,
                }
            },
            final(self).scheduler_owned() == old(self).scheduler_owned(),
            final(self).life() == old(self).life(),
        no_unwind
    {
        if !matches!(self.life, HarnessLife::Complete) {
            JoinRead::Pending
        } else {
            self.join_owned = false;
            let mut stage = TaskStage::Empty;
            core::mem::swap(&mut self.stage, &mut stage);
            match stage {
                TaskStage::Output(value) => JoinRead::Output(value),
                TaskStage::Cancelled(id) => JoinRead::Cancelled(id),
                TaskStage::Panicked(id) => JoinRead::Panicked(id),
                _ => unreached(),
            }
        }
    }

    /// Dropping JoinHandle detaches an incomplete task, or consumes the exact
    /// completed output/error so deallocation cannot drop it on another thread.
    pub fn drop_join(&mut self) -> (dropped_stage: Option<TaskStage>)
        requires old(self).well_formed(), old(self).join_owned(),
        ensures
            !final(self).join_owned(), final(self).life() == old(self).life(),
            old(self).life() == HarnessLife::Complete ==> dropped_stage == Some(old(self).stage())
                && final(self).stage() == TaskStage::Empty,
            old(self).life() != HarnessLife::Complete ==> dropped_stage.is_none()
                && final(self).stage() == old(self).stage(),
        no_unwind
    {
        self.join_owned = false;
        if matches!(self.life, HarnessLife::Complete) {
            let mut stage = TaskStage::Empty;
            core::mem::swap(&mut self.stage, &mut stage);
            Some(stage)
        } else { None }
    }
}

pub fn verify_spawn_pending_wake_ready(future: u64, output: u64)
{
    let mut task = TaskHarnessModel::spawn(future);
    let first = task.begin_poll();
    assert(first);
    let pending = task.poll_normal(FuturePoll::PendingAndWake);
    assert(pending == HarnessPoll::Resubmit);
    let second = task.begin_poll();
    assert(second);
    let ready = task.poll_normal(FuturePoll::Ready(output));
    assert(ready == HarnessPoll::Complete);
    assert(task.stage() == TaskStage::Output(output));
}

pub fn verify_abort_join_once(future: u64, error: u64)
{
    let mut task = TaskHarnessModel::spawn(future);
    let completed = task.abort_and_poll(error);
    assert(completed);
    let joined = task.join_read();
    assert(joined == JoinRead::Cancelled(error));
    assert(task.stage() == TaskStage::Empty);
    assert(!task.join_owned());
}

} // verus!
