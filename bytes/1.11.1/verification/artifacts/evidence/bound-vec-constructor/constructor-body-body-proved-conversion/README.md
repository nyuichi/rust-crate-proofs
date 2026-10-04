# Source-extracted constructor gate, body-proved conversion

This run uses the current `src/bytes_mut.rs` extraction and the reviewed
NonNull-based `RawAllocation::into_bound_ptr_at_zero` body proof. The probe
passed all 35 generated Coma artifacts; this directory contains exactly 35
`.coma` files and 35 `proof.json` files under `proof-artifacts/`.

The extracted source manifest records exact source ranges and FNV-1a hashes
for `BytesMut`, `Shared`, `KIND_VEC`, `KIND_MASK`, `from_vec`, the restricted
constructor predicate/slot/release methods, `vptr`, and `invalid_ptr`. The
source snapshot records SHA-256 digests for the runtime modules and probe.
`raw_vec.rs` has SHA-256
`ab8a196943d9457f1825bec87a101038b1f59d2351ac95de101d03fc9502a56a`.

The native tests passed 2/2. They verify pointer identity, capacity, length,
contents, and four-word native `BytesMut` layout. They reconstruct the Vec
explicitly because this extraction does not include the runtime `Drop` impl.

This gate verifies the extracted constructor followed by its explicit unique
release method. It still depends on the existing B1/B3 trusted physical bridge
contracts and does not claim proof of full-runtime `BytesMut` drop, promotion,
split, reference counting, or shared-storage behavior.
