# Independent append_value audit

Verdict: **accept the actual `append_value` body as proved relative to its recorded standard-library contracts and local tail-bound premise**. The five-root batch contains one actual body root and four imported literal-true support roots. It does not establish other mutation bodies or global HeaderMap invariants.

This audit ran no solver and edited no implementation source. `audit.py` verifies all 177 source files in the inherited attempt-6 freeze, the separately archived Map/Name source bytes, invocation-context source bytes, COMA identities, proof JSON and log hashes, and the proof exit status. It independently prints both complete task streams with Why3, without selecting a prover or applying a transformation. Full compressed stdout/stderr, commands, source diffs, and hashes are recorded alongside `report.json`.

## Source and task reconciliation

The proof target is archived COMA `9e78217e81492788561261720835d27cce4ea7f8ee412abaa69c13cee9977f74`, emitted from Map `9e2dbdc382acb0375d3d8592784077799386f75dc252205658759b2604610edf` and Name `bc3be7d995c314b74f5aaca5b7259f9b7afd9dcd846b528264be1c43eb3ad5ca`. The inherited source-freeze hash is `a3b1e4fbee1d42af4145680ee345c3a989d43a9eb05050771b56d01e0e82c899`.

The successful package invocation compiled Map `91dd661a5e40e9c15ace2ddd53bd64b50b1de21d04c785558658b2d8c296e202` and Name `ed50c0e4bf945279fe497399e86ebbdbcf7f2a9a8aa58f7a1aab01ba5a6511ed`. Map differs only by the public `get` postcondition. Name contains separate comparator implementation work, which must not be described as a mere source-coordinate change. Nevertheless, the entire `append_value` contract and body slice is byte-identical between the two archived Map sources, and **the complete printed task streams of the archived COMA and invocation-context regenerated COMA are byte-identical without any normalization or goal-only filtering**. The latter COMA has hash `40406433a410b276248d1a0619c4bd6b25d3db2dc89a262778fad854cc0263c6`. Thus the separate Name changes introduce no task-context change for this target.

The unique ignored `append_value_resume.coma` target was prepared by a checked byte-for-byte copy of the archived COMA; its command, proof-output origin, result, and cleanup are recorded by the owner. The initial outside-`verif/` invocation failed before starting any VC and contributes no proof result. The accepted run exited 0.

## Independent root check

| Direct root | Classification | Recorded outcome |
|---|---|---|
| `vc_append_value_T` | Actual checked body | Successful direct prover leaf |
| `vc_elim_Some` | Imported literal-true support | Successful direct prover leaf |
| `vc_index_mut_Vec_ExtraValue_T_Global` | Imported literal-true support | Successful direct prover leaf |
| `vc_len_ExtraValue_T` | Imported literal-true support | Successful direct prover leaf |
| `vc_push_ExtraValue_T` | Imported literal-true support | Successful direct prover leaf |

The independently printed root set equals the fresh proof JSON root set exactly. All five nodes are terminal successful prover records; there are no nested transformations, null children, or hidden additional roots. Each support classification was checked against its complete formula `[@coma:solid] true`. The actual function is emitted as a checked `(! bb0 ...)` body, with both `Some(links)` and `None` branches present.

## What the body proves

For a valid borrowed entry and extra-value vector, with `links.tail < extra.len()` when links exist, the eleven explicit postconditions establish:

- Extra storage gains one element carrying the input value and the specified entry/previous-tail links.
- The entry's original hash, key, and first value remain unchanged; its link metadata selects the new tail, preserving the existing head or creating the first head as appropriate.
- Every original extra value and previous-link field is unchanged. Only the previous tail's next-link field changes when extending an existing list; all other next links are preserved.

These facts describe the actual local updates. They do not require an abstract model for generic `T`: its actual values are preserved and compared. `Link::deep_model` is a visible, inlined definition distinguishing `Entry(index)` from `Extra(index)`; no opaque-model bridge is missing.

The `Some` precondition is satisfiable, for example with one extra element and tail zero; the `None` case allows an empty vector. There is no selected impossible external precondition, false assumption, trusted HTTP axiom, or generic-type collapse. The `false` branches in the Option eliminator and link postcondition are ordinary excluded cases, not contradictory global hypotheses. The body uses genuine Vec length, push, and indexed mutable-borrow specifications; their caller support roots do not prove those library bodies.

The premise gives only a bounded tail. It does not assert entry ownership of the whole chain, acyclicity, link reachability, disjointness between chains, index-table coverage, or map-wide readiness. Those properties require separate caller/global-invariant closure. The proof also does not establish hash/equality coherence, insertion/growth/removal algorithms, or an integrated actual-Map proof. This accepted helper is a concrete basis for continuing that local work.
