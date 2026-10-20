# Comparison extraction harness

This crate exists only to make selected `bytes` comparison implementations
reachable from Charon. It depends on `bytes-adapted` and calls the public
cross-type `PartialEq` and `PartialOrd` methods from small helper functions.
The harness does not modify the `bytes-adapted` Rust sources and is not itself
counted as verified library behavior.

`Cargo.toml` pins 50 helper roots and repeats the exact 209-pattern opacity
profile in `verification/opaque-baseline.json`. The roots expose 46 distinct
comparison bodies from `src/bytes.rs` and `src/bytes_mut.rs`; generic
reference-method roots share the corresponding generic source bodies.

From the repository root, replay extraction with:

```sh
verification/extensions/comparisons/extract.sh
```

The script checks the frozen Rust hashes and opacity set, runs Charon with
`--include bytes`, translates the LLBC with Aeneas, and records the raw
translation hashes in `verification/extensions/comparisons/extraction.json`.
The extension's `Generated.lean` differs from raw Aeneas output only by the
additional `public import FloatTypes` needed by the generated module.
