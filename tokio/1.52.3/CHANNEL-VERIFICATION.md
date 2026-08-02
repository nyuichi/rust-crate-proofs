# Tokio channel verification status

This file records the scoped verification added after SetOnce and oneshot. The
models are executable Verus bodies with no assumptions in their
channel-specific transitions. Production correspondence is checked by the
exact Tokio integration and bounded loom cases in `verify-all.bash`.

## oneshot

`oneshot_refinement` connects the earlier state, poll, close, payload, and drop
proofs to one production-shaped resource model. It proves the exact four state
bits with no unknown-bit states, weak-CAS retry and lifecycle-bit monotonicity,
task-bit/Waker correspondence, endpoint and local-Arc conservation, final
cleanup, and the outer `Option<Arc<Inner<T>>>` transitions. Sender publication
is deliberately split at its real interference point: `inner.take()` and the
physical `Option<T>` store create a staged sender capability; receiver close
may then win; completion either publishes `VALUE_SENT` and transfers the slot
capability to the Receiver or returns that exact staged value to the Sender.
This also proves the production behavior that `try_recv` releases its outer
Receiver option without inventing a CLOSED bit.

Production `OneshotValue<T>` has layout and store/take/observe regression tests,
and both `rt`-only and `sync`-only feature builds are integrated. The raw
UnsafeCell semantics are frozen, but connecting the proved state capability to
the vstd permission inside production's shared-`&self` Tokio wrapper is a
Tokio-specific, still-unclosed representation refinement—not a frozen adapter.
Repeated terminal `try_recv` and terminal `close` no-op behavior are now proved.
The remaining Tokio-specific gaps are `coop::poll_proceed`/`trace_leaf`
orchestration; `blocking_recv` through the `rt` runtime-context/BlockingRegion
and non-`rt` CachedParkThread paths owned by the runtime/park rows (not T01);
Future repoll-after-Ready panic preservation; task-bit/Waker transition-in-
progress states and their concurrent atomic invariant; arbitrary panic in the
unstable tracing branch; and public error/Debug/Display/Error/Clone trait paths.
`oneshot` and `oneshot_value` use
the identical `cfg(any(feature = "rt", all(windows, feature = "process")))`
predicate as source-equivalence evidence, but the Windows process-only
cross-build remains unexecuted because that target is unavailable locally.
Therefore
S02 remains **L / R(partial) / I**, not current-closed.

## watch

`watch_protocol` proves:

- logical generation publication and independent per-Receiver seen versions;
- exact-value replacement and `borrow_and_update` state;
- register-before-check and the post-registration lost-wakeup recheck;
- Sender and Receiver reference counts;
- closure only by the final Sender, and reopening by subscription while a
  Sender remains.

`watch_completion` and `watch_refinement` additionally prove the exact
closed-bit/even-version encoding including wrap, arbitrary post-update values
for modified/unmodified/panicking closure outcomes, the separate lock-held
publication and post-unlock notification phases, circular shard selection,
all-eight BigNotify call-generation fanout, registration snapshots, repeated
notification, and latest-version recheck after multiple publications.

The production RwLock and future/Waker execution remain frozen adapters. Full
machine-word cycles remain explicit ABA boundaries for both the watch version
and a newly-created Notify future's call-generation snapshot. Additionally,
Notify's production WAITING counter increment panics at its terminal value in
overflow-checking builds, whereas the abstract shard model wraps. Predicate
orchestration in `wait_for`, endpoint/refcount/closed registration races, and
the RNG-backed selector's connection to U02 remain production-refinement gaps.
The integration target has 22 tests. Three bounded loom cases cover
multi-Receiver publication, concurrent final-Sender drop, and value/lock
consistency. The upstream `wait_for_test` scheduler search is intentionally not
integrated because it exhibits combinatorial explosion even with bounded
preemptions; predicate closure execution remains outside the model.
[`WATCH-MUTATION-AUDIT.md`](WATCH-MUTATION-AUDIT.md) records the rejected
register-after-check mutation and the deliberately unclaimed atomic-ordering
mutation.

## broadcast

`broadcast_protocol` proves:

- unique tail reservation and Receiver-count capture for each send;
- slot generation, exact eviction, and remaining-reader conservation;
- per-Receiver cursors, empty/closed distinction, and exact lag distance;
- recovery to the oldest retained position after lag;
- last-Sender and last-Receiver closure plus resubscription.

`broadcast_completion` additionally proves arbitrary-length Receiver draining
with one cursor invariant, Clone-success/panic guard release, and intrusive
waiter membership/Waker replacement at the non-pointer protocol level.

The logical proof uses non-wrapping positions. Production u64 wrapping, Mutex
and atomic implementations, `T::clone` execution, intrusive waiter links, and
Waker delivery remain adapters. The integration target has 32 tests. Three
bounded loom cases cover ring overwrite/lag, two Receiver value delivery, and
Receiver drop while values remain.
[`BROADCAST-MUTATION-AUDIT.md`](BROADCAST-MUTATION-AUDIT.md) records rejected
lag-cursor and remaining-reader mutations.

## mpsc

`mpsc_protocol` proves:

- bounded conservation: `available + reserved + queued == capacity`;
- reservation success/full/closed outcomes;
- borrowed/owned permit commit and cancellation accounting;
- Receiver close, draining of previously reserved sends, and Receiver drop;
- FIFO slot claims and exact message ownership;
- a later producer publishing first cannot bypass an earlier busy slot;
- all values precede the final Sender close marker;
- the mandatory pop, AtomicWaker registration, pop-again polling shape.

`mpsc_completion` lifts the local proof to arbitrary logical slot ownership
using tracked claimed/ready/consumed sets, specifies the all-consumed condition
for block reclamation, proves reserve-many iterator conservation, and proves
that weak Senders cannot resurrect a channel after the final strong Sender.

`mpsc_endpoint_refinement` connects that rule to production's exact
`tx_count`/`tx_weak_count` updates for both bounded and unbounded wrappers. Its
invariant is phase-accurate: each atomic count includes live, constructing, and
retiring wrappers, so a count read during `downgrade`, clone, upgrade, or Drop
is not misidentified as merely the number of fully returned handles. It proves
strong/weak clone and Drop accounting, spurious-CAS Retry preservation,
successful upgrade construction, and terminal no-resurrection. Strong owners
are split into live Sender and live OwnedPermit counts: successful
`reserve_owned` moves ownership without changing `tx_count`, `send`/`release`
return the same owner as Sender, and OwnedPermit Drop retires it exactly once.
The final strong `fetch_sub`
establishes only `CloseRequested`; raw-list close insertion and wake execution
remain outside this refinement. Three module-local tests connect sequential
count observations and terminal upgrade behavior for bounded, unbounded, and
owned-permit paths. Count additions are proved only under the explicit finite
`endpoint_count_room` observation window. Production full-count behavior and
the concurrent interval between Tokio's count increment and following Arc
clone are not proved safe by this refinement.

`mpsc_refinement` now proves the production branch order for single-message
`Chan::recv` and `recv_many` after the trace/cooperative gates. It separates
first pop, Waker registration, and second pop; first-pop Value/Closed never
register, while post-registration Value, TX_CLOSED, receiver-close+idle, and
Pending priorities preserve the latest Waker. A receiver closed with an
outstanding permit remains Pending. Caller witnesses cover both continuations:
commit/send then Value then terminal, and cancel/drop then idle terminal. Value
delivery composes with `MpscCapacity` to restore exactly one bounded permit.
`recv_many` uses only the logical-oracle observation count as its progress
measure, returns that many permits once, returns any nonempty observed prefix
before registration, and handles `limit == 0` only after the outer gates. A
bounded batch witness composes two observations with two capacity returns. The
unbounded refinement proves the exact `(messages << 1) | receiver_closed`
encoding, a single `fetch_sub(2)` effect per Value, preservation on
Pending/Closed observations, and the positive-count no-underflow condition.
Bounded and unbounded public poll behavior plus these capacity/batch cases are
exercised by five module-local production tests; they do not expose whether a
particular Value arrived on production's first pop or post-registration pop.

The two-slot executable queue is a local proof of the production block-list
ordering rule, not a proof of arbitrary raw block allocation and reclamation.
The refinement consumes `QueueRead` as the already-established logical pop
oracle; it does not connect that oracle to the raw list/UnsafeCell chain. Its
gate lemma preserves only modeled Waker/returned-permit/progress bookkeeping;
queue, caller buffer, and capacity snapshots are not part of that lemma.
Trace/coop execution itself and untouched raw queue state remain their owning
boundaries. Block pointers, index wrapping, block reuse, full-count and
pre-Arc-clone endpoint overflow behavior, last-strong raw-list close/wake
completion, `try_recv`'s
CachedParkThread Busy loop, recv-many buffer-allocation/panic-guard/drop paths,
arbitrary destructors, and scheduler liveness remain open. The source audit
also found non-wrapping additions in `Block::grow` and `Block::has_value` at the
final aligned machine-word block; debug can panic and release `has_value` can
misclassify the wrapped range. Production was not changed; see
[`MPSC-WRAP-AUDIT.md`](MPSC-WRAP-AUDIT.md). The ordinary mpsc target has 100
tests and its weak-Sender target has 28. Three bounded loom cases cover bounded
close, unbounded close, and the send-versus-Receiver-close race. The upstream
`try_recv` loom case is excluded from integration after its scheduler search
exceeded one minute; ordinary tests and the deterministic Busy/FIFO proof cover
its channel-specific logic.
[`MPSC-MUTATION-AUDIT.md`](MPSC-MUTATION-AUDIT.md) records three independently
rejected production mutations plus formal receive-orchestration witnesses.

## Status terminology

Watch's generation protocol, broadcast's logical ring protocol, and mpsc's
arbitrary logical slot ownership, permits, close ordering, and reclamation
precondition are proved under the adapters listed above. The raw block pointer
representation is connected through the exact-value two-slot refinement and
production tests, while pointer validity and allocation remain the declared
common adapter. This is scoped channel-protocol completion, not a no-trust
proof of the foundational runtime implementations.
