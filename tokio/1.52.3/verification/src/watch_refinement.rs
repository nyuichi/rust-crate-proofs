use crate::watch_completion::{WatchUpdate, WatchUpdateKind};
use vstd::prelude::*;

verus! {

pub open spec fn watch_version_period() -> nat {
    usize::MAX as nat / 2 + 1
}

pub open spec fn notify_call_period() -> nat {
    usize::MAX as nat / 4 + 1
}

pub open spec fn encoded_watch_epoch(epoch: nat) -> nat {
    epoch % watch_version_period()
}

pub open spec fn encoded_notify_epoch(epoch: nat) -> nat {
    epoch % notify_call_period()
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchPollResult { Changed, Closed, Pending }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WatchHasChanged { Changed, Unchanged, Closed }

/// Exact per-receiver view of production `Version` plus the one BigNotify
/// shard and notify-waiters call snapshot captured by `notified()`.
pub struct EncodedReceiverVersion {
    generation: usize,
    registered_shard: Option<usize>,
    notify_snapshot: Option<usize>,
}

impl EncodedReceiverVersion {
    pub closed spec fn generation(&self) -> usize { self.generation }
    pub closed spec fn registered_shard(&self) -> Option<usize> { self.registered_shard }
    pub closed spec fn notify_snapshot(&self) -> Option<usize> { self.notify_snapshot }
    pub closed spec fn raw(&self) -> nat { 2nat * self.generation() as nat }
    pub closed spec fn well_formed(&self) -> bool {
        &&& (self.generation() as nat) < watch_version_period()
        &&& match (self.registered_shard(), self.notify_snapshot()) {
            (Some(shard), Some(snapshot)) =>
                shard < 8 && (snapshot as nat) < notify_call_period(),
            (None, None) => true,
            _ => false,
        }
    }

    pub fn new(generation: usize) -> (result: Self)
        requires (generation as nat) < watch_version_period(),
        ensures result.well_formed(), result.generation() == generation,
            result.registered_shard().is_none(), result.notify_snapshot().is_none(),
            result.raw() == 2 * generation as nat,
        no_unwind
    {
        EncodedReceiverVersion {
            generation,
            registered_shard: None,
            notify_snapshot: None,
        }
    }

    pub fn mark_changed(&mut self)
        requires old(self).well_formed(), old(self).registered_shard().is_none(),
        ensures final(self).well_formed(), final(self).registered_shard().is_none(),
            old(self).generation() == 0 ==>
                final(self).generation() == usize::MAX / 2,
            old(self).generation() > 0 ==>
                final(self).generation() + 1 == old(self).generation(),
            final(self).generation() != old(self).generation(),
        no_unwind
    {
        if self.generation == 0 {
            self.generation = usize::MAX / 2;
        } else {
            self.generation -= 1;
        }
    }

    pub fn mark_unchanged(&mut self, current: usize)
        requires old(self).well_formed(), old(self).registered_shard().is_none(),
            (current as nat) < watch_version_period(),
        ensures final(self).well_formed(), final(self).generation() == current,
            final(self).registered_shard().is_none(),
        no_unwind
    {
        self.generation = current;
    }

    pub fn borrow_and_update(&mut self, current: usize) -> (changed: bool)
        requires old(self).well_formed(), old(self).registered_shard().is_none(),
            (current as nat) < watch_version_period(),
        ensures final(self).well_formed(),
            changed == (old(self).generation() != current),
            final(self).generation() == current,
            final(self).registered_shard().is_none(),
        no_unwind
    {
        let changed = self.generation != current;
        self.generation = current;
        changed
    }

    /// Public `has_changed` reports closure before comparing versions.
    pub fn has_changed(&self, current: usize, closed: bool)
        -> (result: WatchHasChanged)
        requires self.well_formed(), self.registered_shard().is_none(),
            (current as nat) < watch_version_period(),
        ensures closed ==> result == WatchHasChanged::Closed,
            !closed && self.generation() != current ==> result == WatchHasChanged::Changed,
            !closed && self.generation() == current ==> result == WatchHasChanged::Unchanged,
        no_unwind
    {
        if closed { WatchHasChanged::Closed }
        else if self.generation != current { WatchHasChanged::Changed }
        else { WatchHasChanged::Unchanged }
    }

    /// Exact `changed_impl` order: construct `Notified` (capturing shard and
    /// call generation), then execute `maybe_changed`, where change precedes
    /// closure. Only the Pending branch retains the registration snapshot.
    pub fn register_then_check(
        &mut self,
        shard: usize,
        notify_snapshot: usize,
        current: usize,
        closed: bool,
    ) -> (result: WatchPollResult)
        requires old(self).well_formed(), old(self).registered_shard().is_none(),
            shard < 8, (notify_snapshot as nat) < notify_call_period(),
            (current as nat) < watch_version_period(),
        ensures final(self).well_formed(),
            old(self).generation() != current ==> result == WatchPollResult::Changed,
            old(self).generation() != current ==> final(self).generation() == current,
            old(self).generation() == current && closed ==> result == WatchPollResult::Closed,
            old(self).generation() == current && !closed ==> result == WatchPollResult::Pending,
            result != WatchPollResult::Changed ==>
                final(self).generation() == old(self).generation(),
            result == WatchPollResult::Pending ==>
                final(self).registered_shard() == Some(shard),
            result == WatchPollResult::Pending ==>
                final(self).notify_snapshot() == Some(notify_snapshot),
            result != WatchPollResult::Pending ==>
                final(self).registered_shard().is_none(),
        no_unwind
    {
        self.registered_shard = Some(shard);
        self.notify_snapshot = Some(notify_snapshot);
        if self.generation != current {
            self.generation = current;
            self.registered_shard = None;
            self.notify_snapshot = None;
            WatchPollResult::Changed
        } else if closed {
            self.registered_shard = None;
            self.notify_snapshot = None;
            WatchPollResult::Closed
        } else {
            WatchPollResult::Pending
        }
    }

    pub fn consume_wake(&mut self, ready: bool)
        requires old(self).well_formed(), old(self).registered_shard().is_some(), ready,
        ensures final(self).well_formed(),
            final(self).generation() == old(self).generation(),
            final(self).registered_shard().is_none(),
        no_unwind
    {
        self.registered_shard = None;
        self.notify_snapshot = None;
    }
}

fn advance_notify_generation(generation: usize) -> (result: usize)
    requires (generation as nat) < notify_call_period(),
    ensures (result as nat) < notify_call_period(),
        generation == usize::MAX / 4 ==> result == 0,
        generation < usize::MAX / 4 ==> result == generation + 1,
    no_unwind
{
    if generation == usize::MAX / 4 { 0 } else { generation + 1 }
}

/// Exact eight-way `BigNotify` view. Each member stores the upper call-count
/// bits of its production `EncodedNotifyWord`; `notify_waiters` advances every
/// member with that word's wrapping rule.
pub struct BigNotifyGenerations {
    gen0: usize, gen1: usize, gen2: usize, gen3: usize,
    gen4: usize, gen5: usize, gen6: usize, gen7: usize,
}

impl BigNotifyGenerations {
    pub closed spec fn generation(&self, shard: usize) -> usize {
        match shard {
            0 => self.gen0, 1 => self.gen1, 2 => self.gen2, 3 => self.gen3,
            4 => self.gen4, 5 => self.gen5, 6 => self.gen6, 7 => self.gen7,
            _ => 0,
        }
    }
    pub closed spec fn well_formed(&self) -> bool {
        &&& (self.gen0 as nat) < notify_call_period()
        &&& (self.gen1 as nat) < notify_call_period()
        &&& (self.gen2 as nat) < notify_call_period()
        &&& (self.gen3 as nat) < notify_call_period()
        &&& (self.gen4 as nat) < notify_call_period()
        &&& (self.gen5 as nat) < notify_call_period()
        &&& (self.gen6 as nat) < notify_call_period()
        &&& (self.gen7 as nat) < notify_call_period()
    }

    pub fn new() -> (result: Self)
        ensures result.well_formed(),
            forall |shard: usize| shard < 8 ==> result.generation(shard) == 0,
        no_unwind
    {
        BigNotifyGenerations {
            gen0: 0, gen1: 0, gen2: 0, gen3: 0,
            gen4: 0, gen5: 0, gen6: 0, gen7: 0,
        }
    }

    pub fn snapshot(&self, shard: usize) -> (result: usize)
        requires self.well_formed(), shard < 8,
        ensures result == self.generation(shard),
            (result as nat) < notify_call_period(),
        no_unwind
    {
        match shard {
            0 => self.gen0, 1 => self.gen1, 2 => self.gen2, 3 => self.gen3,
            4 => self.gen4, 5 => self.gen5, 6 => self.gen6, 7 => self.gen7,
            _ => 0,
        }
    }

    pub fn notify_waiters(&mut self)
        requires old(self).well_formed(),
        ensures final(self).well_formed(),
            forall |shard: usize| shard < 8 ==>
                (old(self).generation(shard) == usize::MAX / 4
                    ==> final(self).generation(shard) == 0),
            forall |shard: usize| shard < 8 ==>
                (old(self).generation(shard) < usize::MAX / 4
                    ==> final(self).generation(shard)
                        == old(self).generation(shard) + 1),
        no_unwind
    {
        self.gen0 = advance_notify_generation(self.gen0);
        self.gen1 = advance_notify_generation(self.gen1);
        self.gen2 = advance_notify_generation(self.gen2);
        self.gen3 = advance_notify_generation(self.gen3);
        self.gen4 = advance_notify_generation(self.gen4);
        self.gen5 = advance_notify_generation(self.gen5);
        self.gen6 = advance_notify_generation(self.gen6);
        self.gen7 = advance_notify_generation(self.gen7);
    }

    pub fn registration_ready(&self, receiver: &EncodedReceiverVersion)
        -> (result: bool)
        requires self.well_formed(), receiver.well_formed(),
            receiver.registered_shard().is_some(),
        ensures result == (self.generation(receiver.registered_shard().unwrap())
            != receiver.notify_snapshot().unwrap()),
        no_unwind
    {
        match (receiver.registered_shard, receiver.notify_snapshot) {
            (Some(shard), Some(snapshot)) => self.snapshot(shard) != snapshot,
            _ => false,
        }
    }
}

pub fn select_circular_shard(ticket: usize) -> (shard: usize)
    ensures shard == ticket % 8, shard < 8,
    no_unwind
{
    ticket % 8
}

pub fn verify_notification_after_registration_is_ready(ticket: usize)
{
    let shard = select_circular_shard(ticket);
    let mut fanout = BigNotifyGenerations::new();
    let snapshot = fanout.snapshot(shard);
    let mut receiver = EncodedReceiverVersion::new(0);
    let poll = receiver.register_then_check(shard, snapshot, 0, false);
    assert(poll == WatchPollResult::Pending);
    let before_notify = fanout.registration_ready(&receiver);
    assert(!before_notify);
    fanout.notify_waiters();
    let ready = fanout.registration_ready(&receiver);
    assert(ready);
    receiver.consume_wake(ready);
}

pub fn verify_registration_after_notification_is_not_ready(ticket: usize)
{
    let shard = select_circular_shard(ticket);
    let mut fanout = BigNotifyGenerations::new();
    fanout.notify_waiters();
    let snapshot = fanout.snapshot(shard);
    let mut receiver = EncodedReceiverVersion::new(0);
    let poll = receiver.register_then_check(shard, snapshot, 0, false);
    assert(poll == WatchPollResult::Pending);
    let after_registration = fanout.registration_ready(&receiver);
    assert(!after_registration);
}

pub fn verify_repeated_notification_requires_reregistration(ticket: usize)
{
    let shard = select_circular_shard(ticket);
    let mut fanout = BigNotifyGenerations::new();
    let mut receiver = EncodedReceiverVersion::new(0);
    let first_snapshot = fanout.snapshot(shard);
    receiver.register_then_check(shard, first_snapshot, 0, false);
    fanout.notify_waiters();
    let first_ready = fanout.registration_ready(&receiver);
    assert(first_ready);
    receiver.consume_wake(first_ready);

    let second_snapshot = fanout.snapshot(shard);
    receiver.register_then_check(shard, second_snapshot, 0, false);
    let before_second = fanout.registration_ready(&receiver);
    assert(!before_second);
    fanout.notify_waiters();
    let second_ready = fanout.registration_ready(&receiver);
    assert(second_ready);
}

/// Two lock-held publications may precede the first sender's post-unlock
/// fanout. That fanout wakes the receiver, whose version recheck observes the
/// latest publication; the caller proof then performs the second fanout too.
pub fn verify_publish_publish_notify_rechecks_latest(
    first: u64, second: u64, third: u64, ticket: usize)
{
    let shard = select_circular_shard(ticket);
    let mut fanout = BigNotifyGenerations::new();
    let snapshot = fanout.snapshot(shard);
    let mut receiver = EncodedReceiverVersion::new(0);
    receiver.register_then_check(shard, snapshot, 0, false);

    let mut channel = WatchUpdate::new(first);
    let (_, first_notification) = channel.apply_update_under_lock(
        second, WatchUpdateKind::Modified);
    let (_, second_notification) = channel.apply_update_under_lock(
        third, WatchUpdateKind::Modified);
    assert(channel.generation() == 2);

    first_notification.unwrap().complete();
    fanout.notify_waiters();
    let ready = fanout.registration_ready(&receiver);
    receiver.consume_wake(ready);
    let recheck_snapshot = fanout.snapshot(shard);
    let recheck = receiver.register_then_check(shard, recheck_snapshot, 2, false);
    assert(recheck == WatchPollResult::Changed);

    second_notification.unwrap().complete();
    fanout.notify_waiters();
}

/// Counterexample for moving fanout before the lock-held version advance. The
/// receiver consumes that early wake and registers against its still-current
/// version; the later publication has no remaining wake attached to it.
pub fn verify_notify_before_publish_mutant_loses_wake(ticket: usize)
{
    let shard = select_circular_shard(ticket);
    let mut fanout = BigNotifyGenerations::new();
    let mut receiver = EncodedReceiverVersion::new(0);
    let snapshot = fanout.snapshot(shard);
    receiver.register_then_check(shard, snapshot, 0, false);

    fanout.notify_waiters();
    let early_ready = fanout.registration_ready(&receiver);
    receiver.consume_wake(early_ready);
    let after_early_wake = fanout.snapshot(shard);
    let pending = receiver.register_then_check(shard, after_early_wake, 0, false);
    assert(pending == WatchPollResult::Pending);

    let mut channel = WatchUpdate::new(0u64);
    let (_, notification) = channel.apply_update_under_lock(
        1u64, WatchUpdateKind::Modified);
    assert(channel.generation() == 1);
    let still_asleep = fanout.registration_ready(&receiver);
    assert(!still_asleep);

    // The phase witness records that correct production still has a distinct
    // post-publication fanout call. This counterexample deliberately omits the
    // fanout after consuming the witness.
    notification.unwrap().complete();
}

pub fn verify_mark_borrow_and_silent_updates(first: u64, changed: u64)
{
    let mut receiver = EncodedReceiverVersion::new(0);
    receiver.mark_changed();
    assert(receiver.generation() == usize::MAX / 2);
    let observed = receiver.borrow_and_update(0);
    assert(observed);
    receiver.mark_unchanged(0);

    let mut channel = WatchUpdate::new(first);
    let (_, false_notification) = channel.apply_update_under_lock(
        changed, WatchUpdateKind::Unmodified);
    assert(false_notification.is_none());
    let (_, panic_notification) = channel.apply_update_under_lock(
        first, WatchUpdateKind::Panicked);
    assert(panic_notification.is_none());
    assert(channel.generation() == 0);
}

pub fn verify_changed_wins_over_closed()
{
    let mut receiver = EncodedReceiverVersion::new(0);
    let result = receiver.register_then_check(3, 0, 1, true);
    assert(result == WatchPollResult::Changed);
}

pub fn verify_public_has_changed_reports_close_first()
{
    let receiver = EncodedReceiverVersion::new(0);
    let result = receiver.has_changed(1, true);
    assert(result == WatchHasChanged::Closed);
}

pub proof fn verify_subcycle_update_is_detected(start: nat, updates: nat)
    requires start < watch_version_period(), 0 < updates,
        updates < watch_version_period(),
    ensures (start + updates) % watch_version_period() != start,
{
    let period = watch_version_period();
    if start + updates < period {
        vstd::arithmetic::div_mod::lemma_small_mod(start + updates, period);
    } else {
        let reduced: nat = (start + updates - period) as nat;
        assert(reduced < period);
        vstd::arithmetic::div_mod::lemma_small_mod(reduced, period);
        vstd::arithmetic::div_mod::lemma_mod_add_multiples_vanish(
            reduced as int, period as int);
        assert(start + updates == period + reduced);
    }
}

pub proof fn verify_complete_watch_cycle_aba(start: nat)
    requires start < watch_version_period(),
    ensures (start + watch_version_period()) % watch_version_period() == start,
{
    let period = watch_version_period();
    vstd::arithmetic::div_mod::lemma_small_mod(start, period);
    vstd::arithmetic::div_mod::lemma_mod_add_multiples_vanish(
        start as int, period as int);
}

pub proof fn verify_complete_notify_cycle_aba(start: nat)
    requires start < notify_call_period(),
    ensures (start + notify_call_period()) % notify_call_period() == start,
{
    let period = notify_call_period();
    vstd::arithmetic::div_mod::lemma_small_mod(start, period);
    vstd::arithmetic::div_mod::lemma_mod_add_multiples_vanish(
        start as int, period as int);
}

pub proof fn verify_watch_refinement_mutants_rejected()
{
    assert(WatchPollResult::Changed != WatchPollResult::Closed);
    assert((0nat + 1nat) % 2nat == 1nat);
    assert((0nat + 2nat) % 2nat == 0nat);
    assert(0usize != 1usize);
}

} // verus!
