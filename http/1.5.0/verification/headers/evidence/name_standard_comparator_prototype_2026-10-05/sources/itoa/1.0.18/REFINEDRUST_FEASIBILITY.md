# RefinedRust feasibility for the runtime memory boundaries

## Finding

RefinedRust is a strong research candidate for the unsafe-memory obligations in
principle, but its current frontend cannot prove the unchanged Phase 9/10 itoa
code end to end. Annotations alone will not close these gaps. This assessment
uses RefinedRust development HEAD
`5846daaff196fbf29260b1876bd8d0549f1d6035` (2026-09-29), whose frontend pins
`nightly-2026-08-28`.

The first blocker is the actual `core::mem::MaybeUninit<u8>` element type in
`Buffer.bytes`. Rust defines `MaybeUninit` as a union, and RefinedRust's current
frontend rejects union ADTs (`translator.rs`, `register_adt`, lines 1092–1097).
The stdlib `mem` shim does not currently map this Rust union to RefinedRust's
logical `maybe_uninit` type. That logical model does exist: it represents an
initialized element as ownership of the inner `T` and an uninitialized element
as uninitialized memory of the same layout (`maybe_uninit.v`, lines 5–41).
The model is useful precedent, but it is not yet connected to the Rust type.

The second blocker is the suffix conversion. RefinedRust currently rejects slice
types (`translator.rs`, lines 1829–1833). References to `str` and slices are
temporarily translated as `Unit`, with a TODO about fat pointers
(`translator.rs`, lines 1753–1762). This representation cannot retain the
selected byte range, its values, or the facts needed to prove the returned
string. The commented slice/fat-pointer development sketch in
`stdlib/vec/theories/slices.v` is consistent with this limitation.

There is meaningful precedent for the first boundary's permission reasoning.
The checked-in `case_studies/minivec` has an array modeled as
`array_t cap (maybe_uninit T)` and implements `get_unchecked_mut` by computing
`ptr.add(index)` and forming `&mut *p`. Its Coq proof is checked in alongside
the example. This shows that RefinedRust can verify raw-pointer reborrows into
owned arrays while preserving initialized versus uninitialized element state.
It does not establish the itoa cast from a borrowed 40-element stack array to a
shorter typed array prefix, or the untouched-tail frame.

## Smallest useful research spikes

1. Add a frontend/stdlib mapping for `core::mem::MaybeUninit<T>` to the existing
   `maybe_uninit T` semantic type, including `uninit` and `write`. Then try a
   concrete `N = 4` helper over `&mut [MaybeUninit<u8>; 40]` that casts to and
   reborrows `&mut [MaybeUninit<u8>; 4]`, proving the same base, exclusive
   lifetime, and unchanged `[4, 40)` tail. This tests Boundary A without the
   generic associated type complicating the first probe.
2. For Boundary B, add faithful slice/fat-pointer support before attempting the
   unchanged helper. The proof must preserve the suffix address, length,
   provenance, and borrow lifetime while converting each initialized
   `MaybeUninit<u8>` payload to a readable `u8`; then prove ASCII implies valid
   UTF-8 and relate the resulting `&str` bytes to that suffix. Current `&str` /
   slice erasure cannot support this proof.

So the practical verdict is **no-go with today's unmodified RefinedRust
frontend; plausible as a tool-extension project**. Boundary A is the smaller
spike after the `MaybeUninit` mapping. Boundary B needs broader slice and string
support. This note records a source audit only; no RefinedRust build, proof, or
installation was run.

## Sources

- RefinedRust overview and PLDI 2024 paper: <https://plv.mpi-sws.org/refinedrust/>.
- Pinned frontend union handling:
  <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/rr_frontend/translation/src/types/translator.rs#L1092-1097>.
- Pinned slice and `str` handling:
  <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/rr_frontend/translation/src/types/translator.rs#L1753-1762> and
  <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/rr_frontend/translation/src/types/translator.rs#L1829-1833>.
- Logical `MaybeUninit` model:
  <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/theories/refinedrust/rust_typing/primitive/maybe_uninit.v#L5-41>.
- Raw-pointer reborrow precedent and checked-in proof:
  <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/case_studies/minivec/src/lib.rs#L447-454> and
  <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/case_studies/minivec/output/minivec/proofs/proof_Vec_T_get_unchecked_mut.v#L10-37>.
- Pinned frontend toolchain: <https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev/-/blob/5846daaff196fbf29260b1876bd8d0549f1d6035/rr_frontend/rust-toolchain.toml#L1-2>.
- itoa boundaries and operation locations: [`CROSS_TOOL_BOUNDARIES.md`](CROSS_TOOL_BOUNDARIES.md), [`RUNTIME_MEMORY_LEDGER.md`](RUNTIME_MEMORY_LEDGER.md), and [`src/runtime.rs`](src/runtime.rs).
