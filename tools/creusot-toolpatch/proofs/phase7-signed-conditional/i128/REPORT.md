# Scratch-only signed i128 suffix proof

The patch is based on the older `phase7-u128-suffix-probe` scratch crate, not
on a current repository commit. It preserves the signed writer's
`unsigned_abs`, one-byte leading gap, slice-to-array conversion, actual unsigned
formatter call, and optional `'-'` write. The proof adds a state-sequence
projection to connect the array suffix frame to the containing buffer. The
existing canonical-byte, initialization, length, and frame postconditions are
retained.

Scratch targets completed as follows:

- suffix projection helpers: 1/1 each;
- `write_u128_suffix_i128`: 29/29;
- `signed_write_i128`: 14/14;
- `i128::MIN` output witness: 10/10, including all 40 initialized bytes.

These caller proofs consume the Phase 6 `u128::fmt` contract. They remain
conditional until the actual `u128::fmt` body is integrated and its Phase 4
`div_rem_1e16`/`mulhi` dependencies are proved in the integrated crate. This
scratch result is not current-main evidence and the patch is not directly
applicable to current main.
