# `skip_empty_lines` verification checkpoint — 2026-10-05

**Status: the extracted `skip_empty_lines` implementation is integrated into the crate and its selected proof closure is Valid. The full `httparse` proof remains OPEN.**

## Implementation and model

`src/lib.rs` includes the shared `src/skip_empty_lines.rs` body at the original helper location. `Request::parse_with_config_and_uninit_headers` still calls `skip_empty_lines` at entry. The independent model is loaded only under `cfg(creusot)` from `src/verification/empty_lines.rs`.

The model tracks input, starting mark, cursor, and end. It distinguishes complete LF and CRLF pairs, an incomplete trailing CR, and a CR followed by a non-LF byte. It records that the latter byte is consumed before `Error::NewLine`, while the entry mark remains unchanged. The runtime-body comparison against the pre-extraction `lib.rs` snapshot passed: 104 normalized tokens, SHA-256 `cc731c7e0eeb496982e24cd47b3e563091f7a49afbbf00ed0c7b07947bdc8296`.

## Bounded proof run

The fresh translation used the rebuilt string-model profile and compiler SHA-256 `29fcf8914166db07a910c45e4c8462f5665cfe4eee482509b885e629d3ff826e`. It emitted 64 CoMa files. The final import-normalized translation passed Why3 `--type-only` on the same 20-target closure. The proof was run sequentially with one Z3 4.15.3 worker, 30 seconds per goal, and 1000 MiB per goal under the shared lock.

Across the selected 20 target files, the raw logs contain **171 Valid solver results**, zero non-Valid results, and one `TrivialTrueNoTask` target (`complete_line_ending_at`, whose only CoMa goal is literally `true` and has no body). That no-task target contributes zero proof goals. The actual extracted `skip_empty_lines` body contributes 49 Valid goals; the model contributes 66 and its `Bytes`/`Iterator` dependency closure contributes 56.

The earlier timeout at the CRLF extension was from the pre-induction candidate and remains preserved as historical evidence. In this structurally inductive candidate, the concat helper, cursor induction, both finite extensions, endpoint recurrence, loop bridge, outcome model, and actual helper body all passed.

After the bounded helper proof, `lib.rs` was wired to include that exact helper file and to load the model under `cfg(creusot)`. The helper's duplicate prelude import was then removed; the isolated harness now brings those names into its own scope. A new translation and type-only check confirmed all 20 targets still typecheck. Nineteen selected CoMas are byte-identical; the actual helper CoMa has only source-coordinate line shifts, and its normalized content is identical to the proved CoMa. The proof therefore still matches the final runtime and contracts.

Detailed source snapshots, exact selected CoMas, target order, tool identities, hashes, type-only logs, and raw solver output are in [`evidence/proof-replay-20261005T162057Z/`](evidence/proof-replay-20261005T162057Z/). The fresh translation and its emitted CoMa inventory are in [`evidence/translation-replay-20261005T160339Z/`](evidence/translation-replay-20261005T160339Z/).

## Native validation and remaining boundary

From `httparse/1.10.1`, both crate-scoped offline test configurations passed after the import-scope correction:

- `cargo test --offline --manifest-path Cargo.toml`: 105 unit tests, 263 URI tests, and 6 doc tests passed.
- `cargo test --offline --no-default-features --manifest-path Cargo.toml`: 101 unit tests, 263 URI tests, and 6 doc tests passed.

The `Bytes` helper and unsafe slice-construction contracts remain part of the Creusot standard-library trust boundary. The proof covers this helper's extracted runtime body and dependencies; it does not prove the enclosing `Request`/`Response` bodies, the complete crate, or all feature and target configurations.

Older attempts and their raw logs remain unchanged under the earlier evidence directories. The fresh 2026-10-05 proof run is the accepted result for this checkpoint.
