# Direct-source `BytesMut` proof evidence

On this verification fork based on crates.io `bytes` 1.11.1, selected
constructor bodies establish that `with_capacity(n)` returns an empty buffer
with capacity at least `n`, and `zeroed(n)` returns `n` zero bytes with
capacity at least `n`. The selected crate-private `from_vec(v)` conversion
preserves `v`'s length and bytes; `new` and `Default` establish an empty
buffer. These results apply to this fork; its native source refactors are not
shown equivalent to the unmodified release.

`proof.json` records the fresh proof run from the crate's live source. The
`coma/` and `results/` trees retain the exact selected COMA bodies and
successful prover results, together with the feature tree and translation log.
All selected targets translate from this crate's original source; no separate
Std-source proof targets are included. [`TCB.md`](TCB.md) describes the selected
guarantees and their assumptions.

For the generic Vec-storage path, `VecRawParts::take` and
`VecStorage::hold_owned` now have body proofs composing the B1 capture with
the body-proved recovery-token and physical-region construction. The
`VecStorage::capture_vec_owner` primitive that binds the consumed native Vec's
pointer, length, capacity, and contents to a sealed witness is trusted; it
does not create those affine resources or establish any `Bytes` owner,
refcount, or retirement rule. The private promotion helper calls `hold_owned`
to build its initial Shared storage state. `hold_owned` also returns a ghost
allocation identity tuple (namespace, base pointer, capacity) computed from
that captured Vec, alongside the storage authority, physical region and base
descriptor. The tuple identifies the allocation; it grants no access
authority. Later Shared lifecycle operations remain outside this proof.

The native Shared reserve-growth path now compacts its initialized visible
bytes to the Vec base before calling `Vec::reserve`, so a hidden prefix left
uninitialized by another disjoint handle is not included in the Vec length.
The existing regression test covers that sequence. Reserve remains
`no_translate`; this is a tested native behavior change, not a proof of Shared
ownership or lifecycle.

The selected source targets also include body proofs for cursor-relative
prefix helpers, Vec metadata packing and position updates, `BoundPtr`
projections, `set_visible_len`, `advance_vec_layout`, `advance_shared_layout`,
`SharedOwnerPacket::advance_cursor`, `advance_unchecked`, and
`BufMut::advance_mut`. The selected constructors establish an offset-zero
`unique_vec_rebuildable` Vec state, including the known initialized prefix
needed to reconstruct the original Vec. The Vec `advance_unchecked` branch is
proved under its rebuildable-state, `count <= len`, capacity, and
representable-position preconditions; it preserves the allocation-slot frame.
The unique layout relation also records that the exact cursor pointer is the
allocation base offset by the hidden prefix, and the selected Vec advance body
preserves that relation while reindexing the visible suffix. This is pointer
metadata only. A selected body proof covers the original-source
`BytesMut::rebuild_vec` path called by `promote_to_shared_count_one`: it applies
the signed offset to the actual cursor pointer, restores the hidden prefix in
the Vec length and capacity, then passes the actual B1 recovery and full
physical region to the generic B2 restoration. The selected precondition
requires that actual recovery capability; other callers that pass no proof,
including the public Vec-backed `freeze` route, are not covered by this body
proof. The generic signed pointer contract is limited to a one-byte pointee and
a non-wrapping numerical address; it grants no allocation or dereference
authority. No safe public call chain to the nonzero-offset promotion branch is
established.
The packet-backed Shared branch is proved under its internal layout and
initialized-visible-prefix preconditions. It advances the handle pointer and
packet cursor, reduces length and capacity, preserves the packet capsule,
ticket, region, base and all allocation slots, and specifies the visible bytes
as the old suffix after `min(count, old_len)`. The selected packet invariant
ties each ticket's allocation identity to its region namespace, base and
capacity; the capsule protocol ties the same identity to its current protected
storage authority. The generic ledger split and drop bodies preserve this
identity. It remains metadata only. Both Vec-backed and packet-backed ARC
branches of the public split wrappers are proved for valid, normal-return
inputs. Native clone, automatic-drop and other unselected refcount transitions
are not. The native regression
`shared_advance_past_len_within_capacity_keeps_empty_view` checks the Shared
path when count exceeds length but stays within capacity.
The `BufMut::advance_mut` implementation body has a proof under its
cursor-validity and initialized-slot preconditions, but the generated trait
refinement obligation is not proved because those implementation preconditions
are stronger than the trait prestate.
No selected proof establishes a safe public-call chain that produces a
nonzero cursor. The public `Buf::advance` wrapper is not translated.
A separate body proof now covers the private
`promote_to_shared_count_one` helper: it moves a rebuildable Vec into a Shared
header and binds the nonzero cursor, whole allocation region, count-one native
counter permission, and root owner ticket into the capsule. The native test
`count_one_promotion_preserves_nonzero_offset_view` exercises this helper at
offset 3 with zero additional advance and checks the visible view. Neither
result proves a safe public call chain reaches the promotion branch.

The selected source targets include body proofs for the generic `OwnerLedger`
algebra and current-value refcount operation wrappers. Each `OwnerTicket`
consumes an exact affine singleton fragment for one native strong reference.
Its binding records the allocation identity, a `Mutable`/`Readonly` class and
a half-open interval. Every interval stays in bounds; Mutable intervals are
disjoint from every other registration, while Readonly intervals may overlap
one another. The selected bodies cover root and two-owner registration,
Readonly registration, same-ticket reclassification, splitting a Mutable
range, dropping a registration and replacing a ticket binding while preserving
ticket and authority identities.

`SharedPayload` places one exact native counter permission model, storage
authority, physical pool, all-reference ledger and phase state under a generic
event invariant. The protocol equates the ledger count with the counter
permission value and defines the pool as the in-bounds complement of Mutable
intervals. The selected count-one promotion and count-two Vec split producer
construct payloads with matching tickets and physical regions. Generic pool
retirement and full-range recovery operations are body-proved. The selected
ARC split and explicit disposal paths connect their actual owner tickets and
physical regions to the ledger and pool. The common release carrier also
body-proves the Readonly-ticket branch: it consumes the actual ticket without
changing the pool and uses the same final-owner path. The selected private
freeze helper carries that actual Readonly packet and pointer-cell permission
into `Bytes`; the selected `Bytes::dispose_shared` body consumes them and
calls the common release body. The public `BytesMut::freeze` body and native
static vtable identity, readonly byte reads, native clone, recovery and
automatic-drop transitions remain unproved.

The public `truncate` and `clear` bodies, and unsafe `set_len`, are also
body-proved from `buffer_view_valid`, a common precondition for a stable
unique or packet-backed Shared layout with an initialized visible prefix.
`truncate(n)` proves the `min(n, old_len)` result, preserves the visible byte
prefix, native pointer, capacity, header and exact proof sidecars, and frames
every allocation slot. `clear` proves the corresponding empty-view transition.
Growing `set_len` requires the newly exposed slots to be initialized and proves
that only the logical length changes. A body-proved bidirectional lemma
connects the common prefix observer to the unique and Shared slot models. In
the Shared case, the `BytesMut` type invariant still carries the
`SharedOwnerPacket` capsule/ticket/ledger conditions. These modular method-body
proofs do not establish that every safe producer or lifecycle path supplies
`buffer_view_valid`; they do not prove the public truncate behavior for every
safely produced state.

The private `BytesMut::as_slice` body is also checked under
`buffer_view_valid`: it borrows the actual Vec region or the actual Shared
packet region through the existing B4 read boundary and returns bytes equal to
`buffer_bytes()`. The native `from_raw_parts` read remains in that source path;
this does not prove its `Buf::chunk`, `AsRef`, or `Deref` callers, or readonly
`Bytes` reads.

The selected private `Bytes::as_slice` body also proves its boxed-sidecar
branch under `original_boxed_valid`. It borrows the matching actual full
physical region through the same B4 boundary and returns the saved boxed bytes.
This is limited to that internal proof carrier; it does not prove public
`From<Vec>` or `From<Box>` contracts, `Buf::chunk`, `AsRef`, `Deref`, other
readonly `Bytes` states, or destruction.

The original `free_boxed_slice` body now shares its actual base/view/length
capacity calculation with an explicit unsafe `Bytes::dispose_boxed` entry.
That proof entry consumes the boxed sidecar's real B1 recovery and full-region
authority through the existing B3 boundary. It applies only to the unpromoted
full boxed view; ordinary vtable-driven Drop and cleanup callbacks remain
unproved.

This increment also body-proves `make_count_two_packets`, a generic constructor
for one count-two event payload and its two Mutable owner packets. Under a precondition that
the actual active-owner map covers the full allocation, it places the exact
counter permission, stored authority, ledger, and empty physical pool into
one shared state, then returns the actual disjoint ticket and physical-region
fragments for `[0, cut)` and `[cut, capacity)`. `new_promoted_pair` proves that
coverage for its pair of ranges. The original-source private Vec split
producer, `BytesMut::split_unique_to_shared`, now calls this constructor and
has a body proof that installs the two actual pointer/length/capacity views
with the returned packets. It preserves the allocation frame and each
visible initialized prefix under its strong unique-Vec precondition and the
generic B2 restoration and non-wrapping pointer-offset assumptions described
in [`TCB.md`](TCB.md). The public `split_off`, `split_to`, and `split` wrappers
are selected proof targets for valid, normal-return Vec-backed and packet-backed
ARC branches. These proofs do not establish every safe producer or lifecycle
state, so they do not show that every safe state satisfies the common
precondition for `BytesMut::truncate`.

The selected targets prove the one-shot `RefCountCell` bootstrap extraction
body and the generic `EventBoundInvariant<T>` container and accessors. The
container keeps the exact counter permission outside its protocol payload; its
constructor and extraction preserve the counter model and protocol state.
Generic atomic adapters use one-shot callbacks that preserve this model and
protocol across each unchanged native atomic operation and ordering. These
adapter contracts are trusted and limited to sequential normal-return use:
they do not specify concurrent read coherence, synchronization handoff or
acquire-imported resources. The count-one and count-two constructors tie the
event payload to the corresponding ledger and native counter value, and the
unique-Vec split helper uses the count-two constructor. The selected ARC split
and explicit disposal paths also connect their native source operations to
those registrations. Native clone, the public `BytesMut::freeze` body and its
static vtable identity, readonly byte reads, recovery, and automatic-drop
transitions remain unproved, so this checkpoint does not
establish count-to-reference correspondence across every native transition or
public `BytesMut::truncate` behavior for every safely produced state.

The selected targets also body-prove `PhaseAuthority`'s one-shot close action,
`shared_capsule::make_count_one_packet`, and `promote_to_shared_count_one`. For
this private promotion helper, the proof constructs a count-one Shared capsule
from the actual Vec storage and exact native counter permission, with one root
ticket over the allocation's full physical region and an empty pool. Selected
ARC split and explicit disposal paths connect their source operations to the
registration. Native clone, public `BytesMut::freeze`, readonly reads,
recovery, and automatic Drop remain unproved, and count-one exclusivity and reclamation
are not established for every safe Shared state.

The selected targets also body-prove the generic append-only
`ReadyRegistry<R>` resource mover. Registration transfers the actual affine
`Fragment<R>` into a newly appended permanent slot; taking moves that same
capability out and leaves a tombstone, and returning it fills the slot with
that capability. The contracts preserve each other slot's complete resource
id and RA state. `ReadyCap::update` delegates to the paired authority update and
preserves the fragment id and registry key. The selected private promotion,
split, and explicit-disposal paths use this registry to move their actual
header capability through the caller's Context. `freeze_into_readonly` instead
borrows the caller's tokens and preserves the existing Context entries.
These selected routes do not prove every producer or lifecycle path needed for
`BytesMut::truncate`.

The selected `ownership_proof::shared_context` targets body-prove the affine
`OwnedSharedContext` and `BorrowedSharedContext` carrier. `from_tokens` moves an
actual caller-supplied token set into an owner with an empty registry; the
owner and borrowed-view reborrows preserve the exact token namespaces and
registry final-state observations. `register`, `take`, and `return_cap` move
the actual fragment through the registry while framing its other slots. The
carrier does not create tokens, fragments, or a header permission. Selected
promotion, split, freeze-packet, and explicit-disposal paths use this actual
context; public freeze, readonly reads, and other unselected producer and
lifecycle paths to `buffer_view_valid` or `truncate` remain unproved.

The checked `freeze_into_readonly` path consumes an actual Mutable packet
ticket and its actual physical region, returns a Readonly ticket and returns
that region to the pool. It preserves the reference count, storage,
lease/release authority, caller Tokens, and Context entries. The selected
private `freeze_shared_view` body carries the actual Readonly packet into a
`Bytes` sidecar and preserves the supplied vtable input, but the public
`BytesMut::freeze` body and native static vtable identity remain unproved.
`Bytes::dispose_shared` consumes the actual sidecar and its pointer-cell
permission, then reaches the common release body with the caller's Context.
The native regression `explicit_readonly_dispose_releases_shared_split_once`
checks split, truncate, freeze, and disposal of both related handles, including
the 2-to-1 counter transition and surviving bytes. Readonly byte reads and
ordinary automatic Drop remain unproved; selected operation sequences
explicitly dispose each related handle and assume no Context state after an
untracked automatic Drop.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
