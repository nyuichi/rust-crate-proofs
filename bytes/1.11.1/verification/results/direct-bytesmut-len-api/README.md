# Direct-source `BytesMut` proof evidence

`proof.json` records the fresh proof run from the crate's live source. The
`coma/` and `results/` trees retain the exact target COMA bodies and successful
prover results used by that run, together with the feature tree and translation
log. [`TCB.md`](TCB.md) describes the selected guarantee and its assumptions.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
