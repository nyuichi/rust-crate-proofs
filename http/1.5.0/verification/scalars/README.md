# HTTP scalar verification

This leaf harness includes the published `src/status.rs`, `src/version.rs`, and `src/ascii.rs` directly. It proves those production bodies under the `status` feature and `http_status_leaf` translation configuration.

## Current proof result

At the proved source snapshot, a clean translation emitted the scalar targets. A fresh bounded `--no-cache` run passed all 80 supported targets and 381 leaves: 133 own-goal leaves and 248 callee-contract leaves. It excluded the two named status-array capability probes. Target-by-target COMA and proof JSON hashes, source hashes, solver versions, and run details are in [final-2026-10-05.json](evidence/final-2026-10-05.json).

A later clean emission from the current `status.rs`, `version.rs`, and `ascii.rs` produced full, untransformed Why3 task streams byte-identical to the proved streams for all 80 accepted targets. The complete recorded proof trees and all 381 successful leaves therefore remain applicable without another solver run. Independent solver-free `split_vc` checks matched all seven root tactic nodes (60 child tasks); there were no nested tactic nodes. Exact emitted streams, proof-tree reuse checks, arities, and report hashes are recorded under [current-source-reconciliation-20261005](evidence/current-source-reconciliation-20261005/).

## Covered behavior

For `StatusCode`, the current proof set covers numeric and byte construction, `as_u16`, all five status-class predicates, exact `canonical_reason` results, all 900 decimal `as_str` results, `Default`, `Clone`, `PartialEq`, `PartialOrd`, `Ord`, `Hash`, `Debug`, and `Display` bodies and refinements. It also covers `FromStr`, the three `TryFrom` conversions, conversions to and from `u16`, and both directions of equality against `u16`.

`InvalidStatusCode` construction and its `Debug`/`Display` formatter append-preservation contracts are covered. For `Version` and its private `Http` representation, the proof set covers the exact `Http::numeric_value` helper, `Clone`, `Eq`, `PartialOrd`, `Ord`, `Default`, `Hash`, and `Version` `Debug` bodies/refinements. `Version::Default` proves the exact numeric value 11.

`StatusCode`, `InvalidStatusCode`, and `Version` formatting proofs establish formatter append preservation. `StatusCode` Display output, formatting flags, and partial-write behavior are checked separately by a runtime comparison with the former `write!` implementation. Hash proofs establish valid hasher-invariant preservation only; they do not claim a digest value.

The status digit table is generated as a by-value `[u8; 2700]` constant and returns the same static decimal text for every supported code. The tests cover codes 100 through 999.

## Exclusions and translator boundary

The 62 associated numeric `StatusCode` constants are not included in the leaf proof. A bounded six-caller attempt failed during constant translation with `unsupported constant expression pattern_type!(u16 is 1..)`, at the `NonZeroU16` niche field of the macro initializer. Changing the initializer to checked `NonZero::new` cannot avoid the same evaluated field type. The exact attempt and diagnostic are recorded in [status-constants-pilot-2026-10-05.txt](evidence/status-constants-pilot-2026-10-05.txt); no numeric-constant proof claim is made.

The capability probes `status_array_model_probe` and `status_array_model_borrowed` are excluded from the accepted proof batch. A leftover proof JSON without a current COMA target is also not counted; it is identified in the final evidence file.

## Runtime regression

From this directory, the final runtime suite passed 3 integration tests:

```sh
RUSTFLAGS='-Zcrate-attr=feature(hasher_prefixfree_extras,stmt_expr_attributes,proc_macro_hygiene)' \
  cargo test --tests --features status --locked --offline
```

These tests cover the Version Hash callback trace, decimal text for all 900 codes, and StatusCode Display flags/chunks/errors.

The suite was rerun against the reconciled current source and passed all 3 integration tests; the command and output are preserved under [current-source evidence](evidence/current-source-reconciliation-20261005/runtime-test.command).

## Reproduction

With the repository tool environment activated, emit the current scalar targets with:

```sh
CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http cargo clean -p http-scalar-proofs --offline
RUSTFLAGS='--cfg http_status_leaf' CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http \
  cargo creusot --simple-triggers=false -- --features status --locked --offline
```

Run the 80 accepted proof targets through the shared wrapper, leaving out the two named capability probes:

```sh
mapfile -d '' targets < <(find verif/http_scalar_proofs_rlib -name '*.coma' \
  ! -name 'status_array_model_probe.coma' \
  ! -name 'status_array_model_borrowed.coma' -print0)
RUSTFLAGS='--cfg http_status_leaf' ../../scripts/run-proof.sh cargo creusot \
  --simple-triggers=false prove --no-cache "${targets[@]}" -- \
  --features status --locked --offline
```

These are isolated source-body proofs, not integrated verification of the complete HTTP crate or dependencies such as `bytes`. NonZero implementation bodies and standard formatting/hasher callbacks remain library contract boundaries, with claims limited to their recorded specifications.
