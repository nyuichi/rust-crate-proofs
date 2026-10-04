# Runtime-dispatch atomic probe

This probe separates the actual global-static blocker from two atomic-call
questions. The operation-level `model-dispatch::get_runtime_feature` caller
has passed its 15 Why3 VCs. That result does not close static initialization
or concurrent invariant access.

`src/lib.rs` has three relevant modes:

- `static-cache` keeps the `AtomicU8` statics and actual load/detect/store body.
  Creusot stops at `DefKind::Static` (`ItemType::Unsupported`).
- `parameterized-cache` keeps the raw standard atomic type and call sequence,
  replacing CPU detection with a parameter. The raw atomic calls have no
  history contract, so this is not a proof route.
- `model-dispatch` calls the existing Creusot `AtomicU8` wrapper and carries
  its `Perm`/`Committer` history resource. The `cpu_avx2` and `cpu_sse42`
  parameters are fixed CPU capability facts; history entries must be `0`, NOP,
  AVX2 when AVX2 is supported, or SSE4.2 when SSE4.2 is supported. The function
  promises a supported result and preserves that history invariant. The
  generated Coma retains the AVX2/SSE4.2/NOP priority branches, the cache
  zero-test, and existing relaxed atomic contracts. This is a parameterized
  single-access caller model with an explicit mutable history permission; it
  does not model overlapping calls through a persistent `AtomicInvariant`,
  connect a static initializer to one history resource, or check invariant
  reentrancy. The module is public so Creusot emits each caller as its own Coma
  file for exact positive and negative selection. The `negative-store` feature
  adds a caller with the same preconditions that writes `4` while promising
  the same invariant. Its postcondition remains unknown: one split VC child
  timed out under Z3 at 10, 10, and 20 seconds under Why3find retry, so this
  is not a counterexample or a proved invalid-store result.

Run the tracked wrapper probe from this directory:

```sh
source /workspace/proof-tools/activate.sh
timeout 600s /workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash \
  cargo creusot prove \
  verif/httparse_static_atomic_probe_rlib/model_dispatch/get_runtime_feature.coma \
  -- --features model-dispatch --offline

timeout 120s /workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash \
  cargo creusot prove \
  verif/httparse_static_atomic_probe_rlib/model_dispatch/malformed_store.coma \
  -- --features 'model-dispatch,negative-store' --offline
```

The negative attempt writes its Why3 result to
`verif/httparse_static_atomic_probe_rlib/model_dispatch/malformed_store/proof.json`.
An unresolved child is recorded as `null`; it must not be described as a
counterexample. Keep positive and negative runs separate so the expected
negative status does not obscure the positive proof result.

The raw-atomic extension experiment lives in
`/workspace/scratch/httparse-static-probe`. It uses an isolated copy of
`creusot-libs`, adds `std::sync::raw_atomic::AtomicU8HistoryExt` with the
existing `Container`/`HasTimestamp` and `Committer` history contracts, and
translates the parameterized actual dispatch body. The local raw `new/load/store`
helpers have `#[erasure(AtomicU8::new/load/store)]` checks. After installing
`rust-src` and building std, this command passes in error mode:

```sh
source /workspace/proof-tools/activate.sh
CARGO_NET_OFFLINE=true cargo creusot --erasure-check=error -- -Z build-std=std
```

The helper-to-standard-operation erasure checks pass. Creusot warns that the
standard atomic calls have no ordinary function contract. The extension methods
that attach the history contracts are trusted, so Creusot deliberately skips
their body erasure check. Their runtime bodies directly invoke the corresponding
standard atomic operations, but the history semantics are an added standard
atomic TCB contract, just as they are for the existing wrapper. This experiment
does not yet prove that trusted history contract against Rust's atomic
implementation, and it does not verify static initialization.

## Work still required for generic static state

Do not route `Static` through the current `Constant` expansion. A distinct
translation must give each static one stable identity, evaluate its initializer
once, and make one persistent ghost resource available at all accesses. The
resource must retain the existing atomic history across loads/stores, reject
stores that break the feature predicate, allow a post-store load to observe the
stored value, prevent minting a second exclusive permission for the same
static, and reject reentrant opening of its invariant. The initialization path
must establish the initial history from `AtomicU8::new(0)` exactly once without
adding a dispatch-specific trusted postcondition.

The first translator slice would touch `creusot/src/ctx.rs` (`ItemType` and
`item_type`), `creusot/src/backend.rs` (static dispatch), and
`creusot/src/backend/clone_map/elaborator.rs` (a static expansion separate from
`expand_constant`). It also needs the MIR/place/reference path in
`creusot/src/translation/function.rs` and a global resource/invariant entry
point that does not currently exist. `backend/logic.rs` and
`translation/constant.rs` may need a static-symbol case depending on the chosen
representation. Add positive and negative compiler tests for unique identity,
one-time initialization, preserved history, invalid stores, and reentrant
opens. A translator-only symbol experiment is several Rust/Why3 files plus
tests; the correct shared-resource and initializer semantics are a separate,
larger change, estimated at multiple engineering days before full proof and
soundness review. No implementation of that bridge is included here.
