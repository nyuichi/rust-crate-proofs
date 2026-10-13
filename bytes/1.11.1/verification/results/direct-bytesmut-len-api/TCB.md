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
`truncate`, `clear`, and unsafe `set_len` have body proofs from the common
`buffer_view_valid` precondition. It requires a stable layout and initialized
visible prefix, with either the Vec-backed unique layout or the packet-backed
Shared view. In the Shared case, `BytesMut`'s type invariant also carries the
`SharedOwnerPacket` invariant; this modular proof does not remove or bypass its
capsule, ticket, or ledger conditions. `truncate(n)` establishes length
`min(n, old_len)` and preserves the visible byte prefix, native pointer,
capacity, header word, exact proof sidecars and every allocation slot. `clear`
establishes an empty visible sequence with the same frame. Unsafe `set_len(n)`
requires `n <= capacity` and known initialized slots for any growth, then
changes only the length. A body-proved bidirectional observer lemma converts
the canonical prefix fact to the unique or Shared slot model. These body proofs
do not establish that every safe producer or lifecycle transition supplies
`buffer_view_valid`; shared-state reachability remains open. The implementation
body of `BufMut::advance_mut` still uses its unique-state precondition and
initialized-slot bounds for growth. Its separate generated trait-refinement
obligation is not proved because the trait prestate does not establish those
proof-only preconditions.
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
next-count function, and the `try_increment`/`increment` loop. Compile-time
assertions check the cell's size, alignment, and drop behavior. The existing
native layout test checks `Shared` field offsets; it does not exercise the
`Owned` prefix cast. That cast relies on the `repr(C)` cell with its counter as
the first field, and is not claimed as a separate runtime test.

The generic `CounterProtocol`, `EventState<T>`, and
`EventBoundInvariant<T>` abstraction keeps one exact `Perm<NativeCounter>` in
the invariant owner and a protocol payload outside it. Its body-proved
constructor, extraction, and callbacks preserve the exact counter model, public
mapping, protocol, and current scalar value. Shared payloads use this wrapper
to carry the allocation storage, all-reference ledger, physical pool, and
lifecycle phase under the same event state. The Shared payload protocol relates
the ledger's reference count to that exact counter permission value and binds
its allocation identity to the stored authority. The initial count-one
promotion and count-two Vec split construction have body proofs that establish
those relations and return the matching tickets and regions.

The token-aware `EventBoundInvariant::open_with_tokens` body is also proved.
Its callback precondition and postcondition explicitly expose the payload
protocol, equality between the payload counter model and owner permission, and
agreement between the payload and owner scalar value; the body delegates to the
generic `AtomicInvariant::open` contract and preserves those facts and the
owner permission frame. This proves the generic opener's body and callback
obligations. It does not mint namespace tokens or establish a native Shared
operation's access to them.

The Shared payload's header-read lease is a verification-only affine resource,
not a native reference count. Selected body proofs cover lease-authority
creation, activation and deactivation, checking an active fragment, the exact
payload take/restore transitions, their projection helpers, and the loan-bound
`loaned_native_counter` accessor. Packet `try_take_header_read_loan` and
`try_restore_header_read_loan` open the capsule with an explicit namespace
token and have body-proved callback contracts that preserve the public tuple,
counter model, protocol and owner-value agreement. A successful take moves the
exact boxed header permission into a loan whose affine fragment binds the
generation, header pointer, counter, storage authority and allocation binding,
Vec metadata, ledger and phase. Restoring requires that matching fragment and
returns that exact permission while restoring the idle lease state.

The packet precondition and namespace token establish permission to attempt an
open; they do not establish that the root is present. The try-take operation
therefore returns `Option`, and no proof in this checkpoint makes the `None`
case unreachable for an arbitrary valid packet. The selected public Shared
producer and lifecycle bodies do not yet scope these transitions around their
native counter operations. This lease layer consequently does not close safe
producer/lifecycle reachability for the common `BytesMut::truncate`
precondition. The exact selected bodies and prover results are listed in
[`proof.json`](proof.json).

The native atomic adapters that open this generic event invariant are trusted
primitives. Their contracts preserve the same permission and counter model,
and require a one-shot callback to preserve the protocol after the exact
native operation and ordering. FnGhost purity checks reject native operations
from the callbacks, and the adapter does not permit callback reentry or a
second state. These contracts have a sequential normal-return interpretation:
they do not describe concurrent read coherence, synchronization handoff, or
acquire-imported resources. The crate retains its ordinary concurrent native
behavior, but these proofs make no concurrent `Bytes` claim. The generic
platform abort contract accounts for the overflow error not returning from
`increment`.

The `OwnerLedger` is one affine inventory of all native strong references.
Each `OwnerTicket` owns the exact singleton fragment for one registration and
records the common allocation identity, its classification, and a half-open
interval. Mutable intervals are disjoint from every other registration;
Readonly intervals may overlap one another. Body proofs cover readonly
registration and same-ticket reclassification, root and two-Mutable-owner
registration, splitting a Mutable interval, removing a registration, and
preserving the authority identity when replacing a ticket binding. The
ledger's count is the map cardinality. Its generic operations alone do not
connect arbitrary native lifecycle operations to the registrations.

The selected `shared_context` bodies prove a proof-only affine owner and
borrowed view for an actual caller-supplied `Tokens` value and one
`ReadyRegistry`. Construction preserves the token namespaces and creates an
empty registry; reborrow bodies tie the view to the owner's existing fields
and frame the registry's final slot states. Register, take, and return move
the exact supplied `Fragment` through that registry. These bodies create no
tokens, fragments, header permission, or new trusted boundary. Token
membership for a particular capsule namespace remains a separate precondition
at each open. The context has not been connected to a `BytesMut` producer or
native lifecycle body, so it does not establish a ready Shared header,
producer reachability for `buffer_view_valid`, or a verified release path.

For Shared payloads, the physical pool contains exactly the in-bounds
complement of active Mutable intervals. Readonly intervals do not claim
physical mutation authority. The count-one packet constructor body-proves the
initial full-range Mutable ticket and empty pool. The count-two constructor
body-proves one event payload with two Mutable tickets and physical regions
partitioning the full allocation. The original-source private
`split_unique_to_shared` Vec producer consumes a Vec and installs those two
actual views and packets under its unique-Vec precondition. These proofs do not
show that the public `split_off`/`split_to` wrappers reach this helper, or that
ARC split, native clone, freeze, drop, and recovery transitions update the
ledger and pool. They also do not establish count-one exclusivity or
reclamation for every safe Shared state.

The added `terminal_phase` bodies prove creation of a matching affine close
action, its one-shot transition to a retained empty authority, and the
contradiction between a closed authority and a still-valid matching action.
The generic `PhysicalPool::retire` and `finish` operations are body-proved,
but the selected production paths do not yet invoke a complete last-reference
retirement and recovery sequence. The Vec `advance_unchecked` branch assumes a
representable position, so it does not establish reachability of the promotion
branch, and public `Buf::advance` is untranslated.

Although `truncate`, `clear`, and unsafe `set_len` have body proofs from
`buffer_view_valid`, their modular precondition still includes a packet-backed
Shared value's type invariant and the corresponding capsule, ticket, ledger,
and resource conditions. The new all-reference ledger operations do not yet
close the safe-produced-state reachability of that precondition across ARC
split, clone, drop, freeze, `to_mut`, and recovery. The proof therefore does
not yet establish the requested public `BytesMut::truncate` guarantee for all
safe states. Immutable-view lifetime and read permissions, concurrent
behavior, and unwind behavior also remain outside this scope.

The added B2 `VecStorage::resume_owned` adapter and strengthened generic
`STD-PTRWRAP-01` assumption are described above. Generic Creusot Std ghost and
resource contracts also participate in the selected proofs; the Vec observer
and resource assumptions are documented in the [Std overlay](../../std-support/README.md).
Verification-only exclusions for unrelated comparison, ownership, iterator,
and Shared-backed bodies are documented beside the source in
[`bytes.rs`](../../../src/bytes.rs) and [`bytes_mut.rs`](../../../src/bytes_mut.rs).
