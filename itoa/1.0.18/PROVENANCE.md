# itoa 1.0.18 provenance and verification scope

**Verification status: complete-equivalent.**

The established contract and proof record below describes the current
`cfg(creusot)` recursive decimal model. Production-code coverage is being added
phase by phase; see [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md) for its
separate scope and proof status.

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
- `Buffer::format` returns exactly the signed decimal ASCII representation of
  its argument, with no leading zeroes other than the representation of zero;
- the public orchestration and all supporting arithmetic, sequence, and
  representation lemmas have proved bodies.

The original recursive-model baseline proved 67 translated files in both
configurations. The latest integrated runtime matrix is recorded in
[RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md): default proves 196
libraries / 1,696 VCs and all-features proves 197 / 1,700. It includes the
actual unsigned `u128` formatter body and signed `i128` buffer writer with an
`i128::MIN` witness; the `u128` path remains conditional on the accepted
trusted `mulhi` result contract. The all-features configuration includes the
upstream optional `no-panic` dependency.

## Explicit trusted boundaries

Two narrow boundaries are currently trusted:

- The recursive verification model's conversion of its already-proved ASCII
  suffix into a borrowed `str`. Its contract states that the result's bytes
  are exactly that suffix. This pre-existing model boundary does not justify
  the native runtime's raw `MaybeUninit` slice-to-`str` conversion, which is
  still outside the runtime proof. The decimal algorithm, sign handling,
  buffer bounds, and returned contents are not trusted.
- The `u128_ext::mulhi` wrapper's exact result equation,
  `result == x * y / 2^128`, accepted by the user on 2026-10-02. The actual
  limb algorithm lives in `mulhi_core`; its operation bounds, casts, shifts,
  and overflow checks are verified, but the wrapper's assumed equation is not
  yet derived from the core's returned value.

Removal conditions: replace the string boundary when Creusot can prove ASCII
UTF-8 validity and model the slice-to-`str` reference conversion without a
raw representation cast. Remove the `mulhi` trust when the high-half equation
is proved from `mulhi_core` while retaining its operation checks.

The signed `i128` proof also uses narrow external models for core array
`IndexMut`, mutable-slice-to-array borrowing, and primitive `unsigned_abs`.
These are standard-library assumptions, not additional local `#[trusted]`
functions; their exact bounds, view/frame contracts, source basis, and removal
conditions are documented in [RUNTIME_VERIFICATION.md](RUNTIME_VERIFICATION.md).

Run `./verify-all.bash` in this directory to reproduce the proof matrix. The
ordinary upstream suite passes 11 integration tests and 2 documentation tests.
The optimized all-features integration-test build with the `no-panic` feature
also passes all 11 tests. Generated Cargo and Why3 artifacts are intentionally
not tracked.
