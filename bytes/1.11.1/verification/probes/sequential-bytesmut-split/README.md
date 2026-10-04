# Actual BytesMut split, mutable access, and explicit release

This probe extracts the exact `BytesMut`, `Shared`, and `SharedBuffer` declarations
and actual `from_vec`, `promote_to_shared`, `shallow_clone`, `split_to`,
`split_off`, `advance_unchecked`, `as_slice_mut`, `set_len`, `truncate`, and `clear` bodies from `src/bytes_mut.rs`. Source
offsets and hashes are recorded at build time. The gate starts with a freshly
detached Vec at offset zero, performs one `split_to` or `split_off`, borrows both resulting
mutable slices, and explicitly releases the handles in either order.

Promotion moves the constructor's existing Recovery and full PhysicalRegion
into a private pending owner with the typed Shared permission and exclusive
counter authority. It does not detach a second Vec. The shallow copy duplicates
metadata only. The first split initializes the existing two-ticket registry at the
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

Archived mutable-access checkpoint:

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

Archived split-off and length-change extension:

- All 77 positive proof files pass, including the previous mutable-access bodies.
- `split_off` accepts every position up to capacity, including beyond length.
  Lengths are min/saturating differences, capacities partition exactly, and every
  Known or Unknown slot is preserved on its side. A returned empty view over
  spare capacity does not turn its Unknown slots into readable bytes.
- `truncate` and `clear` retain all owned slots. `set_len` requires registered
  ownership, an in-capacity length, and Known slots throughout the new visible
  prefix. The caller shrinks and then restores the old Known prefixes.
- Seven native tests cover empty/full/spare buffers, all capacity positions,
  both release orders, simultaneous writes, and retained-Known regrowth. Native
  allocator instrumentation records one S allocation, two A/S frees for nonzero
  capacity (one S free at capacity zero), and zero realloc calls.
- The new negative uses actual `split_off` over spare capacity, proves a valid
  registered region and an Unknown first slot, and attempts actual `set_len(1)`.
  Its initialized-prefix precondition rejects this growth: one intended failed
  leaf (15/16 in the caller), among 78 proof files.

The `split_off_both` and `shrink_split_off` harnesses clamp their requested index
to the capacity returned by the actual constructor, because the unchanged Vec
sequence model has no capacity field. Native edge tests supply valid indices.
The actual `split_off` implementation does not clamp: its proved precondition
and native assertion require `at <= capacity`.

Current retained-prefix advance extension:

- All 79 positive proof files pass. The actual internal `advance_unchecked`
  method supports registered interior views up to capacity, using saturating
  visible length and exact capacity reduction. The separate `Buf::advance`
  trait implementation is not extracted or claimed here.
- Each handle retains its entire original affine packet, including discarded
  prefixes. Its pointer-relative visible window is a suffix of that region.
  Advance preserves every absolute packet slot, and subsequent mutable access
  frames the discarded prefixes. Explicit retirement returns the entire packet,
  so full allocation recovery needs no prefix stash or additional resources.
- First-split contracts preserve exact original allocation bindings at offsets
  zero and `at`; generalized interior validity does not replace those facts.
- Nine native tests cover counts zero, visible length and capacity, including
  advances beyond length into spare capacity, both release orders, and exact
  allocation/free/realloc events.
- A guarded negative advances into Unknown spare capacity and attempts actual
  `set_len(1)` after proving valid interior geometry and an Unknown first slot.
  Exactly the initialized-prefix precondition fails (19/20 in the caller),
  among 80 proof files.

The `advance_split_off` harness clamps each requested count to that side's
capacity. The actual internal method requires `count <= cap`; its native pointer,
length and capacity operations are unchanged. This extension adds no trusted
boundary: only body-proved ownership-view and frame contracts changed.

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
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_advanced_unknown
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_split_off_unknown
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_unknown_access
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_pending_access
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_stale_contents
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_missing_split_ticket
cargo test --offline --locked --manifest-path verification/probes/sequential-bytesmut-split/Cargo.toml --tests
```

Canonical evidence is under
`verification/artifacts/evidence/sequential-bytesmut-split/retained-prefix-advance/`;
the previous split-off/length checkpoint is under `split-off-shrink/`, and
the earlier mutable-access checkpoint remains under the sibling `mutable-access/`.
