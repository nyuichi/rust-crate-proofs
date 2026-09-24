use crate::mpsc_endpoint_refinement::{
    MpscEndpointRefinement, MpscFiniteExecutionResources, StrongOwner,
};
use crate::mpsc_protocol::{MpscCapacity, ReserveResult};
use crate::mpsc_queue_refinement::{RawMpscQueue, RawRead, RawValueSlot};
use crate::mpsc_recv_compiled_refinement::{CompiledManyPhase, CompiledRecvMany};
use crate::mpsc_refinement::{MpscRecvRefinement, RecvEnvironment, RecvPoll};
use crate::mpsc_try_recv_refinement::{MpscTryRecvRefinement, TryRecvOutcome};
use vstd::prelude::*;

verus! {

/// Thin composition: raw list publication/read moves the same payload into the
/// public recv result, and bounded capacity is restored once. Leaf details are
/// kept out of this orchestration VC.
pub fn verify_raw_first_recv_composes_bounded(value: u64)
{
    proof {
        let tracked mut queue = RawMpscQueue::new(2);
        let index = queue.claim_value();
        let tracked mut slot = RawValueSlot::claimed(index);
        queue.publish_slot(&mut slot, value);
        let read = queue.classify_head();
        assert(read == RawRead::Value);
        let raw_value = queue.pop_slot(&mut slot);
        assert(raw_value == value);
    }

    let mut capacity = MpscCapacity::new(1);
    let reserved = capacity.reserve();
    assert(reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let env = RecvEnvironment::new(false, false, false);
    let mut recv = MpscRecvRefinement::new(None);
    let result = recv.first_value(value);
    assert(result == RecvPoll::ReadyValue(value));
    let received_capacity = capacity.receive();
    assert(received_capacity);
    assert(capacity.available() == 1 && capacity.queued() == 0);
}

/// The endpoint transition requests close, then the raw list claims and
/// publishes the unique FIFO close marker and wakes the receiver. The public
/// recv closure branch consumes no payload or capacity.
pub fn verify_last_strong_composes_raw_close_and_recv(
    resources: &MpscFiniteExecutionResources,
)
    requires resources.well_formed(),
{
    let mut endpoints = MpscEndpointRefinement::new(resources);
    endpoints.begin_drop_strong(resources, StrongOwner::Sender);
    let last = endpoints.finish_drop_strong(resources);
    assert(last && endpoints.close_requested());

    proof {
        let tracked mut queue = RawMpscQueue::new(2);
        let close = queue.claim_close();
        queue.publish_close();
        let read = queue.classify_head();
        assert(read == RawRead::Closed);
        assert(close == queue.head());
        let before_wake = queue.wake_count();
        queue.wake_receiver();
        assert(queue.wake_count() == before_wake + 1);
    }

    let env = RecvEnvironment::new(true, false, true);
    let mut recv = MpscRecvRefinement::new(None);
    let result: RecvPoll<u64> = recv.first_closed(&env);
    assert(result == RecvPoll::ReadyClosed);
    assert(recv.permits_returned() == 0);
}

/// Two raw pops are appended in FIFO order. The compiled guard's pending count
/// is then consumed by the same one bounded bulk-return represented in the
/// capacity model, with no Vec spare-capacity premise.
pub fn verify_raw_recv_many_composes_growth_and_bulk_return(first: u64, second: u64)
{
    proof {
        let tracked mut queue = RawMpscQueue::new(2);
        let first_index = queue.claim_value();
        let tracked mut first_slot = RawValueSlot::claimed(first_index);
        let second_index = queue.claim_value();
        let tracked mut second_slot = RawValueSlot::claimed(second_index);
        queue.publish_slot(&mut first_slot, first);
        queue.publish_slot(&mut second_slot, second);
        let popped_first = queue.pop_slot(&mut first_slot);
        assert(popped_first == first);
        let popped_second = queue.pop_slot(&mut second_slot);
        assert(popped_second == second);
    }

    let mut many = CompiledRecvMany::new(2, Ghost(Seq::<u64>::empty()));
    many.record_raw_pop(Ghost(first));
    assert(many.pending() == 1 && many.pending() < many.limit());
    many.push_succeeded();
    many.record_raw_pop(Ghost(second));
    assert(many.pending() == 2 && many.pending() == many.limit());
    many.push_succeeded();
    assert(many.phase() == CompiledManyPhase::Returned);
    assert(many.accounting_returned() == 2);
    assert(many.buffer() == seq![first, second]);

    let mut capacity = MpscCapacity::new(2);
    let first_reserved = capacity.reserve();
    assert(first_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    let second_reserved = capacity.reserve();
    assert(second_reserved == ReserveResult::Permit);
    capacity.commit_permit();
    capacity.receive_many(2);
    assert(capacity.available() == 2 && capacity.queued() == 0);
}

/// A raw Busy observation is the only reason production enters its wake and
/// park path. The next raw Value is passed unchanged to the public try_recv
/// result; park construction and wake execution remain their frozen adapters.
pub fn verify_raw_busy_composes_try_recv(value: u64, park_waker: u64)
{
    proof {
        let tracked mut queue = RawMpscQueue::new(2);
        let index = queue.claim_value();
        let tracked mut slot = RawValueSlot::claimed(index);
        let busy = queue.classify_head();
        assert(busy == RawRead::Busy);
        queue.publish_slot(&mut slot, value);
        let ready = queue.classify_head();
        assert(ready == RawRead::Value);
        let popped = queue.pop_slot(&mut slot);
        assert(popped == value);
    }

    let env = RecvEnvironment::new(false, false, false);
    let mut recv = MpscTryRecvRefinement::new();
    let initial = recv.initial_observe::<u64>(
        crate::mpsc_protocol::QueueRead::Busy, &env);
    assert(initial == TryRecvOutcome::Continue);
    recv.wake_previous_and_prepare(None, park_waker);
    recv.register_iteration();
    let result = recv.recheck(crate::mpsc_protocol::QueueRead::Value(value), &env);
    assert(result == TryRecvOutcome::Value(value));
    assert(recv.permits_returned() == 1);
}

} // verus!
