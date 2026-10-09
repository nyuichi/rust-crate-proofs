# AQ initial diagnostic archive audit

## Result

The archived input is internally consistent and its proof tree was independently recounted. It remains a historical diagnostic capture, not a current positive gate or admission: its target policy is marked diagnostic and records correspondence exit status 2.

## Archive and targets

- `aq-positive-diagnostic-v2.tar.gz` SHA-256: `aecd8c7011c932b7c4746d25e48daae9899fbeb2e4560cbdb10acb75f72ea973` (matches the captured claim).
- The tar contains 1170 unique files; all 1170 member hashes match, with no missing, extra, duplicate, or changed members.
- All 139 unique COMA/proof pairs match the hash rows in the receipt. The included target list equals those COMA paths; exclusions and features are empty.
- Independently counted proof tree: **1261 prover leaves, 3 null leaves, 0 structural failures**. Only `promotion/slice_view.coma` contains null leaves. The run log reports `vc_slice_view` unproved (60/63) and one unproved file.

## Archived null tasks

All three null leaves are in `vc_slice_view` from `probe/verif/bytes_original_shared_slice_views_rlib/promotion/slice_view.coma`. I extracted that exact COMA member and reprinted each task with `/workspace/work/print_null_task`; the printed output byte-matches the durable sidecars under `evidence/aq-first-null-tasks/`. The proof tree paths contain an initial `0` for `compute_specified`; the local printer runs that transform itself, so the CLI split paths omit that first index. Passing the full path first returned `Failure("nth")`; the corrected three invocations exited 0. Plugin and COMA warnings appeared on stderr, while stdout was complete and matched the sidecars.

| Proof-tree path | CLI split path | Printed goal | Durable sidecar SHA-256 |
|---|---|---|---|
| `0.21.1.0.1.0` | `21,1,0,1,0` | `view_valid (T_Bytes'mk (result3.f0'1) result1 (result.data) (result.vtable) (View f0'18 (result3.f1'1)))` | `e5a827df204b52c848bfebe7772c70ef38186f3278d480e170c3b653280dee49` |
| `0.21.1.1.1.0` | `21,1,1,1,0` | `view_content (T_Bytes'mk (result3.f0'1) result1 (result.data) (result.vtable) (View f0'18 (result3.f1'1))) = (view_content source)[t'int (range.start)..t'int (range.end')]` | `785c0e94b53513d1daedbe3f953aaa50f6f88f9efd10c8e6f1fa4dd5744c0f84` |
| `0.40.1.1.0` | `40,1,1,0` | `not (resolve_Atomic_ptr_unit (ret.data) /\ match ret.original_shared with \| Root x -> resolve_RootDescriptor x \| Child x -> resolve_ChildProof x \| View x x1 -> resolve_ChildProof x /\ resolve_BoundPtr x1 \| Empty x -> resolve_EmptyViewProof x end)` | `8756352fb7c0bcb2014ee4a610e683d16648355e11a8471856d9e495b8f7d7ef` |

The first goal is the selected view's `view_valid` predicate. The second is equality of the result's `view_content` with the source content projected to the requested range. The third is a resolution obligation over `ret.data` and the `Root`/`Child`/`View`/`Empty` cases of `ret.original_shared`; its exact formula is kept in the sidecar and is not treated here as a separate byte-content theorem.

## Limits

The archive predates later Astra source/generator/checker changes. It records a diagnostic failure, not a final correspondence result, full-crate verification, or admission. No proof or build was rerun for this audit; only archive verification and task printing were performed.

See [AQ_INITIAL_DIAGNOSTIC_AUDIT.json](AQ_INITIAL_DIAGNOSTIC_AUDIT.json) for all member, target, sidecar, and printer hashes.
