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

Production `OneshotValue<T>` has layout and store/take/observe regression tests.
The generic body-proved `vstd_ext::shared_pcell` now separates the PCell from
its exact permission, allowing `oneshot_shared_refinement` to retain
production's shared-`&self` access shape while transferring that permission
through Sender, staged, Receiver, Inner-drop, and released roles. RX/TX Waker
slots likewise carry exact Vacant/Writing/Published/Detached/Released
capabilities, so the task bit is set exactly while the Waker is fully
initialized and readable.

`oneshot_surface` closes trace-before-coop ordering, budget restore/commit,
Ready termination and later-poll panic, then instantiates the proved
runtime-context/BlockingRegion and CachedParkThread block-on model for oneshot
Value/Closed. Repeated terminal calls, public error traits, unstable tracing,
`full`, `sync`-only, `rt`-only, and Windows process-only cfgs are integrated.
The remaining interfaces are only the frozen atomic/UnsafeCell/Arc,
Pin/Poll/Context/Waker, arbitrary user trait/Drop, and scheduler/parking
boundaries; taskdump callback mechanics remain assigned to R08. Therefore S02
is **C / R / I**. See `ONESHOT-CLOSURE-CHECKLIST.md`.

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
`tx_count`/`tx_weak_count` updates and Arc-owner phases for both wrapper
families. Count-before-Arc gaps are explicit and bounded by the number of live
threads in `MpscFiniteExecutionResources`, while completed owners are bounded
by the physical Arc/execution environment. Strong/weak clone and Drop,
spurious-CAS Retry, successful upgrade, no resurrection, Sender/OwnedPermit
moves, and exactly-once retirement are body-proved. The final strong decrement
composes in `mpsc_orchestration` with the raw queue's unique close marker and
one receiver wake. Arc and Waker execution remain frozen foundations.

`mpsc_try_recv_refinement` covers `Chan::try_recv` terminal priorities. Initial
Value/Closed/Empty branches return without Busy-side
effects. Initial Busy performs the one pre-parker AtomicWaker wake, prepares a
park Waker, and each modeled loop step registers before rechecking. A Busy
recheck permits exactly one park before the next registration; Value restores
one bounded permit or decrements the unbounded encoded count once, and Empty
distinguishes receiver-close+idle from an outstanding permit. A two-iteration
witness directly composes the wake/register sequence with the proved S09
AtomicWaker machine; the generic register transition is abstract protocol
bookkeeping rather than a direct S09 composition. Two module-local production
tests cover the observable non-Busy bounded and unbounded branches. They do not
force a raw-list Busy schedule.
`mpsc_try_recv_raw_refinement` derives Busy from a claimed but unpublished FIFO
head and proves arbitrary finite register/park repetition using one `nat` rank.
CachedParkThread construction and Waker/park execution remain frozen generic
adapters; finiteness is supplied by the scheduler-liveness foundation. The upstream loom
`try_recv` case was again stopped after its preemption-1 search did not finish
within the bounded run; it is not integration evidence.

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
before registration, and handles `limit == 0` only after the outer gates. Its
conditional preallocated-buffer refinement requires the caller prefix plus
`limit` to fit within an explicit logical Vec capacity. Under that condition it
preserves the caller prefix, proves every Value append stays within capacity,
returns the exact `number_added`, and composes once with bounded
`add_permits(number_added)` or unbounded
`fetch_sub(number_added << 1)`. The latter has the explicit
`number_added * 2 <= usize::MAX` normal-path precondition and proves the shift
equals that multiplication; the unbounded receiver-closed bit is preserved.
An `accounting_applied` phase bit is consumed by either bulk operation, so the
same completed batch cannot apply its accounting twice. A recv-many-specific
environment permits TX_CLOSED observation before the deferred bulk return; the
Closed witness establishes semaphore idle only after applying that return.
Limit completion, short nonempty Empty/Closed completion, and zero-value
Closed versus receiver-close+idle versus outstanding-permit priorities are
covered by proof witnesses. Two pre-reserved production tests assert physical
`capacity() - len() >= limit` before each bounded and unbounded call and confirm
capacity remains unchanged. The current vstd Vec interface does not formally
connect physical capacity to the logical capacity field, so the buffer-content
proof remains conditional. Outside that window, a call-local production guard
records every successful pop before `Vec::push`. Source review and two-pop
zero-sized-Vec regressions cover its capacity-overflow control flow, exact
bounded capacity restoration, and unbounded count decrement. A separate linear
Verus projection shows that a recorded pending count can be consumed once by
either accounting model; it does not directly refine raw pop, Vec unwind, or
compiled Drop execution. Nine module-local tests now exercise the
bounded/unbounded recv poll and batch surfaces; they do not expose whether a
particular Value arrived on production's first pop or post-registration pop.

`mpsc_queue_refinement` derives raw-list observations from logical claims,
ready/close bits, and the unique receiver cursor, and moves arbitrary payloads
through linear slot permissions. It proves release-before-detach and
reuse-or-free traversal leases over absolute generations. Raw allocation and
pointer validity remain foundations. `mpsc_recv_compiled_refinement` frames the
complete queue/buffer/accounting state across trace/coop and connects pop,
guard recording, Vec append, bulk return, and unwind return without a spare
capacity premise. `mpsc_orchestration` and `mpsc_surface` compose first recv,
recv-many, last-close/wake, raw Busy, cooperative budget, blocking cfg result,
identity, and error mappings. The
previously reproduced `Vec<()>` capacity-overflow queue/accounting mismatch is
repaired by exact unwind-time accounting and recorded in
[`MPSC-RECV-MANY-UNWIND-AUDIT.md`](MPSC-RECV-MANY-UNWIND-AUDIT.md). Allocation
and arbitrary destructors remain agreed boundaries. The
source audit found final-word boundary issues in `Block::grow`,
`Block::has_value`, and the reclamation comparison. `Block::grow` now uses
explicit wrapping and has an exact regression. `Block::has_value` and
`Rx::reclaim_blocks` remain unchanged and are the two explicit S05 residuals;
see
[`MPSC-WRAP-AUDIT.md`](MPSC-WRAP-AUDIT.md). The ordinary mpsc target has 100
tests and its weak-Sender target has 28. Three bounded loom cases cover bounded
close, unbounded close, and the send-versus-Receiver-close race. The upstream
`try_recv` loom case is excluded from integration after its scheduler search
exceeded one minute; the raw Busy model and arbitrary finite-prefix rank cover
its safety/conditional-termination obligation instead.
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
