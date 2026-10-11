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
the exact copied or initialized prefix and unknown spare slots. `clear`
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
`truncate`, `clear` and unsafe `set_len` require `unique_valid`, the
whole-allocation, offset-zero Vec-backed state. `set_len` also requires
capacity bounds and known initialized slots for any growth. The listed
constructor bodies establish this state for their selected paths; other
constructors and transitions are not shown to establish it. Shared and
nonzero-offset states, immutable peers, lifecycle operations, concurrency and
unwind behavior are outside this proof scope.

The supporting `split_view_region` body preserves the full physical allocation
while splitting the cursor-relative visible region, including any hidden
prefix. Five selected pure `release_sources` algebra bodies are also proved
from the pinned Std source. They specify set propagation over abstract event
history only; they do not prove native atomic source selection or any Bytes
synchronization property.

The selected bodies use generic Creusot Std ghost/resource and logic
contracts, plus the address-only `wrapping_add_u8` specification. Their Vec
observer and resource assumptions are documented in the
[Std overlay](../../std-support/README.md). Verification-only exclusions for
unrelated comparison, ownership, iterator and Shared-backed bodies are
documented beside the source in [`bytes.rs`](../../../src/bytes.rs) and
[`bytes_mut.rs`](../../../src/bytes_mut.rs).
