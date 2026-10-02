# Phase 7 signed proof scratch evidence

This directory preserves two signed proof attempts only. Neither patch has
been applied to the main runtime source. The i8 candidate is based on current
main source and remains unproved; the i128 candidate is from an older scratch
baseline and remains conditional on Phase 4/6.

## i8 candidate

`i8/PHASE7_MAIN_I8_CANDIDATE.patch` was developed against the source at
`999f8c3` and preserved as a zero-context diff against current main `b098c6a`;
it passes `git apply --unidiff-zero --check`. It extracts the original i8
signed suffix path and calls the actual shared `u8::fmt` body. Its only new
standard-library model is the type-specific `i8::unsigned_abs` magnitude
contract. The source translates, but `write_u8_suffix_i8` has seven open goals
and `signed_write_i8` has four. `check_signed_i8_write` passed one goal using
those helper contracts; that caller goal is not body proof. The stored proof
JSONs show the open states in `i8/evidence/`.

## i128 scratch candidate

`i128/PHASE7_I128_SIGNED_SUFFIX_SCRATCH.patch` applies only to the older
`phase7-u128-suffix-probe` scratch baseline described in its report. It is not
a patch against current main. Its scratch results were 29/29 for the suffix
writer, 14/14 for the signed adapter, and 10/10 for an `i128::MIN` output
witness. These results consume a Phase 6 `u128::fmt` contract and are
conditional on completing the Phase 4 divider/`mulhi` proof and integrating the
Phase 6 formatter. The associated report records the exact scope and boundary.

No fresh proofs or native tests were run while packaging these artifacts.
