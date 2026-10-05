# Checked unique advance transition

This probe extracts the live `BytesMut` layout and proof predicates, the exact
`bytes_proof_valid_handle` invariant, the exact `RawTransition` implementation,
and the exact `advance_unchecked`, `kind`, `get_vec_pos`, and `set_vec_pos`
methods from `src/bytes_mut.rs`. It records source hashes and the extracted
items under `extraction/`.

Under `all(creusot, bytes_proof_valid_handle)`, the checked invariant accepts
either the canonical empty descriptor or a unique-owned initialized handle.
For a unique `KIND_VEC` handle, the actual `advance_unchecked` body installs a
canonical empty placeholder, moves every field into `RawTransition` with
fieldwise `mem::replace`, and forgets the emptied shell. The carrier retains
the original Recovery and PhysicalRegion, advances the actual `BoundPtr`,
updates packed position metadata, and builds the valid result. A final
`mem::replace` installs that result and forgets the resource-free placeholder,
so `BytesMut::drop` never sees an unarmed shell. This does not use `ptr::read`
or add trusted ownership resources.

The default native branch and the shared-handle path retain their existing
code. The verified checked invariant deliberately excludes registered Shared
handles; this proof gate covers the unique Vec transition and the canonical
empty no-op. The extraction does not include production `Drop for BytesMut`,
so the proof establishes the field transition and explicit destructor
suppression, not deallocation behavior or destructor dispatch.

The latest focused run proves the extracted `RawTransition::from_valid`,
`RawTransition::advance_to_valid`, and the exact production
`BytesMut::advance_unchecked` bodies (`Proved (3 files)`). The empty-handle
preconditions are selected only by `bytes_proof_valid_handle`; other Creusot
probes retain the existing non-empty pointer and ownership preconditions.
Generated Coma files, proof reports, extraction hashes, and logs are retained
under `verif/` and `logs/` for this run.

The focused body proof uses the exact extracted helper and method bodies:

```sh
BYTES_PROVE_PATTERN='verif/bytes_advanced_transactional_rlib/actual/impl_RawTransition/* verif/bytes_advanced_transactional_rlib/actual/impl_BytesMut/advance_unchecked.coma' bash verify.sh
```

Run `cargo check --offline` after activating the bytes proof toolchain for a
native build. The Creusot proof wrapper must run outside the sandbox and
serializes on `/tmp/itoa-creusot-proof.lock`. Logs and retained proof files are
saved below this directory.
