# Scalar actual-source verification

This harness includes the published `src/version.rs` and `src/status.rs`
directly with `#[path]`. It proves source bodies from those files; it does not
copy or replace their runtime implementations.

## Verified targets

The following results were obtained with `scripts/run-proof.sh`, the pinned
Why3 setup, one prover, and a 1024 MiB memory limit:

| Actual source target | Result |
|---|---:|
| `Version::default` body and refinement | 2 VCs proved |
| `StatusCode::from_u16` | 8 VCs proved |
| `StatusCode::from_bytes` | 6 VCs proved |
| `StatusCode::as_u16` | 2 VCs proved |
| `StatusCode::is_informational`, `is_success`, `is_redirection`, `is_client_error`, and `is_server_error` | 10 VCs proved |
| Shared `NonZeroU16` consumer: construction plus `get` round trip | 4 VCs proved |
| Shared `NonZeroU16` consumer: equality and order refinements | 6 VCs proved |

The HTTP scalar targets total 26 VCs; adding the two `Version::default` VCs
gives 28. The separate standard-boundary consumer adds 10 VCs. Those consumer
proofs show how callers use the standard-library contracts; they do not verify
the bodies of `core::num::NonZero`.

## Standard-library boundary

`creusot-std/src/std/num.rs` defines an opaque `nonzero_value` observer and
exact external contracts for `NonZero<T>::new`, `get`, and `new_unchecked`.
The local primitive model has an implementation only for `u16`. `DeepModel`
for `NonZeroU16` unfolds to that same observer, so `StatusCode` has one
consistent scalar model. The contracts state that `new` returns `Some` exactly
for nonzero inputs and preserves the payload, `get` returns the payload, and
`new_unchecked` requires and preserves a nonzero payload.

These are explicit trusted contracts about the Rust standard library. Creusot
does not prove the `NonZero` implementation in `libcore`. The `StatusCode`
constructor/accessor bodies are proved against that boundary.

## Harness-only exclusions

Status proofs run with `RUSTFLAGS='--cfg http_status_leaf'`. This custom cfg is
not set by normal `http` builds or the crate's full Creusot configuration. It
omits only these items from this scalar harness because the translator cannot
currently handle their source expressions:

- `StatusCode`'s `Display` implementation, whose `write!` invocation uses the
  formatting literal `"{} {}"`;
- the generated public status constants, whose const initializers call
  `NonZeroU16::new_unchecked` with the registry literals;
- `StatusCode::default`, which refers to the excluded `StatusCode::OK` constant.

The actual `http` source retains all three in normal builds and in full-crate
Creusot builds without this harness-only cfg. They remain unproved. In
particular, the exclusions do not establish the constants or formatting
behavior through substitute models.

The harness can translate the remaining `StatusCode` source, but only the
targets listed above were proved. Other generated/derived traits, conversions,
`as_str`, `canonical_reason`, `Default`, formatting/debugging, and constants
remain unproved. The `as_str` translation also warns that its slice indexing
has no external contract.

For `Version`, only `Default` and its refinement are proved here. Its constants,
derived comparison/hash/clone traits, and `Debug` body remain unproved.

## Reproduction

Run from `http/1.5.0/verification/scalars`:

```sh
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove \
  'verif/http_scalar_proofs_rlib/version/impl_Default_for_Version/default.coma' \
  'verif/http_scalar_proofs_rlib/version/impl_Default_for_Version/default__refines.coma' \
  -- --locked --offline

RUSTFLAGS='--cfg http_status_leaf' ../../scripts/run-proof.sh cargo creusot \
  --simple-triggers=false prove from_u16 -- \
  --features status --locked --offline

RUSTFLAGS='--cfg http_status_leaf' ../../scripts/run-proof.sh cargo creusot \
  --simple-triggers=false prove \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/from_bytes.coma' \
  -- --features status --locked --offline

RUSTFLAGS='--cfg http_status_leaf' ../../scripts/run-proof.sh cargo creusot \
  --simple-triggers=false prove \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/as_u16.coma' \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/is_informational.coma' \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/is_success.coma' \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/is_redirection.coma' \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/is_client_error.coma' \
  'verif/http_scalar_proofs_rlib/status/impl_StatusCode/is_server_error.coma' \
  -- --features status --locked --offline
```

Run the standard-boundary consumer proofs from `http/1.5.0/verification/nonzero`:

```sh
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove \
  checked_nonzero_round_trip nonzero_equality_matches_u16 \
  nonzero_order_matches_u16 -- --locked --offline
```

An unfiltered normal check also passed from `http/1.5.0`:

```sh
scripts/run-proof.sh cargo check --locked --offline
```

This leaf evidence does not establish a successful integrated Creusot run for
the full HTTP crate.
