use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UpgradeStep { Retry, Closed, Constructed }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StrongOwner { Sender, OwnedPermit }

/// Narrow physical-resource contract for mpsc's synchronous count-before-Arc
/// clone windows. Every such window occupies one distinct live thread and
/// executes no user code or await. Completed Arc owners and live threads are
/// independently limited by Arc/the execution environment, so their sum is at
/// most `usize::MAX - 1`. This contract is scoped only to mpsc endpoint
/// clone/downgrade construction; it says nothing about cumulative messages.
pub struct MpscFiniteExecutionResources {
    live_threads: usize,
}

impl MpscFiniteExecutionResources {
    pub closed spec fn live_threads(&self) -> usize { self.live_threads }
    pub closed spec fn well_formed(&self) -> bool {
        self.live_threads() <= usize::MAX / 2
    }
    pub fn new(live_threads: usize) -> (result: Self)
        requires live_threads <= usize::MAX / 2,
        ensures result.well_formed(), result.live_threads() == live_threads,
        no_unwind
    {
        MpscFiniteExecutionResources { live_threads }
    }
}

/// Phase-accurate projection of `Chan::{tx_count,tx_weak_count}` and the Arc
/// owners carried by Sender, WeakSender, OwnedPermit, Receiver, and an upgrade
/// call. Count-before-Arc gaps are explicit and bounded by live threads rather
/// than a finite-prefix assumption on the public counters.
pub struct MpscEndpointRefinement {
    strong_atomic: usize,
    weak_atomic: usize,
    sender_live: usize,
    owned_permit_live: usize,
    weak_live: usize,
    strong_gaps: usize,
    weak_gaps: usize,
    strong_retiring: usize,
    weak_retiring: usize,
    upgrade_arc_held: usize,
    arc_owners: usize,
    close_requested: bool,
}

impl MpscEndpointRefinement {
    pub closed spec fn strong(&self) -> usize { self.strong_atomic }
    pub closed spec fn weak(&self) -> usize { self.weak_atomic }
    pub closed spec fn sender_live(&self) -> usize { self.sender_live }
    pub closed spec fn owned_permit_live(&self) -> usize { self.owned_permit_live }
    pub closed spec fn weak_live(&self) -> usize { self.weak_live }
    pub closed spec fn strong_gaps(&self) -> usize { self.strong_gaps }
    pub closed spec fn weak_gaps(&self) -> usize { self.weak_gaps }
    pub closed spec fn gaps(&self) -> int {
        self.strong_gaps() as int + self.weak_gaps() as int
    }
    pub closed spec fn strong_retiring(&self) -> usize { self.strong_retiring }
    pub closed spec fn weak_retiring(&self) -> usize { self.weak_retiring }
    pub closed spec fn upgrade_arc_held(&self) -> usize { self.upgrade_arc_held }
    pub closed spec fn arc_owners(&self) -> usize { self.arc_owners }
    pub closed spec fn close_requested(&self) -> bool { self.close_requested }
    pub closed spec fn strong_live(&self) -> int {
        self.sender_live() as int + self.owned_permit_live() as int
    }

    pub closed spec fn well_formed(&self, resources: &MpscFiniteExecutionResources) -> bool {
        &&& resources.well_formed()
        &&& self.strong() as int == self.sender_live() as int
            + self.owned_permit_live() as int + self.strong_gaps() as int
            + self.strong_retiring() as int
        &&& self.weak() as int == self.weak_live() as int
            + self.weak_gaps() as int + self.weak_retiring() as int
        &&& self.arc_owners() as int == 1int + self.sender_live() as int
            + self.owned_permit_live() as int + self.weak_live() as int
            + self.strong_retiring() as int + self.weak_retiring() as int
            + self.upgrade_arc_held() as int
        &&& self.arc_owners() <= usize::MAX / 2
        &&& self.gaps() <= resources.live_threads() as int
        &&& self.close_requested() == (self.strong() == 0)
    }

    pub fn new(resources: &MpscFiniteExecutionResources) -> (result: Self)
        requires resources.well_formed(),
        ensures result.well_formed(resources), result.strong() == 1,
            result.weak() == 0, result.sender_live() == 1,
            result.owned_permit_live() == 0, result.weak_live() == 0,
            result.arc_owners() == 2, result.gaps() == 0,
            !result.close_requested(),
        no_unwind
    {
        MpscEndpointRefinement {
            strong_atomic: 1, weak_atomic: 0, sender_live: 1,
            owned_permit_live: 0, weak_live: 0, strong_gaps: 0,
            weak_gaps: 0, strong_retiring: 0, weak_retiring: 0,
            upgrade_arc_held: 0, arc_owners: 2, close_requested: false,
        }
    }

    pub fn begin_clone_strong(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).sender_live() > 0,
            old(self).gaps() < resources.live_threads() as int,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong() + 1,
            final(self).strong_gaps() == old(self).strong_gaps() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
            final(self).sender_live() == old(self).sender_live(),
        no_unwind
    {
        self.strong_atomic += 1;
        self.strong_gaps += 1;
    }

    pub fn finish_clone_strong(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).strong_gaps() > 0,
            old(self).arc_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong(),
            final(self).strong_gaps() + 1 == old(self).strong_gaps(),
            final(self).sender_live() == old(self).sender_live() + 1,
            final(self).arc_owners() == old(self).arc_owners() + 1,
        no_unwind
    {
        self.strong_gaps -= 1;
        self.sender_live += 1;
        self.arc_owners += 1;
    }

    pub fn begin_construct_weak(
        &mut self,
        resources: &MpscFiniteExecutionResources,
        from_strong: bool,
    )
        requires old(self).well_formed(resources),
            from_strong ==> old(self).sender_live() > 0,
            !from_strong ==> old(self).weak_live() > 0,
            old(self).gaps() < resources.live_threads() as int,
        ensures final(self).well_formed(resources),
            final(self).weak() == old(self).weak() + 1,
            final(self).weak_gaps() == old(self).weak_gaps() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_gaps() == old(self).strong_gaps(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak_retiring() == old(self).weak_retiring(),
        no_unwind
    {
        self.weak_atomic += 1;
        self.weak_gaps += 1;
    }

    pub fn finish_construct_weak(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_gaps() > 0,
            old(self).arc_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).weak() == old(self).weak(),
            final(self).weak_gaps() + 1 == old(self).weak_gaps(),
            final(self).weak_live() == old(self).weak_live() + 1,
            final(self).arc_owners() == old(self).arc_owners() + 1,
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_gaps() == old(self).strong_gaps(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak_retiring() == old(self).weak_retiring(),
        no_unwind
    {
        self.weak_gaps -= 1;
        self.weak_live += 1;
        self.arc_owners += 1;
    }

    pub fn reserve_owned_move(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).sender_live() > 0,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() + 1 == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
        no_unwind
    {
        self.sender_live -= 1;
        self.owned_permit_live += 1;
    }

    pub fn return_owned_permit(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).owned_permit_live() > 0,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live() + 1,
            final(self).owned_permit_live() + 1 == old(self).owned_permit_live(),
            final(self).arc_owners() == old(self).arc_owners(),
        no_unwind
    {
        self.owned_permit_live -= 1;
        self.sender_live += 1;
    }

    pub fn begin_drop_strong(
        &mut self,
        resources: &MpscFiniteExecutionResources,
        owner: StrongOwner,
    )
        requires old(self).well_formed(resources),
            owner == StrongOwner::Sender ==> old(self).sender_live() > 0,
            owner == StrongOwner::OwnedPermit ==> old(self).owned_permit_live() > 0,
        ensures final(self).well_formed(resources),
            final(self).strong() == old(self).strong(),
            final(self).strong_retiring() == old(self).strong_retiring() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_gaps() == old(self).weak_gaps(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            owner == StrongOwner::Sender ==>
                final(self).sender_live() + 1 == old(self).sender_live(),
            owner == StrongOwner::OwnedPermit ==>
                final(self).owned_permit_live() + 1 == old(self).owned_permit_live(),
        no_unwind
    {
        match owner {
            StrongOwner::Sender => self.sender_live -= 1,
            StrongOwner::OwnedPermit => self.owned_permit_live -= 1,
        }
        self.strong_retiring += 1;
    }

    pub fn finish_drop_strong(&mut self, resources: &MpscFiniteExecutionResources)
        -> (last: bool)
        requires old(self).well_formed(resources), old(self).strong_retiring() > 0,
        ensures final(self).well_formed(resources),
            final(self).strong() + 1 == old(self).strong(),
            final(self).strong_retiring() + 1 == old(self).strong_retiring(),
            final(self).arc_owners() + 1 == old(self).arc_owners(),
            last == (old(self).strong() == 1),
            last ==> final(self).close_requested(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_gaps() == old(self).weak_gaps(),
            final(self).weak_retiring() == old(self).weak_retiring(),
        no_unwind
    {
        self.strong_atomic -= 1;
        self.strong_retiring -= 1;
        self.arc_owners -= 1;
        if self.strong_atomic == 0 {
            self.close_requested = true;
            true
        } else { false }
    }

    pub fn begin_drop_weak(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_live() > 0,
        ensures final(self).well_formed(resources),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() + 1 == old(self).weak_live(),
            final(self).weak_retiring() == old(self).weak_retiring() + 1,
            final(self).arc_owners() == old(self).arc_owners(),
        no_unwind
    {
        self.weak_live -= 1;
        self.weak_retiring += 1;
    }

    pub fn finish_drop_weak(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_retiring() > 0,
        ensures final(self).well_formed(resources),
            final(self).weak() + 1 == old(self).weak(),
            final(self).weak_retiring() + 1 == old(self).weak_retiring(),
            final(self).arc_owners() + 1 == old(self).arc_owners(),
        no_unwind
    {
        self.weak_atomic -= 1;
        self.weak_retiring -= 1;
        self.arc_owners -= 1;
    }

    /// `WeakSender::upgrade` clones its Arc before entering `Tx::upgrade`.
    pub fn begin_upgrade_arc(&mut self, resources: &MpscFiniteExecutionResources)
        requires old(self).well_formed(resources), old(self).weak_live() > 0,
            old(self).arc_owners() < usize::MAX / 2,
        ensures final(self).well_formed(resources),
            final(self).upgrade_arc_held() == old(self).upgrade_arc_held() + 1,
            final(self).arc_owners() == old(self).arc_owners() + 1,
            final(self).strong() == old(self).strong(),
        no_unwind
    {
        self.upgrade_arc_held += 1;
        self.arc_owners += 1;
    }

    pub fn upgrade_step(
        &mut self,
        resources: &MpscFiniteExecutionResources,
        observed: usize,
        cas_succeeds: bool,
    ) -> (result: UpgradeStep)
        requires old(self).well_formed(resources), old(self).upgrade_arc_held() > 0,
            observed == 0 ==> old(self).strong() == 0,
            cas_succeeds ==> observed > 0 && observed == old(self).strong(),
        ensures final(self).well_formed(resources),
            observed == 0 ==> result == UpgradeStep::Closed,
            observed > 0 && !cas_succeeds ==> result == UpgradeStep::Retry,
            observed > 0 && cas_succeeds ==> result == UpgradeStep::Constructed,
            result == UpgradeStep::Retry ==> *final(self) == *old(self),
            result == UpgradeStep::Closed ==>
                final(self).upgrade_arc_held() + 1 == old(self).upgrade_arc_held()
                    && final(self).arc_owners() + 1 == old(self).arc_owners()
                    && final(self).strong() == old(self).strong(),
            result == UpgradeStep::Constructed ==>
                final(self).upgrade_arc_held() + 1 == old(self).upgrade_arc_held()
                    && final(self).arc_owners() == old(self).arc_owners()
                    && final(self).strong() == old(self).strong() + 1
                    && final(self).sender_live() == old(self).sender_live() + 1,
        no_unwind
    {
        if observed == 0 {
            self.upgrade_arc_held -= 1;
            self.arc_owners -= 1;
            UpgradeStep::Closed
        } else if cas_succeeds {
            self.upgrade_arc_held -= 1;
            self.strong_atomic += 1;
            self.sender_live += 1;
            UpgradeStep::Constructed
        } else {
            UpgradeStep::Retry
        }
    }
}

pub fn verify_full_count_clone_window(resources: &MpscFiniteExecutionResources)
    requires resources.well_formed(), resources.live_threads() > 0,
{
    let mut endpoints = MpscEndpointRefinement::new(resources);
    endpoints.begin_clone_strong(resources);
    assert(endpoints.strong() == 2);
    endpoints.finish_clone_strong(resources);
    assert(endpoints.sender_live() == 2);
}

pub fn verify_weak_upgrade_cannot_resurrect(resources: &MpscFiniteExecutionResources)
    requires resources.well_formed(), resources.live_threads() > 0,
{
    let mut endpoints = MpscEndpointRefinement::new(resources);
    endpoints.begin_construct_weak(resources, true);
    endpoints.finish_construct_weak(resources);
    endpoints.begin_drop_strong(resources, StrongOwner::Sender);
    let last = endpoints.finish_drop_strong(resources);
    assert(last && endpoints.close_requested());
    endpoints.begin_upgrade_arc(resources);
    let result = endpoints.upgrade_step(resources, 0, false);
    assert(result == UpgradeStep::Closed);
    assert(endpoints.strong() == 0);
}

} // verus!
