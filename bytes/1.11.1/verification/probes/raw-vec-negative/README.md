# B2 raw Vec recovery negative probes

These probes import the production B2 types and functions directly from
`src/ownership_proof/owned_region.rs` and `src/ownership_proof/raw_vec.rs`.
They do not change the ordinary `Vec` model, add trusted declarations, or call
any invalid recovery function at runtime. Each feature checks that one missing
B2 precondition is rejected while the other relevant preconditions are stated
on the probe function.

| Feature | Probe function | Intended B2 precondition failure |
| --- | --- | --- |
| `wrong_half_region` | `wrong_half_region_recovery` | A region `[0, 1)` cannot recover an allocation of capacity at least two. Observed `Coma.vc_wrong_half_region_recovery: ✘ (13/14)`; one split obligation remained unproved. |
| `wrong_unknown_prefix` | `wrong_unknown_prefix_resume` | Slot zero is `Unknown`, so a one-byte Vec prefix cannot be resumed. Observed `Coma.vc_wrong_unknown_prefix_resume: ✘ (13/14)`; one split obligation remained unproved. |
| `wrong_namespace` | `wrong_namespace_resume` | Full caps from a different sealed namespace cannot be combined with this raw descriptor. Observed `Coma.vc_wrong_namespace_resume: ✘ (12/14)`; the two unproved split leaves are the raw/recovery and raw/region namespace equalities. |

The feature functions are abstract contract callers over sealed B2 values;
they do not manufacture `RawAllocation`, `Recovery`, or `PhysicalRegion`. The
negative checks target the trusted B2 boundary's requirements. A rejected VC
is not a runtime test and does not invoke `Vec::from_raw_parts`.

Run each feature independently through `scripts/verify-bytes.sh
raw-vec-negative --no-default-features --features <feature>`. Expected goals and
compiler/translation errors are recorded in `logs/`. All three runs reached
Why3 without compiler or translation failures; each rejected only the intended
precondition goal(s). Coma files and proof JSON are archived separately under
`verification/artifacts/evidence/raw-vec-negative/<feature>/`.
