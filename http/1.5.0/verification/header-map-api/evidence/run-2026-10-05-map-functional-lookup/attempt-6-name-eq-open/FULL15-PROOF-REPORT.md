# Attempt 6 Map lookup/getter batch — proof report

Verdict: **passed selected 15-target batch as conditional modular evidence**. Do not describe this as full `HeaderMap` verification or as completion of mutation/composition proofs.

## Fresh result

The bounded command was run with `--no-cache` through the HTTP proof wrapper (one prover, 1024 MiB), using the 15 COMAs archived in this attempt. Exit status was 0. All 15 fresh proof JSONs are archived, and their root names match the independently printed Why3 roots exactly. The batch contains 37 direct roots: 10 actual function-body goals, 5 trait refinements, and 22 literal-true imported support stubs. The tactics split these into 40 terminal leaves; all 40 are proved, with no null or unproved leaf.

`&str` `Sealed::find` now passes its actual body root. The task proves the delegating model equality equation is visible and reaches the text/parsed-header relation. The `String` and `&String` adapters also pass. This closes the prior attempt-5 open VC caused by an opaque `PartialEqModel::eq_model` bridge.

The 22 literal-true tasks are imported caller summaries and are not body proofs of the summarized callees. In particular, this batch does not prove the internal body of core `HeaderMap::find`, nor does it establish crate-integrated verification.

An independent read-only Astra audit reprinted and reconciled all 15 COMAs, 37 direct roots, and 40 successful terminal leaves, including the nested `&str` split. Its report is `independent-astra-functional-audit/REPORT.md` (SHA-256 `1f3c34dd6f4ee618c01af53b6dee8a84e61effca69d4f3d7dbb3d6d9012f999c`); the root/task audit JSON is `report.json` (SHA-256 `bd9ff46e1ef66219549fcd48b36ed0698f0f090e8cc6bba4d0465892ddf1e4c6`) and its source/context comparison is `context-report.json` (SHA-256 `45b39f0692b792f934500076e95504b63376793f71dee06b1dbb8ef142d69fbe`). Astra confirms conditional modular acceptance and the same API-contract limits below.

## Exported contract strength

| Target | What the proved target establishes |
|---|---|
| `HeaderMap::get2` | If `Some(value)`, there is an in-range stored entry with that value and a key matched by the query. `None` has no absence-completeness postcondition. |
| Public `HeaderMap::get` | Its body and call obligations pass under `header_map_find_ready`; the public method currently has no functional `ensures`. |
| `HeaderMap::get_mut` | Its body and mutable-index/type-invariant obligations pass under readiness; there is no value prophecy, frame, or readiness-preservation postcondition. |
| `HeaderMap::get_all` | `Some(index)` implies an in-range matching key. The contract does not identify the returned view's map with the input or prove iteration/absence behavior. |
| `HeaderMap::contains_key` | `true` implies an in-range matching entry. `false` does not imply absence. |
| `Sealed::find` for `HeaderName`, `&HeaderName`, `&str`, `String`, and `&String` | Each selected body and corresponding trait refinement passes. The string adapters are modularly conditional on the selected `&str` contract and imported parser/core-lookup contracts. |

The `get` and `get_mut` body successes must not be described as functional API verification. Adding functional posts for public `get`, a prophecy/frame contract for `get_mut`, and a returned-view/map relation for `get_all` remains follow-up work.

## Source and evidence boundaries

The selected Map source is `9e2dbdc382acb0375d3d8592784077799386f75dc252205658759b2604610edf`; selected Name source is `bc3be7d995c314b74f5aaca5b7259f9b7afd9dcd846b528264be1c43eb3ad5ca`. All 177 captured source files were rehashed after the proof with zero changes and zero missing files. The successful native `http 1.5.0 --lib` check is separately recorded in attempt 5 against these exact Map and Name hashes.

The frozen Name file also contains its StandardHeader absence-helper work. This Map batch does not prove that helper's body. Map append/mutation/rebuild/grow, actual map composition, mutable framing beyond current body safety, and complete lookup absence semantics are outside this batch. Prior attempt-5 artifacts remain preserved unchanged.
