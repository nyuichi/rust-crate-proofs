# `parse_newline` isolated proof checkpoint

**Status: the isolated helper proof closure is green.** The 12-target closure
contains 78 Valid leaves across 11 nontrivial targets; the open logic model
target reduces to literal `true` and creates no solver task. All 12 selected
targets passed type-only preflight. This is not an integrated parser proof.
Production `src/lib.rs` still invokes `newline!` directly; this helper is not
wired into the parser in this checkpoint.

## Runtime behavior represented by the model

The helper source at `src/parse_newline.rs` invokes the existing
`newline!(bytes)` macro and returns `Complete(())` after that macro succeeds.
The macro expansion retains its existing `next!`, `expect!`, and `Bytes::slice`
operations. At a future caller substitution, `complete!(parse_newline(&mut
bytes))` yields unit on success and propagates the same partial/error outcomes
as the current macro call. No production caller was changed here.
`src/verification/newline.rs` models the complete observable cursor, mark,
result, and input bounds for all branches:

- LF consumes one byte and commits the cursor with `slice()`.
- CRLF consumes two bytes and commits the cursor with `slice()`.
- EOF before the first byte returns partial without changing the cursor or
  mark. EOF after a consumed CR returns partial at `end` and keeps the old
  mark.
- An invalid first byte is consumed before `Error::NewLine`; an invalid byte
  following CR is also consumed. Both error branches preserve the old mark.

The model function is `#[logic(open)]`, so the full finite branch definition
is visible in the generated actual-helper COMA. Its standalone CoMA contains
the literal goal `vc_parse_newline_model: true`; it is recorded as a no-task
definition and excluded from the Valid count. The actual helper body is proved
against the exact model.

## Isolated translation

Run `./verification/probes/newline-harness/translate.sh translate` from
`httparse/1.10.1`. It uses the isolated string-model compiler environment and
the `string/Cargo.toml` dependency copy. The command exited 0 and emitted 56
COMA files. The relevant generated targets are:

| Target | SHA-256 |
|---|---|
| `string/verif/httparse_newline_string_harness_rlib/parse_newline.coma` | `9f35bee713232f3526a0dee224c903fd87f9bc15229e59c8d5f5eb2f8bc30765` |
| `string/verif/httparse_newline_string_harness_rlib/verification_newline/parse_newline_model.coma` | `1cb5474304eebf379d61a56bd840124880b96bc248a802038430974784664239` |

The actual helper and model COMAs passed an initial two-target Why3
`--type-only` preflight. The 10 required `Bytes` dependency COMAs also passed
type-only and were proved with the actual target in the bounded closure. The
remaining COMAs in the 56-file tree were inventoried and hash-frozen but were
not type-checked or proved in this pass.

## Bounded proof result

The frozen target order and hashes are in
`evidence/selected-proof-closure-20261005/TARGETS.tsv`. The required direct
`Bytes` call chain is included: `Iterator::next` calls `peek`/`bump` and the
checked head lemma; `Bytes::slice` calls commit and slice construction. The
result breakdown is:

| Scope | Result | Evidence |
|---|---|---|
| Ten exact `Bytes`/Iterator dependencies | 56/56 Valid leaves | `evidence/direct-why3-selected-20261005T064400Z` |
| Exact `parse_newline` helper body | 22/22 Valid leaves | `evidence/direct-why3-parse-newline-20261005T064700Z` |
| Exact outcome model | No solver task (`vc_parse_newline_model: true`) | `evidence/selected-proof-closure-20261005/TRIVIAL_GOALS.tsv` |

The first corrected 12-target batch type-checked all targets, proved the ten
`Bytes` dependencies, then stopped when the model-only target produced no
solver JSON. That is a no-task definition, not a failed or Valid VC. The
actual helper was run separately after that classification. A previous
attempt stopped before solver execution because of an incorrect temporary
symlink; its diagnostic is preserved in the selected-closure evidence and is
not a proof result. Each solver run used `Z3,4.15.3`, a 30-second per-goal
limit, 1000 MiB, and one prover through the shared proof lock. The raw logs,
Why3 configs, package/stdlib manifests, exact commands, and run identities are
retained beside each run.

The 56-COMA and source-input hash manifests are in
`evidence/translation-20261005/`; their hashes are recorded in
`evidence/selected-proof-closure-20261005/SNAPSHOT.txt`. The selected-closure
manifest separately hashes the target plan, no-task classification, and
result summary.

## Reproduce the frozen artifacts

To regenerate the translation from a clean harness package:

```bash
cd /workspace/rust-crate-proofs/httparse/1.10.1/verification/probes/newline-harness
cargo clean --manifest-path string/Cargo.toml
./translate.sh translate
```

To replay the ten nontrivial `Bytes` dependencies from the frozen COMAs, use a
new output directory so the preserved run remains unchanged. The target list
is the first ten rows in `evidence/direct-why3-selected-20261005T064400Z/targets.txt`;
the model row is intentionally excluded because its goal is literal `true` and
creates no prover task:

```bash
crate=/workspace/rust-crate-proofs/httparse/1.10.1
harness=$crate/verification/probes/newline-harness
run=$harness/evidence/replay-bytes
mkdir -p "$run/logs" "$run/context"
ln -s "$harness/string" "$run/context/string"
mapfile -t targets < "$harness/evidence/direct-why3-selected-20261005T064400Z/targets.txt"
targets=("${targets[@]:0:10}")
cd "$crate"
./run-proof.bash ./verification/probes/spaces-harness/proof-child.sh "$run/context" "$run" "${targets[@]}"
```

Replay the actual helper separately with its recorded one-target list:

```bash
run=$harness/evidence/replay-parse-newline
mkdir -p "$run/logs" "$run/context"
ln -s "$harness/string" "$run/context/string"
mapfile -t targets < "$harness/evidence/direct-why3-parse-newline-20261005T064700Z/targets.txt"
cd "$crate"
./run-proof.bash ./verification/probes/spaces-harness/proof-child.sh "$run/context" "$run" "${targets[@]}"
```

The wrapper locks the shared solver slot; the child loads the isolated string
environment, checks every requested COMA with `--type-only`, then calls Why3
directly with the recorded Z3 version and limits. The run identities record the
compiler, Why3 package, configuration, stdlib, and prover hashes. The three
original raw runs, including the initial preflight setup error, each have an
adjacent `SHA256SUMS` manifest.

This checkpoint proves the isolated helper and the exact selected dependencies
only. It does not prove a parser caller, outer `Request`/`Response` parsing, or
full-crate integration. No new trust boundary was introduced.

`evidence/translation-20261005/COMA-SHA256SUMS` inventories the full emitted
tree. `INPUTS-SHA256SUMS` fingerprints the 30 Rust source files referenced by
the COMA source spans plus the harness manifests, translation script, and
compiler environment files. Verify the manifests from the indicated roots:

```bash
cd /workspace/rust-crate-proofs/httparse/1.10.1/verification/probes/newline-harness/string
sha256sum -c ../evidence/translation-20261005/COMA-SHA256SUMS
cd /workspace/rust-crate-proofs/httparse/1.10.1/verification/probes/newline-harness/evidence/translation-20261005
sha256sum -c SHA256SUMS
cd /workspace
sha256sum -c rust-crate-proofs/httparse/1.10.1/verification/probes/newline-harness/evidence/translation-20261005/INPUTS-SHA256SUMS
```
