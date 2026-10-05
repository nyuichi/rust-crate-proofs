# Native environment full positive checkpoint

This frozen bundle records the native-environment compiler checkpoint after all 21 selected positive COMA target files were checked. It contains 28 named verification conditions, all `Valid`. The three actual derived equality body goals were proved in the initial checkpoint with saved Why3 sessions; the other 18 COMA files yielded 25 `Valid` named goals under direct Why3.

## Scope and outcomes

- `positive-coma/` and `negative-coma/` preserve all 26 positive and 14 negative translated COMA files.
- `inputs/proof-positive.targets` lists the 21 selected positive files. `inputs/proof-nativeenv-eq.targets` lists the three selected equality bodies. `inputs/proof-nativeenv-remaining.targets` lists the other 18. `inputs/proof-negative.targets` lists two deliberately false claims; those were type checked only with this compiler and were not sent to a solver.
- `eq-proofs/` preserves the three selected body COMAs, proof JSON, and Why3 sessions. `logs/nativeenv-*-eq-prover-results.jsonl` preserve raw first-stage prover output.
- `remaining-proofs/NN.log` preserves direct Why3 JSON for each remaining target; `results.tsv` records each named goal result, and `selected-coma-sha256.tsv` binds every selected target to its exact input hash.
- `remaining-proofs/run-selected.sh` runs selected files sequentially with a fixed 30 second limit and exits on the first non-Valid goal. The recorded run used `-P 'Z3,4.15.3' -t 30 -m 1000` under the isolated Why3 config/package and the coordinated one-prover wrapper.
- `run-identity-after.txt` records the compiler, prelude, config, package path, prover, limits, and target counts. The global and isolated Why3 configs had the same recorded SHA-256.
- `toolchain/` preserves the native-env compiler patch and build inputs. It includes the original builder used for the recorded binary and the current replay builder. If the old scratch library seed is absent or its pinned `num.rs`, `string.rs`, or `convert.rs` differs, the replay builder copies the frozen versions from `toolchain/stdlib/` into an isolated fallback seed; it does not use the changing shared `num.rs` for those inputs. `inputs/original-derived-traits/` preserves the original harness source/manifests that the patch sequence builds on. This is an isolated compiler checkpoint; the active compiler was not replaced or adopted.

The positive closure covers only the 21 listed files. The Debug body/refinement establishes `formatter_extends`, not the exact rendered byte sequence. Exact formatting text, the two deliberately false negative claims, the Unicode byte examples from the historical string-model checkpoint, and full crate/parser integration remain open. `REPORT.snapshot.md` freezes the companion report.

`SHA256SUMS` checks every file in this bundle except itself.
