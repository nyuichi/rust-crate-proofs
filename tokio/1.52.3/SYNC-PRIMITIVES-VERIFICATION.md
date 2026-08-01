# Tokio sync primitive verification status

This document extends the frozen channel verification scope to Tokio sync
primitives. The common trusted foundation remains: Rust weak-memory atomics;
Mutex/RwLock/Arc/UnsafeCell; Pin/Poll/Context/Waker; raw pointer validity and
allocation; arbitrary Clone/Drop execution; and scheduler liveness. Everything
listed below is Tokio-specific protocol logic above that foundation.

## Semaphore and batch_semaphore

`semaphore_protocol` body-proves:

- global conservation of available, publicly held, queue-escrowed, and
  intentionally forgotten permits;
- exact success and failure conditions for immediate acquisition;
- partial assignment to a queued acquire and exact return on cancellation;
- arbitrary-length FIFO order, including head-of-line blocking for a large
  request ahead of a smaller request;
- close as an irreversible state which detaches every queued waiter;
- the common borrowed/owned permit lifecycle: split, same-semaphore merge,
  forget, and drop/release.

`semaphore_refinement` additionally proves the production representation:

- low-bit CLOSED encoding, `PERMIT_SHIFT`, and the reserved `MAX_PERMITS` bound;
- exact successful/failed try-acquire subtraction while preserving CLOSED;
- saturating `forget_permits` with mandatory CLOSED-bit reattachment;
- unqueued release addition and its overflow precondition;
- production `push_front` plus `last/pop_back` is exactly FIFO.

The production correspondence is:

| Production operation | Proved transition |
|---|---|
| `try_acquire`, uncontended `poll_acquire` | `SemaphoreAccounting::try_acquire` |
| partial `poll_acquire` | `escrow_available` + `SemaphoreFifo::enqueue` |
| `add_permits_locked` / `Waiter::assign_permits` | `SemaphoreFifo::assign_front` |
| completed acquire | `complete_waiter` |
| `Acquire::drop` | `SemaphoreFifo::cancel` + `cancel_waiter` |
| `close` | both protocol `close` transitions |
| permit `split`, `merge`, `forget`, `Drop` | `PermitValue` plus accounting transfer |

The atomic bit encoding, mutex exclusion, intrusive pointer validity, Arc
identity/lifetime representation, and actual Waker execution remain only the
declared foundation adapters. The integrated run exercises both borrowed and
owned public API suites and bounded exact loom cases for basic acquisition,
concurrent cancellation, and multi-permit FIFO assignment.

The raw atomic operation remains the declared memory-model adapter. The
Tokio-specific values passed to and returned from that adapter are now directly
refined. [`SEMAPHORE-MUTATION-AUDIT.md`](SEMAPHORE-MUTATION-AUDIT.md) records
the formally rejected encoding and queue-order mutations.

## Notify

The earlier `NotifiedCore` and `IntrusiveWaiters` proofs establish each
future's Init/Waiting/Done/drop lifecycle, generation rechecks, waker
replacement, guarded broadcast-list ownership, and unlink-before-publication.
`notify_protocol` now closes the remaining global public protocol by proving:

- the single stored notify-one permit and coalescing of repeated notifications;
- exact FIFO `notify_one` and LIFO `notify_last` selection;
- `enable`/first-poll priority for a newer broadcast generation over a stored
  permit;
- atomic `notify_waiters` generation advance and arbitrary-length queue drain;
- cancellation of a waiting node and forwarding of an unconsumed notify-one
  marker using its original FIFO/LIFO strategy;
- identical borrowed `Notified` and Arc-owned `OwnedNotified` logical behavior.

`notify_refinement` additionally proves the production state-word formulas:

- the upper `notify_waiters` call count and the EMPTY/WAITING/NOTIFIED low bits;
- increment-by-four and machine-word generation wrap without corrupting state;
- preservation of an already stored notify-one permit across a broadcast;
- exact locked notify-one selection and final-waiter state repair;
- first-poll branch priority: a newer broadcast is observed before consuming a
  stored permit.

The raw state-word atomic operation, mutex, intrusive pointer representation,
Pin/Poll/Context/Waker operations, and arbitrary waker destruction remain the
declared common adapters. The Tokio-specific values and branch ordering around
those adapters are directly refined. The integrated run includes both public
Notify test suites and bounded exact loom races for notify-one, broadcast, and
cancellation forwarding/drop. [`NOTIFY-MUTATION-AUDIT.md`](NOTIFY-MUTATION-AUDIT.md)
records the rejected state-word and poll-order mutations.

## Barrier

`barrier_protocol` composes the trusted Mutex adapter with the proved watch
generation protocol and body-proves:

- `Barrier::new(0)` normalization to a one-party barrier;
- arrival counts remain below the configured party count between releases;
- exactly the nth arrival is recorded as the unique leader of a generation;
- followers become ready only after that generation is published;
- all pending followers are released together and the arrival count resets;
- later generations reuse the barrier without confusing earlier followers;
- the documented non-cancel-safe behavior: cancelling a pending wait removes
  its future but deliberately leaves its arrival counted for that generation.

`barrier_refinement` additionally proves the exact production critical-section
values: zero-party normalization, bounded `arrived + 1`, nth-arrival leader
selection, publication of the captured generation, cohort reset, and explicit
machine-word rollover. Production now uses `wrapping_add(1)`, eliminating its
former debug-only panic after publishing generation `usize::MAX`. The exact
`published >= captured` follower predicate is also proved to accept the
leader's equal generation token.

Production's synchronous Mutex and watch storage/waker execution remain the
frozen adapters. A follower suspended across an entire machine-word generation
cycle remains a finite-counter liveness boundary under the already-frozen
scheduler-liveness assumption; safety and unique leadership do not rely on
excluding it. The exact public Barrier suite, including ten generations of 100
parties, and the rollover regression are included in the integrated run.
[`BARRIER-MUTATION-AUDIT.md`](BARRIER-MUTATION-AUDIT.md) records the rejected
production mutations.

## AtomicWaker

`atomic_waker_protocol` body-proves the Tokio-specific two-bit protocol:

- the exact WAITING, REGISTERING, REGISTERING|WAKING, and WAKING states;
- exclusive logical ownership of the optional waker slot;
- replacement by a successful registration and preservation after a panicking
  waker clone;
- wake taking a registered value at most once;
- wake racing with registration is completed by the registering thread;
- registration racing with an active wake directly wakes its input;
- redundant wake and competing-register paths cannot access the slot;
- every critical-section path restores WAITING.

`atomic_waker_refinement` additionally proves the literal production values
WAITING=0, REGISTERING=1, WAKING=2, and REGISTERING|WAKING=3, including:

- the complete `compare_exchange(WAITING, REGISTERING)` result table;
- the complete `fetch_or(WAKING)` table and which result alone owns the slot;
- the release-CAS/swap return branches and final WAITING restoration;
- preservation of the old slot when Waker cloning panics;
- exact callback ownership in the register+wake race: on successful
  replacement both the displaced old Waker and the new Waker are consumed,
  while the panic branch consumes the old Waker before resuming the panic.

This proves the wake-state algorithm and callback identities, not Waker callback
execution or the Rust atomic memory model. Those are explicitly frozen
adapters. The integrated run adds Tokio's ordinary AtomicWaker tests and
bounded exact loom cases for multiple racing notifications and a panicking
waker clone. [`ATOMIC-WAKER-MUTATION-AUDIT.md`](ATOMIC-WAKER-MUTATION-AUDIT.md)
records the rejected bit, lock-ownership, and unwind mutations.
