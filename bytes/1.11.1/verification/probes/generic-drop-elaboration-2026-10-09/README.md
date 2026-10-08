# Generic normal-edge Drop elaboration (label 2026-10-09)

This is the next bounded architecture experiment recommended by Astra after the
full-domain constructor increment. It reopens the stock Drop-to-Goto obstruction
with a different input to Creusot: a checked external effect elaboration before
borrow and liveness analysis. It does not modify Creusot and does not assume any
bytes-specific destructor, last-owner or complete-issuance theorem.

`native.rs` is the unchanged native input for this experiment. SetTrueOnDrop
reproduces the effect shape of the retained automatic-drop diagnostic; ToggleOnDrop
adds a non-idempotent exact-once discriminator. Three native scope witnesses
cover true, toggle, and a final toggle after an original write. Native tests run
these actual scope-exit destructors, not proof shadows.

The pinned Rust native MIR after ElaborateDrops records the real normal Drop
edges and separate unwind successors. The elaborator generates a proof shadow
with identical fields/lifetimes but no Drop trait implementation. It copies each
native destructor body into an ordinary &mut helper and inserts one call at each
supported normal Drop edge. This avoids retaining an implicit destructor in
addition to the inserted call. The copied destructor, effect/frame summary and
caller postcondition must all be body-proved; there is no #[trusted] helper or
caller. `generated/active.rs` is the selected proof input, never a production file.

The helper frame preserves the original nested borrow's future:
`^(guard.0) == ^((^guard).0)`. Specifying only the updated field's value leaves
the original loan unconstrained and cannot propagate the effect to the caller.
An unconditional future value assertion would be unsound because the caller may
legally mutate the guard later. The new relational frame is itself body-proved.

An independent correspondence checker reconstructs the mapping from native
source/MIR and selected shadow; it checks target, place, type, order, exact count,
and copied destructor body. Generator hashes alone are not correspondence
proof. Native compiler/spec erasure and checker adequacy remain **explicit generic
tool TCB**, including preservation of borrow interpretation. The removal path is
supported native MIR Drop/liveness effect lowering passing the same controls.
Std mem::drop only ensures resolve(t), so it does not already supply this effect.

Scope is freshly declared local guards, straight-line normal return, known
monomorphic Drop bodies, scalar checked operations and fields without independent
drop glue. Unknown calls, alias escape, unsafe/raw use, threads, forgotten guards,
unsupported moves/drop flags or field glue are rejected. Unwind edges are retained
in the mapping and excluded from the normal-return theorem; panic completion and
exceptional cleanup are not established by this gate.

Positive proof is accepted only with a successful independent correspondence
check and every selected Coma target proven. Diagnostic variants deliberately
omit, duplicate, reorder or replace a call, or give a false helper summary.
Correspondence failures and semantic VC failures are reported separately.
Toggle makes omission and duplication observably wrong. No failed feature is
positive evidence. Unknown effects are checker controls, not guessed semantics.

The complete bytes target remains NOT ADMITTED. Success here would establish a
small tool-effect prerequisite, then Astra should be consulted again to choose
its extension to actual closed Bytes clients and complete event tracking. It
would not itself prove native Bytes automatic Drop, all Clone issuance, promotion,
branch-aware cleanup, or the remaining API.

The initial positive-v1 body gate proved seven files / twelve actual prover
leaves / zero nulls: the three scope callers, two destructor effects, set_true,
and a helper-interface caller that mutates the same borrowed flag after the
effect. The latter proves false after a later write, distinguishing the sound
relational loan frame from an unconditional future-value promise. Only the three
native scope witnesses receive source/MIR correspondence claims.

The retained positive-v1 capture is historical development evidence. Final
correspondence hardening, control results, immutable matching-source capture and
independent audit will be recorded after completion.


Diagnostic results (all seven targets retained):

| Feature | Correspondence | Semantic null leaves |
| --- | --- | ---: |
| omit_drop_call | rejected | 3 (all native scope callers) |
| duplicate_drop_call | rejected | 2 (toggle callers) |
| wrong_drop_target | rejected | 2 (toggle callers) |
| early_drop_call | rejected | 1 (toggle after write) |
| wrong_drop_summary | structurally accepted; body proof required | 5 (both helpers and three scope callers) |

A false summary is not a structural mismatch: its body proof must fail. All
five captures are failed diagnostic evidence. The positive target list is fixed
to all seven expected bodies, so a missing body target cannot silently disappear.
Astra additionally required exact original set_true signature/attributes and
rejection of trust/axioms throughout the shadow call chain, plus exact normal
and cleanup successor blocks. These checks are part of the independent checker.


The restored positive gate proves **seven files / fourteen actual prover leaves /
zero nulls** after all five diagnostics. The body scope is unchanged; two helpers
retain finer proof decomposition from the negative summaries. The original
positive-v1 (7/12/0) archive is never relabeled as this restored result.

`python3 check_checker_controls.py` checks independent structural
controls, including unknown callbacks, escapes, threads, moves/forgotten guards,
unsupported field glue/panic paths, body mutation, hidden trusted false summary
with a refreshed mapping, and extra normal/cleanup successor statements. These
are coverage failures, not Why3 nulls. Its result is stored in
`fixtures/checker-control-results.json`.

Run `bash run-proof.sh` elevated for the positive gate. A negative run is
`bash run-proof.sh FEATURE --diagnostic`; diagnostic mode records the checker
failure and still translates/proves the intentionally wrong source to preserve
semantic counterevidence. It is never accepted as positive proof. The wrapper
uses the shared proof lock, one prover, the pinned1024MiB configuration and no
sc-drf. It forces this package to retranslate after feature switches.

`evidence.py capture LABEL LOG --status proved|failed|diagnostic` freezes exact
inputs, all seven selected Coma/proof pairs, correspondence/checker records,
private Std, tool provenance/configuration and logs. `evidence.py audit LABEL`
checks member/target/proof hashes and recursive statistics; independent replay
of source/MIR/shadow checks is an additional requirement. The canonical published
receipt is evidence/drop-positive-final2-2026-10-09.json with its independent
audit. Historical v1 and final captures retain earlier checker versions and
are never relabeled as matching the published checker.


Correspondence tokenization also masks comments and string literals before
recognizing executable calls/items, so a commented or quoted fake call cannot
stand in for an actual Drop effect. The control count and exact checker hash in
the final matching-source receipt supersede development control reports.


The final structural corpus contains **22 controls**, including comment/string
fake calls, fake item/Drop headers and comments between contracts and function
headers. Item discovery and call matching mask non-code while preserving original
spans. Exact set_true visibility/signature/attributes and effect-helper visibility,
parameter types and summaries are independently checked. The published independent
receipt is `INDEPENDENT_AUDIT.json`.
