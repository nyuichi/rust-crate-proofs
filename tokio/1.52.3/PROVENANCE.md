# tokio 1.52.3 verification provenance

**Verification status: body-proved SetOnce ownership model; production
concurrency refinement not yet connected.**

This source tree is copied from the `tokio` 1.52.3 package published on
crates.io. The published archive has SHA-256 checksum
`8fc7f01b389ac15039e4dc9531aa973a135d7a4135281b12d7c1bc79fd57fffe`.
Its `.cargo_vcs_info.json` records upstream revision
`d87569164fb61145e79e7ffe0b25783569cc8f93` and path `tokio`.

The complete upstream source and ordinary public API are retained unchanged.
The first Verus milestone is intentionally isolated in the nested
`verification` crate so Verus does not translate unrelated Tokio subsystems.
It defines the value-preserving ownership model that a later refinement proof
must connect to `sync::SetOnce<T>`.

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

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| abstract SetOnce states and value view | yes | spec | no | yes |
| construction and empty observation | yes | yes | no | yes |
| publish-once transition | yes | yes | no | yes |
| value-returning observation | yes | yes | no | yes |
| take/consuming transitions | yes | yes | no | yes |
| representative callers | yes | yes | no | yes |
| production `sync::SetOnce<T>` bodies | partial | no | excluded | no |
| production `wait()` and wake protocol | no | no | excluded | no |

The abstract model contains no `external_body`, `assume`, or trusted transition.
It is not a proof of Tokio's production representation or concurrent execution.

## Production boundaries and removal conditions

Production `SetOnce<T>` uses Tokio's loom-compatible `AtomicBool`,
`UnsafeCell<MaybeUninit<T>>`, and the waiter-list lock inside `Notify`.
`NotifyGuard` serializes writers, while a Release store and Acquire loads
publish the initialized value to readers. Those implementation facts are not
yet connected to the abstract model.

Three proof-library or adapter components are needed to remove this boundary:

1. an exact contract transferring the single writer permission through
   `Notify::lock_waiter_list` and its guard;
2. an Acquire/Release atomic ghost specification relating `value_set` to the
   initialized-slot permission;
3. a persistent read-only publication abstraction that can return `&T` with
   the lifetime of `&SetOnce<T>` after the atomic invariant is closed.

The current vstd `PCell` borrow is tied to the lifetime of its points-to
permission, so it cannot directly justify production `get() -> Option<&T>`
after closing an invariant. `wait()`, cancellation safety, waker registration,
`Drop`, arbitrary destructor behavior, and the `Send`/`Sync` implementations
also remain outside this milestone.

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
2. checks the pinned Verus version;
3. verifies the nested SetOnce model with the locked vstd revision;
4. runs the oneshot polling connection probe.

The pinned Verus version is `0.2026.07.27.31579f0`; vstd is pinned to commit
`31579f0b8542a8a9ae4ae5604c16107ccde23ef2`. Generated Cargo and Verus build
artifacts are intentionally not tracked.
