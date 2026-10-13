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
storage authority, region, base descriptor and ghost allocation identity
(namespace, base pointer and capacity) computed from the same captured Vec.
The private count-one promotion helper passes that identity into the owner
ledger and packet constructor. It grants no access authority and does not prove
later Shared lifecycle operations. The proof uses an opaque transparent native
Vec wrapper and an outer C-layout storage struct with a zero-sized ghost field;
compile-time assertions compare its
size and alignment with `Vec<u8>`. The native `VecRawParts::take` branch keeps
the original direct extraction implementation. No trusted Bytes-specific
ownership, refcount, destructor, or reclamation law is used here.
Removing the capture assumption requires deriving the exact native metadata
and consumed-owner transfer from verified Vec, allocator, and `ManuallyDrop`
contracts; proving only `bind_consumed_vec` would leave that assumption in
place.

The source now also defines `VecStorage::resume_owned` as a narrow generic B2
restoration boundary used by the selected private unique-to-Shared producer
proof. It consumes the matching existing `RawAllocation`, `Recovery`, and
full-capacity `PhysicalRegion`, requires every slot in the restored Vec prefix
to be known,
and calls `Vec::from_raw_parts` exactly once before immediately wrapping the
Vec in the opaque `NativeStoredVec(ManuallyDrop<Vec<u8>>)` field. Its contract
preserves the same native pointer word, allocation base and capacity, every
full-capacity slot (including known spare slots), the exact recovery resource
and the exact full-region resource. It returns the corresponding
`StoredAuthority`, region, and offset-zero `BoundPtr`; it returns no usable Vec
and mints no B1 namespace, recovery, or region. The caller must have no
outstanding physical borrow into the allocation. The boundary assumes the
matching B1 tokens still represent the live original global-allocator `u8`
allocation and that native `Vec::from_raw_parts` plus the opaque wrapper
transfer its sole owner without changing allocation bytes. It states no Bytes
owner, refcount, split, or reclamation rule. The selected body proof for the
private original-source Vec split producer calls this adapter and proves its
own use of the returned exact resources; the adapter itself remains trusted,
and the proof does not establish public wrapper reachability. Removing this
assumption requires a generic Std/B2 contract for exact native owner
reconstruction and full-allocation preservation, followed by a body proof of
the opaque storage adoption and metadata binding.

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
The packet invariant ties each ticket's ghost allocation identity (namespace,
base pointer and capacity) to its physical region; the capsule protocol ties
that same identity to its current protected storage authority. This tuple is
identity metadata and grants no allocation access. The selected proof does not
establish every production path that creates or preserves the relation, or
Shared reads, recovery, exclusivity or lifecycle behavior. No selected proof
establishes a safe public call chain that produces a nonzero offset: public
`Buf::advance` is untranslated. The trait-refinement
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
the bounded pointer relationship used by the cursor helpers. Generic
`STD-PTRWRAP-01` is a strengthened external specification for raw `u8`
`wrapping_add`: it retains the modular-address guarantee and, when the source
address plus offset does not wrap, assumes that the exact result pointer equals
the Std logical `offset_logic` result for that same source pointer and offset.
This pointer-word identity is a generic standard-pointer assumption used by
`BoundPtr::advance_within` and the original Vec split helper; it is not inferred
from equal numeric addresses. The assumption grants no `Perm`, `PtrLive`, B1/B2
resource, allocation liveness, or dereference authority. The selected bodies
also use generic Creusot Std ghost/resource and logic contracts. Their Vec
observer and resource assumptions are documented in the
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
transitions or derive count-one exclusivity for recovery. Terminal recovery in
a production path, Shared allocation retirement and destruction, and end-to-end
`BytesMut::truncate` across Shared, split, or nonzero-offset states remain
unproved.

The selected crate targets body-prove the generic
`ownership_proof::ref_count_ledger` module. Each `OwnerTicket` contains the
actual affine `Fragment` for one singleton owner ID. The body-proved
`OwnerTicket::from_fragment` constructor consumes that fragment once and
establishes its ledger resource ID, owner ID, exact singleton map and stored
binding. `replace_binding` preserves the ticket identity and explicitly frames
the authority resource ID while updating the matching map entry; the
`OwnerBindingUpdate` trait-refinement obligation is also selected and proved.

An `OwnerBinding` contains the common allocation identity tuple (namespace,
base pointer and capacity) and a half-open range `[lo, hi)`. The ledger
invariant requires each active range to lie within the allocation and requires
active ranges to be pairwise disjoint. Body proofs cover a root ticket over
`[0, capacity)`, a promoted pair split at a supplied cut, `split_owner` splitting
one live parent interval into adjacent parent and child intervals, and
`drop_owner` removing one registration. `sole_active_id` characterizes the
range represented by the only active ticket. These interval and allocation
fields are metadata: they grant no allocation access authority.

The root-ticket constructor is connected to the selected production
count-one promotion helper. The two-ticket constructor is connected to the
selected private original-source Vec split helper, which consumes the actual
Vec state and installs the two returned packets. The generic `split_owner` and
drop transitions remain unconnected to native handle operations; no selected
production clone, drop, conversion, or later refcount transition updates the
ledger. Consequently the proofs do not establish that later native refcount
changes remain matched to ledger changes, count-one uniqueness for recovery,
or a Shared allocation lifecycle.

The added `terminal_phase` bodies prove creation of a matching affine close
action, its one-shot transition to a retained empty authority, and the
contradiction between a closed authority and a still-valid matching action.
`shared_capsule::make_count_one_packet` is body-proved to compose the supplied
header, exact counter permission, storage authority, root ticket, whole
physical region and open terminal phase into a count-one capsule. The packet
invariant binds the ticket's allocation identity and `[lo, hi)` range to the
region namespace, base, capacity and bounds. `SharedState` states that its
physical pool contains exactly the in-bounds complement of active owner
ranges. The selected constructor proves the initial case: the ticket owns the
whole `[0, capacity)` interval and the pool is empty, using the body-proved
`PhysicalPool::empty_from` constructor and its unit-normalization helper. The
generic `PhysicalPool::retire` and `finish` operations are also body-proved.
No selected production path invokes them to retire regions into the pool or
recover a full region from it. The private `promote_to_shared_count_one` body
supplies these initial values, including the count-one counter and root ticket.
This is the only selected production owner-ledger composition; no
clone/split/drop update path is connected yet.
The Vec `advance_unchecked` branch assumes a representable position, so it
does not establish reachability of the promotion branch, and public
`Buf::advance` is untranslated. Its packet-backed Shared branch proves cursor
movement under private packet predicates, whose allocation identity is bound
to current protected storage as described above. The proof adds no trusted
native count-to-ticket law,
count-one exclusivity, terminal recovery, or allocation-lifecycle guarantee.

The new `make_count_two_packets` target body-proves a generic count-two
capsule constructor. Its precondition requires the exact supplied native
counter permission and stored allocation authority, two distinct tickets
bound to the same ledger and allocation with adjacent ranges, and an
active-owner map covering the full allocation. The body puts that one ledger,
storage authority, counter permission, terminal phase, and an empty pool into
one `SharedState`, then returns packets carrying the actual two ticket and
physical-region fragments cut from the full region; both packets share the
same capsule. The generic `new_promoted_pair` constructor proves the matching
full-range active-owner coverage for its particular `[0, cut)` and
`[cut, capacity)` pair. These proofs establish constructor behavior under
their stated preconditions. The body-proved `split_unique_to_shared` helper
calls `make_count_two_packets`, then installs the actual left and right
pointer/length/capacity views with those packets while preserving their common
capsule and full allocation-slot frame. Its proof depends on the trusted B2
`VecStorage::resume_owned` adapter and the generic non-wrapping pointer-offset
assumption above. It establishes this private Vec producer body under its
unique-state preconditions; it does not prove the public `split_off` or
`split_to` wrappers, safe call-chain reachability, the ARC split producer, or
subsequent ARC split, clone/drop/refcount lifecycle transitions.
