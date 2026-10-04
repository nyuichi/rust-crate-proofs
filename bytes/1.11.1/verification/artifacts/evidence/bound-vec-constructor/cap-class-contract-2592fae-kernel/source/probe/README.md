# Bound Vec constructor probe

The default proof includes the shared `ownership_proof::bound_ptr::detach_bound_vec`
helper and its real B1 implementation by path. It checks that detaching a
`Vec<u8>` produces a one-word bound pointer descriptor, preserves the exact
initialized prefix/capacity map, and carries the full matching recovery and
region capabilities. The positive path explicitly consumes those capabilities
through the bound B3 deallocation helper; it does not rely on automatic Rust
`Drop` effects.

With the `actual_from_vec` feature, `build.rs` extracts the exact `BytesMut` and
`Shared` declarations, `from_vec`, the constructor predicate/slot/release
methods, pointer helpers, and kind constants from `src/bytes_mut.rs`. It
records source line ranges and FNV-1a identifiers and compiles the extracted
fragments directly. The probe uses the real capacity, provenance, and
ownership-proof modules by path; it omits the rest of the runtime
implementation, including `Drop`. This feature calls the extracted
constructor and its explicit `proof_release_unique_at_zero` method, which
forgets the moved handle after deallocation. The result verifies that
constructor path and explicit cleanup in the extracted context, not automatic
drop behavior or the full runtime type.

The native `actual_from_vec` tests check that construction preserves the
allocation pointer, capacity, length, and bytes; encodes a nonzero
original-capacity class in the metadata word; and keeps `BytesMut` at four
machine words. They manually rebuild the Vec because the isolated extraction
intentionally has no `BytesMut` destructor.

Run the positive helper proof, its negative control, and the source-extracted
constructor gate from the crate root with the pinned toolchain:

```sh
./scripts/verify-bytes.sh bound-vec-constructor
./scripts/verify-bytes.sh bound-vec-constructor --features negative_unbound_cleanup
./scripts/verify-bytes.sh bound-vec-constructor --features actual_from_vec
cargo test --locked --features actual_from_vec
```

`negative_unbound_cleanup` intentionally erases the binding while preserving
the same pointer bits. Its call to the bound B3 helper fails at the matching
namespace/capacity/offset precondition.

Initial positive/negative helper artifacts and the first actual-constructor
gate are archived under `verification/artifacts/evidence/bound-vec-constructor/`.
