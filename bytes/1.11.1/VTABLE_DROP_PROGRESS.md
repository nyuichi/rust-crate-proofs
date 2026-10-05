# Bytes clone table and destructor frontier (2026-10-05)

The actual `src/bytes.rs` clone callbacks now receive the current vtable as a
fourth private argument. `static_clone` and `owned_clone<T>` preserve that
argument instead of looking up their own containing table. The free
`src/bytes_mut.rs::shared_v_clone` callback likewise preserves its caller's
table. Promotable callbacks continue selecting the shared table as before.
Pointer, length, owner type, reference-count operations and public API are
unchanged. No verification-only implementation or trusted contract was added.

`BYTES_TRANSLATE_ONLY=1 scripts/verify-bytes.sh runtime` now gets past both
former mutual-recursion errors. Its next five diagnostics are the existing
`Bytes` Send/Sync implementations requiring trusted annotations, and the
`Bytes` Deref plus `BytesMut` Deref/DerefMut implementations requiring ghost
purity. This is translation progress, not a successful whole-crate translation
or proof. The actual callback dispatch remains unverified.

Validation logs under `verification/probes/vtable-leaf-integration/logs/`:

- `runtime-passed-vtable.log`: actual crate, five diagnostics above; no proof.
- `native118.log`: `cargo test --locked --test test_bytes`, 118 passed.
- `no-std.log`: `cargo check --locked --no-default-features`, passed.
- `runtime-diagnostic-ghost-deref-first.log`: diagnostic copy under `/tmp`,
  with temporary trusted Send/Sync scaffolding and three ghost Deref
  annotations; fails on calls to actual `Bytes::as_slice`, `BytesMut::as_ref`
  and `BytesMut::as_mut` from ghost context. These diagnostic annotations are
  not present in the repository and establish no thread-safety proof.
- `runtime-diagnostic-ghost-accessors.log`: extending ghost annotations through
  those accessors in the diagnostic copy exposes seven exact purity errors:
  `Bytes` raw `slice::from_raw_parts`, and `BytesMut::kind`, B4
  `borrow_bound`/`borrow_bound_mut`, and shared `borrow_packet`/
  `borrow_packet_mut`. Merely labeling the trait methods ghost is insufficient.
  Adding ghost purity to trusted B4 leaves would change the trusted interface
  and has not been adopted as a fix.
- `runtime-diagnostic-skipped-deref.log`: temporarily also trusting the three
  Deref bodies, solely to inspect downstream translation, next stops on the
  raw-pointer array load in `src/buf/buf_impl.rs`'s `buf_try_get_impl` macro.
  This is a separate, potentially removable source integration issue. Existing
  reduced vtable probes still establish unsupported function-pointer calls;
  they are not yet the first full-crate diagnostic.

`verification/probes/vtable-leaf-integration/prepare-diagnostic.py` reconstructs
the three explicitly unsound-for-proof frontier configurations from current
source in a fresh output directory. Its `sequential-accessors` mode instead
omits the unsafe Send/Sync implementations from the proof configuration and
adds ghost annotations through the accessors, with no new trusted annotation.
This sequential candidate reaches the same seven concrete purity failures
(`runtime-sequential-no-new-trust.log`). It does not establish a working
full-crate sequential configuration or verify concurrency. The printed runtime
wrapper command is translation-only and uses the shared proof lock. Never use
the temporary trusted configurations as proof evidence or copy their
annotations into the actual crate.

The subsequent `buf_try_get_impl` edit in `src/buf/buf_impl.rs` replaces that
raw array load with `copy_from_slice` into a fixed-size array. `get(..SIZE)`
already establishes the exact length; numeric conversion, buffer advancement,
and fragmented-input fallback are unchanged. No trusted annotation was added.
Native `test_buf` (820 tests), `test_bytes` (118 tests), and the no-default-
features check pass (`native-array-copy.log`, `no-std-array-copy.log`). The
slice-Buf proof gate passes 82 files with zero unproved leaves
(`slice-buf-overrides-regression.log`). It is a regression of the concrete
slice overrides; it must
not be counted as a proof of this generic default macro. The default macro's
Buf trait contracts and missing primitive endian-conversion contracts remain
separate integration obligations.

The private `_assert_trait_object` functions in `buf_impl.rs` and `buf_mut.rs`
are now native-only. Ordinary Rust still checks object safety; these functions
have no runtime behavior. Dynamic Buf/BufMut dispatch is explicitly outside
this proof configuration. With the temporary trusted trait scaffolding only
in the diagnostic copy, translation then advances to the raw transparent
wrapper cast in `src/buf/uninit_slice.rs:37`
(`runtime-diagnostic-no-dyn-asserts.log`). This is an additional representation
bridge obligation, not a successful full-crate translation.

The tempting mutable-B4 ghost annotation has a concrete unsoundness
counterexample. A temporary copy adds only `#[check(ghost)]` to the already
trusted `borrow_bound_mut`. `ghost-b4-erased-write.rs` detaches `[1]`, writes 2
through that bridge inside `ghost!`, reads the allocation normally, and frees
it explicitly. The single `erased_write` VC proves the postcondition
`result == 2`, while the native test observes `result == 1` because the write
was erased. Proof source, exact diagnostic bridge source and task are retained
in `ghost-b4-counterexample/`, with source hashes in
`ghost-b4-erased-write-manifest.json`; logs use the `ghost-b4-erased-write-`
prefix. This rejects the proposed trusted purity extension. The actual B4
bridge retains its ordinary program classification; no claim is made that
the current ordinary bridge has this flaw. Removing the temporary annotation
rejects the same caller at translation with "called non-ghost function
`raw_vec::borrow_bound_mut` in ghost context"
(`ghost-b4-current-interface-rejection.log`). A safe DerefMut integration needs
a representation/permission interface whose ghost projection cannot write
physical runtime memory, not an extra purity annotation.

Automatic destruction remains distinct from explicit cleanup. Vanilla
Creusot's `translation/function/terminator.rs` translates MIR `Drop` to
`Goto`; the standard `core::mem::drop` contract only ensures `resolve(t)`.
Consequently explicit consuming cleanup calls can prove normal-return callers
that invoke the existing reviewed cleanup routines. They cannot prove that
Rust's externally inserted scope-exit or unwind destructors invoke those
routines. Rewriting crate-controlled consuming callers to invoke cleanup
directly is viable; proving arbitrary downstream implicit Drop requires a
destructor-aware compiler/model. The existing exact-body cleanup ownership
gates retain their stated scope.
