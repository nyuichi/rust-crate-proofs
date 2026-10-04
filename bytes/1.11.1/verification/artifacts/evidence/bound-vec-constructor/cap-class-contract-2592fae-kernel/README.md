# `from_vec` capacity-class contract gate

The source-extracted constructor gate passed all 35 generated proof artifacts.
The native probe passed 3/3 tests, including a nonzero capacity-class case.
The complete stdout logs are `proof.log` and `native_constructor_tests.log`;
`proof-artifacts/` contains exactly 35 `.coma` and 35 `proof.json` files.

This checkpoint adds a pure logical view of the original-capacity class and a
`from_vec` postcondition connecting the class bits in `data` to
`original_capacity_to_repr(result.cap)`. The logical view reuses the native
`leading_zeros` specification. It adds no runtime code or trusted contract.
The generated source and extraction manifest under `source/generated/` match
the exact `from_vec` body and declarations extracted from
`source/src/bytes_mut.rs`.

To avoid racing in-progress pool edits, this proof used the committed
2592fae snapshots of `raw_vec.rs` and `owned_region.rs`, together with the
current constructor, capacity, provenance, and probe sources. Their exact
SHA-256 hashes are listed in `source/sha256.txt`. This verifies the stronger
constructor contract over that kernel snapshot; it does not verify the newer
physical-pool code.

The gate covers the extracted constructor and explicit unique-at-zero release
method. It does not prove full-runtime `BytesMut` integration, automatic
`Drop`, promotion, split, reference counting, or shared-storage behavior.
