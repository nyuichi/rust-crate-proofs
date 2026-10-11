# Direct-source `BytesMut` proof evidence

`proof.json` records the fresh proof run from the crate's live source. The
`coma/` and `results/` trees retain the exact selected COMA bodies and
successful prover results, together with the feature tree and translation log.
All selected targets translate from this crate's original source; no separate
Std-source proof targets are included. [`TCB.md`](TCB.md) describes the selected
guarantees and their assumptions.

The selected source targets include body proofs for the generic `OwnerLedger`
algebra and the current-value refcount operation wrappers and increment loop.
The ledger remains disconnected from production `Bytes` and `BytesMut` owner
handles. The refcount proof state is a scalar operation model, not an owner
inventory. These results do not establish that the native refcount equals the
number of registered handles, that an observed count of one is exclusive, or
that Shared storage may be reclaimed.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
