# bytes 1.11.1 runtime proof checkpoint

Full runtime verification is **not complete**. Isolated helper and storage
proofs establish useful parts of the implementation, and the helpers are wired
into selected native call sites. They do not verify complete `Bytes` or
`BytesMut` ownership, trait, reference-count, or destruction behavior.

The latest crate-level `verify-all` attempt still stops during translation in
`src/buf/buf_impl.rs` and `src/buf/buf_mut.rs` on illegal recursive `Buf` and
`BufMut` traits. It also reproduces a rustc internal compiler error while
normalizing the `Bytes: PartialOrd<T>` `DeepModel` projection. See
`verification/artifacts/logs/runtime-entry.log`. The default-std integrated
tests (997 tests and 246 doctests) and the `no_std` check pass; these are build
and behavior checks, not a successful full-runtime proof. The compiler API
inventory contains 993 items, with 64 helper function rows linked to isolated
exact-source proofs. The final component replay passed all 19 targets: 184
Coma/proof files and 540 named VCs, including repeated dependency helpers.
These totals are not a completion percentage or a count of unique obligations.
All 38 negative controls failed at their intended VCs. Source/configuration
hashes and paired proof artifacts are saved in
`verification/artifacts/component-evidence.json`.

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
regions, and invalid Box recovery. The corresponding logs live beside each
probe. These negative probes check that the specified properties reject the
mutations; they do not expand runtime coverage.

The default-std integrated test/doctest and no-std logs are under
`verification/artifacts/logs/`. Component probes are under
`verification/probes/`. `./verify-all.bash` currently reproduces the runtime
translation blockers noted above. Proof commands require elevated execution
because Why3 uses Unix-domain sockets.

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
