# Original checked conversion body checks

Kani verifies the actual `ToAsciiChar::to_ascii_char` implementations for u8, i8, u16, u32 and char in unchanged ascii 1.1.0. Unsigned inputs succeed exactly at codes 0–127; i8 succeeds exactly when nonnegative; char succeeds exactly for ASCII codepoints. Every Ok value has exactly the input ASCII code. Other inputs return ToAsciiCharError.

The separate harness package depends on the artifact root via `path = "../.."`. Original Rust files and Cargo.toml are archive-compared before each run. The harnesses directly call each public implementation, match the real Result, and observe an Ok value through the original public as_byte method. They contain no copied conversion implementation or private/raw-memory observer.

Each integer harness quantifies its entire bit width. The char harness ranges over every Unicode scalar value using an arbitrary u32 constrained only by the exact Rust char validity conditions, constructs it with char::from_u32, and asserts its codepoint roundtrip. These paths have no loops; no sample-size or unwind override is used.

A sixth, separate dependency check verifies `transmute::<u8,AsciiChar>` preserves the byte for all valid discriminants 0–127. It assumes this valid domain before construction, and makes no claim about invalid enum states or unchecked public methods on invalid inputs.

Reproduce with `./verify-kani-conversions.bash` at the artifact root. Kani 0.68.0 / CBMC 6.11.0 use nightly-2026-08-21 on x86_64 with default features. Trust includes the compiler and Kani/CBMC standard/intrinsic semantics. These body checks are separate from the twenty-one Lean API claims, and do not establish a mechanized Kani-to-Lean semantic bridge.
