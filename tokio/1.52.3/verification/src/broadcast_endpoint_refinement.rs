use vstd::prelude::*;

verus! {

/// Narrow environment adapter authorized for broadcast's two synchronous
/// counter-before-Arc-clone gaps (`downgrade` and successful `upgrade`).
///
/// Each gap occupies one distinct live execution thread and neither gap calls
/// user code nor awaits. Thus one thread owns at most one gap. Arc-backed
/// completed endpoints and live threads are each bounded by `isize::MAX`, so
/// their sum is at most `usize::MAX - 1`. This is not an atomic or Arc axiom and
/// must not be reused outside these two production paths.
pub struct FiniteExecutionResources {
    live_threads: usize,
}

impl FiniteExecutionResources {
    pub closed spec fn live_threads(&self) -> usize { self.live_threads }
    pub closed spec fn well_formed(&self) -> bool {
        self.live_threads() <= usize::MAX / 2
    }

    pub fn new(live_threads: usize) -> (result: Self)
        requires live_threads <= usize::MAX / 2,
        ensures result.well_formed(), result.live_threads() == live_threads,
        no_unwind
    {
        FiniteExecutionResources { live_threads }
    }
}

pub struct BroadcastEndpoints {
    strong_atomic: usize,
    weak_atomic: usize,
    strong_complete: usize,
    weak_complete: usize,
    receivers: usize,
    arc_endpoint_owners: usize,
    downgrade_gaps: usize,
    upgrade_gaps: usize,
    closed: bool,
}

impl BroadcastEndpoints {
    pub closed spec fn strong(&self) -> usize { self.strong_atomic }
    pub closed spec fn weak(&self) -> usize { self.weak_atomic }
    pub closed spec fn strong_complete(&self) -> usize { self.strong_complete }
    pub closed spec fn weak_complete(&self) -> usize { self.weak_complete }
    pub closed spec fn receivers(&self) -> usize { self.receivers }
    pub closed spec fn arc_endpoint_owners(&self) -> usize { self.arc_endpoint_owners }
    pub closed spec fn downgrade_gaps(&self) -> usize { self.downgrade_gaps }
    pub closed spec fn upgrade_gaps(&self) -> usize { self.upgrade_gaps }
    pub closed spec fn gaps(&self) -> int {
        self.downgrade_gaps() as int + self.upgrade_gaps() as int
    }
    pub closed spec fn closed(&self) -> bool { self.closed }

    pub closed spec fn well_formed(&self, resources: &FiniteExecutionResources) -> bool {
        &&& resources.well_formed()
        &&& self.strong() as int
            == self.strong_complete() as int + self.upgrade_gaps() as int
        &&& self.weak() as int
            == self.weak_complete() as int + self.downgrade_gaps() as int
        &&& self.arc_endpoint_owners() as int
            == self.strong_complete() as int + self.weak_complete() as int
                + self.receivers() as int
        &&& self.arc_endpoint_owners() <= usize::MAX / 2
        &&& self.gaps() <= resources.live_threads() as int
        &&& self.closed() == (self.strong() == 0)
    }

    pub fn channel(resources: &FiniteExecutionResources) -> (result: Self)
        requires resources.well_formed(),
        ensures result.well_formed(resources), result.strong() == 1,
            result.weak() == 0, result.receivers() == 1,
            result.arc_endpoint_owners() == 2, !result.closed(),
        no_unwind
    {
        BroadcastEndpoints {
            strong_atomic: 1,
            weak_atomic: 0,
            strong_complete: 1,
            weak_complete: 0,
            receivers: 1,
            arc_endpoint_owners: 2,
            downgrade_gaps: 0,
            upgrade_gaps: 0,
            closed: false,
        }
    }

    /// `Sender::clone` and `WeakSender::clone` first clone Arc, then increment
    /// the corresponding count. Arc normal-return capacity is the frozen Arc
    /// adapter, not a channel-specific finite-prefix assumption.
    pub fn clone_sender(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).strong_complete() > 0,
            old(self).arc_endpoint_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong() + 1,
            final(self).strong_complete() == old(self).strong_complete() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners() + 1,
            final(self).weak() == old(self).weak(),
        no_unwind
    {
        self.arc_endpoint_owners += 1;
        self.strong_complete += 1;
        self.strong_atomic += 1;
    }

    pub fn clone_weak(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_complete() > 0,
            old(self).arc_endpoint_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).weak() == old(self).weak() + 1,
            final(self).weak_complete() == old(self).weak_complete() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners() + 1,
            final(self).strong() == old(self).strong(),
        no_unwind
    {
        self.arc_endpoint_owners += 1;
        self.weak_complete += 1;
        self.weak_atomic += 1;
    }

    /// Linearization of downgrade's weak `fetch_add`, before its Arc clone.
    pub fn begin_downgrade(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).strong_complete() > 0,
            old(self).gaps() < resources.live_threads() as int,
        ensures final(self).well_formed(resources),
            final(self).weak() == old(self).weak() + 1,
            final(self).downgrade_gaps() == old(self).downgrade_gaps() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners(),
            final(self).strong() == old(self).strong(),
            final(self).strong_complete() == old(self).strong_complete(),
        no_unwind
    {
        self.weak_atomic += 1;
        self.downgrade_gaps += 1;
    }

    pub fn finish_downgrade(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).downgrade_gaps() > 0,
            old(self).arc_endpoint_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).weak() == old(self).weak(),
            final(self).downgrade_gaps() + 1 == old(self).downgrade_gaps(),
            final(self).weak_complete() == old(self).weak_complete() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners() + 1,
            final(self).strong_complete() == old(self).strong_complete(),
            final(self).strong() == old(self).strong(),
        no_unwind
    {
        self.downgrade_gaps -= 1;
        self.weak_complete += 1;
        self.arc_endpoint_owners += 1;
    }

    /// Successful upgrade CAS. `strong > 0` is tested before the CAS, so zero
    /// can never transition back to one. The gap owns its current thread until
    /// the immediately-following Arc clone returns or aborts.
    pub fn begin_upgrade(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_complete() > 0,
            old(self).strong() > 0,
            old(self).gaps() < resources.live_threads() as int,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong() + 1,
            final(self).upgrade_gaps() == old(self).upgrade_gaps() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners(),
            !final(self).closed(),
        no_unwind
    {
        self.strong_atomic += 1;
        self.upgrade_gaps += 1;
    }

    pub fn finish_upgrade(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).upgrade_gaps() > 0,
            old(self).arc_endpoint_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong(),
            final(self).upgrade_gaps() + 1 == old(self).upgrade_gaps(),
            final(self).strong_complete() == old(self).strong_complete() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners() + 1,
        no_unwind
    {
        self.upgrade_gaps -= 1;
        self.strong_complete += 1;
        self.arc_endpoint_owners += 1;
    }

    pub fn upgrade_closed(&self, resources: &FiniteExecutionResources) -> (none: bool)
        requires self.well_formed(resources), self.weak_complete() > 0,
        ensures none == self.closed(), self.closed() ==> self.strong() == 0,
        no_unwind
    {
        self.strong_atomic == 0
    }

    pub fn drop_sender(&mut self, resources: &FiniteExecutionResources) -> (last: bool)
        requires old(self).well_formed(resources), old(self).strong_complete() > 0,
        ensures final(self).well_formed(resources),
            final(self).strong() + 1 == old(self).strong(),
            final(self).strong_complete() + 1 == old(self).strong_complete(),
            final(self).arc_endpoint_owners() + 1 == old(self).arc_endpoint_owners(),
            last == (final(self).strong() == 0), last == final(self).closed(),
            final(self).weak_complete() == old(self).weak_complete(),
            final(self).weak() == old(self).weak(),
        no_unwind
    {
        self.strong_atomic -= 1;
        self.strong_complete -= 1;
        self.arc_endpoint_owners -= 1;
        if self.strong_atomic == 0 { self.closed = true; }
        self.strong_atomic == 0
    }

    pub fn drop_weak(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_complete() > 0,
        ensures final(self).well_formed(resources),
            final(self).weak() + 1 == old(self).weak(),
            final(self).weak_complete() + 1 == old(self).weak_complete(),
            final(self).arc_endpoint_owners() + 1 == old(self).arc_endpoint_owners(),
            final(self).strong() == old(self).strong(),
        no_unwind
    {
        self.weak_atomic -= 1;
        self.weak_complete -= 1;
        self.arc_endpoint_owners -= 1;
    }

    pub fn subscribe(&mut self, resources: &FiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).strong_complete() > 0,
            old(self).arc_endpoint_owners() < usize::MAX / 2,
            old(self).receivers() < usize::MAX,
        ensures final(self).well_formed(resources),
            final(self).receivers() == old(self).receivers() + 1,
            final(self).arc_endpoint_owners() == old(self).arc_endpoint_owners() + 1,
            final(self).strong() == old(self).strong(),
        no_unwind
    {
        self.arc_endpoint_owners += 1;
        self.receivers += 1;
    }

    pub fn drop_receiver(&mut self, resources: &FiniteExecutionResources) -> (last: bool)
        requires old(self).well_formed(resources), old(self).receivers() > 0,
        ensures final(self).well_formed(resources),
            final(self).receivers() + 1 == old(self).receivers(),
            final(self).arc_endpoint_owners() + 1 == old(self).arc_endpoint_owners(),
            last == (final(self).receivers() == 0),
        no_unwind
    {
        self.receivers -= 1;
        self.arc_endpoint_owners -= 1;
        self.receivers == 0
    }
}

pub fn verify_upgrade_cannot_resurrect(resources: &FiniteExecutionResources)
    requires resources.well_formed(), resources.live_threads() > 0,
{
    let mut endpoints = BroadcastEndpoints::channel(resources);
    endpoints.begin_downgrade(resources);
    endpoints.finish_downgrade(resources);
    let last = endpoints.drop_sender(resources);
    assert(last);
    let none = endpoints.upgrade_closed(resources);
    assert(none);
    assert(endpoints.strong() == 0);
}

} // verus!
