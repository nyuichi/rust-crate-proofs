# Current trusted assumptions and scope

The constructor proof path uses generic trusted B1 boundary
`VecRawParts::take`. Its shared native implementation consumes the original
`Vec<u8>` with `ManuallyDrop` and reads `as_mut_ptr`, `len` and `capacity`
once; the runtime `BytesMut` fields use that exact tuple. The trusted contract
binds it to the Vec model and to one recovery token plus the full slot region:
the initialized prefix matches the Vec contents, and spare-capacity slots are
unknown. The zero-capacity pointer is dangling metadata for an empty region;
the contract grants no allocation-liveness or dereference authority.

This generic interpretation assumes the default global allocator and `u8`
layout, exclusive ownership transferred by moving the Vec, capacity at most
`isize::MAX`, and a nonwrapping base-plus-capacity address range. The body of
`bind_consumed_vec` constructs the logical resource split and slot ledger from
the sealed witness. The selected `from_vec`, `with_capacity`, `new`, `zeroed`,
`From<&[u8]>`, `From<&str>`, and `Default` bodies establish this state, with
the exact copied or initialized prefix and unknown spare slots. They also
establish `unique_vec_rebuildable`: the visible initialized bytes and any
hidden prefix needed when reconstructing a Vec are known; spare-capacity slots
remain unknown. `clear`
preserves the state and all capacity-slot values while setting length to zero.
The body of `is_empty` is checked without a representation precondition. The
capacity encoder and decoder, and local ownership/provenance helpers are
checked against the original crate source. No trusted Bytes-specific
ownership, refcount or destructor law is used in this call cone.
The replacement path is to verify Vec, allocator and `ManuallyDrop`
representation/transfer contracts that derive this exact native-tuple to
affine-allocation binding; proving only `bind_consumed_vec` would not remove
this trust.

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
allocation slots. The private `advance_unchecked` body is proved only with a
rebuildable Vec state, `count <= len`, capacity bounds and representable packed
position; the selected branch is the Vec no-promotion path. This does not prove
that any safe public call chain produces a nonzero offset. The public
`Buf::advance` wrapper is not translated, and the trait-refinement obligation
for `BufMut::advance_mut` remains unproved because its implementation body has
stronger proof preconditions than the trait prestate. Promotion, Shared states,
immutable peers, lifecycle operations, concurrency and unwind behavior remain
outside the selected `BytesMut` proof scope.

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
value while leaving the cell sidecar empty. No production `Bytes` or `BytesMut`
path calls this helper, and it does not establish a handle or owner-ledger
registration.
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

The scalar refcount protocol does not contain owner tickets. The body proofs do
not connect constructors, clones, conversions, drops, or handle transfers to
the `OwnerLedger`; they establish neither count-to-registration equality nor
count-one exclusivity. Terminal recovery, Shared allocation retirement,
destruction, and end-to-end `BytesMut::truncate` across Shared, split, or
nonzero-offset states remain unproved.

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
