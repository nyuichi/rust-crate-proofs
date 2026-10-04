# Runtime source correspondence

The original runtime sources for this crate were extracted from the published
`bytes` 1.11.1 release. The verification branch connects pure helpers to
selected runtime call sites and now includes proof-only `cfg(creusot)` structural
adapters for trait translation and handle comparisons, plus pointer-address
helpers. These edits retain the published public representation and normal
build trait implementations. Component proofs, source checks, and ordinary
tests do not establish the surrounding runtime ownership, trait,
reference-count, or destruction behavior.

The correspondence below records which source operations are extracted and
where the runtime calls them. Component proof logs and the latest integrated
build/test logs are listed in `STATUS.md`.

| Extracted helper | Published operation represented | Runtime call sites and proof boundary |
|---|---|---|
| `src/slice_ops.rs`, `src/slice_read_ops.rs`, `src/slice_wide_read_ops.rs` | Slice advance/copy and fixed-width checked reads | Used by `buf/buf_impl.rs`; read helpers also compose with codec helpers. The proofs cover the isolated helper contracts, not the full `Buf` trait methods |
| `src/cursor_ops.rs` | Cursor remaining length, visible chunk, and advancing position | Used by the `Cursor` implementation in `buf/buf_impl.rs`; the `Buf` caller and cursor trait composition remain unproved |
| `src/chain_ops.rs` | Saturating remaining-length sum and split count | Used in `buf/chain.rs`; it does not prove the `Chain` ownership or trait implementation |
| `src/comparison_ops.rs` | Byte-slice equality and ordering | The normal `Bytes`/`BytesMut` trait implementations still delegate to this helper under `cfg(not(creusot))`. Proof-only handle adapters were added under `cfg(creusot)`, but no adapter or trait caller is translated/body-proved: the latest full-crate run stops before Coma generation on vtable cycles |
| `src/capacity_ops.rs` | `BytesMut` original-capacity conversion and packed metadata operations | Used by `bytes_mut.rs` for initial capacity, metadata packing/extraction, and vector position. The pure helper proof does not cover adjacent pointer/integer casts or pointer reconstruction |
| `src/byte_codec_ops.rs`, `src/byte_codec_wide_ops.rs`, `src/endian_ops.rs` | Fixed-width integer encoding/decoding in big- and little-endian forms | Used by `buf/buf_mut.rs` and read helpers in `buf/buf_impl.rs`; the component probes prove the codec/endian helpers, not each trait caller |
| `src/slice_mut_ops.rs`, `src/uninit_ops.rs` | Mutable slice advance/copy/fill and uninitialized-prefix initialization/fill | Used by selected `BufMut` methods in `buf/buf_mut.rs`; caller ownership and trait composition are not part of the isolated proofs |
| `src/variable_read_ops.rs` | Unsigned variable-width u64 reads | Used by the `Buf` `try_get_uint` and `try_get_uint_le` overrides in `buf/buf_impl.rs`; the helper has isolated unsigned-read evidence. Fixed-width signed i64/i128 readers are also connected via `signed_wide_ops` |
| `src/bounded_ops.rs` | Bounded minimum, prefix, and remaining-budget operations | Used by Take, Limit, Reader, and Writer adapters; generic trait-level composition remains unproved |

`src/signed_wide_ops.rs` provides fixed-width signed i64/i128 readers. Its
range-split conversion models exact two's-complement values without relying on
an unspecified narrowing cast. Eight slice `Buf` get/try-get overrides use the
checked BE/LE readers; short inputs stay unchanged. The standalone probe proves
24 files and rejects the wrong-signed negative control.

The `capacity_ops` helpers preserve the native bit operations. In particular,
`original_capacity_to_repr` uses native `usize::leading_zeros` and the
`core::cmp::min` cap from `bytes_mut.rs` around lines 1502--1516;
`original_capacity_from_repr` preserves the corresponding power-of-two shift.
The probe proves the three-bit representation range and useful reconstruction
properties using the standard leading-zero logical model, plus metadata pack,
unpack, vector-position round-trip, low-bit preservation, and kind-flag
preservation. This evidence was run on the 64-bit target; it does not establish
a 32-bit proof. Runtime pointer/integer casts remain at native `BytesMut` call
sites and are explicitly outside the pure arithmetic proof.

## Storage and deallocation probes

The storage-foundation probe uses actual Box allocations and standard
`Perm`/`PtrLive` permission primitives. The region-permissions probe splits a
uniquely owned `Box<[u8]>` allocation into disjoint borrowed regions, mutates
each region, and recovers the original Box with other bytes preserved. These
are foundations only; they do not prove Bytes aliasing, pointer provenance,
reference counts, or automatic `Drop` behavior.

The deallocation probe reproduces the actual `free_boxed_slice` body with only:

1. a ghost live-allocation witness;
2. a permission-aware wrapper erasing to native `offset_from`;
3. a token-consuming wrapper erasing to native `dealloc`.

`scripts/check-erasure.py` removes these reviewed substitutions and compares
the result to the source function body. Its hash result is recorded in
`verification/artifacts/deallocation-correspondence.json`. The compiler's
`#[erasure]` checking is an additional check. This source check is not a
mechanical proof of the entire compiler's erasure or of Bytes caller resource
supply.

No replacement storage model, permission fabrication, vtable replacement, or
stronger atomic ordering has been adopted for the runtime crate. The new
`src/provenance_specs.rs` module adds the narrowly scoped `STD-PTRWRAP-01`
extern specification, which is a TCB assumption for numerical address
calculation only; it supplies no provenance or dereference permission. The
other pure helper modules add no trusted contracts. Pointer, layout, allocator,
and permission foundation proofs rely on the existing `creusot-std` contracts
for those standard primitives.

## Current proof-only structural boundaries

The ordinary comparison trait implementation bodies remain enabled under
`cfg(not(creusot))`. Under `cfg(creusot)`, the current source adds named
`Bytes`/`BytesMut` comparison adapters that call `as_slice`, `as_ref`,
`as_bytes`, and the proved `comparison_ops` helpers. They have no semantic
postcondition relating those raw-backed views to logical handle contents. They
are source adapters only: the latest full-crate formal entry fails during
translation before producing any Coma tasks for them, so they are neither
translated nor body-proved and do not establish caller correspondence.

`buf::proof_convenience` is compiled only for `cfg(creusot)`. It routes selected
`copy_to_bytes` operations through free wrappers for the actual `Take` and
`Chain` constructors, avoiding recursive convenience-method trait signatures
in Creusot. The normal build retains the existing trait methods and bodies;
the wrappers call the actual constructors, whose own specs describe stored
field preservation, and add no adapter-level content/advance or I/O behavior
contract. Source acceptance of these proof paths did not produce body proof
artifacts because full-crate translation stops earlier at the vtable cycles
described below.

The latest formal runtime entry has no report of the earlier recursive
`Buf`/`BufMut` trait rejection or `Bytes: PartialOrd<T>` normalization ICE. It
now stops at two mutually recursive translation cycles: `static_clone` through
`STATIC_VTABLE`, and `owned_clone` through `Owned::VTABLE`. These are verifier
translation cycles around runtime vtable constants, not claims of runtime
recursion. No Coma tasks are generated. See
`verification/artifacts/logs/runtime-ownership-frontier-proof-entry.log` and
`verification/artifacts/logs/runtime-ownership-frontier-translation.log`.

## Pointer-address source variants

`src/provenance_specs.rs::pointer_addr` uses `ptr.addr()` for `cfg(miri)` and
`cfg(creusot)`, while optimized non-Miri/non-Creusot builds retain the existing
`ptr as usize` implementation. `bytes.rs::ptr_map` similarly uses
`pointer_with_address` and raw-pointer `wrapping_add` on the original `*mut u8`
in the Miri/Creusot branch, while the optimized branch retains integer
extraction and reconstruction. `without_provenance` and
`bytes_mut.rs::invalid_ptr` create null-derived pointers for integer metadata
fields only.

`STD-PTRWRAP-01` states only the one-byte pointer's numerical wrapping-address
equation. The isolated `provenance-ops` probe proves eight helper files/23 VCs
using the `.addr()` proof branch and rejects a negative dereference through a
null-derived metadata pointer at the permission VC. It does not prove the
normal optimized pointer cast, provenance preservation, allocation identity,
live-range validity, `Perm`/`PtrLive`, or `Bytes`/`BytesMut` ownership.

## Bounded adapter extraction

`src/bounded_ops.rs` is compiled into the normal runtime and included unchanged
by the `bounded-ops` proof crate. Take/Limit remaining and Reader/Writer transfer
sizes call `bounded_len`: the original `cmp::min` expression is unchanged
inside the helper. Take::chunk calls `bounded_chunk`, which uses that same
minimum and the original prefix slice operation. Take/Limit budget decrements
call `decrease_limit`, retaining the original subtraction after the underlying
operation and retaining the original preceding assertions. No trait contract,
ownership, or `Drop` behavior is assumed to claim whole-adapter coverage. The
proved helpers are connected at these runtime call sites; generic adapter
composition remains pending.

## Refactoring behavior checks

The slice overrides preserve the original panic guards and error sizes. Fixed
checked reads test the same minimum input length before reading or consuming;
variable unsigned reads retain the width > 8 panic before the short-input check,
and width zero succeeds with zero and no consumption. Native codec comparison
tests exhaust all u16 bit patterns and sample boundary/mixed u32/u64/u128 values;
wide tests are not exhaustive. `MaybeUninit<u8>` fills/copies assign
`MaybeUninit::new(byte)` without reading prior uninitialized storage; replacing
its value invokes no payload destructor. These changes do not alter atomic
Ordering. Performance has not been benchmarked.

`scripts/check-runtime-extractions.py` checks reviewed inverse substitutions for
Take/Limit/Reader/Writer and the slice/Cursor Buf changes against the pinned
baseline. It treats new read overrides as separately reviewed implementations,
not a compiler-checked general equivalence proof. Box helper write-back uses a
proved sequence lemma and whole-borrow frame to recover copied content and
untouched prefix/suffix; that proves the actual Box caller, not a Bytes handle.
