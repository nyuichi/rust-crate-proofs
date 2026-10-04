# Actual Shared control gate

This probe extracts the exact `Shared`, `SharedBuffer`, and restricted
`sequential_shared_control` helper block from `src/bytes_mut.rs`. The build
script records source offsets/hashes. It selects the source's proof field
adapters and includes the exact `SharedBuffer::drop` body in native tests.
Automatic Drop remains outside Creusot's modeled effects.

The helper allocates a real boxed Shared with one buffer descriptor, consumes
real B1 byte capabilities into the two-ticket registry, and connects the native
counter to the number of pending registrations. Both release orders execute
native Release decrements. Only the native final-count branch obtains full
byte capabilities, disarms the descriptor, calls B3 for the byte allocation,
and consumes typed Shared permission through ordinary `Perm::drop`.

Results: 45 proof files pass. The missing-empty-ticket configuration has one
intended failure (16/17 in that caller): the byte region is fully returned but
one registration remains, so `shared_protocol::finish` is unavailable. Native
smoke and allocator-count tests pass; they observe one Shared allocation and
two frees for nonzero byte capacity, or only the Shared free for zero capacity.

The new helper is not the existing `BytesMut::from_vec -> promote_to_shared ->
split_to` path. That connection, concurrent use, existing `release_shared`, and
automatic Drop are not proved here. Unsupported old atomic proof paths have
explicit false-precondition adapters; they grant no counter authority.

No new trusted protocol function is introduced. This gate uses existing
physical B1/B3 contracts, the four documented sequential native counter
contracts, and standard Creusot typed permissions/invariants/resources. It
makes no formal physical-deallocation-event claim from `Perm::drop`; allocator
counts separately check the actual native frees.

```
./scripts/verify-bytes.sh sequential-shared-control
./scripts/verify-bytes.sh sequential-shared-control --features negative_missing_ticket
cargo test --offline --locked --manifest-path verification/probes/sequential-shared-control/Cargo.toml --tests
```

Canonical source/configuration/generated proof evidence is in
`verification/artifacts/evidence/sequential-shared-control/`.
