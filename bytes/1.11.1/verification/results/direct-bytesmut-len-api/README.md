# Direct-source `BytesMut` proof evidence

`proof.json` records the fresh proof run from the crate's live source. It
contains 40 selected Bytes COMA targets and five body-proved algebra targets
translated directly from the pinned private `creusot-std` source package. The
`coma/` and `results/` trees retain the exact COMA bodies and successful prover
results used by that run, together with the feature tree and translation log.
[`TCB.md`](TCB.md) describes the selected guarantees and their assumptions.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
