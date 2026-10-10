# Fresh active-byte `Bytes` proof artifacts (2026-10-05)

This evidence bundle records the fresh active-byte-compiler refresh at
`src/iter.rs` SHA-256
`369662bbf36c68ba814750ce3a91c4aa78ab40e9453af0f7e07f30d869103625`.
All 31 selected CoMa targets passed: 121 valid split leaves in the
`memory-pointer` probe and 18 in `memory-array-conversion`, 139 distinct
`(target, leaf)` pairs total. Each target's CoMa, `proof.json`, and
`why3session.xml` is copied under `targets/`. `TARGETS.tsv` gives the path,
recursive valid-leaf count, and SHA-256 for each artifact; `SHA256SUMS` covers
the entire bundle except itself.

The translation ran from the two package directories with
`CARGO_NET_OFFLINE=true`, `RUSTUP_TOOLCHAIN=nightly-2026-02-27`, and
`CARGO_TARGET_DIR=/tmp/httparse-memory-pointer-active-20261005`, using
`cargo creusot -- --manifest-path Cargo.toml`. `CREUSOT_RUSTC` was unset, so
`cargo-creusot` selected the activated byte compiler at
`/workspace/proof-tools/creusot-data/toolchains/nightly-2026-02-27/bin/creusot-rustc`
(SHA-256 `1ee46c7a5e05f3dbbdbd03804dff6468338c36134711840372e606bfae0d4b7e`).
Both translations completed without warnings.

Proof batches used `why3find prove --no-cache -s -j 1` through
`httparse/1.10.1/run-proof.bash` from the exact probe working directory. The
wrapper held one prover and a 1000 MiB limit. The prover was Z3 4.15.3; the
checked Why3 config SHA-256 was
`e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`, and
both probe `why3find.json` files have SHA-256
`a5609701e7760be9f89cada433de5274372ea89856f2e01803c619586e808aca`.
The compiler binary, `why3find`, config, and wrapper were unchanged across the
refresh. Exact target counts are in `TARGETS.tsv`; the full proof commands are
listed in `../../NEXT-PROOF-MANIFEST.md`.

Why3 sessions for `produces_refl` and `produces_trans` retain an earlier
unsplit parent attempt (`timeout` and `unknown`, respectively). In each case,
`split_vc` produced two child goals and both are `valid`; the enclosing goal
and file are marked proved. Validation counted the successful transformed
leaves recorded in `proof.json` and checked that the corresponding successful
Why3 proof paths contain the same number of valid leaves. These historical
parent attempts are not unresolved obligations in this snapshot.

The current proof boundary and trusted standard-library assumptions are
recorded in [`../../REPORT.md`](../../REPORT.md). This is a selected
`Bytes`/iterator component proof, not a proof of the complete HTTP parser.
