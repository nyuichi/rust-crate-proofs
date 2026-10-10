# Bounded direct Why3 run

Result: all 11 selected COMA files type-checked and all 69 split subgoal
results were `Valid`. The results contain 11 unique named verification
conditions (one per selected file); `skip_spaces.coma` contributed 26 split
results. Raw output is retained in `logs/NN.log`, and `results.tsv` records
every split result.

Each proof invocation used the actual fresh COMA path, the isolated Creusot
package, the harness `verif` load path, and:

```text
why3 prove --json -F coma -a split_vc -P 'Z3,4.15.3' -t 30 -m 1000
```

The run was serialized by the crate's shared `run-proof.bash` lock. The
preflight checked the isolated Creusot compiler, the scratch Why3 config,
Why3/Z3 executables, package path and package theories, and all 11 fresh COMA
files. It type-checked all 11 targets before invoking any prover. The scratch
environment's `_opam` entry is a symlink; `run-identity.txt` records the
requested base path and resolved Why3 base/stdlib. The run-local effective
config loadpath was checked against that resolved standard library. Both
original environment configs were left unchanged.

The input bundle contains the current harness, runtime/model source closure,
translation inputs, the complete current generated COMA tree (58 files,
including the exact 11 selected targets), the exact run scripts, environment
and config identity, package/stdlib hash manifests, preflight log, raw target
logs, result table, and pre/post hashes for all 11 selected targets. The proof
invocation did not use cached Why3 proof JSON.

This proves the extracted `skip_spaces` body and its selected independent
dependencies. It does not prove a crate-integrated parser or full-crate run.
