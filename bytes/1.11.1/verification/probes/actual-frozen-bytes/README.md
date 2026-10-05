# Actual frozen Bytes gate

This gate extracts the actual cfg `BytesMut::freeze`, `Bytes::with_vtable`,
`Bytes::as_slice`, `proof_share_frozen` and explicit ticket-return methods.
A sequential `FrozenOwner` holds allocation recovery while overlapping immutable
receivers hold affine lifetime fractions. Returning every fraction permits
explicit deallocation; no bytes protocol assumption is added.

The 2026-10-05 resumed positive gate passed 64 proof files with zero unproved
leaves; the native receiver/read/close matrix also passed. Proof JSON and emitted
Coma are archived in `evidence/cloud-resume/positive.tar.gz`, whose digest and per-file hashes are
pinned by `evidence/cloud-resume/manifest.json`. The freeze contract now preserves the actual pointer,
which connects absolute initialized slots to the receiver's relative slice.
The historical `artifacts/positive.log` records the older 14/15 caller failure;
the archive's `logs/positive.log` is the successful replay.

This is the unique-at-zero freeze configuration with an explicit coordinator.
The ordinary native freeze/Clone/vtable/automatic Drop lifecycle is outside the
gate. `proof_share_frozen(&mut self)` is an explicit affine splitting adapter;
it is not a proof of `Clone::clone(&self)`. Historical diagnostics preserve the
unsupported dynamic vtable call and shared-receiver affine mutation attempts.

`negative_missing_fraction` deliberately omits one returned ticket from the
join; reclaim must remain blocked. `negative_shared_clone` and
`negative_actual_clone` preserve the distinct native Clone frontiers.
Run `bash run-proof.sh`, or append `--features FEATURE` for a negative control.
