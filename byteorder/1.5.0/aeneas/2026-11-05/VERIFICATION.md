For byteorder 1.5.0, the original BigEndian and LittleEndian write_u16, write_u32, write_u64, and write_u128 bodies write the specified endian bytes into the beginning of every sufficiently long output slice. All remaining bytes are preserved. The handwritten BigEndian::default and LittleEndian::default implementations always trigger an explicit panic; their empty enum types cannot be constructed by these methods. The eight write-method proofs quantify over arbitrary input integers and arbitrary output contents and establish normal return under the stated length precondition.

This report covers eight integer-write methods and the two handwritten Default methods on x86_64 with default features. It does not establish behavior of short buffers, read methods, variable-width integers, signed or floating-point wrappers, bulk slice operations, I/O extensions, or whole-trait integration. Rust source and package manifest are byte-identical to crates.io. No custom standard-library models or axioms are introduced.

Trusted assumptions: The official Aeneas standard-library models and Rust-to-Lean translation are the semantic boundary; Lean checks the stated theorems with only propext, Classical.choice, and Quot.sound.

Environment: Charon 08d7c9d50144bd84848212c24701253585e90d1f; Aeneas be86ab0f1989d04944292b345b24c435be3ad10a; Rust nightly-2026-09-17; Lean 4.35.0; x86_64 default features.

The extraction selector calls only the original public APIs. Its provenance guards require the sixteen actual original method bodies and their source spans; no method implementation is copied into the selector. See evidence/ and tools/ for reproduction.
