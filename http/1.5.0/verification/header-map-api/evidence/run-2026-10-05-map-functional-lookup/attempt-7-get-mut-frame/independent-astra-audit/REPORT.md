# Independent attempt-7 contract and task audit

Verdict: **the new getter contracts have a sound, non-vacuous shape; no blocking precondition or translation defect was found**. This report records source/COMA review and solver-free task printing only. It does not claim that the new body obligations have been proved or reconcile any proof JSON produced afterward.

The audit edits no source or COMA and invokes no prover. `audit.py` verifies the hashes of all 177 frozen sources and all 13 selected COMAs, prints each complete Why3 task stream without filtering or preprocessing, and archives full stdout/stderr plus commands and hashes in `task-audit.json`. Every `.stdout.gz` and `.stderr.gz` contains the full original stream, compressed without textual normalization.

Frozen Map is `bfdb950399fc2612c4a4f2135a876922d052a921b5b1c19cc685e080b0c8d9a8`; frozen Name is `ed50c0e4bf945279fe497399e86ebbdbcf7f2a9a8aa58f7a1aab01ba5a6511ed`. The get_mut COMA hash is `0a5f4d0c1e83b1be6ebdbf722d9bdd304655f78ee5c274c74ef8f16d61c1f18d`. Its independently printed complete stdout hash is `5b2abf633f209ce1644891ec867ac00a97668030f28ee0a0144d02041ceaf21b`.

## Direct task inventory and context comparison

There are 13 selected COMAs and 30 direct roots: 8 actual function-body roots, 5 trait-refinement roots, and 17 imported support roots. Each support root was classified only after checking that its full formula is literally `[@coma:solid] true`.

The entire raw stdout streams for 11 of the 13 targets are byte-identical to the accepted attempt-6 streams, without normalization. Only public `get` and `get_mut` change. The unchanged set consists of `get2` and all five selected Sealed find implementations with their refinements. Prior-stream references and hashes are retained in `task-audit.json`.

`get_mut` has four direct roots: `vc_find_K`, `vc_elim_Some`, `vc_index_mut_Vec_Bucket_T_Global`, and its actual body root `vc_get_mut_T`. No nested transformation was requested or replayed in this audit; terminal proof-tree reconciliation is a separate step after the proof run.

## New contract shape

Public `get` now exports the existing `get2` relation: a returned value corresponds to an in-range stored entry whose key model matches the query. Its `None` result still makes no absence claim.

The `get_mut` contract contains two independent explicit postconditions. First, `header_map_find_ready(^self)` holds. Second, a `Some(value)` result has one common index that is in range in both the original and future maps, matches the query against the original stored key model, relates `*value` to the original stored value, relates `^value` to the future stored value, and identifies that same index as the sole entry value allowed to change by the frame predicate. A `None` result receives the frame with no selected index, so every stored entry value is preserved.

Here `^self` and `^value` mean their prophecy/future values when the borrow resolves. This is a relation through the returned mutable borrow; it makes no physical-address claim.

The public opaque `header_map_get_mut_frame` has an actual pure definition. It preserves the mask, Danger value, table length and every slot's index/hash, entry count and every stored bucket hash/key model/link structure, extra-value count and each extra value/link model, and every unselected entry value. Each quantifier is separately parenthesized. The independently emitted COMA confirms that the following clauses remain outside preceding quantifier scopes. Future readiness is also a separate top-level postcondition, so it is not accidentally vacuous when the entry vector is empty.

The frame's HeaderName clause preserves `deep_model()` equality, not identity of the entire runtime HeaderName representation. Report it as preservation of key models. The Link model visibly distinguishes `Entry(index)` and `Extra(index)`, and the explicit Links comparison preserves both offsets and the Option variant. Generic `T` values are compared directly; no `T: DeepModel`, PartialEq requirement, or identification of unrelated model types is introduced. Danger equality is ordinary logical equality and requires no executable PartialEq implementation.

## Why the contract is supported by the current interfaces

The emitted Vec IndexMut summary supplies original selected-bucket value, future selected-bucket value, equality of every other bucket, and unchanged vector length. The existing body translation borrows Map.entries, then the selected bucket, then only its value field. Its record updates preserve the surrounding fields while propagating the returned borrow's future value into that selected field. The proposed existential witness is the runtime `found` index. The None branch resolves the map without mutation.

The frame preserves every component read by the readiness predicate: mask, index-table contents and length, entry length, and stored hashes. Empty readiness follows from length preservation. For a nonempty map, the original power-of-two witness and empty-slot witness remain valid; occupied-slot bounds and matching hashes transfer through the preserved fields. No value or link property is needed to obtain readiness, and no prophetic invariant is called by the pure frame definition.

These are interface and translation observations, not a substituted proof result. The new actual `get_mut` body VC must still be closed with its strengthened contract.

## Non-vacuity and limits

The selected preconditions remain ordinary type invariants and the existing readiness premise; there is no new false or stronger caller premise. For example, choose `T = u32`, mask 3, four table slots, two stored entries, slots 0 and 1 pointing to them with matching hashes, empty slots 2 and 3, empty extra storage, valid keys, and `Danger::Green`. Select entry 1, change its value from 7 to 9, and preserve entry 0's value 3 and all structure. This satisfies the proposed frame and both original and future readiness predicates. Thus the frame does not accidentally forbid changing the returned value. The None case can preserve the entire map, and empty maps remain admissible.

The `false` terms in the frame's mismatched Option variants and Option eliminator express excluded cases; they are not global hypotheses or external call preconditions. There is no new trusted HTTP axiom. The unchanged Sealed contexts retain the previously repaired visible Name equality bridge. Warning-bearing RandomState construction and hashing bodies remain unselected, as in attempt 6.

The guarantees remain modular with respect to the selected Sealed/core lookup contracts and genuine standard-library specifications. They do not establish negative lookup completeness, constructor/mutation invariant closure, key uniqueness, physical-reference identity, all-values iteration, hash/equality coherence, termination, or an integrated HeaderMap proof. This audit deliberately omits `get_all`, `contains_key`, and other mutation targets from its selected set.
