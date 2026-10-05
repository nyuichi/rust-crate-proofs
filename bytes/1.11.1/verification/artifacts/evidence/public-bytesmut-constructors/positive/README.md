# Public `BytesMut` constructor probe

This probe extracts the actual `BytesMut::zeroed` and core
`From<&[u8]> for BytesMut` source items from `src/bytes_mut.rs`, along with the
actual `from_vec` constructor, its unique-at-zero predicates, and explicit
release method. It includes the existing bound Vec, capacity, and provenance
modules by path. It omits the runtime `BytesMut` destructor and does not make a
capacity guarantee part of the constructor contracts.

The source contracts state the resulting length and initialized byte contents,
together with the existing unique-at-zero validity predicate. The exact-source
extraction records source line numbers and FNV-1a identifiers, including those
contract attributes. The `From<&[u8]>` impl is the real core trait impl, not a
local replacement; the Creusot caller also includes a generic `T: From<&[u8]>`
call.

Native fixtures pass 3/3. They cover empty and nonempty zero-filled
allocations, empty and nonempty copied slices including zero and `0xff`, and
independence after the source slice is modified. Each fixture reconstructs the
exact Vec from its raw parts so the isolated test frees the allocation
explicitly without claiming anything about automatic `Drop`.

The target-scoped vanilla Creusot 0.13 run proved 60 files. This is isolated
component evidence for these extracted constructor bodies and the included
helpers; it is not an integrated full-crate proof. The gate adds no trusted
contracts. It inherits the existing B1 Vec-detachment and B3 explicit
deallocation contracts in `ownership_proof/raw_vec.rs`. Automatic `Drop`,
capacity guarantees, and shared/split construction remain outside this result.
