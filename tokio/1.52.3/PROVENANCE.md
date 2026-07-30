# tokio 1.52.3 verification provenance

**Verification status: body-proved reusable publication slot and SetOnce
publication kernel; production atomic orderings are encapsulated and exercised
by loom, with the weak-memory refinement kept as one explicit trusted boundary.**

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
- `TokioLoomCell<T>` specializes the proof view to Tokio's
  `UnsafeCell<MaybeUninit<T>>` field and body-proves the production-shaped
  unsafe `get_unchecked() -> &T` contract;
- erased size and alignment are checked against
  `UnsafeCell<MaybeUninit<T>>` for zero-sized, scalar, array, and over-aligned
  payloads;
- `PublicationFlag` body-proves its load/store wrapper contracts over vstd's
  sequentially-consistent atomic primitive;
- production `SetOnceFlag` exposes only operation-specific Acquire, Release,
  and Relaxed methods, so SetOnce call sites cannot select arbitrary orderings;
- `PublishedOnce<T>` maintains `flag == slot.is_init()`, and its
  production-shaped `initialized`, `get`, publish, and take bodies preserve
  that relation;
- `WriterLease<T>` body-proves the locked double-check and first-writer-wins
  portion of production `SetOnce::set`, including exact return of a rejected
  value and preservation of the first value.
- production `SetOnceWriteGuard` now owns the actual `NotifyGuard` and encloses
  the second check, value write, Release publication, and waiter notification;
- a three-writer loom model checks that exactly one writer succeeds and that
  the published value identifies that writer.
- constructors and `new_with` preserve the exact initialized/uninitialized
  state in the proof kernel;
- production `into_inner` and `Drop` share one exclusive `take_inner` path
  which clears the flag before moving the value;
- the proof kernel establishes exact-value first take, empty second take, and
  a well-formed empty representation afterward.
- `WaitProtocol<T>` body-proves publication before registration, publication
  after registration, readiness-only Relaxed recheck, outer Acquire get, and
  cancellation followed by a fresh wait;
- production `poll_waiter` isolates the agreed Pin/Poll/Context/Waker and
  `Notified::poll` trusted surface from the SetOnce-specific wait loop;
- a loom test polls a wait to Pending, cancels it, publishes, and successfully
  waits again, exercising waiter unlinking and re-registration.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| abstract SetOnce states and value view | yes | spec | no | yes |
| construction and empty observation | yes | yes | no | yes |
| publish-once transition | yes | yes | no | yes |
| value-returning observation | yes | yes | no | yes |
| take/consuming transitions | yes | yes | no | yes |
| representative callers | yes | yes | no | yes |
| `PublishedCell<T>` publish/get/take bodies | yes | yes | no | yes |
| Tokio loom-cell proof view and `get_unchecked` | yes | yes | representation correspondence | yes |
| SC flag/slot publication kernel | yes | yes | vstd atomic primitive | yes |
| writer-lease double-check/set body | yes | yes | Notify mutex exclusivity | Verus/loom |
| constructors/new_with | yes | yes | const adapter | Verus/tests |
| owned take/into_inner/Drop protocol | yes | yes | destructor semantics | Verus/tests/loom |
| production Release/Acquire `set` -> `get` | yes | wrapper/protocol | weak-memory refinement | Verus/loom |
| production `sync::SetOnce<T>` bodies | partial | no | representation adapter | tests/loom |
| SetOnce wait/lost-wakeup protocol | yes | yes | poll surface | Verus/loom |
| production `Notified::poll`/waiter list | partial | no | agreed adapter | loom |

The verification crate itself contains no `external_body` or `assume` on
`PublishedCell::{publish,get,take}`, `PublishedOnce::get`, or the writer-lease
set body. Like other vstd physical-memory proofs, it relies on vstd's trusted
`PCell` and atomic primitive implementations. It is not a complete formal
proof of Tokio's production representation or the Rust memory model.

## Production boundaries and removal conditions

Production `SetOnce<T>` uses Tokio's loom-compatible `AtomicBool`,
`UnsafeCell<MaybeUninit<T>>`, and the waiter-list lock inside `Notify`.
`NotifyGuard` serializes writers, while a Release store and Acquire loads
publish the initialized value to readers. Those implementation facts are not
yet connected to the abstract model.

Three proof-library or adapter components remain before the production bodies
are formally connected end to end:

1. replacement of the structurally connected `SetOnceWriteGuard` boundary
   with a direct Verus contract for `Notify::lock_waiter_list`; the production
   critical section and first-writer-wins body are already isolated and tested;
2. replacement of the SC proof primitive with a foundational Acquire/Release
   atomic ghost implementation; production operation selection and the
   SetOnce-specific protocol are already connected;
3. replacement of the checked/trusted representation correspondence with a
   direct vstd contract for Tokio's loom `UnsafeCell` wrapper.

Phase 1 introduces `TokioLoomCell<T>` as the explicit proof view of production
`UnsafeCell<MaybeUninit<T>>`. Its write, `get_unchecked`, and take operations
are body-proved; only correspondence between the production wrapper and the
vstd physical cell remains trusted. Tokio's non-loom wrapper is now explicitly
transparent, and the integrated run checks the erased layout. This is stronger
than a disconnected value model, but it is not yet a direct Verus translation
of Tokio's private wrapper body.

The earlier reference-lifetime blocker is removed: keeping the points-to
permission inside `PublishedCell<T>` lets its body-proved `get` return a
reference with the lifetime of `&PublishedCell<T>`. The remaining difficulty
is transferring that permission through a concurrent atomic invariant while
matching Tokio's loom wrapper and exact Acquire/Release operations. Current
vstd atomics expose sequentially-consistent operations only, so substituting
Tokio's weaker but sufficient ordering is not claimed as body-proved.

The production `SetOnceWriteGuard` is the concrete counterpart of the verified
`WriterLease<T>`. Its remaining trusted fact is that Tokio's waiter-list mutex
grants exclusive ownership of that logical lease. The production loom tests
exercise two- and three-writer races.

The production loom test `set_once_get_publication_test` deliberately reads via
`SetOnce::get` before joining the writer. This makes the production Release
store and Acquire load the publication edge under test. The SetOnce-specific
wait protocol and cancellation state preservation are now proved. As agreed,
Pin/Poll/Context/Waker behavior, `Notified::poll`, intrusive waiter unlinking,
and wake delivery remain one explicit poll-surface boundary exercised by loom.
Arbitrary user destructor behavior, unwind, and the `Send`/`Sync`
implementations remain outside the formal milestone. The ownership transition
used by production `Drop` is proved.

## oneshot polling connection probe

`verification-probes/oneshot-poll` preserves the production-shaped
`Future::poll` signature for `oneshot::Receiver<T>`. The pinned Verus accepts
that signature only after `Pin`, `Poll`, `Context`, and `Waker`, plus the poll
body, are declared as explicit external boundaries. Its expected result is
`0 verified, 0 errors`; this is a translation connection probe, not a body
proof. The probe README records the exact exclusions and removal requirements.

## Reproduction

Run `./verify-all.bash` in this directory. It checks only tokio 1.52.3 and:

1. runs the upstream `sync_set_once` integration target with `full` features;
2. runs the five `loom_set_once` model tests with `full,test-util` features;
3. checks the pinned Verus version;
4. verifies the nested SetOnce and publication models with the locked vstd
   revision;
5. checks the erased loom-cell proof-view layout;
6. runs the oneshot polling connection probe.

The pinned Verus version is `0.2026.07.27.31579f0`; vstd is pinned to commit
`31579f0b8542a8a9ae4ae5604c16107ccde23ef2`. Generated Cargo and Verus build
artifacts are intentionally not tracked.
