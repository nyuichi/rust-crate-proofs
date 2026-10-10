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
the sealed witness. The selected `from_vec` and `with_capacity` bodies,
capacity encoder and decoder, and local ownership/provenance helpers are
checked against the original crate source. No trusted Bytes-specific
ownership, refcount or destructor law is used in this call cone.
The replacement path is to verify Vec, allocator and `ManuallyDrop`
representation/transfer contracts that derive this exact native-tuple to
affine-allocation binding; proving only `bind_consumed_vec` would not remove
this trust.

`len` is proved without a precondition. `truncate` and unsafe `set_len` require
`unique_valid`, the whole-allocation, offset-zero Vec-backed state. `set_len`
also requires capacity bounds and known initialized slots for any growth.
`from_vec` and `with_capacity` establish this state for their selected path;
other constructors and transitions are not shown to establish it. Shared and
nonzero-offset states, immutable peers, lifecycle operations, concurrency and
unwind behavior are outside this proof scope.

The selected bodies use generic Creusot Std ghost/resource and logic
contracts, plus the address-only `wrapping_add_u8` specification. Their Vec
observer and resource assumptions are documented in the
[Std overlay](../../std-support/README.md). Verification-only exclusions for
unrelated comparison, ownership, iterator and Shared-backed bodies are
documented beside the source in [`bytes.rs`](../../../src/bytes.rs) and
[`bytes_mut.rs`](../../../src/bytes_mut.rs).
