# Tokio watch S03 closure checklist

This file is the specification map and live closure checklist for promoting
S03 from legacy/partial refinement to current `C / R / I` status.  It applies
only to Tokio 1.52.3's standard `sync` watch implementation.

## Public contract map

| Production surface | Mathematical state | Required result |
|---|---|---|
| `channel`, `Sender::new`, `Default` | one value, encoded version, one sender and receiver | exact initial ownership and version |
| `send`, `send_replace`, `send_modify`, `send_if_modified` | lock-held value plus terminal version and fanout phase | value/outcome preservation; publish before fanout; panic/false does not publish |
| `borrow`, `borrow_and_update`, `has_changed`, `mark_changed`, `mark_unchanged` | per-receiver observed version | exact seen/unseen and close priority |
| `changed`, `wait_for` | registration snapshot and predicate outcome trace | register-before-check; changed-before-close; predicate panic releases lock while retaining the observed version |
| Sender/Receiver clone/drop, `subscribe`, counts, `closed`, `is_closed` | completed Arc owners, atomic endpoint counts, and one in-flight clone gap | no resurrection, exact last-drop fanout, reopen/recheck semantics, no counter wrap |
| `BigNotify::{notified,notify_waiters}` | eight terminal Notify generations plus selector | selected shard is in `0..8`; every fanout advances all shards; terminal failure is pre-mutation |
| traits/errors/`same_channel` | value-independent public surface | formatting/clone/equality and identity operations preserve channel state |

## Representation map

- The production state word is `2 * generation + closed_bit`.
- A Receiver stores one even version word and at most one selected Notify shard
  snapshot while waiting.
- The RwLock value and the Release/Acquire state word are related at the
  lock-held publication point; the frozen RwLock and atomic memory semantics
  remain foundation adapters.
- BigNotify is an array of exactly eight S07 Notify instances.  The circular
  selector uses `ticket % 8`; the RNG selector uses the high 32 bits of a
  `u32 * 8` product and therefore also returns `0..8`.
- Endpoint counters count completed endpoint handles plus any explicitly
  modeled counter/Arc ordering gap.  Arc implementation and allocation remain
  frozen; Tokio's ordering and count conservation are S03 obligations.

## Proof boundaries and order

1. Thin orchestration: `wait_for`, `closed`, publication/fanout, and public
   result mapping over reviewed leaf contracts.
2. Leaf state machines: predicate outcomes, endpoint lifecycle, registration
   rechecks, selector range arithmetic, and terminal generation transitions.
3. Production refinement: exact branch/order mapping and regression tests.
4. Integrated Tokio-scoped run and independent audit.

No temporary trusted component is planned.  If the watch version is allowed to
wrap, full-cycle ABA cannot satisfy the unbounded changed-detection contract;
the selected terminal-overflow policy or an explicit bounded-history trusted
contract must be recorded before S03 can close.

## Live status

| Component | Contract reviewed | Body proved | Trusted | Integrated |
|---|---:|---:|---:|---:|
| Existing publish/change/BigNotify protocol | yes | yes | no | yes |
| `wait_for` predicate orchestration | yes | yes | no | yes |
| endpoint/refcount/`closed` races | yes | yes | no | yes |
| circular and RNG selector refinement | yes | yes | U02 RNG step only | yes |
| terminal watch version / full-cycle ABA | yes | yes | no | yes |
| public traits/errors/identity/cooperative surface | yes | yes | T01 budget/TLS only | yes |

## Stop conditions

- Redesign a proof interface after two same-shaped failures.
- Stop and report after three failures in one area or 30--45 minutes without
  structural progress.
- Validate leaf milestones with the verification crate only; reserve
  `verify-all.bash` for integrated checkpoints.
