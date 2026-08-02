use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UpgradeStep {
    Constructing,
    Retry,
    Closed,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StrongOwner { Sender, OwnedPermit }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OwnedPermitReturn { Send, Release }

/// Explicit finite observation window for count arithmetic. This is a proof
/// precondition, not a claim that production Arc behavior establishes it.
pub open spec fn endpoint_count_room(count: usize) -> bool {
    count < usize::MAX
}

/// Phase-accurate projection of `Chan::{tx_count, tx_weak_count}`.
///
/// Tokio increments a count before its Arc clone has become a returned wrapper,
/// and decrements it inside wrapper Drop. Therefore an atomic count is the sum
/// of live, constructing, and retiring endpoints—not just completed wrappers.
/// Raw atomic linearization and Arc ownership remain frozen adapters.
pub struct MpscEndpointRefinement {
    strong_atomic: usize,
    sender_live: usize,
    owned_permit_live: usize,
    strong_constructing: usize,
    strong_retiring: usize,
    weak_atomic: usize,
    weak_live: usize,
    weak_constructing: usize,
    weak_retiring: usize,
    close_requested: bool,
}

impl MpscEndpointRefinement {
    pub closed spec fn strong(&self) -> usize { self.strong_atomic }
    pub closed spec fn sender_live(&self) -> usize { self.sender_live }
    pub closed spec fn owned_permit_live(&self) -> usize { self.owned_permit_live }
    pub closed spec fn strong_live(&self) -> int {
        self.sender_live() as int + self.owned_permit_live() as int
    }
    pub closed spec fn strong_constructing(&self) -> usize { self.strong_constructing }
    pub closed spec fn strong_retiring(&self) -> usize { self.strong_retiring }
    pub closed spec fn weak(&self) -> usize { self.weak_atomic }
    pub closed spec fn weak_live(&self) -> usize { self.weak_live }
    pub closed spec fn weak_constructing(&self) -> usize { self.weak_constructing }
    pub closed spec fn weak_retiring(&self) -> usize { self.weak_retiring }
    pub closed spec fn close_requested(&self) -> bool { self.close_requested }

    pub closed spec fn well_formed(&self) -> bool {
        &&& self.strong() as int == self.sender_live() as int
            + self.owned_permit_live() as int + self.strong_constructing() as int
            + self.strong_retiring() as int
        &&& self.weak() as int == self.weak_live() as int
            + self.weak_constructing() as int + self.weak_retiring() as int
        &&& (self.strong() == 0 <==> self.close_requested())
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(), result.strong() == 1,
            result.sender_live() == 1, result.owned_permit_live() == 0,
            result.strong_live() == 1, result.strong_constructing() == 0,
            result.strong_retiring() == 0, result.weak() == 0,
            result.weak_live() == 0, result.weak_constructing() == 0,
            result.weak_retiring() == 0, !result.close_requested(),
        no_unwind
    {
        MpscEndpointRefinement {
            strong_atomic: 1,
            sender_live: 1,
            owned_permit_live: 0,
            strong_constructing: 0,
            strong_retiring: 0,
            weak_atomic: 0,
            weak_live: 0,
            weak_constructing: 0,
            weak_retiring: 0,
            close_requested: false,
        }
    }

    /// Linearization of `Tx::clone`'s Relaxed `fetch_add(1)`, before Arc clone
    /// and construction of the returned Tx finish.
    pub fn begin_clone_strong(&mut self)
        requires old(self).well_formed(), old(self).sender_live() > 0,
            endpoint_count_room(old(self).strong()),
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong() + 1,
            final(self).strong_constructing() == old(self).strong_constructing() + 1,
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.strong_atomic += 1;
        self.strong_constructing += 1;
    }

    pub fn finish_construct_strong(&mut self)
        requires old(self).well_formed(), old(self).strong_constructing() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).strong_constructing() + 1 == old(self).strong_constructing(),
            final(self).sender_live() == old(self).sender_live() + 1,
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live() + 1,
            final(self).strong_live() > 0,
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.strong_constructing -= 1;
        self.sender_live += 1;
    }

    /// Shared first half of Tx::downgrade and WeakSender::clone.
    pub fn begin_construct_weak(&mut self, from_strong: bool)
        requires old(self).well_formed(),
            from_strong ==> old(self).sender_live() > 0,
            !from_strong ==> old(self).weak_live() > 0,
            endpoint_count_room(old(self).weak()),
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak() + 1,
            final(self).weak_constructing() == old(self).weak_constructing() + 1,
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.weak_atomic += 1;
        self.weak_constructing += 1;
    }

    pub fn finish_construct_weak(&mut self)
        requires old(self).well_formed(), old(self).weak_constructing() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_constructing() + 1 == old(self).weak_constructing(),
            final(self).weak_live() == old(self).weak_live() + 1,
            final(self).weak_live() > 0,
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.weak_constructing -= 1;
        self.weak_live += 1;
    }

    /// Successful `reserve_owned(self)` moves the same Tx owner into an
    /// OwnedPermit. No endpoint count changes.
    pub fn reserve_owned_move(&mut self)
        requires old(self).well_formed(), old(self).sender_live() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() + 1 == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live() + 1,
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.sender_live -= 1;
        self.owned_permit_live += 1;
    }

    /// Both OwnedPermit::send and OwnedPermit::release take the stored Tx and
    /// return it as Sender. Their message/capacity effects are owned elsewhere.
    pub fn return_owned_permit(&mut self, _how: OwnedPermitReturn)
        requires old(self).well_formed(), old(self).owned_permit_live() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live() + 1,
            final(self).owned_permit_live() + 1 == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.owned_permit_live -= 1;
        self.sender_live += 1;
    }

    pub fn begin_drop_strong(&mut self, owner: StrongOwner)
        requires old(self).well_formed(),
            owner == StrongOwner::Sender ==> old(self).sender_live() > 0,
            owner == StrongOwner::OwnedPermit ==> old(self).owned_permit_live() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            owner == StrongOwner::Sender ==>
                final(self).sender_live() + 1 == old(self).sender_live(),
            owner == StrongOwner::Sender ==>
                final(self).owned_permit_live() == old(self).owned_permit_live(),
            owner == StrongOwner::OwnedPermit ==>
                final(self).owned_permit_live() + 1 == old(self).owned_permit_live(),
            owner == StrongOwner::OwnedPermit ==>
                final(self).sender_live() == old(self).sender_live(),
            final(self).strong_live() + 1 == old(self).strong_live(),
            final(self).strong_retiring() == old(self).strong_retiring() + 1,
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
        no_unwind
    {
        match owner {
            StrongOwner::Sender => self.sender_live -= 1,
            StrongOwner::OwnedPermit => self.owned_permit_live -= 1,
        }
        self.strong_retiring += 1;
    }

    /// Linearization of Drop's `fetch_sub(1)`. `last` establishes only a
    /// CloseRequested obligation. Raw list close insertion and wake execution
    /// occur afterward and are not claimed by this refinement.
    pub fn finish_drop_strong(&mut self) -> (last: bool)
        requires old(self).well_formed(), old(self).strong_retiring() > 0,
        ensures final(self).well_formed(),
            final(self).strong() + 1 == old(self).strong(),
            final(self).strong_retiring() + 1 == old(self).strong_retiring(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            last == (old(self).strong() == 1),
            last ==> final(self).close_requested(),
            !last ==> final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        self.strong_atomic -= 1;
        self.strong_retiring -= 1;
        if self.strong_atomic == 0 {
            self.close_requested = true;
            true
        } else {
            false
        }
    }

    pub fn begin_drop_weak(&mut self)
        requires old(self).well_formed(), old(self).weak_live() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() + 1 == old(self).weak_live(),
            final(self).weak_retiring() == old(self).weak_retiring() + 1,
            final(self).weak_constructing() == old(self).weak_constructing(),
        no_unwind
    {
        self.weak_live -= 1;
        self.weak_retiring += 1;
    }

    pub fn finish_drop_weak(&mut self)
        requires old(self).well_formed(), old(self).weak_retiring() > 0,
        ensures final(self).well_formed(),
            final(self).strong() == old(self).strong(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).strong_constructing() == old(self).strong_constructing(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() + 1 == old(self).weak(),
            final(self).weak_retiring() + 1 == old(self).weak_retiring(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
        no_unwind
    {
        self.weak_atomic -= 1;
        self.weak_retiring -= 1;
    }

    /// One `compare_exchange_weak` iteration after the temporary Arc clone.
    /// Success creates a constructing Tx; Retry includes spurious failure.
    pub fn upgrade_step(&mut self, observed: usize, cas_succeeds: bool)
        -> (result: UpgradeStep)
        requires old(self).well_formed(), old(self).weak_live() > 0,
            observed == 0 ==> old(self).strong() == 0,
            cas_succeeds ==> observed > 0,
            cas_succeeds ==> observed == old(self).strong(),
            cas_succeeds ==> endpoint_count_room(old(self).strong()),
        ensures final(self).well_formed(),
            observed == 0 ==> result == UpgradeStep::Closed,
            observed > 0 && cas_succeeds ==> result == UpgradeStep::Constructing,
            observed > 0 && !cas_succeeds ==> result == UpgradeStep::Retry,
            result == UpgradeStep::Constructing ==>
                final(self).strong() == old(self).strong() + 1
                    && final(self).strong_constructing()
                        == old(self).strong_constructing() + 1,
            result != UpgradeStep::Constructing ==>
                final(self).strong() == old(self).strong()
                    && final(self).strong_constructing()
                        == old(self).strong_constructing(),
            final(self).strong_live() == old(self).strong_live(),
            final(self).sender_live() == old(self).sender_live(),
            final(self).owned_permit_live() == old(self).owned_permit_live(),
            final(self).strong_retiring() == old(self).strong_retiring(),
            final(self).weak() == old(self).weak(),
            final(self).weak_live() == old(self).weak_live(),
            final(self).weak_constructing() == old(self).weak_constructing(),
            final(self).weak_retiring() == old(self).weak_retiring(),
            final(self).close_requested() == old(self).close_requested(),
        no_unwind
    {
        if observed == 0 {
            UpgradeStep::Closed
        } else if cas_succeeds {
            self.strong_atomic += 1;
            self.strong_constructing += 1;
            UpgradeStep::Constructing
        } else {
            UpgradeStep::Retry
        }
    }
}

pub fn verify_counts_include_constructing_and_retiring_phases()
{
    let mut endpoints = MpscEndpointRefinement::new();
    endpoints.begin_clone_strong();
    assert(endpoints.strong() == 2 && endpoints.strong_constructing() == 1);
    endpoints.finish_construct_strong();
    assert(endpoints.sender_live() == 2 && endpoints.strong() == 2);
    endpoints.begin_construct_weak(true);
    assert(endpoints.weak() == 1 && endpoints.weak_live() == 0);
    assert(endpoints.weak_constructing() == 1);
    endpoints.finish_construct_weak();
    assert(endpoints.weak() == 1 && endpoints.weak_live() == 1);
    endpoints.begin_drop_weak();
    assert(endpoints.weak() == 1 && endpoints.weak_retiring() == 1);
    endpoints.finish_drop_weak();
    assert(endpoints.weak() == 0 && endpoints.weak_retiring() == 0);
}

pub fn verify_upgrade_retry_success_and_no_resurrection()
{
    let mut endpoints = MpscEndpointRefinement::new();
    endpoints.begin_construct_weak(true);
    endpoints.finish_construct_weak();
    let retry = endpoints.upgrade_step(1, false);
    assert(retry == UpgradeStep::Retry);
    let constructing = endpoints.upgrade_step(1, true);
    assert(constructing == UpgradeStep::Constructing);
    assert(endpoints.strong() == 2 && endpoints.strong_constructing() == 1);
    endpoints.finish_construct_strong();
    endpoints.begin_drop_strong(StrongOwner::Sender);
    let first_drop = endpoints.finish_drop_strong();
    assert(!first_drop);
    endpoints.begin_drop_strong(StrongOwner::Sender);
    let last_drop = endpoints.finish_drop_strong();
    assert(last_drop && endpoints.close_requested());
    let closed = endpoints.upgrade_step(0, false);
    assert(closed == UpgradeStep::Closed);
    assert(endpoints.strong() == 0 && endpoints.weak() == 1);
}

pub fn verify_owned_permit_keeps_upgrade_open()
{
    let mut endpoints = MpscEndpointRefinement::new();
    endpoints.begin_construct_weak(true);
    endpoints.finish_construct_weak();
    endpoints.reserve_owned_move();
    assert(endpoints.sender_live() == 0 && endpoints.owned_permit_live() == 1);
    assert(endpoints.strong() == 1);
    let constructing = endpoints.upgrade_step(1, true);
    assert(constructing == UpgradeStep::Constructing);
    endpoints.finish_construct_strong();
    assert(endpoints.strong() == 2);
    assert(endpoints.sender_live() == 1 && endpoints.owned_permit_live() == 1);

    endpoints.return_owned_permit(OwnedPermitReturn::Send);
    assert(endpoints.sender_live() == 2 && endpoints.owned_permit_live() == 0);
    assert(endpoints.strong() == 2);
    endpoints.reserve_owned_move();
    endpoints.return_owned_permit(OwnedPermitReturn::Release);
    assert(endpoints.sender_live() == 2 && endpoints.strong() == 2);

    endpoints.reserve_owned_move();
    endpoints.begin_drop_strong(StrongOwner::OwnedPermit);
    assert(endpoints.strong() == 2 && endpoints.strong_retiring() == 1);
    let dropped_permit = endpoints.finish_drop_strong();
    assert(!dropped_permit && endpoints.strong() == 1);
    assert(endpoints.sender_live() == 1 && endpoints.owned_permit_live() == 0);
}

} // verus!
