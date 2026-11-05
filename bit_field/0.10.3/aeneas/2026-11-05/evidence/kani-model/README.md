# bit_field 0.10.3 — supplementary checks for twelve primitive implementations

Kani 0.68.0 / CBMC 6.11.0 verified 24 harnesses and 600 checks with no failures. The Cargo path dependency is the actual unchanged release at `../bits/bit_field/source`; the harness does not copy or rewrite method implementations. Source and manifest SHA-256 checks before and after the run agree.

For each of `u8/u16/u32/u64/u128/usize/i8/i16/i32/i64/i128/isize`:

- The layout harness checks the concrete target's `size_of::<T>()` and the original `T::BIT_LENGTH` (1/2/4/8/16/8 bytes and 8/16/32/64/128/64 bits respectively for unsigned and corresponding signed types).
- The body harness quantifies arbitrary values, every valid `usize` bit index, both Boolean setter values and an arbitrary replacement value. It checks the original getter against a logical unsigned-bit-pattern reference, the setter against setting/clearing the corresponding unsigned mask, preservation of all other bits, the requested post-set bit, and a write through the returned mutable reference updating the original variable.

Signed values are cast to the corresponding unsigned representation for the reference formulas. All negative values and sign-bit indices are included; there is no nonnegative-value assumption. Every get/set body harness passes 48 checks and every layout harness passes 2. No failure was observed for these valid-index single-bit contracts.

These are supplementary one-step Rust checks, not an in-Lean proof of Rust layout or a mechanically checked connection to Lean models. The run targets x86_64-unknown-linux-gnu (`usize/isize` are 64-bit) and uses Rust nightly-2026-08-21, whereas Aeneas extraction uses nightly-2026-09-17. Invalid-index panics, `get_bits/set_bits`, BitArray methods, other targets, feature configurations and whole-crate guarantees are outside this artifact. In particular, it does not establish unsigned-mask behavior for signed range extraction, where arithmetic right shifts require a separate specification review.

Run `./verify.bash` in the current environment. `results.json` records per-harness counts, tool versions, source hashes and limitations. `verification.log` is the full successful Kani output. No custom Kani contract, stub or trusted function was introduced; the sole input restriction is the documented valid bit-index range.
