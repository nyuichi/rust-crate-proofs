# Acquire/Release publication feasibility result (implemented follow-up)

This spike evaluated whether the pinned Verus/vstd revision can remove the
remaining weak-memory boundary for production `SetOnce::set` and `SetOnce::get`
without adding an opaque assumption.

## Attempt 1: keep the PCell permission in an atomic invariant

The intended design stores the initialized `PCell` points-to permission in an
atomic invariant on Release and retrieves enough permission on Acquire to
justify `get() -> &T`.

This cannot implement the production API with the current invariant rules:

- vstd atomics execute only `SeqCst` operations;
- an atomic invariant may contain only one atomic executable operation while it
  is open; reading through `PCell` is explicitly not atomic;
- the invariant must close at the end of the atomic operation, but the returned
  reference must remain valid for the lifetime of `&SetOnce<T>`.

Consequently a `PCell::borrow` tied to the invariant-held permission cannot
escape as the production reference.

## Attempt 2: derive a persistent read capability

vstd's verified `RwLock` demonstrates the available pattern: an atomic state
machine produces a `ReadHandle`, and the returned reference is tied to a borrow
of that handle. SetOnce cannot expose such a handle without changing its public
`get() -> Option<&T>` API.

Persistent state-machine tokens can preserve duplicable knowledge of the
published value, but they do not provide the `PCell::PointsTo<T>` permission
required to construct a physical reference. Leaking a read handle is also not
valid: `into_inner` and `Drop` must later regain exclusive ownership, relying on
Rust lifetimes to establish that no returned reference remains live.

## Result and selected implementation

The SetOnce-specific Release/Acquire protocol is proved and production
orderings are mutation-tested with loom. The first architectural choice below
was selected:

1. extend vstd with ordering-aware atomics plus an immutable-after-publication
   cell whose shared borrow is tied directly to `&self` after an Acquire token;
2. verify an API that returns an explicit read guard, which would not be the
   Tokio SetOnce API;
3. connect a separate weak-memory verifier and treat its theorem as the
   implementation of the existing Verus publication interface.

`verification/src/release_acquire.rs` now supplies that local vstd-style
extension. Its tokenized state machine and atomic-invariant composition are
body-proved. Release consumes the unique writer token; an Acquire load that
observes publication returns persistent exact-value knowledge; a Relaxed load
returns readiness only. `PublishedOnce<T>` uses that token to justify its
lifetime-bound immutable cell read without changing Tokio's API.

The raw permissioned atomic methods and owned epoch reset remain a deliberately
small trusted bridge to Rust's weak-memory semantics. Their executable bodies
use the exact `Acquire`, `Release`, and `Relaxed` operations. Thus this follow-up
removes the earlier SetOnce-level SC refinement assumption, but it does not
claim to prove Rust's atomic memory model inside Verus. Full removal requires
upstream vstd support or independent validation of those raw contracts.
