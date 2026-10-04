# Physical region retirement probe

Pinned unmodified Creusot/creusot-std 0.13. The sealed physical pool consumes
actual B1 region resources, preserves slot values, and returns a full region
only after exact capacity coverage. B2/B3/B4 remain the reviewed physical
primitives; the pool and supporting resource-algebra bodies add no trust.

The default gate proves 25 files. It covers both explicit retirement orders,
splitting at capacity, and independent B4 writes followed by retirement and
B2 Vec recovery with changed and unchanged byte contents preserved. Two native
tests cover empty/zero capacity, spare capacity, nonempty halves and mutation.
These are explicit helper calls, not actual Shared, refcount, or automatic Drop.

Run from the crate directory with elevated proof execution:

- `./scripts/verify-bytes.sh retired-region-pool`
- `./scripts/verify-bytes.sh retired-region-pool --features negative_half_finish`
- `./scripts/verify-bytes.sh retired-region-pool --features negative_wrong_namespace`

The two feature runs intentionally fail. The half case follows real B1 split
and retirement but omits one fragment; only `Coma.vc_reject_half_finish` has
one failed coverage leaf. The namespace case checks the sealed-token API over
assumed Ghost inputs with equal capacity and disjoint domains, but different
namespaces/resource IDs; only `Coma.vc_reject_wrong_namespace` has two failed
matching leaves. It does not prove constructing those inputs from two B1
allocations. Invalid feature bodies must not be executed as native tests.

Exact per-run sources, logs and tasks are retained under
`../../artifacts/evidence/retired-region-pool/`. Core 23-file evidence is a
separate historical snapshot; the final extended gate has 25 files, while each
negative feature has 26 with only its designated failed leaves.
