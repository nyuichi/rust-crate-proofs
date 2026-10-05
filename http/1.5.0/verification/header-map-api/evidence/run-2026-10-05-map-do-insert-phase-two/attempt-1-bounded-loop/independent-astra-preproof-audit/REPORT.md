# Independent phase-two pre-proof audit

Verdict: **the bounded worker is ready for an initial proof attempt**. The source/COMA contract is non-vacuous and matches the one-ring mutation. The original actual-body VC splits into 25 children; three imported literal-true roots are separate. This report claims no prover success and does not accept any saved proof JSON.

This audit changes no source or COMA and starts no frontend or solver. `audit.py` verifies all 177 archived source hashes and the selected COMA hash, prints the complete unfiltered Why3 task stream, and uses the archived OCaml `replay.ml` to split the original `Coma.vc_do_insert_phase_two` once. It never reparses a printed Why3 task or transforms successful siblings. All commands, complete stdout/stderr, input hashes and full parent/child task renderings are archived in `task-audit.json` and the referenced compressed files.

Frozen Map is `56a201350fa1da592f7bc100a1cc983481ab883c7e900d3328584ca9b5fb82bb`; the selected COMA is `97d23992c74b1c4829532c962744b60336afe094ba3a02ebe5711b3fcf76304d`. Exact frozen Name and source-freeze hashes are in the ledger. The source-after-emission path/hash map equals the 177-entry source freeze. All archived compressed output streams and task renderings are byte-identical after decompression to the initial independent audit in `/tmp/map-phase-two-astra-ud64udn5/`.

## Original roots and independent split

| Root | Classification |
|---|---|
| `vc_do_insert_phase_two` | Actual bounded-loop body |
| `vc_len_Pos` | Imported literal-true support |
| `vc_is_none` | Imported literal-true support |
| `vc_replace_Pos` | Imported literal-true support |

The own root yields exactly 25 children under one `split_vc`. If a proof uses that transformation and closes every child directly, the complete COMA would have 28 successful terminal leaves including support. This is an arity observation, not a completed proof tree.

Useful zero-based child locations are: 14, decreasing variant; 18, preservation of an unvisited empty witness; 20, shifted prefix; 21, untouched suffix; 24, actual empty-slot exit postcondition. Child 7 covers the ordinary while-guard exit, which is unreachable from the empty-witness invariant. Children 5 and 6 contain the false arithmetic branch `to_int(0) != 0` created while splitting the initial carry invariant; that is a routine excluded branch, not a false function precondition. The remaining children cover initialization, arithmetic, slice bounds, length/frame, and carry propagation.

## Contract, termination and mutation shape

The caller premises are a nonempty slice, an in-range starting probe, and an empty slot somewhere among the first `len` cyclic offsets. `phase_two_slot` has a visible actual one-wrap Int definition. The function has no power-of-two, map-readiness, hash-coherence, or incoming-occupied premise.

The single canonical runtime progress value is `step`. The variant `len - step` decreases on every occupied backedge, and there is no separate wrap-only iteration. The invariant supplies an original empty offset at least `step` and below `len`, which prevents exhausting the entire ring. At an occupied current slot that witness must be strictly later, permitting the increment. The normalized slot expression is safe: `probe + step < len` in the first branch; the second branch subtracts `len - probe` from a value at least that large and remains below `len`.

The snapshot records the original sequence and incoming Pos. Visited original slots are nonempty; visited current slots contain the incoming item followed by the preceding original items; unvisited slots equal the snapshot; carry is the incoming item at step zero or the preceding original slot afterward. The emitted contract preserves both fields of every Pos exactly and preserves slice length. It reports the first empty offset as the displacement count. The postcondition for the shifted prefix and the untouched suffix has the intended quantifier and conjunction scopes.

A concrete wrapped witness is a four-slot table starting at slot 3: original slots 3 and 0 contain A and B, while slots 1 and 2 are empty. With incoming X, the result is 2 and final slots 3, 0, 1, 2 contain X, A, B, empty respectively. A one-slot empty table also satisfies the premises and returns zero. Incoming X may itself be empty; the current exact-shift contract remains valid, although an occupancy-increment consequence requires X to be occupied.

The rewrite is equivalent to the original shift loop under its stated premises. It bounds a previously unbounded loop. Calls that violate those premises are not covered: in particular, an out-of-range starting probe used to wrap to zero, whereas the new subtraction requires the in-range premise. Proving that actual callers establish the premise remains necessary before an integrated claim.

## Complete task context and warnings

The complete four-root printer stdout is retained without filtering. It contains no `RandomState`, `Hasher`, `hash_elem`, `build_hasher`, `finish`, `as_str`, or contractless `precondition` symbol. The only Hash-related type is the concrete one-field `HashValue(u16)` record stored in Pos. No hash semantics are used.

The HTTP-specific logical declarations are the visible slot definition, Pos/HashValue records, sentinel, and ordinary resolve predicates. Imported program summaries are slice length, the exact Pos empty test, and genuine `mem::replace` pre/future-value relations. There is no false external call precondition or new trusted HTTP axiom. The printed Bool `False` occurrences are standard overflow-Boolean definitions and ordinary control-flow cases. The saved stderr warnings concern existing Why3 integer extensionality declarations; the emitter's global hashing warnings do not propagate into this selected helper's task context.

## Local path to caller closure

Close this worker first without adding occupancy machinery to its runtime loop. A subsequent checked consequence can model empty slots as the existing finite set `E(S) = interval(0, S.len()).filter(empty-slot predicate)`. For an occupied incoming Pos, the exact shift implies `E(post) = E(pre).remove(consumed_slot)`. Existing FSet cardinality then gives a decrement of one; no additional HTTP axiom is needed.

This gives a direct route through mutations: clearing the table establishes `E.len = n`; rebuild after `j` entries maintains `E.len = n-j`; `entries.len < n` supplies space before every insertion and a remaining empty slot afterward. `try_insert_phase_two` can first consume a local two-empty-slots premise, then connect it to a proved representation/count invariant and reserve capacity. The existing lookup-readiness predicate alone neither constrains an empty map's table shape nor guarantees two empty slots.

Coverage and injectivity can be proved separately from the exact relocation: original occupied offset `k < result` moves to `k+1`, offsets after the consumed slot stay put, and the incoming entry occupies offset zero. These ranges are disjoint, and a fresh incoming entry index preserves uniqueness. Rebuild can maintain bounds/hash correspondence for the processed prefix while separately preserving original keys, values and links. These are subsequent local proof tasks, not dependency blockers and not claims of this pre-proof audit.
