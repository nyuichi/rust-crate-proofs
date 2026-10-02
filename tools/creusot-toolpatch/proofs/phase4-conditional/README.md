# Phase 4 conditional divider evidence

These are isolated proof candidates based on repository commit `51090b4`.
Neither candidate is integrated into `itoa/1.0.18/src` on that commit.
The divider proof consumes the `mulhi` result contract, so its completed
focused goals are conditional until the separate `mulhi` body is proved.

## Divider candidate

`PHASE4_DIVREM_CONDITIONAL.patch` adds the actual `div_rem_1e16` body and
small proof helpers while preserving the multiply-high, right shift, quotient
subtraction, and narrowing cast. Its focused results are recorded in `logs/`:

- `shift_by_51`: 2/2.
- `magic_quotient`: 10/10.
- `math::magic_shift_floor`: 1/1.
- `div_rem_1e16`: 31/31, using the `mulhi` postcondition as a callee contract.
- Negative control: 26/27 after inserting the false assertion `quot < 0`.

Both default and all-features `cargo check --offline` passed. The patch has no
trusted local formatter or divider function. The proof is not a full crate
integration result.

To replay these targets, use a detached `51090b4` worktree and apply the divider
patch. From `itoa/1.0.18`, use the pinned environment in
`../../records/versions.md`; the patched compiler and Why3 overlay are required
for the 128-bit shift and cast proofs. Run the Why3 command outside the
filesystem sandbox because it uses Unix sockets. Focused selections use this
form (replace the selector for each listed component):

```sh
cargo creusot --no-check-version --simple-triggers=false prove --no-cache \
  --why3session runtime/shift_by_51 -- --offline
cargo creusot --no-check-version --simple-triggers=false prove --no-cache \
  --why3session runtime/magic_quotient -- --offline
cargo creusot --no-check-version --simple-triggers=false prove --no-cache \
  --why3session math/magic_shift_floor -- --offline
cargo creusot --no-check-version --simple-triggers=false prove --no-cache \
  --why3session runtime/div_rem_1e16 -- --offline
```

## `mulhi` residual-open candidate

`mulhi-residual-open/MULHI_CANDIDATE.patch` applies independently to the same
base commit and modifies only `itoa/1.0.18/src/u128_ext.rs`. It contains the
actual limb implementation with proof guidance, plus the exact source snapshot,
generated `mulhi.coma`, `proof.json`, and bounded-run log. To replay this
candidate, use a separate detached `51090b4` worktree, apply only this patch,
using `git apply --unidiff-zero`, and use the pinned environment. The captured
run was bounded at 90 seconds and ended at 49/51. The focused command is:

```sh
cargo creusot --no-check-version --simple-triggers=false prove --no-cache \
  --why3session arithmetic/u128_ext/mulhi -- --offline
```

The two unresolved formulas in the generated COMA are:

```text
x * y == result * 2^128 + residual
quotient == result
```

The second is an obligation after a call to
`exact_floor_from_split(x * y, result, residual, 2^128)`. The neighboring
`quotient == x * y / 2^128` formula passed. The
captured report describes this as a caller-side composition issue; this is not
evidence that the formulas are unprovable. Do not apply the divider and mulhi
patches sequentially: both were developed against the same base and overlap in
their `u128_ext.rs` changes. A future integrated patch must merge those source
changes, then prove the combined crate.
