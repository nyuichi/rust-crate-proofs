# AN diagnostic archive audit

Read-only closure and null-task audit for the AN re-clone diagnostic captures. The full machine-readable record is [AN_DIAGNOSTIC_AUDIT.json](AN_DIAGNOSTIC_AUDIT.json). No prover, build, Rust source edit, or proof rerun was performed.

## Captures

- `an-positive-infrastructure-v1.tar.gz` (`00601337…625965`): all 6,239 archived members and 120 target `.coma` inputs match their manifest hashes. No proof results were captured. The run log records successful compilation and then stops because the probe-root `why3find.json` was missing. This is an infrastructure interruption before proof tasks, not a body-level failure.
- `an-positive-diagnostic-v2.tar.gz` (`d15bc8cf…cc20c5f`): all 7,227 members, 120 target inputs, and proof receipts hash correctly. All 120 targets were included, with no exclusions or features; correspondence status is 2 and the capture is diagnostic. Results total 988 prover leaves and two null leaves.

The two nulls are at proof-tree path `[0, 17, 1]`, one each in `vc_even_reclone_checked` and `vc_odd_reclone_checked`. Both parent goals report 38/39. I extracted the archived Coma inputs and replayed the printer with split path `17,1`; both stdout outputs match the bound sidecars byte-for-byte. The printer exited 0. Its stderr had Why3 plugin dynamic-load warnings.

The failing obligation is in the full `reclone_result` postcondition after `shallow_clone_arc_checked`: the returned child proof must accept the final cursor. The v2 helper contract established child validity, content, public identity, cursor model/public preservation, and observation updates, but did not state that returned-child/core acceptance relation. Treat these nulls as the specific helper-interface gap, not as a counterexample to a complete bytes theorem.

## v3 comparison

`an-positive-diagnostic-v3.tar.gz` (`af4b16fd…afbeeb3`) has 7,236 verified members and 120/914/0 results. Comparing its captured source with v2 shows the positive proof-source fix adds only the helper postcondition that a returned `Child(p)` has `p.core.accepts(^input_cursor)`. No resource getter or new generic TCB body was introduced for this fix. The generator separately records a `readonly_reseal` negative feature, which is not enabled in the positive target policy.

V3 remains diagnostic: correspondence status is 2/NotRun. Its proof results do not constitute gate admission.
