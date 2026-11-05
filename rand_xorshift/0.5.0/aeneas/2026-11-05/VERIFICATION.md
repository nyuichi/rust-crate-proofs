# Exact next-word output and state update

For the unchanged `rand_xorshift` 0.5.0 `try_next_u32` body, the Aeneas proof
establishes the exact output and state transition for every four-`u32` internal
state, using three explicit standard-library wrapping-operator models. If
`t = x ^ (x << 11)`, the output is
`r = w ^ (w >> 19) ^ t ^ (t >> 8)` and the new state is `(y, z, w, r)`.
Operations use 32-bit words and logical right shifts. The call returns `Ok`
without model failure or divergence.

The Rust source and official Aeneas frontend are unchanged. The supplied
executable models connect `Wrapping<u32>` shift-left, shift-right, and xor to
existing Aeneas primitives. Kani checks the Rust operator correspondence over
all `u32` operands and 64-bit `usize` shift values. A second Kani harness checks
the actual original crate body against the complete state/output formula over
all four-word states. It embeds the original source bytes and appends only a
private-state observer; it does not transcribe the implementation.

This is an explicitly reviewed semantic bridge, not an in-Lean proof of the
original standard-library operator bodies or of the cross-tool correspondence.
Kani and extraction use different Rust nightlies, so the Kani operator checks
are not direct proofs of the extraction toolchain's std bodies.
The result covers `try_next_u32`, including the all-zero state. Seed construction,
`next_u64`, byte filling, external-RNG initialization, inherited wrappers,
formatting, and serde remain outside scope. It establishes no seed invariant,
period, distribution, or cryptographic property.

## Evidence and reproduction

- [Public theorem and reference formula](proofs/Proofs.lean).
- [Original generated body](proofs/RandXorshift/Funs.lean),
  [generated types](proofs/RandXorshift/Types.lean), and
  [translation manifest](proofs/RandXorshift/translation.json).
- [Executable standard-operator models](proofs/RandXorshift/FunsExternal.lean)
  and [model-correspondence boundary](evidence/std-model-correspondence.txt).
- [Kani harnesses](evidence/kani-models/src/lib.rs),
  [byte-preserving source embedding](evidence/kani-models/build.rs),
  [operator result](evidence/kani-models/std-models.log), and
  [original-body result](evidence/kani-models/original-body.log): two successful
  harnesses, respectively 10 and 107 checks, zero failures.
- [Integrated Lean result](evidence/verification.log) and
  [transitive axiom listing](evidence/axioms.log): only `propext`,
  `Classical.choice`, and `Quot.sound`; no `sorryAx`.
- [Archive/source hashes](evidence/upstream.sha256) and
  [upstream revision](evidence/upstream-vcs.json). The original release archive
  is retained as `evidence/upstream.crate`.

Run `./verify-all.bash` for original tests, fresh targeted extraction,
generated-file equality, Lean build, and axiom audit. Run
`./verify-kani.bash` for the supplementary Rust correspondence checks. Tool
versions and environment assumptions are recorded in `proofs.toml` and the
Kani evidence directory.

Publication uses cargo-proofs 0.4.1 with
[environment CA trust](evidence/cargo-proofs-environment-ca.patch) and the
[upstream audit-variable rename](evidence/cargo-proofs-audit-compat.patch).
The theorem checks and allowed-axiom policy are unchanged.
