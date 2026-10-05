# itoa 1.0.18 provenance and verification scope

**The actual `Buffer::format` source path is included under `cfg(creusot)` and
passes the integrated x86_64 proof in both configurations:** 254 libraries /
2,083 VCs by default and 269 / 2,145 with all features.

To make Creusot translate the public runtime path, the source now uses two
explicitly accepted implementation changes: each sealed writer obtains its
fixed-size typed prefix with a safe slice-to-array `.try_into().unwrap()`, and
the initialized `MaybeUninit<u8>`-to-`u8` slice reinterpretation is isolated in
the narrow trusted `assume_init_slice` helper. This is a limited departure from
the goal of verifying the unmodified published runtime source, recorded with
its exact contract and removal condition in
[RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md). The helper does not
assume ASCII or decimal correctness. The integrated proof establishes the
public method and its callers under the exact local trust described below;
the physical memory-permission transfer inside `assume_init_slice` remains
unproved.

The existing recursive decimal model remains the mathematical specification.
The historical Phase 8 writer-state audit and the prior raw-pointer probe
results are retained as checkpoints in
[RUNTIME_MEMORY_LEDGER.md](RUNTIME_MEMORY_LEDGER.md) and
[CROSS_TOOL_BOUNDARIES.md](CROSS_TOOL_BOUNDARIES.md); their earlier conclusion
that the current source excludes the public runtime method has been superseded
by the bridge changes.

This source tree is copied from the crate published on crates.io as `itoa`
version `1.0.18`. The published archive has SHA-256 checksum
`8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682`.
Its `.cargo_vcs_info.json` records the immutable upstream revision
`af77385d0daf4d0e949e81f2588be2e44f69f086`.

Ordinary builds retain the published API, fixed `MaybeUninit` buffer, lookup
table, and optimized formatting implementation, with the two accepted source
changes documented above. Under `cfg(creusot)`, the actual runtime
`Buffer::format` and optimized sealed writers are now compiled through the
same `[MaybeUninit<u8>; 40]` storage path. The separate recursive model in
`verification.rs` remains available as the mathematical specification; it no
longer replaces the public runtime method during Creusot translation. The
current x86_64 proof target includes all 12 concrete sealed writer bodies and
the actual public method; the integrated result passes in both configurations.
The proof report and archived log are linked from
[RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md).

## Established contracts

The separate recursive verification model defines canonical
most-significant-first decimal ASCII sequences and establishes the following
model facts:

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
- the recursive-model orchestration and its supporting arithmetic, sequence,
  and representation lemmas have proved bodies;
- Phase 8 proves the ASCII range of the existing unsigned, signed, and
  whole-integer decimal models, then checks it against initialized output from
  the actual i8 and `i128::MIN` writers. The current bridge additionally
  supplies ASCII facts to the runtime string-construction proof; the integrated
  proof establishes that composition.

The original recursive-model baseline proved 67 translated files in both
configurations. The latest completed pre-bridge Phase 12 matrix reported 224
libraries / 1,917 VCs in both default and all-features on the current x86_64
target. The focused joint-helper proof discharged the `mulhi_core` body 68/68
and its module 74/74; the `mulhi` wrapper has no `#[trusted]` annotation and
its body proof discharged 2/2 goals. It establishes the exact high-half
equation from the limb implementation, consumed by the actual `u128`
formatter and signed `i128` writer. These counts predate the current runtime
bridge and must not be read as evidence about the newly included public method.
The all-features configuration includes the upstream optional `no-panic`
dependency. The current integrated result is summarized in
[RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md).

The current `CharExt::to_utf8` model is open logic defined through the proved
`utf8_byte` constructor; each passed one focused VC. That proves the
mathematical Unicode-to-UTF-8 encoding used by the ASCII witness, not Rust
core's runtime character encoder. The focused report is
[`stdlib-utf8/REPORT.md`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/stdlib-utf8/REPORT.md).
The `from_utf8_unchecked` standard-library contract used downstream remains a
separate assumption.

## Explicit trusted boundaries

The runtime source now contains one deliberately narrow trusted boundary for
the accepted Boundary B source change:

- `assume_init_slice` reinterprets a borrowed slice of `MaybeUninit<u8>` as a
  borrowed slice of `u8`. Its precondition requires every selected slot to be
  initialized; its postconditions preserve length and each logical byte. It
  carries no ASCII, UTF-8, or decimal-correctness premise. Creusot currently
  cannot express the slice metadata-preserving view conversion from the
  initialized-slot model. Remove this helper when Creusot can prove that
  conversion from the existing initialized-slot permission directly. The
  concrete trust entry and status are in
  [RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md).

The recursive verification model's prior trusted string leaf has been removed.
Its replacement requires the caller-proved ASCII suffix and uses the shared
ASCII-to-UTF-8 lemma; it preserves the existing returned-byte postcondition.
Its body and strengthened caller precondition pass in the integrated proof.
No second local string-construction trust is present in the current source.

The previous arithmetic boundary on `u128_ext::mulhi` is closed in the focused
joint-helper proof. It proves the exact equation
`result == x * y / 2^128` from the limb algorithm, preserving the core's
operation-range and overflow checks. The same proved contract now connects to
the `u128` formatter and signed `i128` writer. The earlier isolated attempts
and their open goals remain documented as history; the current proof result
and evidence are summarized in [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md)
and the [joint-helper report](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-composition/REPORT.md).

The remaining removal condition for runtime trust is to prove the initialized
`MaybeUninit<u8>`-to-`u8` slice view directly from Creusot's memory model. The
`mulhi` result equation is no longer an accepted arithmetic assumption in the
focused proof. The runtime caller can establish the helper's initialization
precondition in Creusot's logical writer model; the helper's raw-view soundness
and physical memory-permission transfer remain inside the trusted boundary.
The historical Verus audit does not prove or correspond to that operation.

The signed `i128` proof also uses narrow external models for core array
`IndexMut`, mutable-slice-to-array borrowing, and primitive `unsigned_abs`.
These are standard-library assumptions, not additional local `#[trusted]`
functions; their exact bounds, view/frame contracts, source basis, and removal
conditions are documented in [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md).

Run `../../tools/creusot-toolpatch/scripts/run-proof.sh ./verify-all.bash` from
this crate directory to reproduce the proof matrix with the registered
Power_sum-pruned Z3 driver variant. Historical native results before the new
reused-buffer regression test covered 11 integration tests and 2 documentation
tests by default, and 11 integration tests with all features. Current native
validation passes 12 integration tests and 2 doctests with default features,
and 12 integration tests in the release all-features configuration with fat
LTO and one codegen unit, with matching rustdoc flags for the doctests. The
no-panic investigation found that private digit, pair, and quad writers had
checker annotations despite caller-preconditioned bounds or value ranges. The
accepted correction removes only those three annotations; public `Buffer::format`
and sealed writer annotations remain. The change affects checker metadata,
not executable bodies or Creusot contracts, and adds no extra trust. Exact
commands and logs are in the [native runtime check record](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/native/README.md).
The separate diagnostic baseline run passed 24 tests across 14 targets after
the same three-annotation removal. Generated Cargo and Why3 artifacts are
intentionally not tracked.
