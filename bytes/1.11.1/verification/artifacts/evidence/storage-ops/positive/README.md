# `storage-ops` positive evidence

This archive records the vanilla Creusot 0.13 proof and native tests for two
standalone loops over caller-owned `MaybeUninit<u8>` slices:

- `fill_uninit_prefix` writes `value` to exactly the first `count` slots.
- `copy_to_uninit_prefix` copies exactly the source length into the destination.

Both helpers preserve the slice length and frame every slot after the written
prefix. The proof run discharged all four generated VCs (the two helper bodies
and their public probe wrappers). The native test log records four passing
tests, including full-prefix initialization from entirely unknown storage.

The helpers have no new trusted contracts or memory operations. This is an
isolated helper result; `BytesMut` runtime integration and its ownership
protocol are outside this proof.

`source/` contains the exact helper, probe, lock/config files, and wrapper used
for the archived run. `proof-artifacts/` contains the generated Coma and proof
JSON. `logs/` contains the actual wrapper and native test output. The
`manifest.sha256` file hashes every archived file except itself.
