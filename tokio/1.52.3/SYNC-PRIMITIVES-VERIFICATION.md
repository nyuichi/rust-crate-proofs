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

This is scoped completion of the logical Semaphore protocol, not yet a direct
translation of the production bodies into Verus.
