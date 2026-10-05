# Independent phase-two completed-proof audit

Verdict: **ACCEPT for the frozen conditional bounded-loop worker body.** The exact four original roots match four direct successful Z3 4.15.3 proof leaves. One is the actual body; three are literal-true imported support. The saved proof tree contains no transformation node, null leaf, missing root, or additional root.

This audit invokes no prover or frontend and changes no source or COMA. Run `python3 audit.py` to reproduce it. Complete unfiltered printer stdout/stderr, four complete individual task files, an exact copy of the proof JSON, input hashes, command arguments, and proof invocation/log/status references are retained in `task-audit.json` and its referenced files.

## Frozen provenance and full-context identity

All 177 frozen source hashes match. The path/hash sets in the source freeze and source-after-emission snapshot agree, and the latter reports zero changed sources. Map is `56a201350fa1da592f7bc100a1cc983481ab883c7e900d3328584ca9b5fb82bb`; Name is `fa9742ee1a16f3967fcf00338c5e6a36cfa98a41c922faaa3738aff004d6ca7d`. The source-freeze SHA-256 is `13a77d2d055dabd484699e39dbbd1e9a1b73e8962f232b65c12ae0f761c8bb1e`.

The original COMA remains `97d23992c74b1c4829532c962744b60336afe094ba3a02ebe5711b3fcf76304d`. Its complete 146874-byte printed task stream is byte-identical, without normalization, to the [pre-proof context audit](../independent-astra-preproof-audit/REPORT.md). The raw stdout SHA-256 is `7f1dce1e3e8318a5799b7d52ee19b9752ffede42224cbee48f92954835051cce`. That audit's full-context, quantifier-scope, and non-vacuity findings therefore apply to these exact successful tasks.

The owner's `direct-task-print.json` is separately hash-linked. Its worker full stdout and all four complete individual worker task files also match this independent printing byte for byte. The two caller COMAs inventoried in that owner record are outside this completed-proof audit.

The successful proof invocation runs direct Why3find on the frozen archive COMA through the shared resource wrapper. It uses `--no-cache`, no preprocessing, and no Cargo re-emission. Its log reports `✔ (4)` and exit status zero. Source stability above concerns emission; the later solver invocation directly consumes the preserved COMA rather than recompiling live source. Both COMA and proof JSON remained unchanged during this independent reconciliation.

## Actual successful proof tree

| Original root | Classification | Saved leaf |
|---|---|---|
| `vc_do_insert_phase_two` | Actual body | Z3 4.15.3, 0.07 seconds |
| `vc_len_Pos` | Imported literal `true` support | Z3 4.15.3 |
| `vc_is_none` | Imported literal `true` support | Z3 4.15.3 |
| `vc_replace_Pos` | Imported literal `true` support | Z3 4.15.3 |

Every proof node has only `prover` and `time` fields. The independently printed root set, individual task root set, prior audit root set, and saved `proofs.Coma` key set are equal. Thus the accepted count is **4 roots / 4 successful terminal leaves / 0 transformations**, comprising one actual-body leaf and three support leaves.

The pre-proof audit independently explored `split_vc` and obtained 25 children for the own root, or 28 candidate leaves including support. That was a diagnostic candidate decomposition. It is not the successful proof tree and is not counted as 25 proved child obligations. No nested arity replay is required to reconcile this direct proof. The direct own-root success establishes its complete unsplit formula, which includes the loop variant and all stated postconditions.

## Accepted behavior and limits

Under a nonempty slice, in-range starting probe, and an original empty slot within one cyclic traversal, the worker's actual body proves arithmetic/index safety, preservation and decrease of its `len-step` variant, and the first-empty stopping relation. The result is below the original length, every earlier original cyclic slot is nonempty, and the result identifies an original empty slot.

The post-state has unchanged length. Its cyclic prefix contains the incoming Pos followed by each preceding original Pos; the remaining cyclic suffix is unchanged. Both index and hash fields are preserved exactly according to that relation. The non-vacuity examples and incoming-empty case described in the pre-proof report remain applicable.

The selected task context contains no RandomState/Hasher or contractless hash/String precondition. The HashValue type is only its concrete u16 record. The imported support roots do not themselves prove slice-length, Pos::is_none, or mem::replace implementations; this is a modular body result using their emitted contracts.

Actual callers still need to establish the helper's premises. This result does not prove spare capacity, two remaining empty slots, constructor/reserve/rebuild invariant closure, key uniqueness, Robin Hood ordering, global readiness preservation, or integrated HeaderMap correctness. In particular, one empty slot suffices for this worker's termination but can be consumed completely; preserving nonempty-map lookup readiness after insertion requires an additional spare-slot/count fact. Those local caller proofs follow separately from this accepted worker milestone.
