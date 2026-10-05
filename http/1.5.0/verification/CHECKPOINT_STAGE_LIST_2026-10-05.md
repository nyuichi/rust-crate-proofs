# First-checkpoint staging list — 2026-10-05

This list is for the root commit/push. The repository owner has already staged
the production HTTP source/Cargo changes and the required `creusot-std` source
support. This sub-agent has not run Git staging or commit commands.

## Stage the checkpoint and runnable harnesses

Stage these documentation and harness files/directories:

- `http/1.5.0/verification/CHECKPOINT_2026-10-05.md`
- `http/1.5.0/verification/CHECKPOINT_STAGE_LIST_2026-10-05.md`
- The checked-in verification harness inputs and area docs in
  `http/1.5.0/verification/{bytes,byte-str,byte-eq,byte-ord,composition,convert-std,error-conversions,error-formatting,hash-std,header-map-api,headers,id-hasher,method,map-capacity,path-scanner,runtime-check,runtime-http,scalars,str-index,uri}/`:
  `Cargo.toml`, `Cargo.lock` where present, `README.md`, `src/`, `tests/`,
  scripts, and `why3find.json` where present. Stage no generated `verif/`
  subtrees through this directory selector.
- `http/1.5.0/verification/evidence/validate_own_target.py`
- `http/1.5.0/verification/runtime-check/evidence/http-native-checkpoint-results-2026-10-05.json`
- `http/1.5.0/verification/runtime-check/evidence/http-native-allfeatures-summary-2026-10-05.txt`
- `http/1.5.0/verification/runtime-check/evidence/http-native-checkpoint-source-2026-10-05T08-03-09Z.json`
- `http/1.5.0/verification/runtime-check/evidence/http-native-checkpoint-source-2026-10-05T08-03-09Z.tar.gz`
- `http/1.5.0/verification/runtime-check/HEADER_NAME.md` and
  `tests/header_name_boundaries.rs`.

The native test evidence is the post-rebase snapshot. Older interim native
snapshots are historical and are not needed for this first checkpoint.

## Stage accepted evidence archives and manifests

Stage only the following curated evidence. Paths are repository-relative; each
listed run directory is a curated archive with a manifest or associated
run-level evidence. The HeaderName archive is intentionally limited to the
single v7 snapshot so old duplicate snapshots and ICE output do not enter the
first checkpoint.

- `http/1.5.0/verification/bytes/evidence/`
- `http/1.5.0/verification/byte-str/evidence/`
- `http/1.5.0/verification/byte-eq/evidence/creusot-std/partial_eq/seq_eq_u8_int_view_transport/`
- `http/1.5.0/verification/byte-ord/evidence/`
- `http/1.5.0/verification/hash-std/evidence/manifest.json`
- `http/1.5.0/verification/hash-std/evidence/run-2026-10-05-hashword-modulo/`
- `http/1.5.0/verification/map-capacity/evidence/manifest.json`
- `http/1.5.0/verification/map-capacity/evidence/run-2026-10-05-manual-debug/`
- `http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-live-bound/`
- `http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-bucket-fields/`
- `http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-iter-mut-next-conditional/`
- `http/1.5.0/verification/composition/evidence/request-entrypoints-2026-10-05.json`
- `http/1.5.0/verification/composition/evidence/run-2026-10-05-request-shortcuts-current/`
- `http/1.5.0/verification/composition/evidence/run-2026-10-05-builder-into-relay/`
- `http/1.5.0/verification/composition/evidence/parts-defaults-2026-10-05.json`
- `http/1.5.0/verification/composition/evidence/run-2026-10-05-default-field-relay/`
- `http/1.5.0/verification/error-formatting/evidence/error-formatters-2026-10-05.json`
- `http/1.5.0/verification/error-formatting/evidence/run-2026-10-05-error-formatters-current/`
- `http/1.5.0/verification/uri/evidence/authority-utf8-boundaries-2026-10-05/`
- `http/1.5.0/verification/uri/evidence/path-api-checkpoint-2026-10-05/`
- `http/1.5.0/verification/headers/evidence/archived_snapshots/value_complete_translation_v7/`
- `http/1.5.0/verification/method/evidence/final-2026-10-05.json` and
  `outcomes.json`
- `http/1.5.0/verification/scalars/evidence/final-2026-10-05.json`,
  `nonhash-2026-10-05.json`, and `outcomes.json`
- `http/1.5.0/verification/str-index/evidence/manifest.json`
- `http/1.5.0/verification/convert-std/evidence/run-2026-10-05-slice-asref-identity/`

For Method and Scalars, the accepted `.coma`/`proof.json` files are generated
under ignored `verification/{method,scalars}/verif/`. Stage only targets listed
as accepted by the corresponding final evidence manifest, matching the
recorded COMA hashes. For Scalars, exclude
`status_array_model_probe` and `status_array_model_borrowed`; they are named
capability probes, not accepted body evidence. Do not add a whole generated
`verif/` tree without this target filter.

## Keep out of this checkpoint

- Every live `verification/**/verif/` tree except the manifest-filtered Method
  and Scalars targets above; the separate archived `.coma`/proof JSON
  directories explicitly listed above are retained.
- `creusot-libs/_creusot_erasure/`, `target/`, `.why3find/` runtime state,
  Why3 sockets/logs, interrupted proof JSONs, and all `rustc-ice-*.txt` dumps.
- `verification/headers/evidence/archived_snapshots/**` other than
  `value_complete_translation_v7/`; the older copies are duplicated and
  historical.
- `verification/composition/evidence/run-2026-10-05-clean-builder/` and
  `verification/header-map-api/evidence/run-2026-10-05-clean-map-api/`;
  these are translation/diagnostic runs, not accepted proof archives.
- `verification/map-capacity/evidence/attempts/`; the derived Debug attempt
  was rejected and is superseded by the accepted manual Debug run.
- `verification/headers/why3find.json` if it contains transient, unaccepted
  target selections; keep the checked-in harness profile only if it matches
  the reviewed source and named profile at freeze.

The map capacity constructor additions and `num.rs` contract remain
unverified source changes in this checkpoint; the checkpoint README labels
them explicitly. Their presence in the source commit is not proof evidence.
