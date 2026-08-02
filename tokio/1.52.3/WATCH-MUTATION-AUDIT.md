# Tokio watch mutation audit

Date: 2026-08-01

The first two source mutations were applied one at a time to a disposable copy
and are not present in the verification target. The remaining mutation rows
are rejected by explicit Verus witnesses; the private runtime/Loom checks
separately exercise the production BigNotify fanout loop.

| Mutation | Target | Result |
|---|---|---|
| move `notify_rx.notified()` after `maybe_changed` in `changed_impl` | `sync::tests::loom_watch::smoke` with two bounded preemptions | rejected: loom reports a deadlock caused by the lost wake-up |
| weaken version `fetch_add` from Release to Relaxed | the same loom smoke case | not detected |
| notify only one `BigNotify` shard instead of advancing all eight | `BigNotifyGenerations::notify_waiters` and `verify_notification_after_registration_is_ready` | rejected: an arbitrarily selected shard must advance from its registration snapshot |
| model a shard as a sticky wake bit | `verify_repeated_notification_requires_reregistration` | rejected: consuming a wake and re-registering captures the current call generation and requires a later advance |
| notify before advancing the watch version | `verify_notify_before_publish_mutant_loses_wake` | rejected: the receiver can consume the early wake, re-register, and sleep through the later publication |
| check CLOSED before version in `maybe_changed` | `verify_changed_wins_over_closed` | rejected: an unseen final value is incorrectly skipped |
| check version before CLOSED in public `has_changed` | `verify_public_has_changed_reports_close_first` | rejected: the API returns `Ok(true)` after channel closure instead of `RecvError` |
| increment the raw state by one instead of `STEP_SIZE = 2` | `verify_watch_refinement_mutants_rejected` | rejected: the increment corrupts the reserved CLOSED bit |
| make `mark_changed` non-wrapping, or fail to mark `borrow_and_update` seen | `verify_mark_borrow_and_silent_updates` | rejected: the synthetic previous even version and subsequent observation transition are lost |
| advance or notify after a false-returning/panicking update closure | `verify_watch_unmodified_and_panic_do_not_notify` and `verify_mark_borrow_and_silent_updates` | rejected: both arbitrary post-closure values preserve the version and return no fanout phase witness |

The first result validates the watch-specific register-before-check proof. The
second is intentionally recorded rather than overstated: RwLock acquisition
also synchronizes production value access, so this test does not isolate the
atomic publication ordering. Raw atomic ordering remains a common memory-model
adapter and is not counted as mutation-validated watch-specific reasoning.

The private eight-shard runtime and Loom tests are regression evidence for the
production loop only; the proof claim comes from the per-shard generation and
snapshot contracts above.

`verify_publish_publish_notify_rechecks_latest` additionally covers the
production overlap in which two lock-held publications precede the first
post-unlock fanout. The first fanout wakes a recheck of the latest version.
This is execution coverage, not a rejected mutation.

The returned `WatchNotification` is an ordinary droppable proof-model value,
not a Rust-level linear obligation coupled to `BigNotify`. Production-shaped
caller proofs establish publication-before-fanout and the two-publication
overlap, but do not claim that the model's type system makes omission of every
fanout call impossible.

`verify_complete_watch_cycle_aba` records a separate finite-counter boundary. The
encoded version has period `usize::MAX / 2 + 1`; after exactly that many
successful modified updates, a receiver that has not observed any intermediate
state again compares equal to its stored version. `verify_subcycle_update_is_detected`
proves the corresponding strict subcycle case for every initial encoded version.
Each underlying Notify call counter now has an all-build terminal gate before
generation zero can be reused. The earlier complete Notify-cycle ABA witness is
retained only as a rejected wrapping-policy counterexample; it is unreachable
under the selected production policy. Watch's own encoded-version period
remains the boundary for unbounded `changed`/`has_changed` completeness.

`WatchUpdateKind` and the post-closure value are unconstrained inputs to the
update-closure refinement. This covers true, false-returning, and unwinding
update closure outcomes without assuming or trusting execution of the arbitrary
user closure itself.

Still-open production refinement includes `wait_for` predicate orchestration,
endpoint/refcount/closed registration races, and the RNG-backed BigNotify
selector's connection to the U02 random helper. Those paths are not promoted to
component-complete status by this audit.
