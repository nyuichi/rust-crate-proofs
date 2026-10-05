# Independent attempt-7 proof reconciliation

Verdict: **ACCEPT for the frozen selected modular getter contracts.** All 30 direct roots have complete saved proof trees, with 43 successful terminal leaves and no null leaves. The only transformation node is independently reproduced from the original COMA and has exactly the recorded 14 children. This audit invokes no solver, frontend, or source mutation.

Run `python3 audit.py` from any directory to reproduce the checks. It compiles the archived `replay.ml` against the installed Why3 OCaml API in a temporary directory. All command arguments, return statuses, input hashes, complete unfiltered printer stdout/stderr, full nested task renderings, and exact proof JSON copies are retained in `task-audit.json` and its referenced files. Compressed files preserve the original bytes; no textual normalization is applied.

## Frozen input and complete-context identity

The script verifies all 177 archived source hashes and the exact hashes of all 13 selected COMAs. Frozen Map is `bfdb950399fc2612c4a4f2135a876922d052a921b5b1c19cc685e080b0c8d9a8`; frozen Name is `ed50c0e4bf945279fe497399e86ebbdbcf7f2a9a8aa58f7a1aab01ba5a6511ed`. The `get_mut` COMA is `0a5f4d0c1e83b1be6ebdbf722d9bdd304655f78ee5c274c74ef8f16d61c1f18d`.

Each original COMA is independently printed with the Why3 driver, without preprocessing or a prover. All 13 complete stdout streams are byte-identical to those archived by the earlier [contract and non-vacuity audit](../independent-astra-audit/REPORT.md). That audit's soundness analysis, concrete mutable-value witness, and source-to-COMA observations therefore apply to exactly these proof tasks. Its report and machine-readable record are linked by SHA-256.

The direct roots consist of 8 actual function-body roots, 5 trait refinements, and 17 imported support roots whose whole goal formula is literally true. For every COMA, the independently printed goal-name set equals the saved `proofs.Coma` key set exactly. Every proof leaf records Z3 4.15.3 or cvc5 1.3.1 success; no unmatched, null, or zero-child tactic node is accepted.

## Transformation reconciliation

`get_mut.coma` contains four direct roots. Three are literal-true imported support (`vc_find_K`, `vc_elim_Some`, and `vc_index_mut_Vec_Bucket_T_Global`). The actual body root `vc_get_mut_T` has `split_vc` with **14** children, all terminal successes. Its library display of **17** includes those three support roots; it is not a split into 17 children.

The OCaml audit reads the original COMA, selects its unique `Coma.vc_get_mut_T`, and applies `split_vc` once. The independently observed arity is 14, matching the saved proof tree. All complete parent and child tasks are archived. No printed Why3 file is reparsed, and no successful sibling is split again. There are no deeper transformation nodes in this batch.

Child index 12 contains the actual `get_mut` postcondition, including the future-map readiness predicate and the result/frame/prophecy relation. Its saved success uses cvc5 1.3.1. The other 13 children cover the call preconditions, Option elimination, and type/borrow invariants and have saved Z3 successes. All other original roots are direct terminal successes. Thus the total is `29 + 14 = 43` terminal leaves.

Classified terminal totals are 21 actual-body leaves, 5 refinement leaves, and 17 literal-true support leaves. Prover totals are 41 Z3 and 2 cvc5 successes: besides `get_mut` child 12, the actual `get2` root also uses cvc5. No imported literal-true support is counted as an implementation proof.

The direct Why3find batch log shows all 13 libraries successful and exit status zero. It uses the shared resource wrapper and the exact frozen archive COMAs without Cargo re-emission. The same `get.coma` had a successful direct pilot before the batch; the batch starts with progress `!2/?28`. Both logs are retained. We claim complete results for the same frozen inputs, not that every stored terminal result originated in the later full batch rather than that pilot.

## Accepted guarantees and limits

Public `get` and `get2` establish that a returned value corresponds to an in-range entry with a matching key model. Public `get_mut` additionally relates the initial returned value to that entry's initial value and the returned borrow's future value to the same entry's future value. It preserves the other stored values and the stated structural frame, and separately ensures `header_map_find_ready(^self)` when the borrow resolves. A `None` mutable result frames every stored value. These are generic `T` values, not a collapsed replacement model.

The frame preserves key models rather than exact runtime HeaderName representation. It does not claim physical pointer identity. `None` does not establish key absence. The source precondition remains the existing readiness premise and ordinary type invariants; the earlier non-vacuity witness permits the selected value to change from 7 to 9 while another value stays 3.

These are conditional modular body proofs against the emitted callee contracts and genuine standard-library interfaces. They do not close the actual core lookup/parser/hash bodies, constructor or insertion readiness, hash/equality coherence, key uniqueness, negative lookup completeness, or whole-crate verification. `get_all`, `contains_key`, and mutations are outside this 13-COMA selection.

Native integration tests are separate execution evidence and do not change this frozen proof-source claim. In particular, the attempt's `native-test/source-snapshot.json` records Map `bfdb9503...` with the later Name `271a0780...`; later native runs on other live revisions are likewise not proof-input substitutions.
