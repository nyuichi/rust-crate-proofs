# Step 1: symmetric x-side product split

This checkpoint adds one proof-only helper to `itoa/1.0.18/src/u128_ext.rs`:
`mulhi_x_split_product`. It has an ordinary `#[cfg(creusot)]` body, no
`#[trusted]` attribute, and does not change the native implementation.

## Focused result

The x-side helper body passes 1/1 VC using
`../../tools/creusot-toolpatch/scripts/run-proof.sh cargo creusot
--simple-triggers=false prove mulhi_x_split_product --why3session --no-cache`
from the crate directory, with one prover and a 1024 MiB memory limit. Its
COMA, proof JSON, and Why3 session are preserved here. The y-side helper remains
independently proved as recorded in the structural split report.

The small representative caller in `mulhi_product_split_from_limbs.rs` invokes
both helpers with the exact split equations as preconditions. Its focused proof
reported 3/4 goals. Both helper preconditions and the caller's `ensures` goal
pass, but the explicit body assertion remains open:

```text
x * y == (x_hi * 2^64 + x_lo) * (y_hi * 2^64 + y_lo)
```

The unresolved Why3 goal is `vc_mulhi_product_split_from_limbs.2.0`. The saved
session records Z3 `unknown` at 1.736 seconds and CVC5 timeout at 9.948 seconds.
The generated COMA and session are preserved beside this report.

A separate candidate that added both helper calls and the exact high-half
postcondition to `mulhi_core` reached the 120-second wall-clock limit without
emitting a proof result. That run is inconclusive. The caller assertion above
is the bounded, reproducible Step 1 result.

## Current trust boundary

Only the independently proved x-side helper is integrated at this checkpoint.
The failed tiny caller remains a source snapshot and is not part of the crate's
verification target. `mulhi` keeps its existing trusted equation
`result == x * y / 2^128`; the core contract and native runtime implementation
are unchanged.

## Checkpoint validation

- `tools/creusot-toolpatch/scripts/run-verify-all.sh /workspace/rust-crate-proofs`:
  passed both default and all-features proof configurations (exit 0).
- `cargo test --offline --locked` via `run-proof.sh`: passed 11 integration
  tests and 2 doctests.
- `cargo test --release --all-features --tests --offline --locked` via
  `run-proof.sh`: passed all 11 integration tests.
