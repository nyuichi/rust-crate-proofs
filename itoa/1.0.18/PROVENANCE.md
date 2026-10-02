# itoa 1.0.18 provenance and verification scope

**Recursive `cfg(creusot)` model: complete-equivalent. Actual runtime public
`Buffer::format`: end-to-end proof incomplete.**

The established contract and proof record below describes the current
`cfg(creusot)` recursive decimal model. Production-code coverage is being added
phase by phase; see [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md) for its
separate scope and proof status. The Phase 8 audit of initialized runtime
suffixes and remaining raw-memory boundaries is in
[RUNTIME_MEMORY_LEDGER.md](RUNTIME_MEMORY_LEDGER.md), with cross-tool contract
status in [CROSS_TOOL_BOUNDARIES.md](CROSS_TOOL_BOUNDARIES.md).

This source tree is copied from the crate published on crates.io as `itoa`
version `1.0.18`. The published archive has SHA-256 checksum
`8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682`.
Its `.cargo_vcs_info.json` records the immutable upstream revision
`af77385d0daf4d0e949e81f2588be2e44f69f086`.

Ordinary builds retain the published API, fixed `MaybeUninit` buffer, lookup
table, and optimized formatting implementation. Under `cfg(creusot)`, the same
public `Buffer` and sealed `Integer` surface is represented by an initialized
40-byte buffer and an equivalent recursive decimal writer. This isolates the
proof from raw-memory representation details without changing runtime code.

## Established contracts

The proof defines canonical most-significant-first decimal ASCII sequences and
establishes the following:

- every published `Integer` implementation (`u8` through `u128`, `usize`,
  `i8` through `i128`, and `isize`) computes the exact mathematical magnitude
  and sign, including `i128::MIN`;
- the recursive digit writer terminates, stays within the supplied suffix,
  preserves all bytes outside its modeled updates, returns the exact start
  index, and writes precisely the canonical unsigned decimal sequence;
- the 40-byte buffer is sufficient for every `u128` magnitude and leaves room
  for the sign of every negative supported integer;
- the recursive model's `Buffer::format` returns exactly the signed decimal
  ASCII representation of its argument, with no leading zeroes other than the
  representation of zero;
- the public orchestration and all supporting arithmetic, sequence, and
  representation lemmas have proved bodies.
- Phase 8 proves the ASCII range of the existing unsigned, signed, and
  whole-integer decimal models, then checks it against initialized output from
  the actual i8 and `i128::MIN` writers. This establishes the writer-to-ASCII
  handoff; it does not prove the runtime raw pointer or string conversions.

The original recursive-model baseline proved 67 translated files in both
configurations. The Phase 12 matrix before the `mulhi` closure reported 218
libraries / 1,884 VCs in each configuration; the latest integrated
`run-verify-all.sh` rerun after closure passed with 224 libraries / 1,917 VCs
in both default and all-features on the current x86_64 target. The focused
joint-helper proof discharged the `mulhi_core` body 68/68 and its module
74/74; the `mulhi` wrapper has no `#[trusted]` annotation and its body proof
discharged 2/2 goals. It establishes the exact
high-half equation from the limb implementation, which the actual `u128`
formatter and signed `i128` writer consume. Raw `Buffer::format` and the
runtime borrowed-`str` conversion remain outside the proof. The all-features
configuration includes the upstream optional `no-panic` dependency. Full
proof evidence and logs are linked from [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md).

## Explicit trusted boundaries

The recursive verification model retains one local trusted boundary:

- The recursive verification model's conversion of its already-proved ASCII
  suffix into a borrowed `str`. Its contract states that the result's bytes
  are exactly that suffix. This pre-existing model boundary does not justify
  the native runtime's raw `MaybeUninit` slice-to-`str` conversion, which is
  still outside the runtime proof. The decimal algorithm, sign handling,
  buffer bounds, and returned contents are not trusted.

The previous arithmetic boundary on `u128_ext::mulhi` is closed in the focused
joint-helper proof. It proves the exact equation
`result == x * y / 2^128` from the limb algorithm, preserving the core's
operation-range and overflow checks. The same proved contract now connects to
the `u128` formatter and signed `i128` writer. The earlier isolated attempts
and their open goals remain documented as history; the current proof result
and evidence are summarized in [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md)
and the [joint-helper report](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-composition/REPORT.md).

The remaining removal condition for the recursive-model string boundary is
for Creusot to prove ASCII UTF-8 validity and model the slice-to-`str` reference
conversion without a raw representation cast. The `mulhi` result equation is
no longer an accepted arithmetic assumption in the focused proof.

The signed `i128` proof also uses narrow external models for core array
`IndexMut`, mutable-slice-to-array borrowing, and primitive `unsigned_abs`.
These are standard-library assumptions, not additional local `#[trusted]`
functions; their exact bounds, view/frame contracts, source basis, and removal
conditions are documented in [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md).

Run `../../tools/creusot-toolpatch/scripts/run-proof.sh ./verify-all.bash` from
this crate directory to reproduce the proof matrix with the registered
Power_sum-pruned Z3 driver variant. The ordinary upstream suite passes 11
integration tests and 2 documentation tests.
The optimized all-features integration-test build with the `no-panic` feature
also passes all 11 tests. Generated Cargo and Why3 artifacts are intentionally
not tracked.
