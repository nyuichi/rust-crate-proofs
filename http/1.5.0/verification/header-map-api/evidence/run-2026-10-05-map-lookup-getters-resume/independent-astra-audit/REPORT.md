# Independent Astra audit of the resumed Map lookup batch

**Decision: accept as conditional modular body/refinement evidence.** It is not
a proof of complete HeaderMap lookup semantics, a closed call graph, or the
integrated HTTP crate. No selected-context impossible precondition, false HTTP
axiom, generic type collapse, or erased named logical setter was found in this
audit. This is a source/context review, not a formal consistency certificate for
the complete Creusot/Why3 trusted base.

The audited snapshot is the frozen run `sources/` and `artifacts/`, not the live
worktree. `map.rs` SHA256 is
`ca22bfb5673eba84281bf3b1738e831a15b94cfcb99b0f8e16cab4fa46f9ce6e`;
`name.rs` SHA256 is
`8c4633906a8ecd6e30e7eb75f8ace00c9b9f89d5e5f4c325f7e6b10a4eee94d6`.
All 177 frozen source files match `source-freeze.json`, whose SHA256 is
`2d458cedc823f93f7c760905426ddc15fdb3687365707de578ca0122fd332b79`.

`audit.py` prints all 13 archived COMAs with `why3 prove -D why3`, without
preprocessing and without selecting a prover. The complete stdout and stderr
streams are retained alongside this report; `task-audit.json` records each
COMA, proof JSON, and full-stream hash. All 43 independent initial goal names
match the corresponding proof JSON keys. Every proof root is a direct successful
prover leaf; there are no nested or null nodes requiring a split audit.

The count is **10 actual function body goals, 3 trait-refinement goals, and 30
imported support stub goals**. All 30 support goals print literally as
`[@coma:solid] true`. They do not establish the imported callees' bodies.

## What the body goals establish

| Selected target | Established contract and limit |
| --- | --- |
| `HeaderMap::find_with_hash` | Under `header_map_find_ready` and a nonempty entries vector, the body respects its local safety obligations. `Some((probe,index))` has in-range positions, matching slot/bucket/input hashes, and the supplied key's `PartialEqModel` relation. `None` has no semantic postcondition. |
| `HeaderMap::find` | Under `header_map_find_ready`, the body establishes in-range `Some` positions, matching bucket/slot hashes and the generic key-model relation. The hash helper's summary supplies an arbitrary `HashValue`. |
| `get`, `get2`, `get_mut` | Local call/index/type-invariant obligations under `header_map_find_ready` and the sealed key callback contract. There is no functional result postcondition relating a value to its key or first insertion. `get_mut` does not export a map frame or readiness-preservation postcondition. |
| `get_all`, `contains_key` | Local call/type-invariant obligations under the same premise. No complete returned-view, membership, or absence theorem is exported. The `GetAll` type invariant does not itself require the stored index to be in bounds. |
| `Sealed::find` for `HeaderName`, `&HeaderName`, `&str` | The bodies establish only `Some` index bounds at this interface. Three refinement goals show that these implementations satisfy the trait's identical bounded-index contract. |

The emitted main bodies contain checked program calls (`! bb0`), including the
loop body. The loop's `false` assertion is the failed `debug_assert!(len > 0)`
branch, whose infeasibility follows from the explicit nonempty-ready premise;
it is not a globally assumed false condition. The two closure definitions are
normal `coma:extspec` callback summaries. Their printed pre/post predicates
retain the actual field projection / Map-call requirements and result relation;
they are not the quarantined named-string logical-setter construction.

## Preconditions and vacuity

The ready predicate has ordinary nonempty witnesses. For example, choose
`mask = 1`, two slots, one entry with a valid HeaderName/value and hash zero,
slot 0 pointing to entry 0 with hash zero, slot 1 with `index = 65535`, empty
extra values, and `Danger::Green`. Then `k = 1` witnesses the power-of-two
condition and slot 1 witnesses emptiness; every occupied slot is in bounds and
has the matching hash. This also satisfies the additional nonempty premise.
This is a direct source-predicate satisfiability witness by inspection, not a
new solver result or a proof that a constructor creates that representation.

The predicate is deliberately weaker than a full Map invariant. Empty entries
make it true regardless of mask/indices, and a nonempty entries vector with all
slots empty also satisfies it. The latter can hide an existing bucket from a
lookup, demonstrating why `None iff absent` does not follow. Neither slot
coverage, Robin Hood probe ordering, key uniqueness, hash/equality coherence,
extra-value link closure nor constructor/mutation preservation is established.

Generic `T`, generic key `K`, and generic key model remain separate abstract
types in the relevant COMAs. No axiom equates arbitrary inhabitants. The
`HeaderName` model/view equation only connects its two observers, and does not
force arbitrary generic values to share a model. Nonempty witnesses are
consistent with the displayed invariants; no vacuity from these premises was
identified.

## Imported contracts and frontend warnings

`find.coma` represents `hash_elem_using_K` as `any`, with only the actual
type-invariant preconditions and no result property beyond returning
`HashValue`. In particular there is no `false` precondition and no asserted
hash/equality coherence. The `vc_hash_elem_using_K` goal here is the trivial
imported stub, not the hash helper's body. Its source body is outside this
selected batch: the Red path still calls external `build_hasher` and `finish`
without contracts. Those warnings are a call-graph closure gap; they do not make
the selected Map caller's proof context contradictory. The selected body proof
must remain conditional on its callees and generic callback contracts.

The `RandomState::new` warning occurs in unselected `Danger::set_red`. The
`String::as_str` warning occurs in unselected `Sealed::find for String`; neither
that implementation nor `&String` is covered by this batch. The selected `&str`
implementation instead uses the standard `str::as_bytes` contract and the actual
`HdrName::from_bytes` callback contract. This batch does not reprove the parser;
its exact source/proof reconciliation remains separate. The bytes dependency is
accepted under the user's premise. Existing standard-library contracts and
Creusot's scalar/string model remain the trusted base, including the recorded
U+10FFFF character-model limitation.

No `#[trusted]` HTTP boundary or new HTTP `extern_spec` was used by these
selected targets. Ordinary modular summaries for locally defined HTTP callees
must still be discharged separately. Imported `desired_pos`, `probe_distance`,
`Pos::resolve`, equality refinements and Name parser evidence are not body
proofs merely because their support stubs appear here.

No selected function requests `check(terminates)` or supplies an explicit
termination variant; do not promote these results to a total-correctness or
termination claim.

## Next concrete work

1. Add checked public functional observers/postconditions through the sealed
   lookup interface: found key/value relation, returned `GetAll` map/index, and
   mutable-reference frame/readiness preservation. Preserve the explicit
   absence limitation until a stronger representation invariant is proved.
2. Close the actual hash helper and the omitted String key paths using genuine
   standard-library contracts or independently proved helpers. Do not replace
   them with a trusted HTTP summary.
3. Prove constructor/mutation closure for a representation invariant that also
   supports slot coverage, probe order and hash coherence before claiming full
   lookup correctness or using such a summary in `Builder::header`.

This audit launched no solver and modified no HTTP source file.
