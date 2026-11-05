# Original XorShiftRng Serialize: concrete protocol supplement

Kani 0.68.0 / nightly-2026-08-21 verified one harness: 536 checks, zero failures, five unreachable, 8.458 seconds. One authorized run under a 60-second process limit; exit0. Raw output is kani.log. This is a separate Kani supplement, not a new Aeneas machine claim.

Original rand_xorshift0.5.0 path dependency, default+serde, exact rand_core0.10.0 / serde1.0.228 Cargo.lock. All original files match prior archive checks before/after. No observer, private-state injection, source or model edits. Existing owned rng-init target cache reused. The root dependency path must resolve to the preserved original; provenance and hashes are in status.json.

All16 symbolic seed bytes are passed to actual public from_seed. The custom finite serializer observes the original struct name XorShiftRng/length4, x/y/z/w field order, and actually invokes each T::serialize. The original Wrapping/u32 Serialize bodies then invoke the custom u32 callback. Independent output is seed bytes in LE order, except the all-zero seed maps to four0x0BAD5EED words. Derived PartialEq checks complete rng state unchanged; the derive itself is not a separate API claim.

One symbolic stage selects failure at any of ten callback entries: struct, fieldx,u32x,fieldy,u32y,fieldz,u32z,fieldw,u32w,end, or complete success. Arbitrary u32 error payload is propagated exactly; call count, completed field count, written prefix and untouched zero tail are exact. A failed u32 callback returns before writing. Unsupported fallible Serializer methods and skip_field return explicit error; no unsupported method succeeds. The unused nonfallible default is_human_readable query is outside this observed protocol.

This proves the actual Serialize call only for this concrete finite callback implementation and the image of all public seeds (including zero-seed fallback). It does not prove arbitrary Serializer callbacks, arbitrary private Rust-valid states, Deserialize, formats/encodings beyond this protocol, other configurations, or general serde correctness. Generic Lean extraction remains unverified due to its documented type-generation and Formatter boundaries.
