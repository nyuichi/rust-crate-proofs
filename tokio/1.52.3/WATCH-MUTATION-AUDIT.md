# Tokio watch mutation audit

Date: 2026-08-01

Mutations were applied one at a time to a disposable copy and are not present
in the verification target.

| Mutation | Target | Result |
|---|---|---|
| move `notify_rx.notified()` after `maybe_changed` in `changed_impl` | `sync::tests::loom_watch::smoke` with two bounded preemptions | rejected: loom reports a deadlock caused by the lost wake-up |
| weaken version `fetch_add` from Release to Relaxed | the same loom smoke case | not detected |

The first result validates the watch-specific register-before-check proof. The
second is intentionally recorded rather than overstated: RwLock acquisition
also synchronizes production value access, so this test does not isolate the
atomic publication ordering. Raw atomic ordering remains a common memory-model
adapter and is not counted as mutation-validated watch-specific reasoning.
