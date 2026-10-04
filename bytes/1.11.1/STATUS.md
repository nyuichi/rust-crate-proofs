# bytes 1.11.1 runtime proof checkpoint

## Current working-tree status (2026-10-04)

The local B1/B2 Vec/raw bridge now passes its ten-file isolated round-trip
gate: detach, return two owned fragments from a helper, join, and restore the
same initialized contents. The two physical Vec representation bridges are
trusted and reviewed; the split/join and caller bodies are proved. This uses
unmodified Creusot/creusot-std 0.13 and does not change the ordinary Vec model.
See the [physical trusted boundary](verification/RAW_VEC_TRUSTED_BOUNDARY.md).
Mutation, explicit cleanup and actual BytesMut integration are separate gates.

The actual `BytesMut::try_unsplit` now uses an address-only comparison helper
for its buffer and Shared pointers. Normal builds retain native thin-pointer
equality; proof builds use the `addr_eq` contract, which grants no logical
pointer/provenance identity. The [helper probe](verification/probes/address-comparison/README.md)
proves six files and rejects the identity-forging negative VC. Fresh ordinary
bytes tests (997), doctests (246), and the no-default-features build pass for
this change. This does not establish a try_unsplit body proof or runtime
ownership integration.

Implementation of the approved ownership design has started. The new
[feasibility results](verification/OWNERSHIP_FEASIBILITY_RESULTS.md) record
vtable translation and automatic Drop as excluded integration paths under the
no-large-compiler-change constraint. A real boxed immutable control block can
be shared through existing lifetime fractions and recovered after joining all
fractions; this is an isolated foundation, not a BytesMut/refcount proof.
The new owned-region kernel proves seven isolated files, including split/join
and exact slot preservation across a returned pair of regions. Its overlap
negative rejects the intended join precondition. It remains a model-only
ledger until a reviewed physical Vec/raw bridge connects it to native
allocation ownership. Separate native RawBuffer bodies have five passing
isolated tests; no physical trusted contracts or BytesMut integration are
established by those tests.

The 21-target replay and ordinary-test counts below refer to source checkpoint
`27cc729`, preceding the ownership implementation changes. Its retained
`checkpoint-manifest.json` records that historical snapshot; it is not a hash
manifest or fresh replay of the current worktree.

Full runtime verification remains **incomplete**. The 19-target inventory in
the historical section below was captured at commit `60e81ad`; its counts,
hashes, and replay results apply only to that snapshot. The current working tree
contains later structural proof changes and `src/provenance_specs.rs`, so those
historical totals must not be read as a replay or validation of the current
tree. The cfg-only Buf/BufMut convenience and comparison changes have passed
the final native-source checks summarized here. No integrated crate proof of
the current runtime is established.

The adopted runtime ownership effort has reached stop condition **D**. The
remaining blocker is a sound memory-resource interface, not another arithmetic
or helper lemma. The concrete target is
`Vec<u8> -> BytesMut::from_vec -> split_to/split_off -> mutate both handles ->
drop both`: the split path promotes the shared backing allocation, gives each
handle a distinct writable interval, and the last drop must recover and destroy
the allocation with the actual Release/Acquire protocol. Current `Perm` and
`PtrLive` support borrowed regions whose lifetimes end with a borrow of one
owner; they do not provide independently transferable owned regions plus a
separate allocation-recovery authority. The actual Vec raw-parts bridge and the
Vec stored in `Shared` also need exact allocation/layout/init tracking and a
suspended-ownership interface. Details and the proposed minimum extension are
in [`verification/BYTESMUT_OWNERSHIP_DESIGN.md`](verification/BYTESMUT_OWNERSHIP_DESIGN.md).

`src/provenance_specs.rs` adds `STD-PTRWRAP-01`, a narrow Creusot extern
specification for one-byte `wrapping_add`. Its postcondition describes only
the numerical address calculation. It does not specify pointer provenance,
allocation liveness, `Perm`/`PtrLive`, or dereferenceability, and it does not
make the tagged-pointer runtime path an ownership proof. Its isolated positive
probe proves 8 files/23 VCs, including address tagging/untagging and metadata
address calculations. The forged-metadata dereference negative control fails
at the intended permission VC. Both are component-level evidence, not an
integrated pointer-ownership proof.

A fresh sequential positive replay of the current working tree passed all 21
configured component targets (194 generated proof files and 581 named VCs,
including repeated dependencies). The new ownership-frontier probe contributes
2 helper-only files/18 VCs; it reuses the proved borrowed `Box` region helper
and does not prove actual `BytesMut` split, mutation, or drop. The 39 recorded
negative controls reject their intended VCs. Default std tests/doctests (997
tests and 246 doctests) and the no-default-features build also pass. These
results are component and behavior checks; no full-runtime proof is established.
The final current-source checks also pass for the `cfg(miri)` compilation path
(compilation only; Miri was not run). The current rustdoc API inventory has 997
items, and the coverage updater links 67 actual helper rows; the normal-build
pointer-address cast is excluded because only the proof-specific address
variant was probed.

The latest fresh full-crate translation now clears the earlier recursive
`Buf`/`BufMut` and `PartialOrd` normalization blockers, but stops before VC
generation on two vtable cycles: `static_clone` with `STATIC_VTABLE` and
`owned_clone` with `Owned::VTABLE`. No Coma tasks are produced for the actual
comparison adapters or constructor bodies, so those are not counted as
translated or proved. See
[`verification/artifacts/logs/runtime-ownership-frontier-translation.log`](verification/artifacts/logs/runtime-ownership-frontier-translation.log).
The formal runtime proof entry also stops during translation with the same two
cycles and produces no Coma tasks; see
[`verification/artifacts/logs/runtime-ownership-frontier-proof-entry.log`](verification/artifacts/logs/runtime-ownership-frontier-proof-entry.log).
This translation failure is separate from the stop-condition-D ownership
resource gap described above. Historical totals and source hashes below remain
specific to commit `60e81ad`; do not combine them with the fresh replay as if
they came from one snapshot.

## Historical component checkpoint at commit `60e81ad`

Full runtime verification is **not complete**. Isolated helper and storage
proofs establish useful parts of the implementation, and the helpers are wired
into selected native call sites. They do not verify complete `Bytes` or
`BytesMut` ownership, trait, reference-count, or destruction behavior.

The crate-level `verify-all` attempt recorded at this historical checkpoint
stopped during translation in
`src/buf/buf_impl.rs` and `src/buf/buf_mut.rs` on illegal recursive `Buf` and
`BufMut` traits. It also stopped at a rustc internal compiler error while
normalizing the `Bytes: PartialOrd<T>` `DeepModel` projection. The corresponding
log at that revision is
`git show 60e81ad:bytes/1.11.1/verification/artifacts/logs/runtime-entry.log`. The
default-std integrated tests (997 tests and 246 doctests) and the `no_std`
check pass; these are build
and behavior checks, not a successful full-runtime proof. The compiler API
inventory contains 993 items, with 64 helper function rows linked to isolated
exact-source proofs. The final component replay passed all 19 targets: 184
Coma/proof files and 540 named VCs, including repeated dependency helpers.
These totals are not a completion percentage or a count of unique obligations.
All 38 negative controls failed at their intended VCs. Source/configuration
hashes and paired proof artifacts are the files as they existed at that
revision, including
`git show 60e81ad:bytes/1.11.1/verification/artifacts/component-evidence.json` and
`git show 60e81ad:bytes/1.11.1/verification/artifacts/checkpoint-manifest.json`. The
current working-tree evidence manifest has since been refreshed for the
21-target replay above.

| Component | Isolated vanilla Creusot 0.13 evidence | Runtime coverage |
|---|---|---|
| Saturating arithmetic and bounded adapter helpers | Bodies proved; bounded helpers cover minimum, prefix, budget decrement, and representative callers (5 files) | Bounded helpers are called by Take, Limit, Reader, and Writer; trait-level composition is unproved |
| Slice, cursor, comparison, and chain helpers | Slice (4), cursor (10), comparison (3), and chain (5) proof files pass | Selected slice/cursor/chain helpers are wired; comparison helpers are used in `Bytes` and `BytesMut` equality/order impls, but those callers remain part of the integration blocker |
| `BytesMut` capacity and packed metadata | 14 proof files pass, including the native `leading_zeros` encoding, reconstruction bounds, packed repr round-trip, and vector-position update/preservation | Pure helpers are wired into `BytesMut`; pointer/integer casts at call sites are outside this proof |
| Fixed-width byte codecs and endian helpers | u16/u32 codecs (7), u64/u128 codecs (14), and endian operations (27) pass | Selected `Buf` and `BufMut` read/write call sites use these helpers |
| Checked fixed-width reads | Narrow reads (12) and wide reads (18) pass | Selected `Buf` read methods use these helpers; whole trait/API proofs remain blocked |
| Signed-wide fixed-width operations | Work in progress for i64/i128 operations | Not wired into the runtime or counted as proved component evidence |
| Mutable slice operations and uninitialized storage | Extracted helper probes pass; uninitialized-storage helpers have 4 proof files | Selected `BufMut` operations are wired; surrounding ownership and trait obligations remain unproved |
| Variable-width unsigned reads | 14 proof files pass for unsigned reads | Wired to the `Buf` `try_get_uint` and `try_get_uint_le` overrides; fixed-width signed i64/i128 readers also have 24 proof files and eight runtime overrides |
| Initialized Box storage | 13 proof files pass in the promoted Astra run | Isolated storage foundation only; does not prove `Bytes`/`BytesMut` ownership or drop behavior |
| Disjoint Box-region permissions | Probe proves a real `Box<[u8]>` can be split into unique borrowed regions, mutated independently, and recovered with other bytes preserved | Foundation only; uses standard `Perm`/`PtrLive` primitives and does not prove Bytes aliasing, provenance, reference counts, or `Drop` |
| `free_boxed_slice` and explicit deallocation | Extracted body and real Box-to-suffix caller prove with reviewed standard pointer/layout/allocator primitives | Bytes callers and resource supply are not connected/proved |
| Full `Bytes`/`BytesMut`, traits, vtables, provenance, reference counts, and automatic `Drop` | Not proved | Blocked by translation and model support; no replacement model is counted as runtime coverage |

No new helper introduces a `#[trusted]` contract. Proofs that use standard
pointer, permission, layout, or allocator primitives rely on the existing
`creusot-std` contracts for those primitives. A proved helper body is distinct
from a proved caller, an adopted trusted contract, and successful full-runtime
verification.

Negative VCs exercise representative wrong results and ownership errors,
including incorrect capacity encoding/reconstruction, lost packed flags,
wrong vector positions, wrong byte order/value, short-read behavior, overlapping
regions, and invalid Box recovery. The corresponding historical logs live
beside each probe at commit `60e81ad`. These negative probes check that the
specified properties reject the mutations; they do not expand runtime
coverage.

The default-std integrated test/doctest and no-std logs for this checkpoint are
in `verification/artifacts/logs/` at commit `60e81ad`. Component probe sources
are in `verification/probes/` at that revision. Its `./verify-all.bash` run
reproduces the historical runtime translation blockers noted above. Proof
commands require elevated execution because Why3 uses Unix-domain sockets.

## Remaining frontier

A final read-only review found no additional small helper that closes the missing
`Bytes` content/ownership relation. A static-only Bytes view is a possible future
restricted target, but still needs a justified pointer/byte model and dispatch
correspondence. General `Bytes::as_slice`, `Buf for Bytes` and capacity-preserving
`BytesMut` writes require allocation/provenance and writable/readable interval
resources carried by the actual handles. Clone, split, promotion, release and
Drop must preserve/recover those resources. Extending arithmetic helpers cannot
establish these premises. The current full-runtime translation failure was
reproduced after the comparison helper extraction.

## Reproduction and configuration

Run `bash scripts/verify-components.sh` for all 19 positive component probes,
`python3 scripts/verify-negative.py` for the 38 intended VC rejections, then
rerun the positive suite to restore default-feature live outputs. The collector
requires every configured component and checks source/configuration/artifact
hashes. Its `integrated_runtime: false` means no full-crate proof; the legacy
`foundation`/`connected_helpers` scope label is a grouping, while runtime call
connections are recorded above and in `SOURCE_CORRESPONDENCE.md`.

All proof results use unmodified Creusot 0.13.0, x86_64, native atomic Ordering
and no `sc-drf`. Default std tests and a no-default-features build were checked;
32-bit, optional features, Miri/Loom/Verus and performance were not validated.
