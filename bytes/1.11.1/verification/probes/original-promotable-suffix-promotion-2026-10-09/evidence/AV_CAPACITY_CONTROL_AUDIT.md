# AV wrong-capacity control: independent null-task reprint

Audited the preserved immutable control snapshot at `/workspace/work/av-capacity-diagnostic-v1-snapshot/bytes/1.11.1/verification/probes/original-promotable-suffix-promotion-2026-10-09`. No Cargo build, proof run, or source mutation was performed.

- The proof-target policy selects only `promotion/shallow_clone_suffix_checked.coma` and excludes the other 166 targets. Its COMA SHA-256 is `e1463e6e3edd8c94c820aaaa8ca7aa14afae48d6aa5942f8011c1f7288174f08`; proof JSON SHA-256 is `75444a957b79223f3cc9238379974d56909d40513b42971e1232b8ef3817ab37`.
- The proof tree independently has 149 prover leaves, 2 null leaves, and no zero-leaf structural targets. The run log reports the single target with 2 unproved obligations.
- The only difference between the saved active source and saved positive source is in `shallow_clone_suffix_checked`: `let cap=distance as usize + len;` changed to `let cap=len;`. Replacing that one statement restores the active file byte-for-byte to the positive file.

I derived the task paths directly from the saved proof JSON and ran `/workspace/work/print_proof_task` on the snapshot COMA with `compute_specified`/`split_vc` only. Both printer commands exited successfully. The full printed outputs exactly match the saved sidecars:

| Null | Goal and proof-tree path | Printed-task SHA-256 | Interpretation |
|---|---|---|---|
| 1 | `vc_shallow_clone_suffix_checked`, `[0,53,2,0]` (`compute_specified:0, split_vc:53, split_vc:2, split_vc:0`) | `e6e8f1e1f67d80cf1393d095a62260fa84550e72a25165fddf7f865dec378ff9` | The `wellformed_Payload` obligation for the reconstructed owner state; `cap=len` fails to preserve the original capacity after a nonzero offset. |
| 2 | `vc_shallow_clone_suffix_checked`, `[0,73,1]` (`compute_specified:0, split_vc:73, split_vc:1`) | `69a5d3ba1ad77d9946a7ceb9a2ecaf5d151185dd0e5ffb3106c386a0cece5747` | The `suffix_clone_result` postcondition, including `result.len + view.offset == descriptor.capacity`, fails with the underreported capacity. |

Both saved task sidecars are byte-for-byte equal to the independent reprints. Saved printer stderr differs only because the recorded run used an absolute COMA path and the independent replay used a relative path; after path-prefix normalization, stderr is byte-identical and contains only Why3 axiom/unused-variable warnings.

This is a targeted proof-sensitivity result for an intentionally mutated proof-source capacity computation. It does not establish a native runtime counterexample. Earlier setup attempts stopped before proof execution: one lacked the AU `generated/positive.rs` input, and a second lacked `original-public-shared-gate/src/relaxed.rs`; they are setup diagnostics, not semantic outcomes. The final saved wrong-capacity run followed source restoration and reached the two recorded null tasks.
