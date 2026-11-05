# Signed range-update counterexample — separate from successful single-bit proofs

With the unchanged bit_field 0.10.3 dependency, `let mut x = 112_i8; x.set_bits(0..4, 1);` terminates normally with `x == 1`. The initial byte is `0111_0000`; setting only bits 0–3 to `0001` while retaining other bits would produce `0111_0001` (113). The same unsigned example produces 113. The index range is valid and the positive inserted value fits four bits.

A positive Kani harness confirms these exact observations (103 checks, zero failures, two unreachable). A separate candidate outside-range-preservation assertion is refuted (one of 57 checks fails; two unreachable). Its nonzero exit is expected diagnostic evidence, not a successful proof of that contract. The original documentation says the method sets the specified range; the frame condition under test is stated explicitly in finding.json.

The implementation constructs the range mask by left/right shifting signed `!0`; arithmetic right shifts retain high ones, explaining the lost outer bits. This is a functional range-update counterexample, not a memory-safety finding. It does not alter the successful twelve-type single-bit get_bit/set_bit proofs or the separate bit_length proof.

The positive harness also observes signed get_bits sign extension. Extraction may intentionally interpret a signed field, so those get_bits examples alone are not called a defect. No claim is made that all other ranges or integer types exhibit this counterexample.

Rust source and manifest remain byte-identical to the archived release; SHA-256 values, target and toolchain are in finding.json. Only the verification package dependency path was relocated. Run verify.bash to reproduce the positive observation and the separately expected failing contract candidate. No custom stubs or trusted contracts are supplied.
