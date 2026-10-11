# Generic Creusot std support for bytes 1.11.1

`scripts/prepare-proof-std.py` prepares a bytes-only copy of the pinned,
normalized `creusot-std 0.13.0` registry package. It applies the reviewed
generic contract patches to the private copy, validates the full stock package
tree and every affected-file hash, and creates the ignored `.cargo/config.toml`
override for `$BYTES_TOOL_ROOT/bytes-proof-std-as-bytes`. The older private copy
is left untouched. The installer migrates the override only when it exactly
matches the previous generated config; custom or conflicting configs are
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
independently required. They add no bytes-specific ownership or refcount theorem and no
termination attributes.

The installer refuses an unexpected stock package, patch hash, private
destination, candidate tree, or conflicting Cargo override. It never replaces
an existing private package. The manifest records the exact reviewed candidate
tree and the complete stock-tree digest.

The generic Vec observation contracts and B1/B2 physical interpretation remain
explicit TCB assumptions; they do not prove bytes ownership or lifecycle laws.
The selected direct-source proof uses the overlay identity recorded in its
receipt. Historical receipts retain their own source and overlay hashes.

The `str::as_bytes` extern spec assumes Rust's native string bytes equal
`Seq<char>::to_bytes()`. That model concatenates the opaque `char::to_utf8`
function; its current assumptions bound each result to one through four bytes
and assert injectivity, but do not prove the encoding matches Rust's native
UTF-8 implementation. Consequently, `BytesMut::from(&str)` content results
depend on this generic encoding assumption. The replacement path is to
body-prove `char::to_utf8` against Rust's UTF-8 encoding and derive the string
byte relation from that result; no bytes-specific string law is needed.
