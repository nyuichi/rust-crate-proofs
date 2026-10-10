# Native environment evidence bundle

This bundle freezes the native-env compiler checkpoint after the first three
actual derived equality body proofs.

## Targets and results

`positive-coma-targets.txt` records all 26 fresh positive COMA files and
`negative-coma-targets.txt` records all 14 fresh negative harness files. The
selected proof target files are copied in `inputs/`:

- `inputs/proof-nativeenv-eq.targets` lists the only three targets sent to a
  solver under the new compiler.
- `inputs/proof-positive.targets` lists the remaining selected positive batch
  together with those three bodies.
- `inputs/proof-negative.targets` lists the two deliberately false selected
  claims. The two negative claims and their dependencies passed type checking
  only; they were not sent to a solver under this compiler.

All 26 positive and 14 negative COMA files passed `why3 prove --type-only`.
The exact selected equality body results are:

| Target | Saved Why3 goal | Result | Session |
|---|---|---|---|
| Error equality body | `vc_eq_Error` | Valid | `eq-proofs/Error/why3session.xml` |
| SignedI8 equality body | `vc_eq_SignedI8` | Valid | `eq-proofs/SignedI8/why3session.xml` |
| SignedI16 equality body | `vc_eq_SignedI16` | Valid | `eq-proofs/SignedI16/why3session.xml` |

Each saved session has one named goal and marks it `proved="true"`. The
corresponding `proof.json` files list the same named goal. The raw prover
JSONL files are preserved under `logs/`. They each include some fast-path
records named `<none>`, including one-second `Timeout` entries; these are not
goals in the saved selected-target sessions. The selected body goals all
finished `Valid`.

The generic fixture in `inputs/failclosed-generic/` intentionally lacks the
`T: DeepModel` predicate needed by the inherited external specification. Its
translation log preserves the expected compiler diagnostic that the
predicate is not proven by the native parameter environment. No solver was
run for this fixture.

## Exact proof invocation

The package preflight resolved `creusot` to the isolated package directory at
`/workspace/scratch/httparse-derived-traits-nativeenv/creusot-data/share/why3find/packages/creusot`.
The isolated and global Why3 configs both had SHA-256
`e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`; the
proof wrapper confirmed one prover and 1000 MiB.

Each target was run separately from the harness root through the elevated
`run-proof.bash` wrapper, using the isolated config and package:

```sh
/workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash env \
  WHY3CONFIG=/workspace/scratch/httparse-derived-traits-nativeenv/why3.conf \
  CREUSOT_DATA_HOME=/workspace/scratch/httparse-derived-traits-nativeenv/creusot-data \
  DUNE_DIR_LOCATIONS=why3find:lib:/workspace/scratch/httparse-derived-traits-nativeenv/creusot-data/share/why3find \
  XDG_CONFIG_HOME=/workspace/scratch/httparse-derived-traits-nativeenv/config \
  XDG_CACHE_HOME=/workspace/scratch/httparse-derived-traits-nativeenv/cache \
  /workspace/scratch/httparse-derived-traits-nativeenv/creusot-data/bin/why3find prove \
  -s -j 1 --no-cache --time 30 \
  --log-prover-results evidence/nativeenv-<target>-eq-prover-results.jsonl \
  verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_<type>/eq.coma
```

The `eq-proofs/` subdirectories contain the exact selected COMA, `proof.json`,
and Why3 session copied after each run. No solver was run on the other
18 selected positive targets, the two negative claims, or any unrelated
Unicode or formatting target in this checkpoint.

## Files and provenance

- `positive-coma/` and `negative-coma/` preserve all generated COMA trees.
- `inputs/` preserves the harness manifests, source, target lists, and the
  actual `src/error.rs` input.
- `logs/` preserves translations, type-only results, the generic rejection,
  and exact prover JSONL output.
- `toolchain/` preserves the native-env patch and build script, relevant
  compiler source, prelude, Why3 config, and the `num.rs`, `string.rs`, and
  `convert.rs` library inputs.
- `toolchain/TOOLCHAIN-HASHES.txt` records compiler, package, config, source,
  and prior-binary hashes. The original derived-traits and string-model
  binaries were left untouched.
- `REPORT.snapshot.md` freezes the sibling report at this checkpoint.

`SHA256SUMS` checks all files in this bundle except itself. Later report edits
do not change this frozen snapshot.
