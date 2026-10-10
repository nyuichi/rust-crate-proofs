# Independent audit: URI equality field conjunction

Audit type: read-only independent review by Astra. No source edits and no solver execution were performed during this audit.

## Own equality proof

The frozen body COMA is `comas/uri_eq_body.coma` (SHA-256 `6a2f938cddeeb5fa5edd566a7d9334c2fe374843611a3c91e6699faf40376d08`); the trait-refinement COMA is `comas/uri_eq_refines.coma` (SHA-256 `3fbd2e706684671a5db513965ee771d0a3a6df1f1522c06266460cfbc1401176`). The body has 26 direct terminal children, and refinement has 4. All 30 proof JSON terminals are Z3 4.15.3 successes: 26 + 4 passed, nulls 0, nested splits 0, maximum reported solver time 0.057 seconds. The captured bounded proof log is `proof-run.log` (SHA-256 `c8d753c7ae3e65ecc13bfa99dd81ea3e728f910f260fa98e41b7ddcb2cfa9618`); recorded command exit status is 0.

Astra independently re-ran Why3 `split_vc -D why3` task printing for the frozen COMAs. All 30 complete `.why` contexts, including the unsuffixed child 0 tasks, matched the archived context bytes. The own `arity/*.stdout` files come from emission with `-o` and are empty; they are not treated as complete Why3 task streams. The independent task printer also captured the full body stream (2,221,527 bytes, SHA-256 `233374ec55fe838d053d83aeff7f7a19a92aace49932b032f476cc63a6101407`) and refinement stream (280,358 bytes, SHA-256 `b1dead9a8bc313ec222617130876d66058f917b2730ce2f3b62679065ec43d5a`); these are audit measurements, not substitutes for the archived `.why` contexts or bounded solver logs.

The source freeze covers 63 files total: 51 project inputs and 12 direct Creusot standard-library dependency inputs. The source hash manifest SHA-256 is `a51310f94e8e79d28a8ac9ce6fde9cb00a505b8f2b52be76050ddc319c3dbe63`. The audited frozen `src/uri/mod.rs` hash is `7224a7fab92245bef2145a7e2304f395b3a7d99e8faa8739684b4e07ebc4393a`. All frozen inputs matched the archived snapshot, and the 288 evidence artifact hashes in the manifest at audit time matched. After archiving the six fresh COMAs and this audit report, the URI owner regenerated the final inventory: 295 evidence files, all 295 hashes match, with no missing, stale, or mismatched entries.

The contract change is equivalent to the original `UriCompareModel` record equality: the model has exactly the four fields `scheme`, `authority`, `path`, and `query`; `uri_models_equal` compares the record values. The explicit conjunction compares those same four fields. The trait-refinement COMA still checks the original record-equality contract. Runtime code, model definitions, helper semantics, and axioms are unchanged by this contract change. No serialized or textual equality was introduced.

The fresh proof JSON contains nine zero-child roots, excluded from the 30 owned obligations: four local HTTP callee-contract roots (`scheme'1`, `authority'1`, `path`, `query'1`), four standard-library boundary roots (`str::as_bytes`, `eq_ref_slice_u8`, and the two Option reference inequality contracts for Authority and Scheme), and one translator root (`elim_Some`). These are modular support contracts, not callee-body proofs. The scheme and authority DeepModel definitions are visible in the parent URI module, and byte equality uses the existing standard-library view-transport contract. No new trusted HTTP lemma or axiom was added.

## Carried-over exact obligations

Astra independently reprinted the old and fresh current-source COMAs for six obligations. Every old/fresh complete raw Why3 stdout stream matched byte-for-byte without normalization; input COMA hashes, stream hashes, byte counts, commands, and stderr diagnostics are recorded in `carryover-six-current-source/full-stdout-comparison.json`. The six stream byte counts and SHA-256 values are:

| Obligation | Bytes | SHA-256 |
| --- | ---: | --- |
| From Authority body | 68,070 | `6c88bf508901aac11c6c99767e56ed6bab01700f60e4ba31954419ccb7cd25cb` |
| From Authority refinement | 133,144 | `6f051451d31ac8ee30fef6ab6459ab68ca31dbffefc43a94c5fcb320d033e666` |
| From PathAndQuery body | 67,746 | `02049b654d2bdbc7d42e80540efdd966c0ed19105adfe23e6c08e281eb0463ad` |
| From PathAndQuery refinement | 133,148 | `bd4c5256757a8e949d64acdede68189b5fc8c11e5066997ab657c741116a72f4` |
| Debug body | 288,717 | `e96832e032a587157a5e27eb4522790006da90dc81603dc6ee3ebfb67a08d4aa` |
| Debug refinement | 214,199 | `377f0cdc8ba1dfe2f9f1dfc3d13f0db30fa47de15aca5c2d2f62f39cd9a0e804` |

Prior proof JSON root assignments give From Authority 1 body + 2 refinement terminals = 3; From PathAndQuery 1 + 2 = 3; Debug 4 + 3 = 7. Thus the six carried-over exact obligations account for 13 prior terminal proofs. They are not added to the current Eq proof's 30 leaves. The current Display (48 + 3 direct tasks) and Uri-to-str equality (49 + 3) targets are source-bound clean-emission and solver-free arity preparations only; neither has a solver proof in this package.
