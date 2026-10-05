# Transactional advanced-handle transition probe

This probe tests a structural retry for unique `BytesMut::advance_unchecked`.
The build extracts the live `BytesMut` layout, `proof_unique_owned`,
`proof_initialized`, `proof_empty_valid`, and the exact current
`advance_unchecked`, `get_vec_pos`, and `set_vec_pos` methods. The extraction is
round-trippable and records method hashes under `extraction/`.

The experiment uses a deliberately strong checked invariant for this probe:
the canonical empty descriptor is valid, or the handle is both
`proof_unique_owned()` and `proof_initialized()`. Those are the actual source
predicates. `proof_unique_owned()` allows a positive advanced offset; it binds
the pointer offset to the packed Vec offset and requires `offset + cap` to
equal the original Recovery capacity. `proof_initialized()` then requires the
currently visible prefix to remain Known. This probe does not claim the
runtime crate's current broader invariant or its Shared states.

The candidate transition replaces `self` with the exact canonical empty
descriptor (zero length/capacity, Vec-kind invalid pointer metadata, unbound
dangling `BoundPtr`, and all proof caps absent). It consumes the old handle
into `RawTransition`, a private field carrier with no `Invariant` impl, using
`ManuallyDrop` and field reads so the old value's `Drop` cannot release the
allocation during the transition. The carrier moves `unique_at_zero`
unchanged, preserving the full Recovery and PhysicalRegion; it also carries
the pending/shared affine fields. It advances the actual `BoundPtr` with
`advance_within(count)`, updates the packed metadata through the actual
`capacity_ops::set_vec_pos_in_data`, applies the source `saturating_sub` and
capacity subtraction, and constructs a fresh `BytesMut` in one struct
expression. No intermediate `BytesMut` has weakened or missing capabilities.

The isolated extraction does not include the production `Drop for BytesMut`
implementation; `ManuallyDrop` is present to exercise the required integration
boundary, but this probe does not verify destructor dispatch or cleanup.

The experiment is intentionally only about this normal-return field
transition. It does not prove automatic Drop, promotion on offset overflow,
Shared transitions, deallocation/recovery consumption, or all public
`advance_unchecked` callers. The original method is retained beside the probe
so reviewers can compare its exact field semantics with the transactional
carrier step. A passing result would validate a candidate integration shape,
not the unchanged production method.

Run `cargo check --offline` after activating the bytes proof toolchain. The
Creusot proof wrapper must run outside the sandbox and serialize on
`/tmp/itoa-creusot-proof.lock`; logs and proof files are saved below this
directory after that run. At the current checkpoint, the normal Cargo check
passes; the serialized Creusot run has not yet reached the prover, so there is
no proof result to count either way.
