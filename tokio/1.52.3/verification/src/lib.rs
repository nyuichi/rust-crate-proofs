#![allow(dead_code, unused_assignments, unused_imports, unused_variables)]

use vstd::prelude::*;

pub mod atomic_waker_protocol;
pub mod atomic_waker_refinement;
pub mod barrier_orchestration;
pub mod barrier_protocol;
pub mod barrier_refinement;
pub mod blocking_recv_refinement;
pub mod broadcast_completion;
pub mod broadcast_endpoint_refinement;
pub mod broadcast_orchestration;
pub mod broadcast_physical_refinement;
pub mod broadcast_protocol;
pub mod broadcast_refinement;
pub mod broadcast_surface;
pub mod coop_refinement;
pub mod coop_tls_refinement;
pub mod defer_refinement;
pub mod intrusive_waiters;
pub mod linearizability;
pub mod mpsc_completion;
pub mod mpsc_endpoint_refinement;
pub mod mpsc_orchestration;
pub mod mpsc_protocol;
pub mod mpsc_queue_refinement;
pub mod mpsc_recv_compiled_refinement;
pub mod mpsc_refinement;
pub mod mpsc_surface;
pub mod mpsc_try_recv_raw_refinement;
pub mod mpsc_try_recv_refinement;
pub mod notified_core;
pub mod notify_guard;
pub mod notify_protocol;
pub mod notify_refinement;
pub mod once_cell_refinement;
pub mod oneshot_atomic;
pub mod oneshot_closed;
pub mod oneshot_drop;
pub mod oneshot_poll;
pub mod oneshot_refinement;
pub mod oneshot_state;
pub mod oneshot_value;
pub mod publication;
pub mod published_cell;
pub mod release_acquire;
pub mod semaphore_protocol;
pub mod semaphore_refinement;
pub mod tokio_loom_cell;
pub mod trait_views;
pub mod unwind;
pub mod vstd_ext;
pub mod wait_protocol;
pub mod watch_completion;
pub mod watch_endpoint_refinement;
pub mod watch_orchestration;
pub mod watch_protocol;
pub mod watch_refinement;
pub mod watch_selector_refinement;
pub mod watch_surface;
pub mod watch_terminal_refinement;
pub mod writer_lease;

verus! {

/// A sequential abstraction of the ownership states used by `SetOnce<T>`.
///
/// This deliberately does not model Tokio's atomics, `Notify`, or returned
/// references.  It is the small value-preserving state machine to which a
/// later production proof can connect those concurrent implementation details.
pub enum SetOnceState<T> {
    Empty,
    Published(T),
    Taken,
}

pub open spec fn state_value<T>(state: SetOnceState<T>) -> Option<T> {
    match state {
        SetOnceState::Published(value) => Some(value),
        SetOnceState::Empty | SetOnceState::Taken => None,
    }
}

pub struct SetOnceModel<T> {
    state: SetOnceState<T>,
}

impl<T> SetOnceModel<T> {
    pub closed spec fn view(&self) -> SetOnceState<T> {
        self.state
    }

    pub fn new() -> (model: Self)
        ensures
            model.view() == SetOnceState::Empty,
        no_unwind
    {
        SetOnceModel { state: SetOnceState::Empty }
    }

    pub fn new_with(value: T) -> (model: Self)
        ensures
            model.view() == SetOnceState::Published(value),
        no_unwind
    {
        SetOnceModel { state: SetOnceState::Published(value) }
    }

    pub fn is_empty(&self) -> (result: bool)
        ensures
            result == matches!(self.view(), SetOnceState::Empty),
        no_unwind
    {
        match &self.state {
            SetOnceState::Empty => true,
            SetOnceState::Published(_) | SetOnceState::Taken => false,
        }
    }

    /// Publish `value` exactly once.  On failure, the input value is returned
    /// unchanged and the previous abstract state is preserved.
    pub fn set(&mut self, value: T) -> (result: Result<(), T>)
        ensures
            match old(self).view() {
                SetOnceState::Empty => {
                    result == Ok(())
                        && final(self).view() == SetOnceState::Published(value)
                },
                SetOnceState::Published(previous) => {
                    result == Err(value)
                        && final(self).view() == SetOnceState::Published(previous)
                },
                SetOnceState::Taken => {
                    result == Err(value)
                        && final(self).view() == SetOnceState::Taken
                },
            },
        no_unwind
    {
        if self.is_empty() {
            self.state = SetOnceState::Published(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Value-returning observer used in place of production `get() -> &T`.
    pub fn get_by_value(&self) -> (result: Option<T>)
        where T: Copy
        ensures
            result == state_value(self.view()),
        no_unwind
    {
        match &self.state {
            SetOnceState::Published(value) => Some(*value),
            SetOnceState::Empty | SetOnceState::Taken => None,
        }
    }

    /// `into_inner`-like transition which leaves `Taken` observable in the
    /// model and moves the published value to the caller at most once.
    pub fn take_into_inner(&mut self) -> (result: Option<T>)
        ensures
            result == state_value(old(self).view()),
            final(self).view() == SetOnceState::Taken,
        no_unwind
    {
        let mut previous = SetOnceState::Taken;
        core::mem::swap(&mut self.state, &mut previous);
        match previous {
            SetOnceState::Published(value) => Some(value),
            SetOnceState::Empty | SetOnceState::Taken => None,
        }
    }

    /// Consuming form corresponding to Tokio's public `into_inner` shape.
    pub fn into_inner(self) -> (result: Option<T>)
        ensures
            result == state_value(self.view()),
        no_unwind
    {
        let mut owned = self;
        owned.take_into_inner()
    }
}

pub fn verify_successful_roundtrip(value: u64)
{
    let mut model = SetOnceModel::new();
    let set_result = model.set(value);
    assert(set_result == Ok(()));
    let observed = model.get_by_value();
    assert(observed == Some(value));
    let taken = model.take_into_inner();
    assert(taken == Some(value));
    let after_take = model.get_by_value();
    assert(after_take.is_none());
    let taken_again = model.take_into_inner();
    assert(taken_again.is_none());
}

pub fn verify_failed_set_preserves(first: u64, second: u64)
{
    let mut model = SetOnceModel::new_with(first);
    let set_result = model.set(second);
    assert(set_result == Err(second));
    let observed = model.get_by_value();
    assert(observed == Some(first));
    let inner = model.into_inner();
    assert(inner == Some(first));
}

pub fn verify_empty_into_inner()
{
    let model = SetOnceModel::<u64>::new();
    let inner = model.into_inner();
    assert(inner.is_none());
}

} // verus!
