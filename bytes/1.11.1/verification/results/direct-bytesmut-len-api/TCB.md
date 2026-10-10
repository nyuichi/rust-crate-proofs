# Selected proof boundary

The checked COMA contains the live bodies for `BytesMut::len`, `truncate` and
`set_len`, plus the local ownership/provenance helper bodies in their proof
cone. These contracts are enabled only by the verification cfg. `len` states
only the modeled length. `truncate` and `set_len` require
`unique_valid`, which describes a whole-allocation, offset-zero Vec-backed
state; `set_len` also requires known initialized slots when it exposes bytes.
The proofs do not establish that constructors or other public methods maintain
that precondition, or cover other representations and lifecycle behavior.

The selected bodies call generic Creusot Std ghost/resource and logic
operations, and use the address-only `wrapping_add_u8` specification. The
generic Vec observer assumptions are documented in the
[Std overlay](../../std-support/README.md). The selected call cone assumes no
trusted Bytes-specific ownership, refcount or destructor laws. Verification-only
exclusions for unrelated comparison,
ownership, iterator, and Shared-backed bodies are documented beside the source
in [`bytes.rs`](../../../src/bytes.rs) and
[`bytes_mut.rs`](../../../src/bytes_mut.rs).
