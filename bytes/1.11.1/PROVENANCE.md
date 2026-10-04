# bytes 1.11.1 provenance and verification scope

**Verification status: isolated runtime helpers and storage/deallocation
foundations proved; complete runtime verification blocked.**

This tree corresponds to the published bytes 1.11.1 archive with SHA-256
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`, upstream revision
`417dccdeff249e0c011327de7d92e0d6fbe7cc43`.

The previous Creusot configuration substituted `src/verification.rs` for the real
runtime. That length/capacity model is preserved but no longer exported or counted
as implementation coverage. Real bytes/buf modules are now translation targets.

The previous trusted subtraction contract was incorrect for a<b. Both arithmetic
helpers now use their unchanged runtime bodies in `src/arithmetic.rs`, with
corrected contracts and no trust; those exact bodies are independently proved on
Creusot 0.13 with a reviewed standard integer-conversion external specification.

The full runtime does not yet translate/prove successfully. Recursive trait bounds,
comparison modeling, indirect vtable calls, exposed-provenance casts and automatic
Drop support are recorded in `TOOLCHAIN_DECISION.md`. General-length Box storage
and a source-corresponding free_boxed_slice body/caller have separate proof
artifacts; these do not prove Bytes constructor/vtable/refcount/Drop paths.

See `STATUS.md`, `VERIFICATION_SCOPE.md`, `CONTRACT_AUDIT.md`, `TRUSTED_BASE.md`,
`SOURCE_CORRESPONDENCE.md` and the coverage/unsafe ledgers for precise boundaries.
`./verify-all.bash` checks the real default-std target and currently reports its
blocker. Isolated proofs have explicit entrypoints in `scripts/verify-bytes.sh`.
