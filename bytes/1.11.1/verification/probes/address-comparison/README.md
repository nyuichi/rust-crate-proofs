# Thin-pointer address comparison in actual BytesMut

The actual `BytesMut::try_unsplit` uses `src/provenance_specs.rs::pointer_addr_eq`
for both thin-pointer comparisons. Normal builds retain `left == right`; proof
builds use `core::ptr::addr_eq`. Both compare numeric addresses for these sized
pointees. The change preserves native behavior and MSRV while withholding
logical provenance identity from a runtime address comparison.

`./scripts/verify-bytes.sh address-comparison` proves six generated files,
including the actual helper and its caller. `--features wrong_identity` rejects
`Coma.vc_rejected_pointer_identity` at its intended VC. It must not infer
physical allocation identity, liveness, or permission from equal addresses.
No new trusted contract was added. This is not a body proof of try_unsplit.

Recorded ordinary bytes checks pass 997 tests and 246 doctests, plus the
no-default-features build. Logs are in `logs/`; generated proof evidence is
retained under `verification/artifacts/evidence/address-comparison/`.
