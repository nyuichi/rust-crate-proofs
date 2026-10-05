# Runtime boundary bridge: accepted source changes and proof status

The original verification goal was to prove the optimized `Buffer::format`
implementation with its published raw-memory expressions intact. Creusot
rejected the physical-buffer raw-pointer dereference and the
`MaybeUninit<u8>`-to-`u8` slice cast during translation. The user explicitly
accepted two small source changes to expose the production path to Creusot,
while asking that every other formatter and arithmetic fact remain proved.
These edits are acknowledged departures from the preferred unchanged-runtime
approach. Boundary A replaces the original raw pointer operation with safe Rust;
the original raw-A cast remains unproved. Boundary B retains a local trusted
raw slice conversion, so its physical memory-permission soundness remains an
assumption.

## A: form the typed prefix with a checked conversion

The sealed writer receives the existing physical
`[MaybeUninit<u8>; 40]` storage. Each concrete implementation takes its known
prefix of length `I::MAX_STR_LEN`, converts that slice to its associated fixed
array type with `.try_into().unwrap()`, and calls the existing optimized
formatter on that typed array. This replaces `as_mut_ptr().cast::<I::Buffer>()`
and `&mut *buf_ptr`.

The conversion itself is safe Rust. For each sealed implementation, the
selected prefix length equals the associated array length and does not exceed
40; the focused proofs establish these facts, so each `unwrap` is proved not
to panic. They also retain the writer's frame and initialized-suffix
contracts. No formatter result or arithmetic fact is trusted here.

This is the accepted source deviation for Boundary A. Its removal condition is
to model/prove the original typed-prefix reborrow directly in Creusot, or to
upstream a Creusot model that supports it, and then restore the original call
without broadening trust.

## B: isolate only the initialized-slice representation conversion

The unsafe `assume_init_slice` helper receives `&[MaybeUninit<u8>]` and returns
`&[u8]`. Its exact intended Creusot contract is:

```text
requires: every slot in the input slice is initialized
ensures:  result length equals input length
ensures:  each result byte equals the corresponding initialized input byte
```

The helper body contains the raw slice reinterpretation and is locally
`#[trusted]` under Creusot because Creusot does not model this metadata-
preserving view conversion. The caller proves the helper's initialization
precondition in Creusot's logical writer model. The helper trusts only that the
representation conversion preserves memory validity and forms the borrowed
byte view; Creusot does not prove the physical memory-permission transfer.
The helper adds no ASCII, UTF-8, or decimal-correctness fact. Its result
lifetime is tied to the input borrow by Rust's reference lifetime rules. The
caller separately proves suffix bounds, ASCII range, and the relation to the
canonical decimal model before using `str::from_utf8_unchecked`.

This is the accepted source deviation for Boundary B. It does not justify
arbitrary `MaybeUninit` reads, writes, offsets, or output bytes. Its removal
condition is for Creusot to prove the byte-slice view from the existing
initialized-slot permission model; then the local trust can be deleted without
changing the writer proof.

## Integrated proof result

The public `Buffer::format` body is now present under `cfg(creusot)` and calls
the same sealed optimized writers. All 12 concrete sealed writers are included
on the current x86_64 target. Focused proofs have discharged the actual
`Sealed::write` body and refinement for all 12 concrete types, including each
`.try_into().unwrap()` length check. The `u8` body used 20 goals; each of the
other 11 bodies used 8, and each refinement used 1. The run is recorded in
[`writers/focused-all/focused-proof.log`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/writers/focused-all/focused-proof.log). The integrated public-method
proof passed in both configurations:

| Configuration | Proof libraries | VCs | Failed goals |
| --- | ---: | ---: | ---: |
| default | 254 | 2,083 | 0 |
| `--all-features` | 269 | 2,145 | 0 |

The proof establishes the `slice_buffer_to_str` bounds and logical
initialization preconditions, composes the ASCII-to-UTF-8 lemma, proves the
returned-byte contract, and shows the `unreachable_unchecked` branch is
unreachable. The all-features run adds 15 proof libraries and 62 VCs for
the extra no-panic instrumentation checks. Its log is
[`integrated/fullsuite.log.gz`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/integrated/fullsuite.log.gz).
Run metadata and source hashes are summarized in
[`REPORT.md`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/REPORT.md).

The result is **proved on x86_64 under the stated Creusot models and local
Boundary B trust**. It does not establish the 16- or 32-bit `usize`/`isize`
implementations or prove the physical memory-permission conversion inside B.

The older Phase 9 and Phase 10 probes remain useful history: they identify the
raw expressions that Creusot rejected before this accepted source change. They
are not the status of the current source. The previous Verus probes are also
historical and are not a proof of this helper's contract. No Verus/Creusot
correspondence claim is made for the locally trusted helper.

The `CharExt::to_utf8` standard-library logic model used by the ASCII witness
is now an open encoding definition built from `utf8_byte`, rather than a
trusted opaque model. Each logic function has a focused one-VC Creusot proof,
recorded in [`stdlib-utf8/REPORT.md`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/stdlib-utf8/REPORT.md).
This proves the mathematical Unicode-to-UTF-8 model used by the ASCII lemma;
it does not prove Rust core's runtime character encoder. The separate
standard-library `from_utf8_unchecked` model remains an assumption in the
Creusot contract chain.

The focused ASCII proof also discharged `ascii_byte_map_to_utf8` in 30 VCs
and `ascii_bytes_are_utf8` in 3 VCs, with no trusted helper. Its caller
composition passed in the integrated proof; the focused proof and solver
sessions are recorded in
[`ascii/focused-final/focused-proof.log`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/ascii/focused-final/focused-proof.log).

Native checks pass 12 integration tests and 2 doctests in both the default
debug/no-features configuration and the release all-features configuration
with fat LTO, one codegen unit, and matching rustdoc flags. Exact commands and
logs are in the [runtime check report](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/native/README.md).
The recursive verification model's former trusted string leaf was removed and
its replacement passed as part of the integrated run.
