# Incremental checkpoint staging list — 2026-10-05

This is the curated file list for the next component checkpoint. The root agent
handles staging and commits. It excludes generated working trees and does not
authorize staging unrelated in-progress source changes.

## Checkpoint documents and source

- `http/1.5.0/verification/CHECKPOINT_2026-10-05.md`
- `http/1.5.0/verification/CHECKPOINT_STAGE_LIST_2026-10-05.md`
- Current source files needed for the accepted groups: `src/header/name.rs`,
  `src/header/value.rs`, `src/header/map.rs`, `src/request.rs`,
  `src/response.rs`, and URI support sources listed by the accepted URI
  manifests. `creusot-libs/creusot-std/src/std/num.rs` supplies the checked
  power-of-two standard-library contract. `src/uri/authority.rs` and
  `src/uri/mod.rs` have small post-emission changes; stage them only with their
  recorded delta and the matching source-at-emission copies below.
- Do not include unrelated or later unproved edits to these shared source
  files; preserve the exact proof/source fingerprint links in the manifests.

## Accepted proof and regression evidence

- HeaderName: `verification/headers/evidence/archived_snapshots/name_hdrname_invariant_v1/`
  `snapshot.json` (SHA-256
  `2cdebd67096b95e4f61e81b15bd82d9f856dfa02314b274d20a6c38570bca5fb`),
  `why3find.json`, all 195 emitted `.coma` files, and only the proof JSON files
  for the 34 targets listed in `snapshot.json`'s `proof_results`. Exclude the
  partial `parse_hdr` proof JSON and failed `HdrName` Debug refinement.
- HeaderValue current helper: the complete
  `verification/headers/evidence/archived_snapshots/value_hex_digit_repair_v1/`
  archive and `verification/headers/evidence/header-value-debug-parity-2026-10-05.json`.
- HeaderValue historical group: the complete
  `verification/headers/evidence/archived_snapshots/value_complete_translation_v7/`
  archive. Keep it labeled historical; it records the older `value.rs` source
  SHA `1ab329864e57e80bd494fcc610ee0da3c7ac77c95c13fcf5a23647166a452e0c`.
- Request/Response composition: the fresh
  `verification/composition/evidence/run-2026-10-05-current-composition-full-clone/`
  archive and its manifest (SHA-256
  `c468e11225502a0f8fc80c05442a56c9f15f0b385ee3ab3b5732fc5901e4dc2a`). It
  records 49 targets, 107 terminal leaves (65 own / 42 support), matching
  proof-tree/run-log arities, and source-map SHA
  `54d174cd6a830a50e101d376ab95209a2aad258d1b7f19d78d2e26a8e5bb1efd`. The
  older 45-target archive is superseded; its old COMA hashes were not reused.
- URI PathAndQuery Display:
  `verification/uri/evidence/path-display-2026-10-05/` (manifest, three COMAs,
  and their three proof JSON files).
- URI `Parts` / conversion partial batch:
  `verification/uri/evidence/parts-uri-constructors-2026-10-05/` (manifest,
  selected COMAs and proof JSONs, `sources-at-emission/` copies, and the
  recursive recorded-tree audit). The manifest SHA-256 is
  `ece93d91b01e0b2ae6497d21c9ace3a87e885750cffc855ae5295f113f9535d2`.
  It records 12 roots with 55 terminal nodes (48 prover results, 7 nulls).
  The audit validates JSON accounting only; expected child arity from COMA is
  still unverified, so this is staged as an explicit partial result. Do not
  stage current URI working files on the strength of this archive; the manifest
  identifies post-emission source changes.
- URI Authority comparisons:
  `verification/uri/evidence/authority-comparison-2026-10-05/` (manifest,
  proof artifacts, proof logs, and `sources-at-emission/src/uri/{authority.rs,mod.rs}`).
  The updated manifest SHA is
  `190f60249b7e91a885646391de26de04420e8cc6f79487cdd0634f43289e098a`; all 69
  declared artifact/log/source-copy hashes match. The manifest records the
  post-emission `Parts::default` delta and Authority hash/helper deltas. The
  source-emission snapshot is the proved comparison source; the later Hash
  body proof is pending. Its focused runtime regression passes 1/1, as does
  the comparison behavior regression.
- HeaderMap conditional `IterMut::next_unsafe`:
  `verification/header-map-api/evidence/run-2026-10-05-iter-mut-next-conditional/`
  (manifest, one COMA, and matching proof JSON). Preserve its nested split
  arity audit and one-call precondition limits.
- HeaderMap capacity constructor pilot:
  `verification/header-map-api/evidence/run-2026-10-05-capacity-constructor-lemma/`
  (manifest, four exact COMA/proof artifacts, and captured source files). It
  records the proved one-leaf power-of-two lemma and `try_with_capacity` body
  proof, plus a passing normal HTTP library check.
- Native regression evidence for the current post-rebase checkpoint:
  `verification/runtime-check/evidence/http-native-checkpoint-results-2026-10-05.json`,
  the matching all-features summary, and the source snapshot referenced by that
  result file.

## Exclude

- All live `verification/**/verif/` trees, proof-server state, Why3 sockets,
  interrupted proof JSON, `rustc-ice-*.txt`, `target/`, and
  `creusot-libs/_creusot_erasure/`.
- HeaderName partial `parse_hdr` proof output and the failed `HdrName` Debug
  refinement.
- HeaderMap capacity proof outputs outside the selected two-target archive;
  in particular exclude stale generated `verif/` files and the earlier failed
  assertion attempt.
- The superseded 45-target composition archive; use the fresh 49-target
  archive above.
- The pre-invariant duplicate HeaderName archives and unrelated historical
  native snapshots.
