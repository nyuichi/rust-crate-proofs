# Generic Creusot std support for bytes 1.11.1

`scripts/prepare-proof-std.py` prepares a bytes-only copy of the pinned,
normalized `creusot-std 0.13.0` registry package. It applies the reviewed
generic Vec contract patches to the private copy, validates the full stock
package tree and every affected-file hash, and creates the ignored
`.cargo/config.toml` override for `$BYTES_TOOL_ROOT/bytes-proof-std`. The
installed registry source and other crates are untouched.

The active patches are:

1. `alloc-capacity.patch` exposes alloc-gated Vec, Box, String, conversion and
   slice contracts, including generic Vec capacity and fallible reservation
   contracts.
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

The former `copy-slot.patch` and three-patch manifest are preserved in
`verification/retired-support/` as historical evidence for the retired
modified API. They are not part of this installer or the active overlay.

The generic pointer observer direction was reviewed with Astra and checked in
`probes/vec-pointer-correspondence-2026-10-08`: the getter caller proves, while
a different-Vec pointer claim fails. The library observation contracts and B1/B2
physical interpretation remain explicit TCB, not bytes ownership proofs.
The previous exact private std tree is retained locally as
`bytes-proof-std-before-pointer-2026-10-08`; archived earlier proof receipts
continue to refer to their old source/tree hashes.
