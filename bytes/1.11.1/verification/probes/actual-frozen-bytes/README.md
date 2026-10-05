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
join. Its fresh replay rejects the final reclamation caller at 13/14 goals;
`evidence/cloud-resume/negative-missing-fraction.{tar.gz,json}` preserves the
source, Coma, proof JSON and log. Run `bash run-proof.sh --features
negative_missing_fraction` to repeat it. Current native Clone diagnostics live
in the separate `native-integration-frontier` diagnostic archive.

The production immutable B4/frozen-reader classification was replayed separately:
all 64 files pass again. `evidence/readonly-positive.{tar.gz,json}` records that
exact source/proof snapshot. The classification changes no native reads or
writes and adds no protocol trust.

After adding the private `Vtable.promotable` field, the exact-layout frozen
gate was replayed again and passed 64 files. The
`evidence/tagged-vtable-layout-positive-v3.tar.gz` archive contains the final source tree, regenerated extraction and
source correspondence, proof COMA/JSON, and run logs; its adjacent SHA-256 and
member manifest pin the archive. The bytes.rs and bytes_mut.rs snapshots match
the final tag source hashes. The native fixture's uncalled forbidden callback
table now supplies `promotable: false` to match the extracted Vtable layout;
the exact-source compatibility test passes 1/1. This replay does not widen the
selected proof scope.
