# Actual BytesMut split, mutable access, and explicit release

This probe extracts the exact `BytesMut`, `Shared`, and `SharedBuffer` declarations
and actual `from_vec`, `promote_to_shared`, `shallow_clone`, `split_to`,
`advance_unchecked`, and `as_slice_mut` bodies from `src/bytes_mut.rs`. Source
offsets and hashes are recorded at build time. The gate starts with a freshly
detached Vec at offset zero, performs one `split_to`, borrows both resulting
mutable slices, and explicitly releases the handles in either order.

Promotion moves the constructor's existing Recovery and full PhysicalRegion
into a private pending owner with the typed Shared permission and exclusive
counter authority. It does not detach a second Vec. The shallow copy duplicates
metadata only. `split_to` initializes the existing two-ticket registry at the
actual boundary and assigns disjoint byte regions. One handle retains the
coordinator until the caller takes it; there are never two mutable coordinators.
Exact slot values are preserved through promotion and partitioning.

The actual mutable view uses a body-proved packet-borrow component and the
B4-bound physical access contract. B4 derives its pointer directly from the
sealed BoundPtr; it takes no arbitrary pointer argument. Nonempty access requires
a matching exclusive initialized region. Empty access performs no pointer
arithmetic and requires no allocation-liveness claim. Prophetic writeback records
the final slice values and preserves every slot outside the borrowed interval,
including Unknown spare capacity. The canonical `slot_known` predicate is
body-proved equivalent to an existing initialized byte value.

The positive caller holds both disjoint slices simultaneously, writes different
bytes, checks the resulting values, and releases both handles. The native Release
decrement chooses the final-release branch. That branch performs the Acquire
load, recovers full byte capabilities, disarms SharedBuffer, frees A through B3,
and consumes typed S permission through native `Perm::drop`. Explicit handle
cleanup ends with `mem::forget(self)` to suppress a second destructor call.

Current mutable-access checkpoint:

- Positive: all 68 proof files pass.
- Native: four integration tests pass, covering simultaneous writes, empty
  views, all split positions and both release orders, and A/S allocation counts.
- Unknown-access rejection: one intended failed leaf, 18/19 in the caller.
  This diagnostic directly exercises B4-bound on a proved Unknown B1 spare slot.
- Pending-access rejection: one intended failed leaf, 6/7 in the caller, after
  the actual shallow clone has produced metadata without registered authority.
- Stale-content rejection: one intended failed leaf, 8/9 in the caller, after
  writing through the actual mutable-view method.
- Missing-empty-ticket rejection: one intended failed leaf, 18/19 in the
  caller; full byte coverage cannot replace an outstanding registration when
  attempting final recovery. Each negative configuration contains 69 proof files.

The initial descriptor/release-only checkpoint remains archived separately:
61 positive proof files and one missing-ticket rejection among 62 files.

The local trusted dependencies are the documented Vec/raw-allocation bridge,
sequential native atomic operations, generic boxed-allocation alignment, and
B4-bound initialized mutable access. The alignment-bit lemma, pending transfer,
ticket registry, packet borrowing, and actual handle protocol bodies are proved.
No trusted BytesMut ownership or reference-count protocol is introduced. Standard
`Perm::drop` is not treated as a formal physical-deallocation event; native
allocator-count tests separately check the frees.

`cfg(bytes_proof_probe)` selects the restricted representation branches for native
execution of the exact extracted source. Normal crate builds retain their handle
layout and operations. The proof representation excludes the native unsafe
Send/Sync impls, preserving its exclusive sequential counter authority. Concurrent
sharing, repeated ARC splitting, reserve, and automatic BytesMut Drop remain
outside this gate.

```
./scripts/verify-bytes.sh sequential-bytesmut-split
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_unknown_access
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_pending_access
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_stale_contents
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_missing_split_ticket
cargo test --offline --locked --manifest-path verification/probes/sequential-bytesmut-split/Cargo.toml --tests
```

Canonical evidence is under
`verification/artifacts/evidence/sequential-bytesmut-split/mutable-access/`.
