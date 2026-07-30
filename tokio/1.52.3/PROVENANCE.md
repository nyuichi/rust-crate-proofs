# tokio 1.52.3 verification provenance

**Verification status: body-proved reusable publication slot and SetOnce
publication kernel; production Release/Acquire path exercised by loom, with
the memory-order refinement still an explicit trusted boundary.**

This source tree is copied from the `tokio` 1.52.3 package published on
crates.io. The published archive has SHA-256 checksum
`8fc7f01b389ac15039e4dc9531aa973a135d7a4135281b12d7c1bc79fd57fffe`.
Its `.cargo_vcs_info.json` records upstream revision
`d87569164fb61145e79e7ffe0b25783569cc8f93` and path `tokio`.

The complete upstream source and ordinary public API are retained unchanged.
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
- `PublicationFlag` body-proves its load/store wrapper contracts over vstd's
  sequentially-consistent atomic primitive;
- `PublishedOnce<T>` maintains `flag == slot.is_init()`, and its
  production-shaped `initialized`, `get`, publish, and take bodies preserve
  that relation;
- `WriterLease<T>` body-proves the locked double-check and first-writer-wins
  portion of production `SetOnce::set`, including exact return of a rejected
  value and preservation of the first value.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| abstract SetOnce states and value view | yes | spec | no | yes |
| construction and empty observation | yes | yes | no | yes |
| publish-once transition | yes | yes | no | yes |
| value-returning observation | yes | yes | no | yes |
| take/consuming transitions | yes | yes | no | yes |
| representative callers | yes | yes | no | yes |
| `PublishedCell<T>` publish/get/take bodies | yes | yes | no | yes |
| SC flag/slot publication kernel | yes | yes | vstd atomic primitive | yes |
| writer-lease double-check/set body | yes | yes | no | yes |
| production Release/Acquire `set` -> `get` | yes | no | memory-order refinement | loom |
| production `sync::SetOnce<T>` bodies | partial | no | representation adapter | tests/loom |
| production `wait()` and wake protocol | no | no | excluded | no |

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

1. an exact contract transferring the single writer permission through
   `Notify::lock_waiter_list` and its guard;
2. an Acquire/Release atomic ghost specification relating `value_set` to the
   initialized-slot permission;
3. a representation adapter relating loom's
   `UnsafeCell<MaybeUninit<T>>` to the proved `PublishedCell<T>` permission.

The earlier reference-lifetime blocker is removed: keeping the points-to
permission inside `PublishedCell<T>` lets its body-proved `get` return a
reference with the lifetime of `&PublishedCell<T>`. The remaining difficulty
is transferring that permission through a concurrent atomic invariant while
matching Tokio's loom wrapper and exact Acquire/Release operations. Current
vstd atomics expose sequentially-consistent operations only, so substituting
Tokio's weaker but sufficient ordering is not claimed as body-proved.

The production loom test `set_once_get_publication_test` deliberately reads via
`SetOnce::get` before joining the writer. This makes the production Release
store and Acquire load the publication edge under test. `wait()`, cancellation
safety, waker registration, `Drop`, arbitrary destructor behavior, and the
`Send`/`Sync` implementations remain outside the formal milestone.

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
2. runs the three `loom_set_once` model tests with `full,test-util` features;
3. checks the pinned Verus version;
4. verifies the nested SetOnce and publication models with the locked vstd
   revision;
5. runs the oneshot polling connection probe.

The pinned Verus version is `0.2026.07.27.31579f0`; vstd is pinned to commit
`31579f0b8542a8a9ae4ae5604c16107ccde23ef2`. Generated Cargo and Verus build
artifacts are intentionally not tracked.
