# Actual BytesMut split, mutable access, and explicit release

This probe extracts the exact `BytesMut`, `Shared`, and `SharedBuffer` declarations
and actual `from_vec`, `promote_to_shared`, `shallow_clone`, `split_to`,
`split_off`, `advance_unchecked`, `as_slice_mut`, `spare_capacity_mut`,
`set_len`, `truncate`, and `clear` bodies from `src/bytes_mut.rs`. Source
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

Archived retained-prefix advance extension:

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

Current spare-initialization extension:

- All 86 positive proof files pass; twelve native integration tests pass.
- Actual `spare_capacity_mut` uses the additional u8-only B4-uninit physical
  reference bridge. It takes a sealed BoundPtr by value and an exclusive region
  borrow. Standard `MaybeUninit<u8>` Option views exactly match Known/Unknown
  slot state; prophetic writeback can initialize or re-uninitialize selected
  slots, while metadata and every other slot are preserved. Empty slices imply
  no allocation liveness. No ownership/refcount protocol is trusted.
- Body-proved callers initialize one spare byte on each side when available,
  using standard `write` or assignment from `MaybeUninit::new`, publish those
  bytes with actual `set_len`, read the values, and release both orders.
- Another caller truncates a Known prefix, writes `MaybeUninit::uninit`, proves
  the ledger is Unknown, rewrites it, and republishes it. A guarded negative
  attempts publication immediately after re-uninitialization; the Known-prefix
  requirement rejects it: one intended failed leaf (16/17 in the caller),
  among 87 proof files.
- Native tests cover zero/full/spare capacity, all split positions, both release
  orders, re-initialization, and unchanged exact A/S allocation/free counts with
  zero realloc calls. The standard Vec and MaybeUninit models are unchanged.

The initial descriptor/release-only checkpoint remains archived separately:
61 positive proof files and one missing-ticket rejection among 62 files.

The local trusted dependencies are the documented Vec/raw-allocation bridge,
sequential native atomic operations, generic boxed-allocation alignment, and
B4-bound initialized access and B4-uninit spare access. The alignment-bit lemma, pending transfer,
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
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_reuninitialized_growth
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_advanced_unknown
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_split_off_unknown
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_unknown_access
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_pending_access
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_stale_contents
./scripts/verify-bytes.sh sequential-bytesmut-split --features negative_missing_split_ticket
cargo test --offline --locked --manifest-path verification/probes/sequential-bytesmut-split/Cargo.toml --tests
```

Canonical evidence is under
`verification/artifacts/evidence/sequential-bytesmut-split/spare-initialization/`;
the previous retained-prefix checkpoint is under `retained-prefix-advance/`,
the previous split-off/length checkpoint is under `split-off-shrink/`, and
the earlier mutable-access checkpoint remains under the sibling `mutable-access/`.

## Read-only access checkpoint

The readonly-access archive contains 90 positive proof files and fourteen native
tests. Actual as_slice supports two shared reads of the left packet while the
right packet is mutated; contents and release in both orders are checked.
B4-read is an explicit physical-access boundary. The Unknown-read configuration
rejects one Known-slot guard among 91 files (caller 17/18). Safe trait dispatch,
unique-state access and a global valid-handle invariant remain subsequent work.

The readonly-access/lifetime-diagnostic archive records Rust E0505 rejecting
release of a handle while a later read keeps its as_slice borrow live. This is
translation/borrow-check evidence only, not a Why3 VC or automatic Drop proof.

## Unique offset-zero access checkpoint

The same actual access methods now accept a freshly detached unique handle as
well as a registered shared handle. Native KIND metadata selects the branch;
no Ghost value selects runtime behavior. The unique caller reads, truncates,
initializes spare storage, publishes the byte with `set_len`, mutates and reads
it, then explicitly returns full allocation authority to B3. Cleanup requires
full unique ownership and an offset-zero bound descriptor; it does not require
Known slots because B3 reconstructs a zero-length Vec and reads no byte.

The positive gate proves 93 files, and all sixteen native tests pass. Unique
allocator checks record no control allocation, no reallocation, and exactly one
buffer free when capacity is nonzero. The B4-bound canonical `slot_known`
postcondition is a redundant consequence of its existing exact byte writeback;
this checkpoint adds no physical or ownership trusted primitive.

`negative_unique_uninitialized_growth` truncates a unique handle, re-uninitializes
its first spare byte, and attempts actual `set_len(1)`. Ownership and capacity
remain valid; the intended rejection is the Known-prefix publication guard.
Unique advancement, safe trait dispatch, and a global handle invariant remain
separate work.

## Unique-at-zero access checkpoint

The unique-access positive archive contains 93 proof files with zero null leaves
and sixteen native tests. The ordinary KIND_VEC branch borrows its existing
constructor region for read/write/spare operations, and composes truncate and
Known-only set_len with explicit unique cleanup. It does not mint a second
region or allocate S. Explicit cleanup can free Unknown slots without reading;
access/publication still need Known bytes. Unique advance, global invariant,
actual traits and automatic Drop remain outside this checkpoint.

The unique-access/negative-unknown-publication archive records exactly one
failed Known-prefix guard in actual set_len after unique spare storage is
re-uninitialized (94 proof files; caller 13/14). Full allocation ownership and
Some(None) are established before that attempted publication.

The in-capacity extension extracts the actual `reserve`, `resize`, and
`extend_from_slice` methods. `reserve` proves its unchanged-storage fast path
under `additional <= capacity - len`; growing/reallocating reserve remains
excluded. Resize fills only newly exposed slots, and append copies only the
requested initialized source bytes. Their contracts preserve the native
allocation/registration identities and every byte slot outside the written
range. The `storage_ops` fill/copy loops run in normal builds as well as proof
builds; no proof-only replacement algorithm or additional trusted operation is
used. Publishing the new length uses the actual initialized-prefix `set_len`
contract.

`noalloc_unique` and `noalloc_split` clamp harness requests to available capacity;
the actual methods have the precise in-capacity preconditions. The split harness
uses actual `split_to`, updates the retained right handle, checks the left view
is unchanged, and explicitly releases in either order. Native fixtures pass
already-valid requests. The allocator-event test checks zero operation-time
allocations/reallocations for unique storage, and only the expected Shared
control allocation for split storage.

Unique view advancement follows the actual `advance_unchecked` Vec branch while
the packed offset fits `MAX_VEC_POS`. The handle retains the original Recovery
and full physical region; its current pointer offset plus remaining capacity
still equals the original allocation capacity. The method preserves every owned
slot and the low packed metadata bits, while visible slots become the old suffix.
`unique_advance` exercises two bounded advances, access and spare publication,
then explicit unique cleanup. That cleanup uses body-proved sealed pointer
retreat to recover the original base and passes the original full capacity to B3.
It does not free an interior pointer or discard the retired prefix. Native
allocator checks verify no Shared control allocation is needed. Offset-overflow
promotion, automatic Drop, and the full Buf trait implementation remain outside
this gate.

`negative_advanced_uninitialized_growth` checks that advancing an empty spare
allocation retains Unknown slots: actual `set_len(1)` still must reject the new
visible Unknown byte despite ownership of the entire original allocation.

The ARC branch of internal advancement also requires a valid affine registration
whose allocation namespace/capacity and control pointer match the descriptor.
This allocation guard permits the temporary geometry between registration and
pointer adjustment during a split, but excludes registration-free metadata. It
is needed for correspondence with the normal native `ptr.add` operation, whose
allocation must remain live; numeric wrapping arithmetic alone is insufficient.
`negative_unregistered_arc_advance` recovers/frees a unique allocation, retains
only copied bound metadata, and checks that manufacturing an ARC tag cannot
satisfy this guard.
