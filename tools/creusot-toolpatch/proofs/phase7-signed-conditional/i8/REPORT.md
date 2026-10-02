# Current-main i8 candidate status

Candidate source base: `999f8c322fcd6e1cf41ac552b42bdcfb5e1e7cf7`. The
preserved zero-context patch is generated against current main `b098c6a` and
touches only `itoa/1.0.18/src/runtime.rs` and
`itoa/1.0.18/src/verification.rs`. It passes
`git apply --unidiff-zero --check` on that base.

The extracted `write_u8_suffix_i8` helper calls the actual shared unsigned
`u8::fmt` body over the three-byte suffix used by the native signed wrapper.
`signed_write_i8` retains `i8::unsigned_abs` and the original optional minus
byte write. The only new external model is the exact i8 unsigned-magnitude
contract. No local formatter helper is trusted.

Creusot translation succeeded. The latest bounded proof state is not green:
`write_u8_suffix_i8` has seven unresolved leaves and `signed_write_i8` has
four. `check_signed_i8_write` passes 1/1, but only by consuming those two helper
contracts; it does not establish their bodies. The three proof JSON files are
in `evidence/`. Stale scratch results from a different base are excluded.

The candidate is not integrated. No proof-suite or native-test result is
claimed for it.
