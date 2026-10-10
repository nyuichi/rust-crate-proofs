# `skip_spaces` isolated proof checkpoint

**Status: the extracted helper, exact model, cursor bridge, and selected
`Bytes` dependency closure are independently proved. The full parser is not
verified or integrated by this probe.**

## Runtime and model boundary

The harness path-includes the actual `src/iter.rs`, `src/macros.rs`,
`src/status.rs`, `src/error.rs`, and extracted `src/skip_spaces.rs`. The
runtime `peek`/match/`bump`/`slice` block was compared against the original
`skip_spaces` body in `src/lib.rs` and is byte-identical. Added executable
syntax is guarded by `cfg(creusot)`. This probe does not wire the helper into a
crate-integrated proof.

The exact model consumes the maximal prefix of literal `0x20` bytes. At the
first non-space byte, it returns complete, leaves that byte at the cursor, and
commits the mark to the cursor. At EOF it returns partial, leaves the cursor
at `end`, and preserves the entry mark because the runtime does not call
`slice()` on that path. Tabs and every other non-space byte terminate the
scan; no error is possible.

The loop uses the single measure `end - cursor`. Its invariants preserve the
input, entry mark, and end; constrain the cursor to the valid range; record
that every byte scanned from the entry cursor is SP; and bound the cursor by
the maximal-prefix endpoint. The `space_prefix_at_cursor` checked lemma links
the opaque endpoint to the loop cursor: at a non-space byte or EOF the cursor
equals the endpoint, while an SP byte puts it strictly before the endpoint.

## Translation and selected proof

Translation used `./verify-string.sh translate` with the isolated string-model
compiler and the default `std` feature. It produced the selected actual source
and model targets plus the required `Bytes` closure. Translation is not a
solver result; the recorded proof used the exact fresh COMA hashes in
[`evidence/proof-targets.tsv`](evidence/proof-targets.tsv).

The direct run is preserved under
[`evidence/direct-why3-20261005T052959319229043Z/`](evidence/direct-why3-20261005T052959319229043Z/).
All 11 selected files passed type-only preflight and yielded 69 `Valid`
results after `split_vc`. These are 69 split subgoal results across 11 unique
named verification conditions, one per COMA file; the actual `skip_spaces`
body produced 26 split subgoals, all `Valid`. The run contains one raw Why3
log per target, the parsed result table, the complete 58-file generated COMA
tree (including the 11 selected targets), selected-target pre/post hashes,
compiler/config/package identity, and hashes of the isolated Why3 standard
library and Creusot package. No cached proof JSON was used.

The 11 targets were proved in this dependency order:

1. `Bytes::byte_permission` (4 split results)
2. `Bytes::peek` (4)
3. `Bytes::advance` (7)
4. `Bytes::bump` (4)
5. `Bytes::commit` (2)
6. `slice_from_ptr_range` (8)
7. `Bytes::slice` (5)
8. `space_prefix_end` (5)
9. `skip_spaces_model` (3)
10. `space_prefix_at_cursor` (1)
11. `skip_spaces` (26)

The direct command for each target used Why3 1.8.2+git, `-F coma -a split_vc`,
`-P 'Z3,4.15.3'`, `-t 30`, and `-m 1000`. The shared
[`run-proof.bash`](../../../run-proof.bash) lock enforced one active prover.
The isolated config and run-local effective-config hashes are recorded in
`run-identity.txt`. The scratch environment's `_opam` path is a symlink to
the active Why3 installation; the run records both requested and resolved
base-data paths and verifies the config's loadpath resolves to that stdlib.
Neither shared config was modified.

One earlier setup attempt stopped at an over-strict stale-loadpath string
check before type-only or solver work. The scratch `_opam` symlink resolves to
the same active stdlib as the canonical path in Why3's effective config. The
corrected run verified that realpath and the stdlib/package hashes; its full
preflight and proof closure passed. The no-solver preflight record remains at
[`evidence/direct-why3-20261005T052759688321450Z/`](evidence/direct-why3-20261005T052759688321450Z/).

The helper has no parser trust attribute or trusted parser substitute. Its
direct `Bytes` dependencies are `peek`, `bump`, and `slice`; the selected
closure also proves `byte_permission`, `advance`, `commit`, and
`slice_from_ptr_range`. The parser model dependencies are
`space_prefix_end`, `space_prefix_at_cursor`, and `skip_spaces_model`. The
dependency map and earlier same-source `Bytes` evidence are recorded in
[`evidence/dependencies.tsv`](evidence/dependencies.tsv). This isolated proof
does not establish the crate-level parser or full-crate verification.

## Reproduction

Run `./verify-string.sh translate` to refresh translation, then
`./verify-proof.sh` to check the manifest hashes, preserve a fresh run bundle,
run type-only preflight, and execute the selected direct Why3 closure through
the shared lock. `proof-child.sh` checks the isolated compiler, config, Why3
base data and Creusot package before proving. The run-local config retains the
effective loadpath after verifying its resolved target. The runner stops on
the first non-`Valid` result and keeps the raw log.
