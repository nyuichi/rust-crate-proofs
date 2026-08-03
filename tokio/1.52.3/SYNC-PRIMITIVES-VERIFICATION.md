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

`notify_refinement` additionally proves the production state-word formulas and
the selected terminal transition:

- the upper `notify_waiters` call count and the EMPTY/WAITING/NOTIFIED low bits;
- increment-by-four without corrupting state below the terminal value;
- mutation-free terminal rejection for EMPTY, NOTIFIED, and locked WAITING;
- preservation of an already stored notify-one permit across a broadcast;
- exact locked notify-one selection and final-waiter state repair;
- first-poll branch priority: a newer broadcast is observed before consuming a
  stored permit.

Production now checks the call-count terminal before either the atomic
EMPTY/NOTIFIED update or locked WAITING mutation. The policy is identical in
all builds: the terminal call panics without advancing the word, unlinking a
waiter, or executing a Waker. Generation zero is never reused.

The raw state-word atomic operation, mutex, intrusive pointer representation,
Pin/Poll/Context/Waker operations, and arbitrary waker destruction remain the
declared common adapters. The Tokio-specific values and branch ordering around
those adapters are directly refined. The integrated run includes both public
Notify test suites and bounded exact loom races for notify-one, broadcast, and
cancellation forwarding/drop. [`NOTIFY-MUTATION-AUDIT.md`](NOTIFY-MUTATION-AUDIT.md)
records the rejected state-word, poll-order, and terminal-policy mutations.

## OnceCell

`once_cell_refinement` composes the closed one-permit Semaphore contract with
the exact Empty, Initializing, and Published OnceCell phases. It body-proves
nonblocking `set`, unique async initialization, waiter observation,
cancellation/panic/error permit return, retry, recursive waiting, mutable
replacement, take/into-inner, and public result/trait mapping. Initializer
Future behavior and generic Clone/Debug/Drop execution remain specification
inputs at the recorded standard trait boundary.

Three loom models connect production ordering: concurrent initializers execute
exactly one closure, cancelling a Pending initializer releases its permit, and
an error from `get_or_try_init` reopens the cell. Eighteen public regressions
cover constructors, all result branches, cancellation, panic retry, recursive
initialization cancellation, take/reuse, traits, and payload destruction.
[`ONCE-CELL-CLOSURE-CHECKLIST.md`](ONCE-CELL-CLOSURE-CHECKLIST.md) records the
scope and frozen adapters. Taskdump trace projection remains assigned to R08.

## Async Mutex

`mutex_refinement` composes S06's closed one-permit semaphore with a
production-shaped guard capability and proves:

- exact immediate success/failure and mutual exclusion for borrowed and owned
  acquisition;
- FIFO grant order and exact removal of a cancelled waiter without reordering
  its survivors;
- borrowed, owned, mapped, and owned-mapped guards all retain the same sole
  permit across `map`, nested map, and both `try_map` outcomes;
- failure of `try_map` returns the original live guard unchanged;
- consuming any of the four guard types releases exactly one permit;
- arbitrary values are conserved across guarded mutation, `get_mut`, and
  `into_inner`;
- Mutex formatting selects data only when `try_lock` succeeds, and the exact
  `TryLockError` message is preserved.

Focused public tests cover all guard transformations, owned Arc retention,
FIFO cancellation, blocking wrappers, value surfaces, formatting, and errors.
Bounded exact loom cases connect the production semaphore/UnsafeCell path to
mutual exclusion, release, and pending-future cancellation. Arc mechanics,
UnsafeCell/raw-pointer validity, atomics, Future polling, arbitrary Drop, and
scheduler liveness remain frozen adapters. No production source was changed.
[`MUTEX-CLOSURE-CHECKLIST.md`](MUTEX-CLOSURE-CHECKLIST.md) records the exact
closure boundary.

## Async RwLock

`rwlock_refinement` composes S06's batch semaphore and proves that production
starts with exactly `mr` permits, each reader holds one, and a writer holds all
of them. This yields shared-read safety, writer exclusion, exact try-operation
results, FIFO writer preference, and cancellation without survivor reordering.

Every borrowed/owned read projection retains one permit. Every borrowed/owned
write and mapped-write projection retains all `mr` permits. Failed `try_map`
and `try_downgrade_map` return the original live guard without release.
Downgrade retains one writer permit as the new read guard and atomically
releases exactly `mr - 1`; a queued writer therefore cannot interleave before
the downgraded reader exists. All read, write, mapped, and owned destructor
amounts are covered by the same linear capability.

Focused public tests cover constructor limits, writer preference/cancellation,
every map family, owned Arc identity, downgrade, blocking wrappers, formatting,
and value ownership. Existing loom tests connect concurrent read/write and
downgrade behavior; the mixed case uses preemption bound one to keep integrated
runtime finite, while the general state space is body-proved in Verus.
Arc/UnsafeCell/raw-pointer mechanics, atomics, polling, arbitrary Drop, and
scheduler liveness remain frozen. No production source was changed.
[`RWLOCK-CLOSURE-CHECKLIST.md`](RWLOCK-CLOSURE-CHECKLIST.md) records the exact
closure boundary.

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
selection, publication of the captured generation, and cohort reset. The exact
`published >= captured` follower predicate is also proved to accept the
leader's equal generation token. The defensive `wrapping_add(1)` leaf is proved
total, but rollover is not claimed reachable through the integrated policy.

`barrier_orchestration` composes those leaves with S03's finite budgets. The
first arrival of a cohort reserves one watch update and all `n - 1` private
Receiver-construction credits before mutating Barrier state. Followers consume
those credits before committing their arrivals, so cancellation does not
return a credit and no admitted cohort can fail midway for watch exhaustion.
Insufficient capacity releases the Mutex and panics at a clean cohort boundary
without changing `arrived`, generation, or reservation state.
The orchestration additionally proves the exact reachable cross-state relation
`Barrier generation = watch update generation + 1`, including terminal
fixtures, so the S03 gate rejects before any Barrier token can be reused.

The production-shaped `BarrierWaitResult(bool)` mapping is body-proved for
leader construction, `is_leader`, and clone observation. Debug/Clone behavior
is runtime-tested, while Send/Sync/Unpin properties and the wait future's
traits are compile-tested; those trait checks are not described as body proofs.

Production's synchronous Mutex and watch storage/waker execution remain the
frozen adapters. S03's terminal gate prevents a full machine-word Barrier
generation cycle, eliminating generation-reuse ambiguity. The exact public
Barrier suite, including cancellation, result traits, ten generations of 100
parties, terminal three-party admission/rejection, and the arithmetic rollover
leaf regression are included in the integrated run.
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
