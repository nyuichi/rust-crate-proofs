# Separate Kani verification of 40 integer byte APIs

`./verify.bash` checks the original path dependency and validates source hashes before and after the Kani run. `source/` is the byte-identical original crates.io archive. `src/lib.rs` contains ten symbolic harnesses with eight useful conversion/roundtrip assertions each. Target compile guards require x86_64 little-endian 64-bit pointers.

See VERIFICATION.md for scope. API identities and exact property bindings are in api-bindings.json; results.json records ten successful harnesses and 220 successful checks. inventory-delta.json proposes separate Kani statuses for the 40 concrete Rust API bodies, retaining owner type arguments. These results are unpublished and are not added to the Aeneas proof count or executable-model claims.

No generic raw-pointer slice API is invoked; the stopped Miri investigation is not used or resumed. The selected integer types admit every bit pattern, and the source transmute operations are reached directly through the original public methods.
