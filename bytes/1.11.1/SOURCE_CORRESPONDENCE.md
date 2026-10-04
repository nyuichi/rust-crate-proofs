# Runtime source correspondence

The original runtime sources for this crate were extracted from the published
`bytes` 1.11.1 release. The current verification branch changes `lib.rs` and
selected call sites in `bytes.rs`, `bytes_mut.rs`, and `buf/` to connect pure
helpers compiled in the normal runtime to isolated Creusot probes. These edits
retain the published public representation. Pure expression extractions retain
their native operations; byte codecs use explicit byte arithmetic and mutable
fills use safe element assignments. Exact component contracts and ordinary
tests support their byte behavior. They do not establish proof of the surrounding
caller, ownership, trait, reference-count, or destruction behavior.

The correspondence below records which source operations are extracted and
where the runtime calls them. Component proof logs and the latest integrated
build/test logs are listed in `STATUS.md`.

| Extracted helper | Published operation represented | Runtime call sites and proof boundary |
|---|---|---|
| `src/slice_ops.rs`, `src/slice_read_ops.rs`, `src/slice_wide_read_ops.rs` | Slice advance/copy and fixed-width checked reads | Used by `buf/buf_impl.rs`; read helpers also compose with codec helpers. The proofs cover the isolated helper contracts, not the full `Buf` trait methods |
| `src/cursor_ops.rs` | Cursor remaining length, visible chunk, and advancing position | Used by the `Cursor` implementation in `buf/buf_impl.rs`; the `Buf` caller and cursor trait composition remain unproved |
| `src/chain_ops.rs` | Saturating remaining-length sum and split count | Used in `buf/chain.rs`; it does not prove the `Chain` ownership or trait implementation |
| `src/comparison_ops.rs` | Byte-slice equality and ordering | Used by `Bytes` and `BytesMut` equality and comparison implementations. The isolated slice comparison is proved; compiler translation of the `Bytes: PartialOrd<T>` caller still ICEs |
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
stronger atomic ordering has been adopted for the runtime crate. None of the new
pure helper modules adds a trusted contract. Pointer, layout, allocator, and
permission foundation proofs rely on the existing `creusot-std` contracts for
those standard primitives.

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
