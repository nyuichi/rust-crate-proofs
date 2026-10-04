# Actual first BytesMut split and explicit release

This probe extracts the exact `BytesMut`, `Shared`, and `SharedBuffer` declarations
and the actual `from_vec`, `promote_to_shared`, `shallow_clone`, `split_to`, and
`advance_unchecked` bodies from `src/bytes_mut.rs`. Source offsets and hashes are
recorded at build time. The gate starts with a freshly detached Vec at offset
zero and performs one `split_to`, followed by explicit release in either order.

Promotion transfers the constructor's existing Recovery and full PhysicalRegion
into a private pending owner, alongside the typed Shared permission and exclusive
counter authority. It does not detach a second Vec. The shallow copy duplicates
metadata only. Once `split_to` knows the boundary, the existing registry issues
two affine tickets and distributes disjoint byte regions. One handle retains the
coordinator until the caller explicitly takes it; the handles never share two
mutable coordinators.

Release consumes each handle and packet. The native Release decrement decides
whether this is the final handle; the final path executes the Acquire load,
recovers full byte capabilities, disarms the Shared buffer descriptor, calls B3
for the byte allocation, and consumes the typed Shared permission through native
`Perm::drop`. `mem::forget(self)` suppresses a second handle destructor call.
Automatic BytesMut Drop, concurrent access, repeated splitting, reserve, and
byte mutation are outside this gate.

Results at the frozen checkpoint:

- Positive: all 61 proof files pass.
- `negative_missing_split_ticket`: 62 files, exactly one failed leaf (18/19 in
  `proof_reject_missing_empty_ticket`). It splits at zero, abandons the empty
  left handle, returns every byte through the right handle, then attempts full
  recovery while the left registration remains outstanding.
- Native: both integration tests pass, including every split position and both
  release orders. Allocator counts observe one Shared allocation and two frees
  for nonzero byte capacity, or one Shared free when byte capacity is zero.

The one added trusted fact is generic boxed-allocation alignment in
`ownership_proof/boxed_alignment.rs`: standard Creusot 0.13 typed `Perm::from_box`
preserves ward/value but omits pointer alignment. The separate alignment-bit
lemma and all pending/registration/handle protocol bodies are proved. Existing
Vec/raw-allocation and sequential atomic bridges remain explicit dependencies.
No formal physical-deallocation event is inferred from standard `Perm::drop`;
allocator-count tests separately check the native effects.

`cfg(bytes_proof_probe)` selects the same restricted representation branches for
native execution of the exact extracted source. Ordinary crate builds retain
the original handle layout and operations. Unsupported ARC cloning and unique
advance branches are explicitly excluded by the restricted preconditions.

```
./scripts/verify-bytes.sh sequential-bytesmut-split
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_missing_split_ticket
cargo test --offline --locked --manifest-path verification/probes/sequential-bytesmut-split/Cargo.toml --tests
```

Canonical source, configuration, logs, extraction records, generated proof code,
and proof JSON are under
`verification/artifacts/evidence/sequential-bytesmut-split/`.
