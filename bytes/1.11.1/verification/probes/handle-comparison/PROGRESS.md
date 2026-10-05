# Handle comparison and concrete iterator proof progress

Comparison changes include one small production fix plus its native regression;
generated Creusot proof models remain confined to this probe.

## Established results

- Semantic adapters: `logs/adapters-positive.log`, **81 proof files passed**.
  `evidence/adapters-positive/manifest.json` hashes the exact generated source,
  contracts, tasks, and proof sessions. Exact `BytesMut::from_vec` and
  `as_slice` bodies connect the checked initialized handle to its `Seq<Int>`
  view. Four unchanged self/slice adapter bodies prove exact equality and
  lexicographic order. Constructor/comparison/explicit-cleanup caller proved.
  This is the archived earlier proof snapshot: its manifest's `build.rs` hash
  predates the later native-only vector test extraction, so the current probe
  has not yet been replayed into a matching manifest.
- Native all-features: `logs/native-final-vector-regression.log`, **4 tests
  passed**. Covers 36 byte-pair comparisons, UTF-8 strings, concrete slice
  iteration, and the exact extracted heterogeneous `Vec<u8>` to `BytesMut`
  partial-order source body. This last case is a native check, not a Creusot
  trait-refinement proof. The production crate regression at
  `../../tests/test_comparison.rs` also passes in
  `logs/native-vec-order-fixed.log`.
- The production library also passes `cargo check --no-default-features --lib`
  in `logs/no-std-library-check.log` after the comparison fix.
- The suspected defect was confirmed by the baseline
  `logs/native-vec-order-counterexample.log`: `[0] < BytesMut([1])` returned
  `Greater`. The production `Vec<u8>` implementation now compares the left
  slice directly with `BytesMut::as_slice()`. Regression cases check equality,
  opposite byte orders, unequal prefixes, and both heterogeneous operand
  directions.
- Concrete iterator initial attempt: body/accessor/size/law proofs succeed;
  one `next__refines` Some branch fails for missing sequence decomposition
  guidance. `iterator/evidence/iterator-refinement-before/` preserves it.
  The subsequent `sequence_head_tail` logical lemma itself proves, but an
  unused reference in `produces` is not exported into refinement dependencies;
  `iterator/evidence/iterator-lemma-not-exported/` preserves that attempt.
- Actual native self trait implementations translate successfully. Initial
  unannotated methods fail three standard refinement files because their own
  inferred contracts promise no result relation; this is expected and is
  preserved in `evidence/traits-missing-contracts/`. The candidate now adds
  explicit semantic ensures to the exact bodies, without stronger method
  preconditions or trust.

## Scope and trust review

The type invariant covers unique initialized storage at offset zero and
canonical empty handles. Shared registrations, advanced handles, automatic
Drop, and lifecycle/concurrency preservation are not covered. Unknown slots
map to zero in the *total* model but this never supplies Known initialization.

The optional readonly gate adds ghost purity to the existing immutable
physical read interfaces only. Their exact bodies, permission requirements,
and lifetimes are unchanged; neither `borrow_bound_mut` nor any physical
write becomes ghost-callable. This is an explicit extension of the existing
trusted immutable boundary, proposed only in generated probe copies. Kind
and address calculations are body-checked. No Send/Sync or Deref ownership
claim is trusted.

The string adapter feature has one precise standard-library extern contract:
`str::as_bytes` returns the modeled UTF-8 encoding. Actual heterogeneous
string trait refinement is not claimed because the stock string deep model
is a character sequence while byte handles use integer byte sequences.

The concrete iterator specializes `IntoIter<T>` to `T = &[u8]`. Rebinding Buf
calls to exact concrete slice methods is inverse-checked. No generic Buf
laws or iterator specification laws are trusted.

## Running follow-ups

The strengthened `next` postcondition states both exact suffix advancement
and singleton-prefix concatenation. The pinned current-source replay passed
**22 proof files**; `iterator/evidence/semantic-next-positive/manifest.json`
hashes the exact generated source, Why3 tasks, proof JSON, and external bytes
source fragments. The earlier failed decomposition attempts remain preserved
under `iterator/evidence/`. A first replay in the reconstructed environment
stopped before proving because that environment had an incompatible
`why3find`; the pinned toolchain was then repaired and the final replay passed.

`logs/readonly-str-traits.log` records a completed **100-file** proof run for
the readonly comparison and string adapter configuration. The independent
`actual-traits` run remains incomplete: `logs/traits-positive.log` has four
failed refinement obligations in the self equality/order methods and caller.
