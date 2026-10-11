# Direct-source `BytesMut` proof evidence

`proof.json` records the fresh proof run from the crate's live source. The
`coma/` and `results/` trees retain the exact selected COMA bodies and
successful prover results, together with the feature tree and translation log.
All selected targets translate from this crate's original source; no separate
Std-source proof targets are included. [`TCB.md`](TCB.md) describes the selected
guarantees and their assumptions.

The selected source targets also include body proofs for cursor-relative
prefix helpers, Vec metadata packing and position updates, `BoundPtr`
projections, `set_visible_len`, `advance_vec_layout`, `advance_unchecked`, and
`BufMut::advance_mut`. `advance_unchecked` is proved only under its internal
`unique_valid`, capacity, and representable-position preconditions; that
contract keeps execution on the Vec no-promotion path.
The `BufMut::advance_mut` implementation body has a proof under its
cursor-validity and initialized-slot preconditions, but the generated trait
refinement obligation is not proved because those implementation preconditions
are stronger than the trait prestate.
The initial Vec constructors establish offset zero. No selected proof
establishes a safe public-call chain that produces a nonzero cursor, and the
promotion branch is outside this proof. The logical model and cursor helpers
are not evidence that those states are reachable through the original API.

The selected source targets include body proofs for the generic `OwnerLedger`
algebra and the current-value refcount operation wrappers and increment loop.
They also prove the one-shot `RefCountCell` bootstrap extraction body, which
moves its event permission and recorded scalar value out of the cell and leaves
the proof-only sidecar detached. No production `Bytes` or `BytesMut` path calls
that extraction yet; the selected scalar adapters still require the sidecar to
contain its invariant. The ledger and refcount proof state therefore remain
disconnected from production owner handles. These results do not establish
that the native refcount equals the number of registered handles, that an
observed count of one is exclusive, or that Shared storage may be reclaimed.
They are a partial direct-source checkpoint, not complete
`BytesMut::truncate` verification across all safe states.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
