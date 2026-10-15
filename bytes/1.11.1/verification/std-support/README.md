# Generic Creusot std support for bytes 1.11.1

`scripts/prepare-proof-std.py` prepares a bytes-only copy of the pinned,
normalized `creusot-std 0.13.0` registry package. It validates the stock tree,
applies the three reviewed generic patches to the private copy, verifies the
candidate tree, and generates the ignored `.cargo/config.toml` override for
`$BYTES_TOOL_ROOT/bytes-proof-std-as-bytes-refcount`. The previous generated
copy is left untouched. The installer migrates the Cargo override only when it
exactly matches the prior generated config; custom or conflicting configs are
refused. The installed registry source and other crates are untouched.

The active patches are:

1. `alloc-capacity.patch` exposes alloc-gated Vec, Box, String, conversion and
   slice contracts, including generic Vec capacity and fallible reservation
   contracts. It also specifies `str::as_bytes` as
   `result@ == self@.to_bytes()`.
2. `address-model.patch` supplies the opaque numeric Vec base-address observer
   and its `as_ptr` / `as_mut_ptr` contracts.
3. `pointer-model.patch` adds the exact mutable Vec getter pointer observation
   and preservation of pointer/capacity. This connects B1/B2 to the native
   pointer word rather than deriving it from a numeric address. The observer
   grants no access permission, allocation injectivity or liveness.

The original `ownership_proof::raw_vec` bridge uses `capacity_model` and
`base_model` to connect its allocation descriptor to the same Vec value. These
are generic library assumptions with no sequence-based allocation identity,
pointer permission, injectivity, or allocation-liveness fact. The exact pointer
observer is metadata only; the sealed matching B1/B2 capabilities remain
independently required. These assumptions add no bytes-specific ownership or
refcount theorem and no termination attributes.

For an element type whose modeled size is exactly one byte, `capacity_model`
is also bounded by `isize::MAX`, matching Rust's maximum Vec allocation size.
This conditional bound does not constrain zero-sized element types.

The pinned Std contracts do not expose Box pointer identity or physical
allocation resources. The original-source proof therefore passes the actual
consumed `Box<[u8]>` through a narrow B1-BOX boundary; the nonempty full-capacity
`Vec<u8>` route uses B1-BOX-VEC to connect that same Vec through
`into_boxed_slice` and the Box raw-pointer transfer. These boundaries preserve
the actual allocation resources used by the source proof and make no Bytes
representation, vtable-identity, or lifecycle claim.

Replacement path: move the one-byte capacity bound into the upstream Vec
capacity contract; replace B1-BOX-VEC with upstream Vec/Box raw-parts contracts
and a body proof of the source transfer; and replace STD-PTRWRAP-01/02 with
upstream exact contracts for the one-byte wrapping-add and wrapping-offset
operations, or body-prove those standard operations. These pointer contracts
state exact nonwrapping offsets only and grant no allocation or dereference
authority.

The installer refuses an unexpected stock package, patch hash, private
destination, candidate tree, or conflicting Cargo override. It never replaces
an existing private package. The manifest records the exact reviewed candidate
tree and the complete stock-tree digest.

The generic Vec observation contracts and B1/B2 physical interpretation remain
explicit TCB assumptions; they do not prove bytes ownership or lifecycle laws.
The `str::as_bytes` extern spec assumes Rust's native string bytes equal
`Seq<char>::to_bytes()`. That model concatenates the opaque `char::to_utf8`
function; its current assumptions bound each result to one through four bytes
and assert injectivity, but do not prove the encoding matches Rust's native
UTF-8 implementation. Consequently, `BytesMut::from(&str)` content results
depend on this generic encoding assumption. The replacement path is to
body-prove `char::to_utf8` against Rust's UTF-8 encoding and derive the string
byte relation from that result; no bytes-specific string law is needed.
