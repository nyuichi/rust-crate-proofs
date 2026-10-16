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

The Std overlay additionally assumes that the opaque `capacity_model` for a
Vec with one-byte elements is at most `isize::MAX`; it makes no such claim for
zero-sized elements. The original-source full-capacity Vec constructor uses
this bound to meet the narrow B1-BOX-VEC boundary's allocation-size
precondition.

The full-capacity Vec-to-Bytes route connects the actual `Vec::into_boxed_slice`
and `Box::into_raw` operations through `raw_vec::detach_full_capacity_vec`.
This B1-BOX-VEC boundary assumes the generic Vec-to-Box transfer preserves the
same allocation and that the consumed Box yields the sealed recovery and full
physical region for that allocation, including its initialized byte slots.
The Box-only B1-BOX boundary also consumes an actual `Box<[u8]>` through the
same source `Box::into_raw` helper and provides the sealed matching resources;
there is no pure Box-content-to-pointer observer. The source body then proves
the internal boxed sidecar and pointer-cell permission from the actual returned
resources. Tagging an even base and untagging its odd tagged cell value use the
generic one-byte `wrapping_add` and `wrapping_offset` contracts. Their exact
pointer-offset relations preserve the input pointer provenance for nonwrapping
offsets, while granting no allocation liveness or dereference permission. The
old integer-to-pointer adapter has been removed. The constructor stores the
vtable selected from its supplied even/odd inputs, but does not prove either
input has the identity of a particular static vtable. Public `From<Vec>` and
`From<Box>` postconditions and the ordinary vtable-driven Drop/callback path
remain unproved.

The original `From<Vec<u8>>` spare-capacity branch now calls the source helper
`from_spare_capacity_vec`. Its body consumes the actual Vec through B1
`raw_vec::detach_vec`, builds the original `bytes::Shared { buf, cap, ref_cnt }`
header, and constructs the same `Bytes` pointer-cell and supplied vtable. The
private carrier preserves the matching recovery and full physical region,
initialized contents and Unknown spare slots, the actual aligned header Box
permission, the exact counter permission with value one, and the actual data
cell permission. This proves only the internal initial carrier for this
helper. It does not connect that distinct `bytes::Shared` header to the
`bytes_mut::Shared` Context/ledger protocol, prove the static `SHARED_VTABLE`
identity, or prove public `From<Vec>` postconditions, clone/refcount updates,
release, or automatic Drop. No VecStorage model or Bytes-specific ownership
contract is used. B1 Vec capture, Box alignment conversion, and the native
counter and pointer-cell constructors remain the generic physical/permission
boundaries; they can be replaced by exact generic Std and allocator contracts
without changing the Bytes-specific proof obligations.

For the same full-view, count-one spare-capacity carrier, the selected original
`shared_to_mut_impl` and private `Bytes::into_mut_spare_shared` bodies prove the
unique recovery branch: one Acquire load, header deallocation without running
`Shared::drop`, exact B2 Vec reconstruction, `BytesMut::from_vec`, and the
original zero-offset advance. The result preserves the input pointer, visible
bytes, length, full allocation capacity and Unknown spare slots. The generic
trusted `offset_from_equal_u8` adapter is used only where the actual native
operands are equal and non-null, yielding offset zero; it does not specify a
general pointer distance, provenance, liveness, or dereference fact. The
replacement path is a matching generic Std contract for `offset_from` on the
same allocation, then a body proof of the adapter. This does not prove the
public `Bytes::try_into_mut` vtable dispatch, the nonunique branch, public
`From<Vec>` postconditions, or automatic Drop.
The original nonunique copy/release fallback remains in the native
`shared_to_mut_nonunique` helper. In the checked count-one entry, its call has a
false precondition derived from the actual count-one carrier and Acquire
result; the helper body and that fallback remain unproved.

For the separate `bytes_mut::Shared` count-one promotion path, the generic
`NativeStoredVec::metadata` boundary observes only the actual stored Vec's
pointer, length, and capacity and preserves the Vec owner. The generic
`NativeStoredVec::offset_from_bound` boundary executes the original native
`offset_from` while that actual Vec and a same-allocation physical-region
borrow keep the allocation live; B1 descriptors bind the exact base and cursor
operands. The standard pointer contract permits equal-address pointers or
pointers within one allocated object. These facts expose pointer distance only,
not byte access or a Bytes ownership rule. Body-checked `VecStorage` wrappers
relate the metadata to its matching stored-allocation authority. This is a
generic Std boundary, not a trusted Bytes-specific promotion rule.

The selected private `Bytes::as_slice` body has boxed and spare-capacity
branches under `original_borrowable_valid`. Each passes the carrier's matching
base and actual physical region to the existing B4 shared-read boundary, and
proves the returned bytes equal that carrier's saved visible contents. This
does not prove a public `From` postcondition, `Buf::chunk`, `AsRef`, `Deref`,
the shared-readonly `Bytes` state, or destruction.

The selected private `Bytes::dispose_spare_shared` body consumes the actual
count-one spare-capacity carrier. It reads the shared pointer from its actual
pointer-cell permission, forgets the handle, and calls the original
`release_shared` source body with the exact header Box permission, counter
permission, recovery token, and full physical region. The selected
`release_shared` body proves the one-owner Release decrement, Acquire load,
and call to the original `free_shared` body; `free_shared` reads the header
through its Box permission, deallocates the exact B1 byte allocation, then
deallocates the header without running `Shared::drop`. The only trusted
boundaries here are generic: owned-permission Release/Acquire atomic
operations, the existing B3 full-allocation deallocation while consuming B1
resources, and sized Box deallocation without its pointee destructor. Their
replacement path is a matching generic Std contract for the atomic permission
operations and Box allocator layout, plus the existing allocator contract.
This establishes only the explicit
unpromoted count-one spare-capacity entry. Public `From<Vec>` postconditions,
cloning or other refcount transitions, vtable callbacks, and automatic Drop
remain unproved.

The original `free_boxed_slice` source body is also translated and checked for
the explicit boxed release entry. It computes capacity from that function's
actual `(buf, offset, len)` operands. The proof-only release payload is required
to tie `buf` exactly to the offset-zero B1 base, `offset` exactly to a bounded
view in the same namespace and capacity, and `view_offset + len` to the full
allocation capacity. Its actual addresses must be ordered and nonoverflowing;
the payload also carries the matching complete Recovery and physical-region
resources. From these facts the same capacity calculation equals the B1
capacity, which is then consumed through the existing B3 deallocator. Native
cleanup and B3 use the same `allocation_ops::deallocate_u8` primitive.

`Bytes::dispose_boxed` is an unsafe original-source entry for an unpromoted,
full-view boxed-slice handle. It reads the actual pointer cell with its carried
permission, exactly untags the even-base representation when needed, forgets
the handle, and calls the original `free_boxed_slice` body with the actual B1
release payload. Its source body is present in native builds as well. This
entry does not establish the auto-Drop path, dynamic vtable dispatch, or
release rights for cloned, sliced, converted, promoted, or other `Bytes` states.

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
own use of the returned exact resources; the adapter itself remains trusted.
The public ARC-backed split route operates on an existing Shared header and
does not call this Vec reconstruction boundary. Removing this assumption
requires a generic Std/B2 contract for exact native owner
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

The private `BytesMut::as_slice` body is checked from `buffer_view_valid` and
returns a slice whose bytes equal `buffer_bytes()`. Its unique and Shared
branches pass the actual cursor and corresponding actual `PhysicalRegion` to
the existing generic B4-read `borrow_bound` boundary, whose native body uses
`slice::from_raw_parts`; this proof adds no access authority and does not prove
the public `Buf::chunk`, `AsRef`, or `Deref` paths.

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

Generic `STD-PTRWRAP-02` supplies the missing signed-offset contract for
`*mut T::wrapping_offset` through `signed_wrapping_offset`. It is restricted
to one-byte pointees and requires the mathematical source address plus signed
offset to stay in the `usize` address range; it returns the same-pointer
`offset_logic` result and grants no `PtrLive`, `Perm`, allocation, or
dereference authority. The selected original-source `BytesMut::rebuild_vec`
body uses this operation to subtract the hidden prefix, restores the original
length and capacity, and then consumes the actual `OriginalUniqueProof`
recovery and full physical region through the existing generic B2
`resume_vec` boundary. Its selected proof precondition requires that actual
capability and the complete initialized prefix. The corresponding native
source operation retains the same signed pointer calculation, length/capacity
updates, and `Vec::from_raw_parts` reconstruction. Callers that supply no
proof capability, including the public Vec-backed `freeze` route, are not
covered by this selected body proof. This contract describes pointer
arithmetic only and does not assert that a pointer is live or that a caller
owns the allocation; the B1/B2 resources provide those separate facts.
This local external specification can be replaced by a matching signed-offset
contract in the pinned Creusot Std overlay; `signed_wrapping_offset` itself is
body-checked.

The selected crate-private `BytesMut::dispose_unique` body requires the actual
rebuildable unique proof state. It uses `rebuild_vec` to consume that proof and
the existing B2 boundary to restore a Vec, then passes the actual Vec through
the body-proved `VecStorage::hold_owned` composition and the existing B3
`retire_stored_vec` boundary. The native body runs the same signed-offset
reconstruction and drops the Vec once. This does not prove callers that pass no
unique proof, public freeze, or automatic `BytesMut::drop`; no Bytes-specific
allocation or destruction trust was added.

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

The selected private count-one promotion and original-Vec split paths pass the
caller-supplied context into packet construction. The packet makers register
the actual idle lease fragment in its `ReadyRegistry`, and packet take/restore
move that same capability through the context. Body proofs cover the
valid-index, normal-return Vec-backed and packet-backed ARC branches of public
`BytesMut::split_off`, `split_to`, and `split`. The Vec route passes its
rebuildable allocation and caller context to the count-two producer. The ARC
route borrows the existing matching idle lease capability, runs the source
relaxed increment once, divides the actual Mutable ticket and physical region,
and restores that capability to the same registry entry. Both routes export
strict Shared layout and initialized visible bytes, pointer/length/capacity and
byte results, distinct owner tickets for the same allocation and lease key,
and the resulting idle Context entry for both handles. These are
normal-return proofs; they do not establish unwind or concurrent behavior.
The selected operation sequences explicitly dispose each related Shared
handle with its caller-supplied context and make no Context-consistency claim
after an untracked automatic `Drop`.
The lease RA has one exclusive idle/active value, and the active stamp stores
the complete `ReadyKey`; idle and active authority therefore cannot compose as
separate product components or lose the slot identity.

The packet borrows the actual caller context and reborrows its actual `Tokens`
for `open_with_tokens`. Packet validity and namespace membership alone permit
an attempt but do not establish that the root is present, so the operation
returns `Option` and may return `None` when the matching idle cap is absent.
Its body-proved result equivalence establishes that a matching registered idle
cap yields `Some`: the packet's actual live `OwnerTicket` and the matching
lease RA state rule out the terminal root-absent protocol and produce the
matching loan. The public split wrappers connect both their Vec and ARC source
branches to packet construction or actual-packet splitting, and return the
same idle capability entry for both resulting handles. This does not cover all
public producer/lifecycle paths. The selected proofs therefore do not close
public reachability for the common `BytesMut::truncate` precondition. The exact
selected bodies and prover results are listed in
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

The body-checked `NativeCounter::load_acquire_with_frame` adapter delegates to
the same one-shot Acquire operation, passing its actual generic Ghost frame to
the callback and binding the callback output to the returned Ghost value. It
adds no trusted boundary or Bytes-specific atomic law.

The proof-only `NativePtr<T>` wrapper has two narrow generic trusted
operations. `new_with_permission` executes one native `AtomicPtr::new(value)`
and returns a permission for that exact cell with modeled pointer value
`value`. `exclusive_read` requires the borrowed cell to be the cell named by
that permission, executes native `get_mut`, returns the permission's pointer
value, and preserves the cell and permission. The checked
`into_raw_with_permission` move retains the identity of the whole atomic cell.
This permission describes only the pointer cell and its stored pointer value;
it grants no authority over the pointed-to header or allocation. The selected
Readonly disposal body uses it only to read its own unchanged cell; the
separate actual packet and root permission remain the source of header access.
These local adapters can be replaced once the pinned Std overlay exposes
reviewed contracts for the actual core `AtomicPtr` constructor and exclusive
`get_mut` with the same cell/value relation. No Creusot change or Bytes-specific
law is needed.

The selected in-capacity `BytesMut::extend_from_slice` body calls the existing
generic `storage_ops::copy_to_uninit_prefix_raw` effect boundary. For typed,
disjoint `&[u8]` source and `&mut [MaybeUninit<u8>]` destination slices with
source length no greater than destination length, its native
`copy_nonoverlapping` call initializes exactly the source-length prefix and
preserves the destination length and suffix. This effect grants no allocation,
liveness, Vec-owner, refcount, or Bytes-specific authority. The verified
`copy_to_uninit_prefix` loop has the same functional specification, but the
selected source body continues to call the original raw-copy helper.

The `OwnerLedger` is one affine inventory of all native strong references.
Each `OwnerTicket` owns the exact singleton fragment for one registration and
records the common allocation identity, its classification, and a half-open
interval. Mutable intervals are disjoint from every other registration;
Readonly intervals may overlap one another. Body proofs cover readonly
registration and same-ticket reclassification, root and two-Mutable-owner
registration, splitting a Mutable interval, removing a registration, and
preserving the authority identity when replacing a ticket binding. The
body of `drop_owner` also proves the exact change to Mutable physical coverage:
removing a Mutable ticket removes only its interval, and removing a Readonly
ticket leaves Mutable coverage unchanged. The ledger's count is the map
cardinality. Its generic operations alone do not
connect arbitrary native lifecycle operations to the registrations.

The checked `freeze_into_readonly` path consumes an actual Mutable packet
ticket and its actual physical region, returns a Readonly ticket and returns
that region to the pool. It preserves the reference count, storage,
lease/release authority, caller Tokens, and Context entries. The selected
private `freeze_shared_view` body connects this transfer to a `Bytes` sidecar,
preserving the actual packet, pointer-cell permission, view and supplied
vtable value. The public `BytesMut::freeze` body, its native static vtable
identity, and public readonly byte-read APIs remain unproved.

With no active physical lend, the live `SharedPayload` protocol states that
every active Readonly ticket's visible interval is contained in the physical
pool and each slot in that interval is known. During an active lend, the exact
lent interval is absent from the pool and is carried by the actual affine
region in the loan; known slots outside it remain in the pool.
`freeze_into_readonly` derives the absolute span from the actual region's
visible-slot facts, and the body-checked mutable-release, mutable-split, and
readonly-release transitions preserve the protocol. The
`prove_readonly_ticket_pool_known` observer requires no active lend, checks the
actual ticket against its live ledger entry, and uses the actual pool resource
for slot facts. This invariant establishes no physical borrow or slice
lifetime. The lease stamp carries the exact optional physical-lend span
marker, and the checked private take/restore transitions pair that marker with
the actual `PhysicalRegion` removed from and returned to the pool.

The checked private `Bytes::copy_shared_readonly_to_vec_with_context` entry
passes its actual Readonly packet and caller Context through the nonunique
shared copy source body. The body borrows the packet view through the loan's
actual B4 region, performs the same `slice::to_vec` copy operation used by the
native helper, ties the returned bytes to the region's known slots with a
value-only witness, restores the affine region, and reaches the original
shared release body. The witness snapshots the old count returned by that
body, and the postcondition preserves the exact ready-slot result: a retained
key and idle cap when the old count exceeds one, or the tombstone when it is
one. This snapshot reports the result but supplies no authority. This proves
the internal nonunique copy and release
path under its actual packet/context preconditions. It does not prove the
outer no-translate vtable callback, native static vtable identity, public
vtable dispatch, or the callback's Acquire/unique branch. The boxed and
spare-capacity branches of `Bytes::as_slice` are proved; its shared-readonly
branch and other public readonly byte-read APIs remain outside the proof.

The private `Bytes::copy_shared_readonly_to_mut_with_context` entry reads the
actual pointer-cell permission and passes its packet and caller Context to the
same all-build `shared_v_to_mut_nonunique` helper called by the original
`shared_v_to_mut` callback's nonunique continuation. That helper calls the
checked copy-and-release body, then calls the original `BytesMut::from_vec`
once. Its body proof preserves the read witness, the exact conditional
post-release Context state, and the copied Vec snapshot's contents, pointer,
capacity, initialized prefix and Unknown spare slots. This proves the shared
copy-to-Mutable helper and its Context-aware entry, but it does not prove that
the outer no-translate callback selects this continuation or establish the
public vtable dispatch or static vtable identity. The native regression
`nonunique_frozen_view_converts_to_mutable_copy_without_changing_sibling`
exercises `split_off`, `freeze`, and `BytesMut::from` while the Mutable sibling
remains alive, then checks the copied view and sibling after truncation.

The selected composition body also starts from a rebuildable unique Vec,
splits it, calls the private freeze helper with an explicit vtable input, and
uses the actual sibling ticket while copying the Readonly side into a new
Mutable Vec. It truncates and explicitly disposes that Vec, then explicitly
disposes the sibling with the same Context. The body proves the actual Context
count transition from two owners, to one idle owner, to the final tombstone.
This sequence does not prove the public freeze dispatch or static vtable
identity, and it bypasses the automatic Drop wrapper.

A second selected operation sequence exercises the unique branch of the
original `shared_v_to_mut` source helper. Starting from a unique Vec-backed
Mutable view, it splits the allocation, explicitly disposes one handle with
the actual Context, and freezes the surviving shared view. The remaining
ticket is then the sole live owner at count one. The original one-Acquire
promotion helper moves that actual Readonly ticket to Mutable over
`[cursor, allocation_capacity)`, lending the actual physical suffix including
Unknown spare slots. The resulting view keeps the allocation pointer and
capacity; the scenario truncates and explicitly disposes it, reaching the
final Context tombstone. A native regression checks the analogous public path
at a nonzero offset, preserving the same pointer, spare capacity, and visible
bytes. This proves the selected private helper and
context-aware entry under their actual packet and Context preconditions. It
does not prove the public `BytesMut::from(Bytes)` callback dispatch, static
vtable identity, or automatic Drop.

The checked common release carrier accepts a Mutable packet with its actual
physical region or a Readonly packet with its actual ticket and no region. Its
Readonly body consumes the ticket, leaves the pool unchanged, and reaches the
same final-owner retirement path. The selected `Bytes::dispose_shared` body
consumes the actual Readonly sidecar and cell permission, reads the pointer
from that same cell, forgets the handle to prevent a second release, and calls
the common release body with its actual packet and Context. This does not
prove public freeze reachability, native static vtable identity, readonly
byte access, or ordinary automatic Drop.

The selected `shared_context` bodies prove a proof-only affine owner and
borrowed view for an actual caller-supplied `Tokens` value and one
`ReadyRegistry`. Construction preserves the token namespaces and creates an
empty registry; reborrow bodies tie the view to the owner's existing fields
and frame the registry's final slot states. Register, take, and return move
the exact supplied `Fragment` through that registry. These bodies create no
tokens, fragments, header permission, or new trusted boundary. Token
membership for a particular capsule namespace remains a separate precondition
at each open. The selected private count-one promotion and
`split_unique_to_shared` paths use this context to register their packet's idle
header capability; packet loan take/restore use it to move the same capability.
The crate-private unsafe `BytesMut::dispose_shared` and `Bytes::dispose_shared`
bodies consume their shared handles with this actual context and call the same
native release body as the ordinary release wrapper. The proof preserves the
other registry slots; the common release path recovers the terminal storage,
full physical region and actual root permission and consumes them in final
native destruction when this is the last owner. It does not establish that public producer
wrappers pass a valid context or that every safely reachable state satisfies
`buffer_view_valid`. The ordinary automatic `Drop` path remains present; its
context-free wrapper is untranslated, so its post-Drop context state is not
verified.

For Shared payloads, the physical pool contains exactly the in-bounds
complement of active Mutable intervals. Readonly intervals do not claim
physical mutation authority. The count-one packet constructor body-proves the
initial full-range Mutable ticket and empty pool. The count-two constructor
body-proves one event payload with two Mutable tickets and physical regions
partitioning the full allocation. The original-source private
`split_unique_to_shared` Vec producer consumes a Vec and installs those two
actual views and packets under its unique-Vec precondition. Body proofs cover
the valid-index, normal-return Vec-backed and packet-backed ARC branches of
public `split_off`, `split_to`, and `split`; they preserve strict Shared
layout, byte and view geometry, and the actual context registration. These
split proofs do not prove native clone, public freeze, public readonly byte reads,
automatic drop, or every recovery transition. They do not establish count-one
exclusivity or reclamation for every safely produced Shared state. The separate
selected count-one sequence above proves one concrete unique-owner promotion
and final disposal, not a general count-one guarantee.

The added `terminal_phase` bodies prove creation of a matching affine close
action, its one-shot transition to a retained empty authority, and the
contradiction between a closed authority and a still-valid matching action.
The generic `PhysicalPool::lend_interval`, `retire`, and `finish` operations
are body-proved. `lend_interval` requires every slot in its requested interval
to be present in the pool, partitions the actual affine resource, and preserves
all slot values including Unknowns. It grants no byte-read authority by itself
and states no Bytes ownership law. The selected `release_native_owner` and
terminal callback connect the actual ticket,
lease, terminal action, ledger removal, pool finish, and root permission for a
final owner; `release_shared_common` then runs the original native decrement,
Acquire load, stored-Vec retirement and Box release sequence. The explicit
`dispose_shared` bodies reach this common native algorithm with their actual
Mutable or Readonly packet and context. For Readonly disposal, the selected
`NativePtr` adapters expose the handle's actual atomic cell value without
converting cell permission into header authority; the packet's root permission
remains the source of header access. The proof covers these internal explicit
paths under their unsafe preconditions. It does not cover every public
clone/freeze/recovery transition or the context-free automatic `Drop` call
chain. The Vec `advance_unchecked`
branch assumes a representable position, so it does not establish reachability
of the promotion branch, and public `Buf::advance` is untranslated.

Although `truncate`, `clear`, and unsafe `set_len` have body proofs from
`buffer_view_valid`, their modular precondition still includes a packet-backed
Shared value's type invariant and the corresponding capsule, ticket, ledger,
and resource conditions. The new all-reference ledger operations and the
selected split/recovery sequences do not close the safe-produced-state
reachability of that precondition for all paths across clone, drop, freeze,
`to_mut`, and recovery. The proof therefore does not establish the requested
public `BytesMut::truncate` guarantee for all safe states. Immutable-view
lifetime and read permissions, concurrent
behavior, and unwind behavior also remain outside this scope.

The added B2 `VecStorage::resume_owned` adapter and strengthened generic
`STD-PTRWRAP-01` assumption are described above. Generic Creusot Std ghost and
resource contracts also participate in the selected proofs; the Vec observer
and resource assumptions are documented in the [Std overlay](../../std-support/README.md).
Verification-only exclusions for unrelated comparison, ownership, iterator,
and Shared-backed bodies are documented beside the source in
[`bytes.rs`](../../../src/bytes.rs) and [`bytes_mut.rs`](../../../src/bytes_mut.rs).

`vec_storage::retire_stored_vec` is a generic stored-Vec destructor boundary.
Its proof-mode preconditions require the actual `StoredAuthority` matching the
consumed storage and the actual matching, full-capacity `PhysicalRegion`; the
caller must have no outstanding physical borrow. Its native body is exactly
`drop(storage.take_for_retirement())`, consuming those proof capabilities and
running the ordinary `Vec<u8>` destructor. This is interpreted for the
matching original global-allocator `u8` allocation and its native pointer,
capacity and layout: a nonzero-capacity allocation is deallocated exactly once;
zero capacity denotes no allocation. The trusted contract states no Bytes
refcount, final-winner, phase, or pool-transition rule; those are proved in the
source call chain.

Two additional narrow generic facts support the empty replacement Vec used by
`mem::take`: `new_empty_vec_u8` returns the native `Vec::<u8>::new()` with
length and modeled capacity zero, and `drop_empty_native_vec` invokes the
ordinary native `ManuallyDrop::drop` for a Vec whose native length and capacity
are both zero, with no allocation to free. They establish no Bytes-specific
owner, refcount, phase, or reclamation rule. `VecStorage::default` and its
empty-owner destructor wrapper remain body-proved around those facts.

The `Shared.vec` field is `MaybeUninit<VecStorage>`, which preserves the native
field size, alignment and offset checked by the layout test. All three Shared
constructors initialize it. The checked accessors require the actual field
view to be initialized and frame the Vec view, original-capacity metadata and
refcount. In the verified terminal release body, `take_vec_storage` installs
the same empty default as `mem::take`, `retire_stored_vec` consumes the original
storage and its actual authority/region, and `drop_empty_vec_field` explicitly
runs the body-proved empty-owner destructor before setting the field view to
`None`. The actual root permission then consumes the Box; the field is not
dropped a second time. The original native atomic/decrement/Acquire and
allocation-retirement order is shared by `release_shared_common` and the
explicit consuming path. The ordinary `BytesMut` automatic `Drop` remains in
the source, but its context-free wrapper is untranslated; no post-Drop context
consistency or all-safe-state `truncate` guarantee is claimed.

Replacing the stored-Vec boundary requires a verified generic transfer of the
unique native stored Vec owner into `Vec<u8>` together with the standard
`Vec` destructor contract, and a body proof that consumes the same authority
and region without changing the allocation model. Replacing either empty-Vec
fact requires verified `Vec<u8>` construction/destruction contracts preserving
the zero-capacity no-allocation case. The selected body proofs leave all
Bytes-specific release rules in the source.
