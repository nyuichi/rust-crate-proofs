# Owner I/O proof evidence

This directory records experiments for the private modified bytes 1.11.1
owner/I/O candidate. Archives preserve the source and generated proof artifacts
for each result. A local target run is not an integrated proof; the full
`verified,std` wrapper is the candidate-level acceptance gate.

## Candidate 08 status

- Native check: `cargo test --offline --locked --lib --no-default-features
  --features verified,std` passed 55/55 tests. Its source manifest and log are
  in `invariant-repair-08-native-pass-source.tar.gz`
  (SHA-256 `179451a25d9f921ed97ab5636641d543431877f92f41a297b51b96c9974e6019`).
- Full proof: failed with the sole unproved file
  `Coma.vc_scatter_segment_splice` at 13/16 children. The null children are
  indices 3, 6, and 9 in the helper's `proof.json`, corresponding to the
  `before_tail`, `after_segment`, and final `updated` `Seq::ext_eq` assertions.
  Astra's review found that all three had length facts but lacked explicit
  pointwise equality facts. The failure source, `.coma`, proof JSON, config
  manifest, logs, and source fingerprints are preserved in
  `invariant-repair-08-full-proof-failure-source.tar.gz`
  (SHA-256 `a061727f5b63f884f7f7e89b6e45ad7ece2663b4ec9c38dad49501f9fdc09ef0`).
- Candidate 08 is **not integrated-proof complete**. No body proof for
  `scatter_segment_splice` is claimed from its separate targeted log; that
  command used an invalid empty goal filter, described below.

## Candidate 09 status

Candidate 09 adds pointwise sequence facts at the three extensionality goals,
using the existing split/frame/read facts and loop index. It adds no
contracts, trusted bytes law, or standard-library source change.

- Native check: 55/55 tests passed, including the 4-byte input written across
  two 2-byte destinations. The exact source manifest and log are in
  `invariant-repair-09-native-pass-source.tar.gz` (SHA-256
  `f5026fd785da59d2c2f6d3d9ed4c9c3a6e8b313f9a803871630d53b5c4dc4ce5`).
- Full proof: the authorized `verified,std` host wrapper passed with
  `Proved (308 files) ✔`. Independent JSON inspection found 308 `proof.json`
  files, 1,303 prover-result leaves, and zero null results. The pre-run source
  manifest matches all 85 source inputs after the run.
- Standard support: the actual private package at
  `/workspace/bytes-proof-tools/bytes-verified-std` independently matches the
  reviewed manifest: 110 files, 711,729 bytes, tree SHA-256
  `878a63ae09dde611de547994499803158b6aa6157ce48f4332f645dff3edb155`; all
  nine affected-file hashes match. The installed standard package was not
  edited.
- The source, generated `.coma` and proof JSONs, feature tree, standard
  manifest, result record, and logs are preserved in
  `invariant-repair-09-full-proof-pass-source.tar.gz` (SHA-256
  `21e87ba182133b8ac70a7e39d2164d7ce94d4ebe3f785ab1f30b73410613f4d2`). The
  machine-readable independent result is
  `invariant-repair-09-full-proof-result.json`.

The pass covers this candidate's concrete modified APIs: Cursor's
`BufRead::fill_buf`/`consume`, initialized-slice gather/scatter, and copied
owners in the sealed `Vec<u8>`, `&[u8]`, `Box<[u8]>`, and `String` set. The
generic owner operation returns its original owner intact and requires explicit
cleanup of the separate copied owner. It does not establish arbitrary `AsRef`
byte laws, standard `IoSlice` trait support, zero-copy owner transfer, or
bytes-specific ownership/refcount theorems.

## Candidate 10 source-matched capture

Candidate 10 changes the native test module gate from `std` to `alloc` and gates
only the `BufRead` test on `std`, so copied-owner and gather/scatter tests run in
the alloc-only configuration. Native results are 55/55 for `verified,std` and
28/28 for `verified` without `std`.

The full wrapper again passed 308 files with zero null JSON leaves and an
unchanged 85-file pre-run source manifest. Its actual `creusot-std` tree was
independently rehashed against the reviewed support manifest. The canonical
read-only capture receipt is
`verification/modified-variant-evidence/runs/positive/bytes-owner-io-candidate10-verified-std/receipt.json`;
the proof/source/actual-Std archive is adjacent `evidence.tar.gz`, SHA-256
`973e20f813c1d715d085172428aa7037c83ab0bf8672a5efc24f75780a99c6e9`.
Independent counts and hashes are in
`invariant-repair-10-full-proof-result.json`.

The capture says `full_coverage: false`. It proves the exact captured bodies
under `verified,std`; it does not establish complete bytes API coverage or
architecture admission. The module visibility/export and combined public
consumer gate are still to be applied in the integration branch.

## Withdrawn targeted-proof results

The following success messages must not be cited as proof evidence:

1. The helper command used `-g scatter_segment_splice` and printed
   `Proved (verif/bytes_rlib/verified/owner_io/scatter_segment_splice.coma) ✔`.
   This filter selected no goals. Why3find's `src/prove.ml` accepts a goal
   filter only when it equals the theory name or `Theory.goal`; its README
   uses `Bar.goal`. The actual goal is `Coma.vc_scatter_segment_splice`.
   The file-level `Proved` message did not report a selected-goal count and was
   vacuous for that filter.
2. The separate `read_scatter_prefix.coma` local-file command printed
   `Proved (...) ✔` without a `-g` filter. That is only a local body result
   which consumes the splice helper's callable contract; it does not prove
   that helper body or the whole candidate. Withdraw it as evidence of
   candidate completion. The full wrapper remains authoritative and failed
   in the helper dependency.

The exact commands, logs, and exit files are bundled in the candidate 08 full
failure archive. The archived helper command used this invalid filter:

```text
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 \
  --why3find-arg=-g --why3find-arg=scatter_segment_splice \
  verif/bytes_rlib/verified/owner_io/scatter_segment_splice.coma
```

For a future diagnostic, use the exact `Coma.vc_scatter_segment_splice` goal
name or select the complete `.coma` file without `-g`, then check generated
goal counts and every JSON child. Candidate completion still requires the
full wrapper to pass.

Why3find's selection implementation is available at
`/workspace/bytes-proof-tools/why3find-source/src/prove.ml` (the `accept`
function) and its selection examples are in that tree's `README.md`.
