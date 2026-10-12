# Current trusted assumptions and scope

The proof-mode B1 path now uses the generic trusted capture
`VecStorage::capture_vec_owner`; `VecRawParts::take` is body-proved composition,
not the trusted boundary. Capture consumes the Vec into opaque native storage,
reads its `as_mut_ptr`, `len`, and `capacity` once, and binds that exact tuple
and the Vec contents to a sealed `VecPartsWitness`. It grants no recovery,
region, allocation-liveness, or dereference authority. Its generic
interpretation assumes the default global allocator and `u8` layout,
exclusive ownership transferred by moving the Vec, capacity at most
`isize::MAX`, and a nonwrapping base-plus-capacity address range. At zero
capacity the dangling pointer is metadata for an empty region only.

The body of `bind_consumed_vec` constructs the logical recovery token and full
slot region from the witness: the initialized prefix matches the Vec contents,
and spare-capacity slots are unknown. The selected bodies for
`VecRawParts::take` and `VecStorage::hold_owned` compose the capture with this
resource construction; `take` suppresses the temporary storage destructor
after transferring its native owner with the existing Std
`core::mem::forget(storage)` operation. `hold_owned` returns the body-proved
storage authority and region; the private count-one promotion helper calls it
to construct the initial Shared state. This does not prove later Shared
lifecycle operations. The proof uses an opaque transparent native Vec wrapper
and an outer C-layout storage struct with a zero-sized ghost field;
compile-time assertions compare its
size and alignment with `Vec<u8>`. The native `VecRawParts::take` branch keeps
the original direct extraction implementation. No trusted Bytes-specific
ownership, refcount, destructor, or reclamation law is used here.
Removing the capture assumption requires deriving the exact native metadata
and consumed-owner transfer from verified Vec, allocator, and `ManuallyDrop`
contracts; proving only `bind_consumed_vec` would leave that assumption in
place.

The selected `from_vec`, `with_capacity`, `new`, `zeroed`, `From<&[u8]>`,
`From<&str>`, and `Default` bodies establish their specified initialized
prefix and unknown spare slots. They also establish `unique_vec_rebuildable`:
the visible initialized bytes and any hidden prefix needed when reconstructing
a Vec are known; spare-capacity slots remain unknown. `clear` preserves the
state and all capacity-slot values while setting length to zero. The body of
`is_empty` is checked without a representation precondition. The capacity
encoder and decoder, and local ownership/provenance helpers are checked
against this fork's source. The proof applies to this verification fork based
on crates.io bytes 1.11.1; the native refactors here are not shown equivalent
to the unmodified release.

The `From<&str>` result uses the generic Std model `Seq<char>::to_bytes()`.
The `str::as_bytes` contract equates native string bytes with that model, whose
opaque `char::to_utf8` operation is not proved here to match Rust's concrete
UTF-8 encoding. The precise assumption and replacement path are documented in
the [Std overlay notes](../../std-support/README.md).

`len` and `is_empty` are proved without a representation precondition.
`truncate`, `clear`, unsafe `set_len`, and the implementation body of
`BufMut::advance_mut` are proved from the cursor-aware `unique_valid`
precondition; `set_len` and `advance_mut` also require capacity and
initialized-slot bounds for growth. The separate generated trait-refinement
obligation for `BufMut::advance_mut` is not proved: the trait prestate does not
establish the proof-only validity and initialized-slot preconditions added to
the implementation body. Therefore this body proof does not establish the
public trait refinement.
The logical invariant and prefix helpers retain the complete allocation frame
at an absolute cursor, and the metadata helpers prove the packed Vec position
updates. The selected constructor bodies establish `unique_vec_rebuildable` at
offset zero. Body proofs for `advance_vec_layout` and `set_visible_len`
establish the two phases of a Vec cursor advance while preserving all
allocation slots. The Vec `advance_unchecked` branch is body-proved with a
rebuildable Vec state, `count <= len`, capacity bounds and representable packed
position. The packet-backed Shared branch is body-proved with the handle's
packet-layout and initialized-visible-prefix preconditions.
`advance_shared_layout` and `SharedOwnerPacket::advance_cursor` preserve the
packet capsule, ticket, physical-region descriptor, base and allocation slots
while advancing the cursor, pointer, visible suffix and remaining capacity.
The packet-layout predicate does not connect each handle's physical region to
the capsule's protected storage authority after later transitions; this proof
therefore does not establish Shared reads, recovery, exclusivity or lifecycle
behavior. No selected proof establishes a safe public call chain that produces
a nonzero offset: public `Buf::advance` is untranslated. The trait-refinement
obligation for `BufMut::advance_mut` also remains unproved because its body has
stronger proof preconditions than the trait prestate. Immutable peers,
refcount-to-owner correspondence, concurrency and unwind behavior remain
outside the selected `BytesMut` proof scope.

The native Shared reserve-growth path now compacts the handle's initialized
visible bytes to the Vec base before setting its length and calling
`Vec::reserve`. This prevents a hidden prefix that another disjoint handle
could leave uninitialized from being included in the Vec length. The existing
regression test exercises that sequence. Shared reserve remains
`no_translate`, so this is native test evidence, not a body proof; it does not
establish Shared ownership or lifecycle properties, nor claim that a prior
execution was observed to have undefined behavior.

The supporting `split_view_region` body preserves the full physical allocation
while splitting the cursor-relative visible region, including any hidden
prefix. The selected `BoundPtr` projection and `advance_within` bodies preserve
the bounded pointer relationship used by the cursor helpers. The selected
bodies use generic Creusot Std ghost/resource and logic contracts, plus the
address-only `wrapping_add_u8` specification. Their Vec observer and resource
assumptions are documented in the
[Std overlay](../../std-support/README.md). Verification-only exclusions for
unrelated comparison, ownership, iterator and Shared-backed bodies are
documented beside the source in [`bytes.rs`](../../../src/bytes.rs) and
[`bytes_mut.rs`](../../../src/bytes_mut.rs).

The refcount operation layer has a private `#[repr(transparent)]`
`NativeCounter` around the existing atomic type and a `#[repr(C)]` `RefCountCell`
with a proof-only zero-sized `Option` sidecar. Native builds execute the
original atomic operations once and preserve their orderings. Selected body
proofs cover the invariant constructor, native-counter getter, Relaxed and
Acquire loads, both compare-exchange wrappers, Release decrement, the checked
next-count function, and the `try_increment`/`increment` loop. They also prove
`take_bootstrap_owner`, which consumes the invariant from a populated sidecar,
returns its unique permission and recorded scalar value, and leaves the cell's
sidecar empty while framing the native counter. The `EventBoundInvariant`
`into_inner` body is proved to move out the same permission and protocol state.
The proof-only `new_with_owner` constructor body is also proved: it creates the
native counter once and returns its exact sole permission with the constructor
value while leaving the cell sidecar empty. The proof branch of
`promote_to_shared_count_one` uses this helper to connect the corresponding
native counter construction to the promotion capsule; native builds use the
ordinary `RefCountCell::new`. The helper by itself creates no handle or
owner-ledger registration.
No production `Bytes` or `BytesMut` path calls this extraction yet; current
scalar adapter callers still require the sidecar to be populated. The increment
body only establishes guarded arithmetic and normal-return behavior. The
generic platform abort contract accounts for overflow error not returning from
`increment`. Compile-time assertions check the cell's size, alignment, and
drop behavior. The existing native layout test checks `Shared` field offsets;
it does not exercise the `Owned` prefix cast. That cast relies on the `repr(C)`
cell with its counter as the first field, and is not claimed as a separate
runtime test.

Contracts for the native adapters are generic trusted primitives with a narrow
single-thread interpretation. While the sidecar is populated, each adapter
assumes it opens the unique `AtomicInvariant` stored there, whose public
projection is bound to that exact `NativeCounter`, lends its current state only
to the one-shot callback, and restores the invariant before returning. The
opening does not expose a second state or permit callback reentry. The patched
FnGhost purity checker rejects native operations from checked callbacks; the
exact tool identity is bound by the receipt. The body-proved bootstrap helper
can consume this invariant once and transfer its permission and current scalar
state out of the cell, but the selected production call cone does not yet
perform that transfer or route operations through the transferred capsule.
Each verified operation is bound to the exact stored counter permission and
current scalar protocol state, and every access to that counter in the selected
proof cone must use these adapters on one thread. These contracts do not
describe concurrent loads, read coherence, synchronization handoff, or
acquire-imported resources. The crate's ordinary native implementation retains
its concurrent behavior, but these proofs make no concurrent Bytes claim.

The scalar refcount protocol does not contain owner tickets. The body-proved
initial promotion path connects one native count of one with one root ledger
ticket in its capsule. The body proofs do not yet connect later increments,
clones, splits, conversions, drops, or handle transfers to the `OwnerLedger`,
so they do not establish count-to-registration equality across those
transitions or derive count-one exclusivity for recovery. Terminal recovery,
Shared allocation retirement, destruction, and end-to-end `BytesMut::truncate`
across Shared, split, or nonzero-offset states remain unproved.

The selected crate targets also body-prove the generic
`ownership_proof::ref_count_ledger` module. Its affine authority map represents
registered owner IDs, and its tickets support root/promotion construction, a
child split, a drop, and the empty-map retirement permit; checked helpers prove
singleton and no-conflicting-ticket facts. This is a reusable ledger component
only. A forgotten handle may leave a registration outstanding, and the module
does not connect any production `Bytes` or `BytesMut` construction, clone,
split, drop, refcount operation, terminal read, or storage recovery to a ticket.
It therefore establishes no native refcount-to-registration equality,
count-one uniqueness, or Shared allocation lifecycle property.

The added `terminal_phase` bodies prove creation of a matching affine close
action, its one-shot transition to a retained empty authority, and the
contradiction between a closed authority and a still-valid matching action.
`shared_capsule::make_count_one_packet` is body-proved to compose supplied
header, exact counter permission, storage authority, one registered owner
ticket, whole physical region, and open terminal phase into a capsule whose
protocol binds those identities and value one. The private
`promote_to_shared_count_one` body now supplies these values on its proof path,
including the count-one counter and root ticket. It is the only promotion
helper whose body is verified in this checkpoint; no clone/split/drop update
path is connected yet.
The Vec `advance_unchecked` branch assumes a representable position, so it
does not establish reachability of the promotion branch, and public
`Buf::advance` is untranslated. Its packet-backed Shared branch proves cursor
movement under private packet predicates, but leaves later storage-authority
binding unproved. The proof adds no trusted native count-to-ticket law,
count-one exclusivity, terminal recovery, or allocation-lifecycle guarantee.
