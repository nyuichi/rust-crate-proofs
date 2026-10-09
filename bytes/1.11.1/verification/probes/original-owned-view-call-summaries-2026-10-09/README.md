# Modular scoped owned-view call summaries (AU)

AU preserves the complete published AT positive at commit
e3a9b0f8003c1dd2eb5de55cd36695cc2e7f74e0 and adds two ordinary proof bodies:
clone_suffix_checked and owned_view_call_scope. Published ancestors and
production bodies are unchanged. D-AU records the distinguishing named-call
summary premise and metadata-only proof-reuse conditions before experiments.

## Modular scoped owned-view call summaries (AU) — 2026-10-09

AU proves an ordinary function boundary that accepts a borrowed owned Shared
Bytes view and returns a fresh owned view. The native clone_suffix(&Bytes,usize)
body calls Clone, advances by any amount<=len, and returns the result; complete
advance retains Shared ownership at zero length. Its reusable body-proved
summary preserves the source, original allocation and BoundPtr namespace/capacity,
exports the exact shifted suffix and inserts a fresh actual lifetime ticket.
The caller uses that summary across arbitrary finite capped runtime steps,
with normal replacement Drop, exact singleton inventory and final recovery/free.

The completed full body proof has 157 targets/1447 prover leaves/zero null and
structural leaves. Admission reuses its exact 157 COMA and 157 proof files from
immutable diagnostic SHA256 ffc00ffe50a408f971f2a56e1d58f7173d3d9856ebeb65e97c7a725d9e9393bd,
with unchanged proof-relevant Rust/Cargo/Std/config inputs. No second solver run
is claimed. Fresh actual Cargo four-artifact capture, full source/native/MIR and
proved-summary correspondence audit, and one refreshed forged-callee registry
control pass separately. Native capture passes 84 cases with 31 selected MIR
bodies, including 29 unchanged production bodies and the actual helper/caller.
Unchanged audited AT native 45 and ownership/Drop controls are reused, not rerun.

The parameterized named_direct_call_v1 correspondence boundary independently
binds source signatures, ordered retained arguments, erased Ghost reborrows,
actual MIR callee/result, and the proved callee/caller COMA summaries. Its v1
scope is two native arguments, trailing Ghost mutable reborrows, one direct
call and normal moved return over audited source/type/MIR instances. Per-instance
native body pins remain outside that generic rule. This is explicit generic
compiler/correspondence TCB; no trusted Bytes summary or ownership law is added.

Canonical admitted_reuse and independent immutable audit PASS: SHA256
0924cfd015be3e7c7284c94b88dc4921ccdbf6e0c9821172774568ebe86ef701,
1,670 members and all 314 origin/current/canonical COMA/proof hashes identical.
The outer admission records fresh correspondence exit 0; the original target
policy retains diagnostic=true and correspondence exit 2. The archived origin
log records `Proved (157 files)`; its numeric process exit status is not recorded. No
second solver run is claimed. Exact nested AT/source/control ancestry closes
the diagnostic's sibling-import gap without outside data. The audit checks 61
production source files, three Cargo manifests, 110 private Std files and seven
tool/config inputs. Eight external executable hashes match the archived manifest;
their payloads are not bundled. Cargo absolute paths remain location-bound,
while archive-only replay uses captured artifacts without a live Cargo target.
Native fixture-count and source-item trailing-newline errors remain separate
preserved diagnostics; they changed no native/proof body.
See verification/probes/original-owned-view-call-summaries-2026-10-09/evidence/AU_CANONICAL_AUDIT.md.

Full original architecture remains NOT ADMITTED. This establishes scoped modular
Shared owner return, not arbitrary untracked escape, general callgraphs/types,
Root/general Static/custom-owner Clone, concurrency, cleanup/unwind, BytesMut
or complete API/configuration coverage. After completed audited publication,
ask Astra「次何するのがいい？」and execute the next recommendation.

## Archive-only replay layout

1. Restore inputs/repository/bytes/1.11.1 as the crate root and its captured
   sibling probes. Place AU probe/ members at the AU probe path.
2. Copy inputs/au-proof-origin/au-full-diagnostic-v1.tar.gz and its .json to
   AU evidence/, preserving exact bytes. The embedded .log records the original
   completed body-proof run; no numeric exit receipt is claimed.
3. Restore AT from the nested canonical-v2 archive's probe/ members plus its
   separately embedded audits/receipts. Restore AS/AP/AQ/AR from their nested
   canonical archives. Materialize historical repository snapshots solely from
   AT v2 inputs/repository/ members for ancestry comparisons.
4. From restored AU, run python3 check_correspondence.py
   --audit-compiled-capture-only. This uses captured Cargo artifacts, invokes
   neither Cargo nor a prover, and needs no live target directory.

See evidence/AU_CANONICAL_AUDIT.md for exact audit scope and restoration details.

## Reconstruct the published archive

GitHub limits single files to 100 MB. The exact audited canonical archive is
published as two ordered parts, with sizes and hashes in
`evidence/au-positive-reuse-canonical-v1.tar.gz.parts.json`. From this probe:

```sh
cat evidence/au-positive-reuse-canonical-v1.tar.gz.part001 evidence/au-positive-reuse-canonical-v1.tar.gz.part002 > evidence/au-positive-reuse-canonical-v1.tar.gz
sha256sum evidence/au-positive-reuse-canonical-v1.tar.gz
```

The reconstructed SHA256 must be
`0924cfd015be3e7c7284c94b88dc4921ccdbf6e0c9821172774568ebe86ef701`
before running the archive audit or restore steps above. Splitting changes no
archive member, proof, input, receipt, or admission claim.
