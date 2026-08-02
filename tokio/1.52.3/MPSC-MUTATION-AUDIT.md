# Tokio mpsc mutation audit

Date: 2026-08-01

Mutations were applied independently to a disposable copy.

| Mutation | Detecting test | Result |
|---|---|---|
| remove the second queue pop after AtomicWaker registration | bounded `loom_mpsc::closing_tx` | rejected: loom reports a lost-wakeup deadlock |
| omit the semaphore permit returned by successful `try_recv` | `sync_mpsc::try_recv_bounded` | rejected: a later `try_send` incorrectly reports Full |
| allow weak upgrade after the final strong Sender is gone | `sync_mpsc_weak::downgrade_upgrade_sender_failure` | rejected: the channel is resurrected |

The mutations cover the receive recheck, bounded capacity conservation, and
strong/weak no-resurrection invariants proved by the scoped mpsc models.

## Formal receive-orchestration witnesses

These are proof-level counterfactuals in
`verification/src/mpsc_refinement.rs`, not additional destructive-copy runs:

| Counterfactual | Rejected property |
|---|---|
| mutate Waker, returned-permit count, or progress bookkeeping when `trace_leaf` or `poll_proceed` returns Pending | the gate projection preserves exactly those three modeled fields before the first pop; queue, buffer, and capacity snapshots remain unconnected |
| register before returning a first-pop Value/Closed result | first-pop terminal helpers preserve the prior Waker and finish directly |
| omit the second pop or turn its Value into Pending | register/recheck returns the exact value, restores one bounded permit, records progress, and retains the latest registered Waker |
| treat `receiver_closed` alone as terminal while a permit is reserved | close + non-idle recheck remains Pending so the reserved permit may still publish |
| lose or duplicate a reserved permit after that Pending result | composed callers cover both commit/send → Value → capacity restoration → terminal and cancel/drop → idle terminal |
| return Pending when receiver-close and idle are both visible | the post-registration terminal priority returns Ready(None) and records progress |
| return `limit` permits after a short `recv_many` batch | the sole `values.len()` progress measure fixes both the logical-oracle observation prefix and exactly-once batch permit count; a bounded-capacity caller restores precisely that count |
| register after `recv_many` has already added a value | any nonempty prefix returns immediately with its exact batch and preserves the pre-existing Waker |
| inspect the queue for `recv_many(limit = 0)` before the outer gates pass | the zero-limit branch exists only after gate success and returns Ready(0) without changing the buffer |
| decrement the unbounded encoded count twice, underflow it, or change it on a no-value terminal/Pending observation | the `(messages << 1) | receiver_closed` refinement requires a positive count for exactly one subtraction and preserves the word on no-value observations |

## Formal endpoint-count witnesses

`verification/src/mpsc_endpoint_refinement.rs` rejects the following
counterfactuals above the frozen atomic/Arc adapters:

| Counterfactual | Rejected property |
|---|---|
| equate an atomic count only with fully returned wrappers | the invariant explicitly includes live, constructing, and retiring phases on both strong and weak sides |
| omit or double the strong increment in `Tx::clone` or successful upgrade | the constructing-strong transition changes `tx_count` and its phase by exactly one |
| omit or double the weak increment in downgrade or WeakSender clone | the constructing-weak transition changes `tx_weak_count` and its phase by exactly one |
| omit or double either strong or weak Drop decrement | retirement preserves the count before linearization and decreases exactly the matching counter once afterward |
| swap the strong and weak counters in any clone or Drop path | every transition preserves the opposite counter and all of its phases |
| let a spurious `compare_exchange_weak` failure change endpoint state | Retry preserves every count and phase |
| upgrade after observing terminal zero | zero returns Closed without constructing a strong endpoint |
| let weak handles delay the final-strong decision | the last result depends only on the strong `fetch_sub(1)` linearization and establishes `CloseRequested` once |
| count reserve-owned as a new endpoint, lose it on send/release, or retire it twice on Drop | owner-kind transitions move exactly one live owner between Sender and OwnedPermit with constant `tx_count`; Drop moves it once through retiring and decrements once |

This proof does not identify `CloseRequested` with completed raw-list close
insertion or wake execution. Count additions use the explicit finite
`endpoint_count_room` window; production full-count behavior and the concurrent
increment-before-Arc-clone interval remain unproved.
