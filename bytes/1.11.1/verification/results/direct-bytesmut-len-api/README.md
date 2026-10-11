# Direct-source `BytesMut` proof evidence

`proof.json` records the fresh proof run from the crate's live source. It
contains 52 selected crate COMA targets and six body-proved targets translated
directly from the pinned private `creusot-std` source package: five pure
`release_sources` algebra functions and the generic
`EventBoundInvariant::new` constructor. The
`coma/` and `results/` trees retain the exact COMA bodies and successful prover
results used by that run, together with the feature tree and translation log.
[`TCB.md`](TCB.md) describes the selected guarantees and their assumptions.

The crate targets include twelve body-proved generic `OwnerLedger` algebra
functions. They establish the affine ticket/map operations used to represent
registered owner IDs, but no production `Bytes` handle is registered through
this ledger yet. They do not prove that the native refcount equals the number
of live handles, that an observed count of one is exclusive, or that Shared
storage may be reclaimed.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
