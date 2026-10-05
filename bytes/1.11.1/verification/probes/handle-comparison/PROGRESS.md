# Handle comparison and concrete iterator proof progress

All changes in this increment are confined to this new probe. Production
source files remain owned by the other integration tasks.

## Established results

- Semantic adapters: `logs/adapters-positive.log`, **81 proof files passed**.
  `evidence/adapters-positive/manifest.json` hashes the exact generated source,
  contracts, tasks, and proof sessions. Exact `BytesMut::from_vec` and
  `as_slice` bodies connect the checked initialized handle to its `Seq<Int>`
  view. Four unchanged self/slice adapter bodies prove exact equality and
  lexicographic order. Constructor/comparison/explicit-cleanup caller proved.
- Native all-features: `logs/native-final.log`, **3 tests passed**. Covers 36
  byte-pair comparisons, UTF-8 strings, and concrete slice iteration.
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
and singleton-prefix concatenation. Its exact runtime body remains unchanged.
The combined readonly self/cross-slice traits and string adapters are queued
under the common prover lock. Results will be appended after completion.
