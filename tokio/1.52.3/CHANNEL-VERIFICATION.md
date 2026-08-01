# Tokio channel verification status

This file records the scoped verification added after SetOnce and oneshot. The
models are executable Verus bodies with no assumptions in their
channel-specific transitions. Production correspondence is checked by the
exact Tokio integration and bounded loom cases in `verify-all.bash`.

## watch

`watch_protocol` proves:

- logical generation publication and independent per-Receiver seen versions;
- exact-value replacement and `borrow_and_update` state;
- register-before-check and the post-registration lost-wakeup recheck;
- Sender and Receiver reference counts;
- closure only by the final Sender, and reopening by subscription while a
  Sender remains.

`watch_completion` additionally proves the exact closed-bit/even-version
encoding including wrap, modified/unmodified/panicking update outcomes,
publication-before-notification, and composition with the Receiver recheck.

The production representation's low closed bit, wrapping version arithmetic,
RwLock guards, BigNotify selection, and future/Waker execution remain adapters.
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

The logical proof uses non-wrapping positions. Production u64 wrapping, Mutex
and atomic implementations, `T::clone` execution, intrusive waiter links, and
Waker delivery remain adapters. The integration target has 32 tests. Three
bounded loom cases cover ring overwrite/lag, two Receiver value delivery, and
Receiver drop while values remain.

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

The two-slot executable queue is a local proof of the production block-list
ordering rule, not a proof of arbitrary raw block allocation and reclamation.
Block pointers, index wrapping, block reuse, Semaphore/AtomicWaker execution,
weak-count atomics, arbitrary destructors, and scheduler liveness remain
adapters. The ordinary mpsc target has 100 tests and its weak-Sender target has
28. Three bounded loom cases cover bounded close, unbounded close, and the
send-versus-Receiver-close race. The upstream `try_recv` loom case is excluded
from integration after its scheduler search exceeded one minute; ordinary
tests and the deterministic Busy/FIFO proof cover its channel-specific logic.

## Status terminology

Watch's generation protocol and broadcast's logical ring protocol are proved
under the adapters listed above. Mpsc's permit lifecycle and local FIFO/close
ordering are proved, but the arbitrary-length lock-free block list is only
represented by a two-slot refinement. Therefore this work is a substantial
channel-core milestone, not a complete end-to-end proof of all three
production modules.
