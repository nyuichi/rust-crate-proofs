# `u64::from_ne_bytes` standard-library boundary provenance

This checkpoint trusts the exact result contract for the standard function
`u64::from_ne_bytes([u8; 8])` for every input array. It does not prove the Rust
function body or `mem::transmute` semantics. The verifier contract maps the
result to the byte sequence interpreted in native-endian order; parser-specific
HTTP literals and equality are not assumptions.

The active compiler identity and target cfg are recorded in `rustc-vV.txt` and
`rustc-target-cfg.txt`. The active target is `x86_64-unknown-linux-gnu` with
`target_endian="little"`; the fresh proof therefore checks the little-endian
model branch only. The big-endian branch remains uncompiled and unproved.

The Rust source is from the active compiler sysroot at
`/workspace/proof-tools/rustup/toolchains/nightly-2026-02-27-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/`.
`uint-from-ne-bytes-template.txt` preserves lines 3989–4031 from
`uint_macros.rs`, whose full-file SHA-256 is
`189d28513c634d88df6a365d3a509ec7aade0d7eeec5ddb3976f22e91ae6e572`.
That macro documents native-endian interpretation and implements
`from_ne_bytes` using `unsafe { mem::transmute(bytes) }`, with the safety
comment that integers are plain old data types.

`u64-integer-macro-callsite.txt` preserves lines 1273–1299 of `num/mod.rs`,
whose full-file SHA-256 is
`6cc32b5143b27ad7691df0ebef327bbf8f4aaae9f07e37636f3ad27872d1aac7`. It shows
the macro instantiated in `impl u64` with `Self = u64`, `BITS = 64`, and the
native byte examples. The open finite packing model and body-checked base-256
injectivity lemmas in the crate are separate from this trusted conversion
boundary.
