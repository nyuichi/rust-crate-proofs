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
| overwrite or discard the caller's existing `recv_many` prefix | the preallocated-buffer view is always the immutable initial prefix followed by exactly the observed Values |
| append beyond the pre-reserved normal-path window | `initial_prefix.len() + limit <= buffer_capacity`, `added <= limit`, and the remaining-versus-spare invariant keep every modeled push within capacity |
| report total buffer length instead of `number_added` | the return and permit count are both exactly the appended suffix length, independent of the initial prefix length |
| return bounded permits one-by-one, apply the same completed batch twice, or decrement the unbounded word by the wrong batch | the general final-nonempty composition consumes `accounting_applied == false`, sets it true, and uses exactly `permits_returned == number_added`; bounded accounting returns that batch, while unbounded accounting proves `number_added << 1 == number_added * 2` under its explicit no-overflow bound and subtracts it once |
| let Empty/Closed discard an already appended suffix, or let receiver-close override an outstanding permit with no appended value | a nonempty suffix returns immediately; the zero-value recheck stays Pending until receiver-close and semaphore-idle are both visible, while TX_CLOSED remains terminal |
| require semaphore idle before processing a nonempty TX_CLOSED batch | the recv-many environment allows Closed observation before its deferred bulk return, and the composed witness establishes bounded idle only after applying that exact batch |
| omit the unwind guard, arm it after `Vec::push`, or apply a different count | the linear guard records each successful pop before push, and its unwind transition consumes exactly the outstanding count; bounded and unbounded witnesses restore/decrement precisely two popped values and leave no reusable obligation |

The preallocated witnesses are conditional normal-path proofs, not an allocator
model. Physical pre-reserved Vec tests assert `capacity() - len() >= limit` and
confirm capacity is unchanged on the covered production branches. The vstd Vec
contract does not formally connect that physical capacity to the logical proof
field. A standard-library capacity-overflow unwind was reproduced after
production had popped a value but before its deferred batch accounting. The
selected repair keeps normal-path batching and uses a post-pop guard to apply
the exact outstanding batch on unwind. Permanent bounded/unbounded safe-API
tests and the linear accounting witnesses are recorded in
[`MPSC-RECV-MANY-UNWIND-AUDIT.md`](MPSC-RECV-MANY-UNWIND-AUDIT.md). Allocation
and arbitrary destructor execution remain foundational rather than modeled.

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

## Formal below-saturation finite-prefix `try_recv` witnesses

`verification/src/mpsc_try_recv_refinement.rs` rejects these control-flow
counterfactuals for every modeled finite Busy prefix whose `usize` proof-trace
counters remain below saturation:

| Counterfactual | Rejected property |
|---|---|
| return Empty or Disconnected immediately for Busy | Busy has only the internal Continue outcome and enters the wake/register path |
| omit or repeat the pre-parker wake | the NeedWake transition performs exactly one wake before any registration or park |
| recheck or park before registering the park Waker | phase preconditions require each recheck after registration and each park after a registered Busy |
| park twice for one Busy observation | one NeedPark transition increments the finite park trace once and requires a new registration before another recheck |
| treat receiver-close alone as Disconnected with a reserved permit | Empty is Disconnected only when receiver-close and semaphore-idle are both observed |
| return a Value without restoring/decrementing accounting exactly once | Value exits with one logical permit and composes with bounded `receive` or unbounded `fetch_sub(2)` accounting |

The raw-list source of Busy, CachedParkThread mechanics, and termination of the
unbounded production loop are not claimed by these witnesses. Only the exact
two-Busy witness directly composes with S09 AtomicWaker; the generic register
transition is abstract protocol bookkeeping. Scheduler liveness is an allowed
foundation, but no instantiated premise here proves this loop terminates.
