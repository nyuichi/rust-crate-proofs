# tokio 1.52.3 verification provenance

**Verification status: SetOnce is complete under the frozen scoped-completion
definition below. Publication, ownership, lock exclusivity, waiter-list
membership, wait, cancellation, and trait views are body-proved models
structurally connected to production and exercised by targeted runtime, loom,
layout, and mutation checks.**

**Oneshot status: its channel-specific protocol is complete under the scoped
definition below. Exact state-bit transitions, payload ownership, ordered
atomic operations, both polling protocols, Waker replacement, two-owner
cleanup, and destructor-state behavior are body-proved models. The production
payload slot is isolated behind the matching operation-specific adapter and the
connection is exercised by targeted runtime, loom, trait, and mutation checks.**

**Channel expansion status: watch, broadcast, and mpsc are complete under the
frozen scoped-channel definition. Their channel-specific generation, ring,
lag, FIFO, permit, endpoint, poll/recheck, unwind, and cleanup protocols are
body-proved models and mutation-tested against production. Raw block pointer
validity and allocation remain common runtime adapters; the channel-specific
reclamation precondition is proved. See
[`CHANNEL-VERIFICATION.md`](CHANNEL-VERIFICATION.md) for the exact boundaries.**

**Watch current closure: S03 is current-closed for standard `sync` cfg. The
production version stops before reuse and before arbitrary update code, with a
final Sender-drop fanout reserved. Receiver construction claims cumulative
last-drop notification credit before Arc/count mutation, while Sender clone is
Arc-first. `wait_for` predicate panic preservation, endpoint/reopen races,
eight-shard RNG/circular selection, cooperative polling, errors, traits, and
identity are body-proved and connected by runtime and Loom regressions.
Taskdump trace projection remains assigned to R08.**

**Sync primitive expansion: Semaphore/batch_semaphore, Notify, Barrier, and
AtomicWaker protocol verification is integrated. Permit conservation, FIFO head blocking, partial
assignment, cancellation return, close, permit transformations, Notify's
stored permit, FIFO/LIFO selection, broadcast generation, enable, and
cancellation forwarding, Barrier's reusable generation and unique-leader
rules, plus AtomicWaker's complete two-bit state protocol, are body-proved above
the frozen common foundation. See
[`SYNC-PRIMITIVES-VERIFICATION.md`](SYNC-PRIMITIVES-VERIFICATION.md).**

**Semaphore production refinement: CLOSED/permit encoding, MAX_PERMITS,
try-acquire, forget, release, and the reversed intrusive FIFO convention are
proved against their exact production formulas. Only raw atomic execution and
pointer validity remain in the frozen foundation.**

**Notify production refinement: the exact generation/state word, increment by
four below the all-build terminal gate, mutation-free terminal rejection,
stored-permit preservation, locked notify-one state repair, and
broadcast-before-permit poll ordering are proved. Raw atomic and
mutex execution, intrusive pointers, and Waker execution remain frozen
foundation adapters.**

**Barrier current closure: exact arrived/generation transitions, leader
publication, reset, follower comparison, cancellation, and public result
mapping are proved. Cohort admission reserves one S03 update and every private
Receiver credit before the first arrival mutates state; exhaustion panics at a
clean boundary and an admitted cohort cannot fail midway. S03's terminal gate
prevents generation reuse, with Barrier generation proved exactly one above
the cumulative watch update count throughout the reachable range. The leader
boolean/result mapping is body-proved; Debug/Clone behavior and auto traits are
runtime/compile evidence. Mutex/watch execution and scheduler liveness remain
frozen foundation adapters.**

**AtomicWaker production refinement: exact two-bit CAS/fetch-or/swap result
tables, slot-lock ownership, clone-panic restoration, and both callback
identities in the register+wake race are proved. Raw atomic/UnsafeCell and
arbitrary Waker execution remain frozen foundation adapters.**

This source tree is copied from the `tokio` 1.52.3 package published on
crates.io. The published archive has SHA-256 checksum
`8fc7f01b389ac15039e4dc9531aa973a135d7a4135281b12d7c1bc79fd57fffe`.
Its `.cargo_vcs_info.json` records upstream revision
`d87569164fb61145e79e7ffe0b25783569cc8f93` and path `tokio`.

The complete upstream source and ordinary public API are retained. The
non-loom `UnsafeCell<T>` wrapper is marked `repr(transparent)` to make its
already single-field runtime representation an explicit adapter guarantee.
The first Verus milestone is intentionally isolated in the nested
`verification` crate so Verus does not translate unrelated Tokio subsystems.
It defines both the value-preserving ownership model and a reusable
`PublishedCell<T>`/`PublishedOnce<T>` kernel. The production source retains
Tokio's representation; a targeted loom test connects its actual `set` and
`get` path to the intended Release/Acquire publication behavior.

## Established model contracts

The model has three canonical states:

```text
Empty -> Published(value) -> Taken
```

The body proofs establish:

- empty and pre-populated construction;
- exact empty-state observation;
- successful publication changes `Empty` to `Published(value)` exactly once;
- a later publication fails, returns the rejected value, and preserves the
  previously published value;
- value observation agrees exactly with the abstract state for `Copy` values;
- taking the inner value moves it to the caller at most once and leaves `Taken`;
- repeated taking returns `None`;
- representative successful, rejected-publication, and empty callers compose
  the method contracts.
- `PublishedCell<T>` keeps its `PCell` permission with the slot and body-proves
  exact-value `publish`, lifetime-correct `get() -> &T`, and `take` operations;
- the reference proof works for a representative non-`Copy` payload;
- production `SetOnceValue<T>` confines the generic loom-cell callback surface
  to the same four operations as the proof view: construction, serialized
  write, publication-justified shared read, and exclusively-owned take;
- `TokioLoomCell<T>` body-proves those operation contracts over vstd's physical
  cell, including the production-shaped unsafe `get() -> &T` contract;
- erased size and alignment are checked against
  `UnsafeCell<MaybeUninit<T>>` for zero-sized, scalar, array, and over-aligned
  payloads; the same test directly compiles Tokio's production wrapper source
  and checks its size, alignment, field address, and `with`/`with_mut` pointer
  behavior;
- `ReleaseAcquireFlag<T>` combines an atomic invariant with a tokenized state
  machine: Release publication consumes the unique writer token, and an
  Acquire load that observes publication returns clonable persistent knowledge
  of the exact value;
- its executable raw bridge uses the exact Rust `Acquire`, `Release`, and
  `Relaxed` orderings; Relaxed observation proves readiness but returns no
  value-publication token;
- production `SetOnceFlag` exposes only operation-specific Acquire, Release,
  and Relaxed methods, so SetOnce call sites cannot select arbitrary orderings;
- `PublishedOnce<T>` maintains `flag == slot.is_init()`, and its
  production-shaped `initialized`, `get`, publish, and take bodies preserve
  that relation;
- `WriterLease<T>` body-proves the locked double-check and first-writer-wins
  portion of production `SetOnce::set`, including exact return of a rejected
  value and preservation of the first value.
- production-shaped `set` and `get` refinement functions identify the successful
  Release publication and observing Acquire checks as their linearization
  points, including both the optimistic and locked rejection paths;
- publication is proved to precede the notification phase, and a production
  test registers a panicking waker to establish that notification unwind leaves
  the value published and rejects later writers;
- production `SetOnceWriteGuard` now owns the actual `NotifyGuard` and encloses
  the second check, value write, Release publication, and waiter notification;
- `NotifyMutexModel` uses vstd's verified concurrent lock and linear
  `WriteHandle`; acquiring produces a non-cloneable writer permission and
  notification consumes it to return the protected state;
- `GuardedSetOnce<T>` proves the exact
  optimistic-check, lock, second-check, publication, and notification shape;
- a three-writer loom model checks that exactly one writer succeeds and that
  the published value identifies that writer.
- constructors and `new_with` preserve the exact initialized/uninitialized
  state in the proof kernel; a constructor-equivalence proof and production
  test connect the runtime and const forms at the value/state level;
- production `into_inner` and `Drop` share one exclusive `take_inner` path
  which clears the flag before moving the value;
- the proof kernel establishes exact-value first take, empty second take, and
  a well-formed empty representation afterward.
- `WaitProtocol<T>` body-proves publication before registration, publication
  after registration, readiness-only Relaxed recheck, outer Acquire get, and
  cancellation followed by a fresh wait;
- `NotifiedCore` body-proves the production `Init`, registering, `Waiting`,
  `Done`, and dropped states; main and guarded broadcast-list membership;
  generation rechecks; notification consumption; waker replacement; and
  cancellation forwarding of an unconsumed notify-one permit;
- the detailed `NotifiedCore` state machine refines the smaller waiter-state
  projection used by the SetOnce wait proof;
- `IntrusiveWaiters` proves global node conservation and disjoint ownership
  across detached, main, and guarded broadcast lists, including
  unlink-before-notification; its broadcast transfers are coupled to
  `NotifiedCore` in a refinement caller;
- production `poll_waiter` isolates the agreed Pin/Poll/Context/Waker and
  `Notified::poll` trusted surface from the SetOnce-specific wait loop;
- a loom test polls a wait to Pending, cancels it, publishes, and successfully
  waits again, exercising waiter unlinking and re-registration.
- another loom test registers two wait futures before publication and checks
  that both return the exact value after `notify_waiters`;
- generic `Clone` and `PartialEq` lifting through SetOnce is body-proved once
  the underlying standard trait call supplies its value-level contract; the
  `SetOnce::Debug` empty/published selection and `SetOnceError`
  Display/Debug/Error-source surfaces are body-proved while actual generic
  formatting remains the standard trait boundary; a concrete `u64` caller
  discharges those boundaries;
- production compile checks establish the positive and negative `Send`/`Sync`
  bounds, and runtime tests cover Default, Clone, Eq, Debug, Display, and Error;
- production unwind tests check that a panicking payload destructor runs once
  both through `SetOnce::drop` and after ownership transfer by `into_inner`;
- `NotificationUnwind<T>` proves the explicit publication-before-waking,
  successful-wake, and exceptional-wake transitions preserve the exact value;
- production panic tests additionally preserve the source across panicking
  Clone and both operands across panicking PartialEq, and exercise a wake-all
  panic between two other waiters whose nodes remain safely detached and ready;
- five deliberate mutations are rejected by the targeted tests, including
  weakened publication orderings and removal of the lost-wakeup recheck.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| abstract SetOnce states and value view | yes | spec | no | yes |
| construction and empty observation | yes | yes | no | yes |
| publish-once transition | yes | yes | no | yes |
| value-returning observation | yes | yes | no | yes |
| take/consuming transitions | yes | yes | no | yes |
| representative callers | yes | yes | no | yes |
| `PublishedCell<T>` publish/get/take bodies | yes | yes | no | yes |
| production `SetOnceValue` / loom-cell proof view | yes | operations yes | std UnsafeCell semantics | yes |
| Acquire/Release flag/token publication kernel | yes | protocol yes | raw atomic bridge | yes |
| writer-lease double-check/set body | yes | yes | loom Mutex representation adapter | Verus/loom |
| NotifyGuard permission lifecycle | yes | yes via vstd WriteHandle | loom Mutex representation adapter | Verus/loom |
| SetOnce set/get linearization refinement | yes | yes | raw atomic memory-model bridge | Verus/tests |
| constructors/new_with/const state equivalence | yes | yes | instrumentation | Verus/tests |
| owned take/into_inner/Drop protocol | yes | yes | arbitrary destructor semantics | Verus/tests/loom |
| notification exceptional-state protocol | yes | yes | Waker panic execution | Verus/tests |
| production Release/Acquire `set` -> `get` | yes | wrapper/protocol | raw atomic/representation adapters | Verus/loom |
| production `sync::SetOnce<T>` orchestration | yes | refinement bodies | representation adapters | Verus/tests/loom |
| SetOnce wait/lost-wakeup protocol | yes | yes | poll surface | Verus/loom |
| `Notified` core and global list ownership | yes | yes | raw pointer/waker adapter | Verus/loom |
| production `Notified::poll`/intrusive links | partial | state/membership yes | Pin/raw-link/waker adapter | Verus/loom |
| generic Clone/Eq state lifting | yes | yes | std trait value contract | Verus/tests |
| Send/Sync bounds | yes | Rust type system | unsafe impl justification | compile tests |
| Debug/Display/Error views | yes | state/error orchestration yes | generic formatting std traits | Verus/tests |
| mutation sensitivity | yes | n/a | no | five rejected mutations |

The verification crate itself contains no `external_body` or `assume` on
`PublishedCell::{publish,get,take}`, `PublishedOnce::get`, or the writer-lease
set body. Its local vstd-style atomic extension deliberately places
`external_body` only on the raw permissioned atomic operations and owned epoch
reset; their executable bodies spell the exact Rust orderings. The persistent
token state machine, atomic invariant, Release publication, Acquire token
extraction, and PublishedOnce composition are body-proved. Like other vstd
physical-memory proofs, this still trusts the primitive bridge to Rust's memory
model and vstd's `PCell`. It is not a complete formal proof of Tokio's
production representation or the Rust memory model.

The frozen scoped-completion definition is: all SetOnce-specific state
machines, value/ownership transitions, ordering selection, writer exclusion,
wait/list membership, cancellation, and unwind-state logic are proved and the
production paths are covered by the integrated tests. The remaining interfaces
are foundational adapters: standard `UnsafeCell` semantics, the raw weak-memory
atomic bridge, correspondence between Tokio's loom Mutex and the verified lock,
Pin/raw intrusive links/Waker execution, and arbitrary user destructor or trait
semantics. Removing them is library, language-model, or toolchain work rather
than additional SetOnce protocol reasoning. This definition is fixed so later
status reports do not silently move the completion threshold.

## Production boundaries and removal conditions

Production `SetOnce<T>` uses Tokio's loom-compatible `AtomicBool`,
`UnsafeCell<MaybeUninit<T>>`, and the waiter-list lock inside `Notify`.
`NotifyGuard` serializes writers, while a Release store and Acquire loads
publish the initialized value to readers. Those implementation facts are not
translated as one production body, but are connected to the abstract model by
the reviewed operation-specific adapters and refinement proofs.

Three foundational components remain before a no-local-trust end-to-end claim:

1. direct verification that Tokio's loom Mutex representation refines vstd's
   verified lock; guard uniqueness and consuming release are already
   body-proved with `WriteHandle`;
2. upstreaming or independently validating the local vstd-style
   Acquire/Release raw atomic bridge; its ghost implementation and
   SetOnce-specific composition are now connected and body-proved;
3. a permission-carrying vstd contract for standard `UnsafeCell`; production
   now isolates this behind operation-specific `SetOnceValue<T>` methods.

`TokioLoomCell<T>` is the explicit proof view of production
`SetOnceValue<T>`. Its write, shared get, and take operations
are body-proved; only correspondence between the production wrapper and the
vstd physical cell remains trusted. Tokio's non-loom wrapper is now explicitly
transparent, and the integrated run checks the erased layout. This is stronger
than a disconnected value model, but it is not yet a direct Verus translation
of Tokio's private wrapper body.

A later direct-translation probe reached the exact remaining toolchain
interfaces: vstd lacks specifications for `std::cell::UnsafeCell::{new,get}`;
after temporary opaque specifications were supplied, generic `FnOnce` callback
preconditions remained unavailable. Those opaque assumptions were not retained.
The probe record under `verification-probes/loom-cell` gives the removal
conditions; the body-proved PCell adapter and direct production representation
test remain the strongest assumption-free connection available here.

The earlier reference-lifetime blocker is removed: keeping the points-to
permission inside `PublishedCell<T>` lets its body-proved `get` return a
reference with the lifetime of `&PublishedCell<T>`. The remaining difficulty
of matching Tokio's loom wrapper is isolated from publication ordering.
`ReleaseAcquireFlag<T>` now transfers persistent value knowledge through an
atomic invariant while its executable primitive calls use the exact weaker
orderings. The SetOnce protocol above that primitive is body-proved; the raw
operation specifications remain the trusted memory-model bridge.

The `verification-probes/weak-publication` feasibility record captures the
earlier design alternatives. The selected implementation separates immutable
cell ownership from persistent publication knowledge: Acquire returns a
clonable exact-value token, then `PublishedOnce::get` uses its stable cell
invariant to justify the lifetime-bound reference. The local raw atomic methods
are the intentionally small `external_body` surface needed because current
vstd atomics do not expose these operation-specific orderings.

The production `SetOnceWriteGuard` is the concrete counterpart of the verified
`WriterLease<T>` and `NotifyGuardPermission`. Permission issuance, consumption,
the second check, and returning the lock on both success and rejection are
body-proved using vstd's concurrent lock rather than a sequential held-bit
model. The remaining trusted fact is narrowed to Tokio's loom Mutex refining
that verified lock representation. The production loom tests
exercise two- and three-writer races.

The production loom test `set_once_get_publication_test` deliberately reads via
`SetOnce::get` before joining the writer. This makes the production Release
store and Acquire load the publication edge under test. The SetOnce-specific
wait protocol and cancellation state preservation are now proved. As agreed,
Pin/Poll/Context/Waker execution, raw intrusive-link validity, and wake delivery
remain explicit poll-surface boundaries exercised by loom. Global node
ownership across both lists, plus the underlying `Notified` state, generation,
membership, notification, and cancellation transitions, are body-proved.
Arbitrary user destructor semantics and the `Send`/`Sync` implementations remain
language/runtime boundaries. The ownership transitions used by production
`Drop` and `into_inner` are proved and their panicking destructor paths are
tested. Notification unwind is modeled explicitly: publication precedes waking,
and a panicking production waker cannot roll the initialized state back or leave
the remaining waiter nodes linked. Clone and equality panics are also checked to
preserve their source cells.

## Oneshot scoped verification

The oneshot proof is split along the production implementation's ownership
boundaries rather than translating the whole async body as one function:

- `oneshot_state` proves the exact four-bit state representation and all
  send-versus-close outcomes;
- `oneshot_value` proves `Option<T>` slot ownership, exact-value send/take,
  rejection, close, and cleanup transitions with `PCell` permissions;
- `oneshot_atomic` exposes only the production orderings—Relaxed observation,
  Acquire load, and AcqRel read-modify-write—as a single tracked-permission
  sequential bridge. It does not itself establish a concurrent atomic
  invariant;
- `oneshot_poll` proves receiver initial observation, Waker registration,
  mandatory completion recheck, stale-Waker replacement, and ready/pending
  ownership results;
- `oneshot_closed` proves the dual sender `poll_closed` protocol and tx-Waker
  replacement;
- `oneshot_drop` proves conservation of the two `Arc`-like endpoint owners,
  unique final cleanup, and exactly-once payload destruction at the model
  level;
- `oneshot_refinement` integrates the exact four production bits, monotonic
  reachable weak-CAS snapshots for the unique active Sender, task-bit/Waker
  state, the outer endpoint options and
  local send Arc, and the physical proof-view slot. Its split
  store/close-or-drop/complete paths prove the principal races where CLOSED is
  set after the payload store but before completion, including exact take-back
  and final local-Arc cleanup. It also proves repeated terminal try/close and
  TX-Waker retention until terminal Receiver cleanup;
- production `OneshotValue<T>` confines loom `UnsafeCell<Option<T>>` access to
  empty construction, sender-only store, state-authorized take, and
  publication-authorized observation; layout and operation regressions check
  the wrapper representation, but do not close its permission correspondence.

The production-shaped channel protocol and its staged physical proof slot are
proved above frozen raw atomic/UnsafeCell semantics and the Arc/Waker
boundaries. In particular, the refinement proves the state capability that
authorizes each slot operation. Connecting it to the permission inside
production's shared-`&self` loom UnsafeCell is still an unclosed Tokio-specific
representation refinement—not part of the frozen raw UnsafeCell adapter—and
there is not yet a concurrent atomic invariant spanning task-bit/Waker
transition-in-progress states. S02 therefore remains partial. Repeated terminal
`try_recv` and terminal `close` are proved; Future repoll after Ready and its
panic preservation are not. Other remaining Tokio-specific obligations are
`coop::poll_proceed`/`trace_leaf`; `blocking_recv` through the `rt`
runtime-context/BlockingRegion and non-`rt` CachedParkThread paths owned by the
runtime/park rows, not T01; unstable tracing-panic preservation; and public
error/Debug/Display/Error/Clone paths. The `oneshot` and
`oneshot_value` declarations have the identical
`cfg(any(feature = "rt", all(windows, feature = "process")))` predicate as
source-equivalence evidence, but were not cross-compiled for Windows locally.

The production tests cover send/receive, close, polling and Waker replacement,
endpoint drops, and panicking payload destructors on both receiver cleanup and
rejected send. The existing `async_send_sync` target compiles the positive and
negative `Send`/`Sync`/`Unpin` assertions for `Sender`, `Receiver`, and
`Sender::closed`. [`ONESHOT-MUTATION-AUDIT.md`](ONESHOT-MUTATION-AUDIT.md)
records three independently rejected production mutations covering publication
ordering, the lost-wakeup recheck, and Waker replacement.

| Oneshot component | Contract reviewed | Body proved | Trusted boundary | Integrated run |
|---|---:|---:|---:|---:|
| state bits and send/close races | yes | yes | no | Verus/loom |
| payload slot ownership | yes | yes | physical cell adapter | Verus/tests/loom |
| Acquire/AcqRel atomic protocol | yes | sequential permission bridge | raw atomic bridge | Verus/loom |
| receiver poll/recheck/Waker state | yes | yes | poll/Waker execution | Verus/loom |
| sender close poll/Waker state | yes | yes | poll/Waker execution | Verus/loom |
| endpoint ownership/final cleanup | yes | yes | Arc/destructor execution | Verus/tests/loom |
| production `OneshotValue<T>` operations | yes | integrated proof-view capability/operations | shared-`&self` loom UnsafeCell correspondence | Verus/tests/loom |
| Send/Sync/Unpin bounds | yes | Rust type system | unsafe impl justification | compile target |
| mutation sensitivity | yes | n/a | no | three rejected mutations |

## T01 cooperative budget and yield slice

`verification/src/coop_refinement.rs` is a production-shaped sequential
refinement of the cooperative budget value and yield state. Its executable
bodies prove the exact `Option<u8>` cases represented as constrained versus
unconstrained budget, including successful decrement through one-to-zero,
Pending only on a later zero-budget poll, and no decrement in the
unconstrained case. A successful poll returns a linear restore ticket holding
the pre-decrement value. Dropping an uncommitted ticket restores that exact
value; `made_progress` replaces it with the unconstrained sentinel so drop
commits the consumed credit.

`vstd_ext::thread_local` supplies the narrow trusted correspondence for generic
standard-library TLS/Cell mechanics: same-thread owned state, one dynamic
`try_with` extent, success closure once versus teardown closure zero, and Cell
get/set within that extent. Outer callback/fallback totality and scoped LIFO
binding are ordinary body-proved Verus code in the same module, not trusted
premises.
`coop_tls_refinement.rs` body-proves Tokio's `ThreadLocalBudgetCell` mapping
above it. It covers successful and teardown-failed
`try_with`, closure cardinality, arbitrary constrained budgets, inaccessible
`with_budget` fallback, and exact LIFO ResetGuard restoration on normal and
unwind exit. Its standard broadcast composition follows production order:
budget gate first, then trace, then the recv core only when budget permits.

`defer_refinement.rs` body-proves the Tokio-specific
`SchedulerContextAccess` mapping for `with_scheduler` selection and nonescaping
same-thread scoped scheduler references. Defer queue behavior is not trusted:
its bodies prove adjacent-only deduplication, exact one-entry push, queue
preservation when Waker clone or allocation fails, pop before arbitrary Waker
execution, and release of every remaining queued token on Drop. The top-level
broadcast recv composition consumes `NeedsRegistration` before returning forced
Pending and exposes either the immediate wake call or the selected scheduler
queue effect. Focused production tests cover deduplication, clone panic,
Waker-panic pop ordering, queued clone release, and cooperative broadcast recv.

The `YieldNow` refinement preserves the source order and captured poll_fn state:
`trace_leaf` is checked
before the yielded-state recheck; only the first trace-ready poll changes the
state, defers the Waker, and returns Pending; only a later trace-ready poll
returns Ready. The existing public outside-runtime test checks Pending+woken
then Ready. Existing upstream loom cases for both current-thread and
multi-thread schedulers check that deferred yield parks before same-thread
rescheduling; both exact cases are part of `verify-all.bash`.

`ConsumeBudgetFuture` separately proves the captured status recheck, trace
Pending preservation, forced-Pending registration, and exactly one committed
budget decrement. `UnconstrainedFuture` proves pin identity, exact inner Poll
mapping, and restoration of the caller budget on Pending, Ready, and unwind.
`ForcedYieldMetric` proves zero-hit/cfg/current-context call cardinality under
the explicit finite-execution premise below `u64::MAX`; atomic metric storage
remains the common atomic boundary. Focused runtime tests exercise all three
scheduler flavors, wrapper unwind, and the unstable metric result.

T01 is C/R/I for the standard `rt` scope. Pin/Poll/Context/Waker mechanics and
scheduler liveness remain frozen. Only the generic `vstd_ext::thread_local`
correspondence is trusted; `ThreadLocalBudgetCell` and
`SchedulerContextAccess` are body-proved mappings rather than TLS assumptions.
Taskdump-gated consumer projections are owned by R08 rather than T01. Scheduler
liveness is not established by these safety proofs.
[`COOP-MUTATION-AUDIT.md`](COOP-MUTATION-AUDIT.md)
records the rejected counterexamples and exact scope. The integrated Tokio
1.52.3 Verus crate currently reaches `994 verified, 0 errors`.

| T01 component | Contract reviewed | Body proved | Trusted boundary | Integrated run |
|---|---:|---:|---:|---:|
| budget decrement and forced-Pending branch | yes | yes | generic TLS/Cell semantics; Waker mechanics | Verus/tests |
| restore ticket commit/rollback | yes | yes | none | Verus/tests |
| unconstrained and initial-budget nesting | yes | yes | none | Verus/tests |
| unwind reset state transition | yes | yes | arbitrary Drop execution | Verus/test |
| scheduler selection and scoped binding | yes | yes, including TLS route and LIFO scope | generic TLS/Cell semantics | Verus/tests |
| Defer queue ownership and panic cleanup | yes | yes | Waker execution; allocation; scheduler liveness | Verus/tests/loom |
| yield trace/defer/recheck state | yes | yes | Pin/Poll/Context/Waker mechanics; scheduler liveness | Verus/test/loom |
| consume_budget poll_fn capture | yes | yes | Pin/Poll/Context/Waker mechanics | Verus/tests |
| Unconstrained pinned poll and unwind | yes | yes | arbitrary inner Future/Poll execution | Verus/tests |
| forced-yield metric consumer mapping | yes | yes | atomics; finite execution below `u64::MAX` | Verus/test |

## oneshot polling connection probe

`verification-probes/oneshot-poll` preserves the production-shaped
`Future::poll` signature for `oneshot::Receiver<T>`. The pinned Verus accepts
that signature only after `Pin`, `Poll`, `Context`, and `Waker`, plus the poll
body, are declared as explicit external boundaries. Its expected result is
`0 verified, 0 errors`; this is a translation connection probe, not a body
proof. The probe README records the exact exclusions and removal requirements.

## Watch, broadcast, and mpsc expansion

The follow-on channel work now contributes to an integrated Verus result of
`886 verified, 0 errors`. It reuses the publication, ownership, register/recheck, endpoint-count,
and cleanup patterns established for SetOnce and oneshot. Watch adds independent
Receiver generations; broadcast adds bounded ring generations and lag recovery;
mpsc adds linear permits and FIFO claim-versus-publication ordering. Its new
production receive refinement additionally proves the exact `Chan::recv` and
`recv_many` first-pop/register/recheck branch priorities, receiver-close with a
reserved permit through both send and cancel continuations, latest-Waker
preservation, bounded single/batch permit restoration, logical-oracle
observation-order batch return, unbounded encoded message-count arithmetic,
and the post-gate zero-limit branch. The bounded and unbounded public poll
surfaces are exercised by module-local production tests; those tests do not
distinguish the internal first-pop and post-registration-pop paths.

Broadcast now uses the reviewed all-build terminal position policy proved in
`broadcast_refinement.rs`. Successful sends reserve through `u64::MAX - 1`;
`tail == MAX` rejects an active-Receiver send before channel mutation and never
reuses zero, while the no-Receiver `SendError` retains priority. The refinement
body-proves the terminal gate, monotonic send/receive/lag cursors, exact 64-bit
distance with 32-bit saturation, direct emptiness, a capacity-two reachable
terminal queried-slot witness, retained drain conservation, and exclusion of
post-snapshot concurrent sends from Receiver Drop. Module-local tests cover all
terminal branches, and an exact loom case connects the concurrent Drop/send
race. Later S04 modules cover generic mask-generation equivalence, direct
physical slot/rem/value coupling, waiter/slot lock orchestration, weak endpoint
lifecycle, public trait/error surfaces, unwind cleanup, and standard
cooperative recv/defer. `blocking_recv_refinement.rs` additionally body-proves
the shared-CONTEXT blocking-region and budget-teardown routes, distinct
CURRENT_PARKER access, cfg(rt)/cfg(no-rt) selection, exact poll/park
cardinality for arbitrary finite Pending repetitions, and terminal result
mapping. Parker/Waker mechanics and scheduling liveness remain frozen. S04 is
C / R / I under its recorded scope. The unstable taskdump-gated `trace_leaf`
projection, including Pending and panic ordering at this consumer, is owned by
R08 rather than being an S04 residual, as recorded in
`BROADCAST-WRAP-AUDIT.md`.

The `recv_many` refinement now also carries an immutable caller prefix and an
explicit logical buffer capacity. With
`initial_prefix.len() + limit <= buffer_capacity`, it proves each Value append
stays inside that capacity, preserves the prefix, reports exactly the appended
count, and performs one exact bounded bulk permit return or one unbounded
`number_added << 1` decrement under the explicit
`number_added * 2 <= usize::MAX` bound, including shift/multiplication
equivalence. A consumed `accounting_applied` state prevents a completed batch
from applying either accounting transition twice. The recv-many-specific
environment reflects production's nonempty Closed order: observe TX_CLOSED,
bulk-return the appended values, then establish semaphore idle. Witnesses cover
limit completion, nonempty
Empty/Closed completion, and the zero-value terminal priorities. Two
module-local tests assert physical `capacity() - len() >= limit` before each
bounded/unbounded call and confirm capacity does not change. The current vstd
Vec contract does not connect physical capacity to the model's logical field;
the proof is conditional. A later zero-sized-Vec capacity-overflow probe
confirmed that production could remove a value and then unwind before applying
its deferred bounded or unbounded batch accounting. The selected repair
installs a post-pop guard while preserving normal bulk accounting and allocation
timing. Permanent safe-API regressions confirm bounded capacity restoration and
removal of the unbounded phantom count. A linear Verus guard model and separate
bounded/unbounded witnesses prove once-only consumption of a recorded pending
count; raw pop, Vec unwind, and compiled Drop execution are source/test
connections rather than direct Verus refinement.
`MPSC-RECV-MANY-UNWIND-AUDIT.md` records the reproducer, selected contract, and
remaining foundational allocation/destructor boundary.

The mpsc endpoint refinement additionally matches `tx_count`, `tx_weak_count`,
and Arc owners through live, count-before-Arc gap, upgrade, and retiring phases.
The mpsc-scoped `MpscFiniteExecutionResources` bounds simultaneous construction
gaps by live threads and completed Arc owners by the physical execution
environment; it replaces the former finite-prefix counter-room assumption.
Strong ownership is split into Sender and OwnedPermit kinds, and weak upgrade
cannot resurrect a zero strong count. The final strong decrement composes with
the raw queue's unique close claim/publication and one receiver wake. Arc
implementation details and Waker execution remain frozen foundations.

The mpsc `try_recv` refinement proves initial terminal priorities and the
Busy control order: wake the displaced poll Waker once, construct the park
Waker, register before every recheck, and park once only after another Busy.
Value exit composes with one bounded capacity return or one unbounded encoded
count decrement; receiver-close without semaphore idle remains Empty. The
two-Busy witness directly uses the proved S09 AtomicWaker machine; the generic
register transition is abstract protocol bookkeeping, not a direct S09
composition. Two production tests cover public bounded/unbounded Value, Empty,
Disconnected, and permit behavior,
but do not force a raw-list Busy interleaving. The added raw queue model derives
Busy from a claimed, unpublished FIFO head even when a later slot is ready. A
single `nat` rank proves register/park safety and termination for every finite
Busy schedule supplied by the scheduler-liveness foundation. CachedParkThread
construction and Waker/park execution remain frozen generic adapters.

These models are intentionally split at production's foundational interfaces.
They do not claim direct verification of RwLock/Mutex or the foundational
AtomicWaker machinery,
raw intrusive links, arbitrary Clone/destructor execution, or raw mpsc block
allocation and pointer validity. Wrapping state behavior, logical waiter
membership, arbitrary slot ownership, and the all-consumed reclamation
condition are proved above those adapters. Watch now additionally refines
BigNotify's eight-shard loop, circular selector arithmetic, per-shard call
generations, registration snapshots, and repeated fanout; Notify's former
checked-overflow/full-cycle boundary is closed by mutation-free terminal
rejection before generation zero can be reused.
The exact transitions, tests,
exclusions, and removal conditions are recorded in
[`CHANNEL-VERIFICATION.md`](CHANNEL-VERIFICATION.md).

For mpsc specifically, the new raw queue model derives Value, Closed, Busy, and
Empty from claimed slots, ready bits, the close marker, and the receiver cursor;
linear `RawValueSlot<T>` permissions move payloads exactly once. A full receive
gate snapshot frames queue, buffer, capacity/count, and Waker state across trace
and cooperative gates. `CompiledRecvMany` connects raw pop, guard recording,
successful Vec append, normal bulk return, and unwind-time return without a
spare-capacity premise. Thin orchestration composes those leaves with bounded
capacity, last-strong close/wake, try-recv, and public result mappings. Raw
allocation/pointer validity, Vec allocation, arbitrary destructor execution,
park/Waker mechanics, and scheduler liveness remain frozen foundations.

S05 remains **L / R(partial) / I** for exactly two production index-boundary
decisions. `Block::grow` is fixed with `wrapping_add(BLOCK_CAP)` and an exact
final-generation regression. `Block::has_value` remains unchanged: its upper
bound can overflow-panic in debug and wrap to a false classification in release.
`Rx::reclaim_blocks` also remains unchanged: ordinary
`required_index > self.index` does not implement the proved traversal lease at
the machine-word boundary. Non-mutating fixtures record both residuals; see
[`MPSC-WRAP-AUDIT.md`](MPSC-WRAP-AUDIT.md).

## Reproduction

Run `./verify-all.bash` in this directory. It checks only tokio 1.52.3 and:

1. runs the exact SetOnce, OnceCell, oneshot, watch, broadcast, mpsc, mpsc-weak,
   Barrier, and cooperative wrapper/metric integration targets;
2. runs the full SetOnce, OnceCell, and oneshot loom modules plus bounded exact
   loom cases for watch, broadcast, mpsc, Semaphore, Notify, AtomicWaker, and
   cooperative scheduler yield;
3. compiles Tokio's existing async Send/Sync/Unpin assertion target;
4. checks the pinned Verus version;
5. verifies 946 nested SetOnce, OnceCell, publication, channel, Semaphore,
   Notify, Barrier, and AtomicWaker model/refinement bodies with the locked vstd
   revision;
6. checks the erased loom-cell proof-view layout;
7. runs the oneshot polling connection probe.

The integrated SetOnce target currently contains 24 ordinary tests and eight
loom model tests. OnceCell contributes 18 ordinary tests and three loom model
tests. The oneshot target contains 24 ordinary tests and eight loom
model tests. The expansion adds 22 watch tests, 32 broadcast tests, 100 mpsc
tests, 28 mpsc weak-Sender tests, and ten bounded exact loom cases.
The Barrier target contributes seven public tests plus four focused unit
regressions for reachable terminal cohort admission/rejection and defensive
generation arithmetic.
[`MUTATION-AUDIT.md`](MUTATION-AUDIT.md) and
[`ONESHOT-MUTATION-AUDIT.md`](ONESHOT-MUTATION-AUDIT.md) record separate
destructive-copy audits; mutations are intentionally not rerun by
`verify-all.bash`.

The channel expansion mutation records are
[`WATCH-MUTATION-AUDIT.md`](WATCH-MUTATION-AUDIT.md),
[`BROADCAST-MUTATION-AUDIT.md`](BROADCAST-MUTATION-AUDIT.md), and
[`MPSC-MUTATION-AUDIT.md`](MPSC-MUTATION-AUDIT.md).

The pinned Verus version is `0.2026.07.27.31579f0`; vstd is pinned to commit
`31579f0b8542a8a9ae4ae5604c16107ccde23ef2`. Generated Cargo and Verus build
artifacts are intentionally not tracked.
