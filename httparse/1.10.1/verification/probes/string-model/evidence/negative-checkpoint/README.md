# String-model negative checkpoint

This frozen snapshot records one elevated Why3 run for exactly these two
negative targets:

1. `negative/verif/httparse_string_model_negative_harness_rlib/deliberately_false_string_claim.coma`
2. `negative/verif/httparse_string_model_negative_harness_rlib/deliberately_false_utf8_claim.coma`

The identical COMA inputs and their Why3 session XML files are copied into this
directory. Their original session roots were:

- `negative/verif/httparse_string_model_negative_harness_rlib/deliberately_false_string_claim/why3session.xml`
- `negative/verif/httparse_string_model_negative_harness_rlib/deliberately_false_utf8_claim/why3session.xml`

The solver was `z3@4.15.3`, under the isolated string-model package/config,
with one prover and 1000 MiB memory. `evidence/negative-prover-results.jsonl`
records `vc_as_bytes` as `Valid`, the false string goal and its split as
`Timeout` (~31.10s), and the false UTF-8 goal as `Timeout` (~36.78s). There is
no `Invalid`/SAT result. The false claims remain unproved; this is not a
counterexample result.

An earlier sandboxed invocation failed to connect to the local Why3 socket
and hit a Why3 timer assertion. Its output is preserved separately at
`evidence/negative-infrastructure-attempt.log`; those labels are excluded
from solver results.

`STAGE.targets` is the exact target list. `REPORT.snapshot.md` freezes the
parent report at this checkpoint, so later report edits do not invalidate the
bundle. `SHA256SUMS` fingerprints the copied inputs, generated
goals/sessions, source manifest, isolated compiler/model artifacts, report
snapshot, and the two result logs. The global/isolated Why3 config and
compiler/package hashes were unchanged by this checkpoint.
