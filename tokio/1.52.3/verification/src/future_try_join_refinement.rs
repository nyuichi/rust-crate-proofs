use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

#[derive(PartialEq, Eq)]
pub enum MaybeDoneState<T> {
    Future,
    Done(T),
    Gone,
}

#[derive(PartialEq, Eq)]
pub enum FutureStep<T> {
    Pending,
    Ready(T),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MaybeDonePoll {
    Pending,
    Ready,
    GonePanic,
}

#[derive(PartialEq, Eq)]
pub enum MaybeDoneDrop<T> {
    Future,
    Output(T),
    Gone,
}

/// Exact three-state projection of production `MaybeDone<Fut>`. Pinning and
/// the arbitrary inner Future call are frozen interfaces; Tokio's state and
/// output ownership transitions are body-proved here.
pub struct MaybeDoneModel<T> {
    state: MaybeDoneState<T>,
}

impl<T> MaybeDoneModel<T> {
    pub closed spec fn state(&self) -> MaybeDoneState<T> { self.state }

    pub fn new() -> (result: Self)
        ensures result.state() == MaybeDoneState::Future,
        no_unwind
    {
        MaybeDoneModel { state: MaybeDoneState::Future }
    }

    pub fn poll(&mut self, step: FutureStep<T>) -> (result: MaybeDonePoll)
        ensures
            match old(self).state() {
                MaybeDoneState::Future => match step {
                    FutureStep::Pending => {
                        result == MaybeDonePoll::Pending
                            && final(self).state() == MaybeDoneState::Future
                    },
                    FutureStep::Ready(output) => {
                        result == MaybeDonePoll::Ready
                            && final(self).state() == MaybeDoneState::Done(output)
                    },
                },
                MaybeDoneState::Done(output) => {
                    result == MaybeDonePoll::Ready
                        && final(self).state() == MaybeDoneState::Done(output)
                },
                MaybeDoneState::Gone => {
                    result == MaybeDonePoll::GonePanic
                        && final(self).state() == MaybeDoneState::Gone
                },
            },
        no_unwind
    {
        match &self.state {
            MaybeDoneState::Future => match step {
                FutureStep::Pending => MaybeDonePoll::Pending,
                FutureStep::Ready(output) => {
                    self.state = MaybeDoneState::Done(output);
                    MaybeDonePoll::Ready
                },
            },
            MaybeDoneState::Done(_) => MaybeDonePoll::Ready,
            MaybeDoneState::Gone => MaybeDonePoll::GonePanic,
        }
    }

    pub fn output_is_available(&self) -> (result: bool)
        ensures result == matches!(self.state(), MaybeDoneState::Done(_)),
        no_unwind
    {
        matches!(self.state, MaybeDoneState::Done(_))
    }

    /// Models mutation through `output_mut`: only Done exposes an output and
    /// replacing it returns the exact previous value.
    pub fn replace_output(&mut self, replacement: T) -> (previous: Option<T>)
        ensures
            match old(self).state() {
                MaybeDoneState::Done(output) => {
                    previous == Some(output)
                        && final(self).state() == MaybeDoneState::Done(replacement)
                },
                MaybeDoneState::Future | MaybeDoneState::Gone => {
                    previous.is_none() && final(self).state() == old(self).state()
                },
            },
        no_unwind
    {
        match &self.state {
            MaybeDoneState::Done(_) => {
                let mut next = MaybeDoneState::Done(replacement);
                core::mem::swap(&mut self.state, &mut next);
                match next {
                    MaybeDoneState::Done(output) => Some(output),
                    _ => unreached(),
                }
            },
            MaybeDoneState::Future | MaybeDoneState::Gone => None,
        }
    }

    pub fn take_output(&mut self) -> (result: Option<T>)
        ensures
            match old(self).state() {
                MaybeDoneState::Done(output) => {
                    result == Some(output) && final(self).state() == MaybeDoneState::Gone
                },
                MaybeDoneState::Future | MaybeDoneState::Gone => {
                    result.is_none() && final(self).state() == old(self).state()
                },
            },
        no_unwind
    {
        if matches!(self.state, MaybeDoneState::Done(_)) {
            let mut previous = MaybeDoneState::Gone;
            core::mem::swap(&mut self.state, &mut previous);
            match previous {
                MaybeDoneState::Done(output) => Some(output),
                _ => unreached(),
            }
        } else {
            None
        }
    }

    /// Makes cancellation/drop ownership explicit without executing arbitrary
    /// user destructors inside the proof model.
    pub fn into_drop(self) -> (result: MaybeDoneDrop<T>)
        ensures
            result == match self.state() {
                MaybeDoneState::Future => MaybeDoneDrop::Future,
                MaybeDoneState::Done(output) => MaybeDoneDrop::Output(output),
                MaybeDoneState::Gone => MaybeDoneDrop::Gone,
            },
        no_unwind
    {
        match self.state {
            MaybeDoneState::Future => MaybeDoneDrop::Future,
            MaybeDoneState::Done(output) => MaybeDoneDrop::Output(output),
            MaybeDoneState::Gone => MaybeDoneDrop::Gone,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BranchState {
    Future,
    Ok(u64),
    Err(u64),
    Gone,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BranchStep {
    Pending,
    ReadyOk(u64),
    ReadyErr(u64),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TryJoinPoll {
    Pending,
    ReadyOk(u64, u64, u64),
    ReadyErr(u64),
    RepollPanic,
}

pub open spec fn branch_after(state: BranchState, step: BranchStep) -> BranchState {
    match state {
        BranchState::Future => match step {
            BranchStep::Pending => BranchState::Future,
            BranchStep::ReadyOk(value) => BranchState::Ok(value),
            BranchStep::ReadyErr(error) => BranchState::Err(error),
        },
        _ => state,
    }
}

pub open spec fn branch_pending(state: BranchState) -> bool {
    state == BranchState::Future
}

pub open spec fn try_join_outcome(
    first: BranchState,
    second: BranchState,
    third: BranchState,
) -> TryJoinPoll {
    match first {
        BranchState::Err(error) => TryJoinPoll::ReadyErr(error),
        _ => match second {
            BranchState::Err(error) => TryJoinPoll::ReadyErr(error),
            _ => match third {
                BranchState::Err(error) => TryJoinPoll::ReadyErr(error),
                _ => match (first, second, third) {
                    (BranchState::Ok(a), BranchState::Ok(b), BranchState::Ok(c)) => {
                        TryJoinPoll::ReadyOk(a, b, c)
                    },
                    _ => TryJoinPoll::Pending,
                },
            },
        },
    }
}

/// Production `future::try_join3` in its fixed left-to-right polling order.
/// Outputs and errors are stable ids, while arbitrary output/drop execution is
/// the frozen generic-code boundary.
pub struct TryJoin3Model {
    first: BranchState,
    second: BranchState,
    third: BranchState,
    terminal: bool,
}

impl TryJoin3Model {
    pub closed spec fn first(&self) -> BranchState { self.first }
    pub closed spec fn second(&self) -> BranchState { self.second }
    pub closed spec fn third(&self) -> BranchState { self.third }
    pub closed spec fn terminal(&self) -> bool { self.terminal }
    pub closed spec fn well_formed(&self) -> bool {
        self.terminal() == (self.first() == BranchState::Gone
            || self.second() == BranchState::Gone
            || self.third() == BranchState::Gone)
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), !result.terminal(),
            result.first() == BranchState::Future,
            result.second() == BranchState::Future,
            result.third() == BranchState::Future,
        no_unwind
    {
        TryJoin3Model {
            first: BranchState::Future,
            second: BranchState::Future,
            third: BranchState::Future,
            terminal: false,
        }
    }

    fn poll_branch(state: &mut BranchState, step: BranchStep)
        ensures *final(state) == branch_after(*old(state), step),
        no_unwind
    {
        match *state {
            BranchState::Future => {
                *state = match step {
                    BranchStep::Pending => BranchState::Future,
                    BranchStep::ReadyOk(value) => BranchState::Ok(value),
                    BranchStep::ReadyErr(error) => BranchState::Err(error),
                };
            },
            _ => {},
        }
    }

    pub fn poll(
        &mut self,
        first_step: BranchStep,
        second_step: BranchStep,
        third_step: BranchStep,
    ) -> (result: TryJoinPoll)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            old(self).terminal() ==> result == TryJoinPoll::RepollPanic
                && final(self).first() == old(self).first()
                && final(self).second() == old(self).second()
                && final(self).third() == old(self).third(),
            !old(self).terminal() ==> result == try_join_outcome(
                branch_after(old(self).first(), first_step),
                branch_after(old(self).second(), second_step),
                branch_after(old(self).third(), third_step),
            ),
            !old(self).terminal() && result == TryJoinPoll::Pending ==>
                !final(self).terminal()
                    && final(self).first() == branch_after(old(self).first(), first_step)
                    && final(self).second() == branch_after(old(self).second(), second_step)
                    && final(self).third() == branch_after(old(self).third(), third_step),
            !old(self).terminal() && matches!(result, TryJoinPoll::ReadyOk(_, _, _)
                | TryJoinPoll::ReadyErr(_)) ==> final(self).terminal(),
            !old(self).terminal()
                && matches!(branch_after(old(self).first(), first_step), BranchState::Err(_)) ==>
                final(self).second() == old(self).second()
                    && final(self).third() == old(self).third(),
        no_unwind
    {
        if self.terminal {
            return TryJoinPoll::RepollPanic;
        }

        Self::poll_branch(&mut self.first, first_step);
        if let BranchState::Err(error) = self.first {
            self.first = BranchState::Gone;
            self.terminal = true;
            return TryJoinPoll::ReadyErr(error);
        }

        Self::poll_branch(&mut self.second, second_step);
        if let BranchState::Err(error) = self.second {
            self.second = BranchState::Gone;
            self.terminal = true;
            return TryJoinPoll::ReadyErr(error);
        }

        Self::poll_branch(&mut self.third, third_step);
        if let BranchState::Err(error) = self.third {
            self.third = BranchState::Gone;
            self.terminal = true;
            return TryJoinPoll::ReadyErr(error);
        }

        match (self.first, self.second, self.third) {
            (BranchState::Ok(first), BranchState::Ok(second), BranchState::Ok(third)) => {
                self.first = BranchState::Gone;
                self.second = BranchState::Gone;
                self.third = BranchState::Gone;
                self.terminal = true;
                TryJoinPoll::ReadyOk(first, second, third)
            },
            _ => TryJoinPoll::Pending,
        }
    }

    /// Cancellation returns the exact still-owned branch resources. A Gone
    /// branch has already transferred its error/output to the caller.
    pub fn cancel(self) -> (result: (BranchState, BranchState, BranchState))
        ensures result == (self.first(), self.second(), self.third()),
        no_unwind
    {
        (self.first, self.second, self.third)
    }
}

/// One complete macro poll visits each branch exactly once in cyclic order.
/// This is the canonical three-branch projection of the generated two-pass
/// skip loop; macro expansion tests cover the supported arities.
pub fn rotated_order(skip: u8) -> (result: (u8, u8, u8))
    requires skip < 3,
    ensures
        skip == 0 ==> result == (0, 1, 2),
        skip == 1 ==> result == (1, 2, 0),
        skip == 2 ==> result == (2, 0, 1),
    no_unwind
{
    if skip == 0 { (0, 1, 2) }
    else if skip == 1 { (1, 2, 0) }
    else { (2, 0, 1) }
}

pub struct Rotator3 {
    next: u8,
}

impl Rotator3 {
    pub closed spec fn next(&self) -> u8 { self.next }

    pub fn new() -> (result: Self)
        ensures result.next() == 0,
        no_unwind
    {
        Rotator3 { next: 0 }
    }

    pub fn num_skip(&mut self) -> (result: u8)
        requires old(self).next() < 3,
        ensures result == old(self).next(), final(self).next() < 3,
            final(self).next() == if old(self).next() == 2 { 0 } else { old(self).next() + 1 },
        no_unwind
    {
        let result = self.next;
        self.next += 1;
        if self.next == 3 { self.next = 0; }
        result
    }
}

pub fn biased_num_skip() -> (result: u8)
    ensures result == 0,
    no_unwind
{
    0
}

pub fn verify_maybe_done_output_lifecycle(value: u64, replacement: u64)
{
    let mut state = MaybeDoneModel::new();
    let pending = state.poll(FutureStep::Pending);
    assert(pending == MaybeDonePoll::Pending);
    let ready = state.poll(FutureStep::Ready(value));
    assert(ready == MaybeDonePoll::Ready);
    let previous = state.replace_output(replacement);
    assert(previous == Some(value));
    let output = state.take_output();
    assert(output == Some(replacement));
    let repoll = state.poll(FutureStep::Pending);
    assert(repoll == MaybeDonePoll::GonePanic);
}

pub fn verify_try_join3_success_and_repoll(first: u64, second: u64, third: u64)
{
    let mut join = TryJoin3Model::new();
    let pending = join.poll(
        BranchStep::ReadyOk(first), BranchStep::Pending, BranchStep::ReadyOk(third),
    );
    assert(pending == TryJoinPoll::Pending);
    let ready = join.poll(
        BranchStep::Pending, BranchStep::ReadyOk(second), BranchStep::Pending,
    );
    assert(ready == TryJoinPoll::ReadyOk(first, second, third));
    let repoll = join.poll(
        BranchStep::Pending, BranchStep::Pending, BranchStep::Pending,
    );
    assert(repoll == TryJoinPoll::RepollPanic);
}

pub fn verify_try_join3_first_error_stops_later_polls(error: u64)
{
    let mut join = TryJoin3Model::new();
    let result = join.poll(
        BranchStep::ReadyErr(error), BranchStep::ReadyOk(2), BranchStep::ReadyOk(3),
    );
    assert(result == TryJoinPoll::ReadyErr(error));
    assert(join.second() == BranchState::Future);
    assert(join.third() == BranchState::Future);
    let remaining = join.cancel();
    assert(remaining == (BranchState::Gone, BranchState::Future, BranchState::Future));
}

pub fn verify_rotator_three_polls()
{
    let mut rotator = Rotator3::new();
    let first = rotator.num_skip();
    let first_order = rotated_order(first);
    assert(first_order == (0, 1, 2));
    let second = rotator.num_skip();
    let second_order = rotated_order(second);
    assert(second_order == (1, 2, 0));
    let third = rotator.num_skip();
    let third_order = rotated_order(third);
    assert(third_order == (2, 0, 1));
    assert(rotator.next() == 0);
}

} // verus!
