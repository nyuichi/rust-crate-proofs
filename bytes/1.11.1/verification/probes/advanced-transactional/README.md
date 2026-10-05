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
dangling `BoundPtr`, and all proof caps absent). It transfers the old handle's
fields into `RawTransition`, a private field carrier with no `Invariant` impl,
using fieldwise `mem::replace` calls. Each source field is replaced with its
canonical empty value, then `mem::forget` suppresses the emptied shell's
destructor. This avoids `ptr::read` and keeps the allocation capability out of
any temporary `BytesMut`. The carrier moves `unique_at_zero` unchanged,
preserving the full Recovery and PhysicalRegion; it also carries the
pending/shared affine fields. It advances the actual `BoundPtr` with
`advance_within(count)`, updates the packed metadata through the actual
`capacity_ops::set_vec_pos_in_data`, applies the source `saturating_sub` and
capacity subtraction, and constructs a fresh `BytesMut` in one struct
expression. The consumed local shell is temporarily normalized field by
field, stays private to the transition, and is forgotten after it reaches the
canonical empty descriptor. The caller-visible handle remains empty while the
raw carrier is prepared. A final `mem::replace` installs the fully constructed
result and returns the empty placeholder; that resource-free placeholder is
immediately forgotten so its destructor cannot dispatch on an unarmed handle.

The isolated extraction does not include the production `Drop for BytesMut`
implementation. The explicit `mem::forget` models the requirement to suppress
the emptied shell's destructor, but this probe does not verify production
destructor dispatch or cleanup.

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
directory after that run. `BYTES_PROVE_PATTERN` accepts whitespace-separated
Why3find file patterns, and `BYTES_TRANSLATE_ONLY=1` stops after translation.
The focused body proof uses these three translated files:

```sh
BYTES_PROVE_PATTERN='verif/bytes_advanced_transactional_rlib/actual/impl_RawTransition/* verif/bytes_advanced_transactional_rlib/actual/impl_BytesMut/advance_transactionally.coma' bash verify.sh
```

The focused pinned Creusot 0.13 run passed all three files. The bodies of
`RawTransition::from_valid`, `RawTransition::advance_to_valid`, and
`BytesMut::advance_transactionally` are proved against their reviewed
contracts. This is a focused body proof, not a full integrated proof of the
probe crate or production `Drop` behavior. The run log and retained Why3 proof
artifacts are in `logs/proof-focused.log` and `evidence/positive/verif/`.
