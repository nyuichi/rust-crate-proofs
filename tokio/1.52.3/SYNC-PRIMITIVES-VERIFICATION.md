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

The production state-word atomics, mutex, intrusive pointer representation,
Pin/Poll/Context/Waker operations, and arbitrary waker destruction remain the
declared common adapters. The integrated run includes both public Notify test
suites and bounded exact loom races for notify-one, broadcast, and cancellation
forwarding/drop.

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

Production's synchronous Mutex and watch storage/waker execution remain the
frozen adapters. The exact public Barrier suite, including ten generations of
100 parties, is included in the integrated run.

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

This proves the wake-state algorithm, not Waker callback execution or the Rust
atomic memory model. Those are explicitly frozen adapters. The integrated run
adds Tokio's ordinary AtomicWaker tests and bounded exact loom cases for
multiple racing notifications and a panicking waker clone.
