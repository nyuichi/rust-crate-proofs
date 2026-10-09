# AN: promoted original-root re-clone and three normal automatic Drops

Development experiment after audited AM `ae7b7e5124c064d2e686ad1a9fb20d52e5c2b795`.
The complete original bytes 1.11.1 architecture remains **NOT ADMITTED**.
The restored canonical gate passes 120 targets / 914 prover leaves /
0 null and 0 structural leaves, with full source/native correspondence status0,
no defect features, no exclusions and no diagnostic flags. Independent
canonical archive audit remains the publication requirement.

The native witness constructs a nonempty Box-backed Bytes, clones the original
once through Raw-to-Shared promotion, clones the same original again through
its existing Shared branch, drops the second then first child at lexical scope
exit, reads the original, saves the returned Vec, and drops the original on
normal return. Both native tag parities are included in the intended body proof;
the execution test is corroboration and does not establish allocator parity.

The distinguishing premise is AL's body-proved first-promotion transition plus
AM's exact terminal-Drop elaboration. The second clone must perform the actual
Acquire pointer load and guarded Relaxed refcount increment/registration. It
must allocate neither a new Shared control nor perform another promotion CAS.
The original retains its owned pointer history; a fresh child receives its own
read-only binding. No ghost value is materialized as a runtime expected pointer.
New bytes-specific protocol bodies must be proved under the existing documented
generic physical, synchronization, erasure, provenance and compiler TCB.

The immutable AL prefix and AM terminal adapters are reused, with new shared-
phase helpers and a three-owner client. The final gate requires all generated
Coma targets, no defect features or exclusions, actual Cargo compiled-record
inputs, exact source/native MIR/erasure correspondence, deliberate semantic and
typing controls, archived inputs/proofs, and independent archive audit.

Normal completion is the bounded scope. Concurrent CAS losers, escaping clones,
unwind, allocation failure, arbitrary clone/Drop contexts, other APIs and build
configurations remain open. A successful AN gate will not imply full-crate
verification. After each audited published increment, ask Astra
「次何するのがいい？」 and execute the next recommendation.

Run scripts use the pinned `/workspace/bytes-proof-tools` installation. Evidence
archives capture source, configuration and binary hashes; tool payloads remain
external. Why3 runs elevated, under the shared proof lock, with one concurrent
prover and a 1024 MiB limit; `sc-drf` is disabled.

## Recorded distinguishing experiments

The first translated body-proof run left one full callback postcondition in
both parity branches unproved (120/988/2). Public metadata equality did not
imply child-to-cursor model identity. Adding the body-proved returned-child
`accepts(final cursor)` export closed that interface without weaker posts or
new trust; the next diagnostic run passed all120 targets. Exact failed Coma
inputs/tasks and the pre-solver missing-why3find-config failure are immutable.

The ten semantic controls leave real null leaves, respectively: wrong ARC
branch2; reset current view4; omit registration2; omit second/first/root Drop
1/1/1; swap child identities1; omit final Acquire2; omit payload/control frees
2/2. Exact tasks are independently replayed from each archived Coma input.
These counts are proof sensitivity, not concrete native execution failures.
The generic missing-Acquire/free features also affect retained Shared bodies;
they are single defects with paired paths, not additional independent coverage.
Missing frees use uninhabitable resource-conjure paths, not a theorem of native
receipt absence. See evidence/AN_SEMANTIC_CONTROL_AUDIT.md.

Five frontend controls reject duplicate second/first/root consumption with
E0382, root retirement across a live read with E0505/E0502, and extraction of
owned history through a borrowed phase for readonly resealing with E0507.
The v2 archives exclude all solver caches and proof tasks. The immutable v1
archives, and four historical AM frontend captures, include old `.why3find`
cache statuses despite overbroad exclusion metadata; that discrepancy is
recorded in evidence/AN_FRONTEND_CONTROL_AUDIT.md. It is not fresh solver
execution or a failed compiler diagnostic, and old archives are not rewritten.

The independently replayed main correspondence controls reject49/49 and
native source/MIR controls reject52/52. Review found a draft dependency-contract
hole and a live-Cargo-only capture omission. Ten imported proof-source pins,
checker pre-import digests, full generated client/extension tokens and four
captured compiled-input artifacts now close the exact reviewed defects.
Astra replayed a contradictory external contract and a proof_assert macro
with refreshed active/mapping hashes; both reject. See TCB.md and the evidence
architecture/checker review notes for precise assumptions and replacement paths.

Canonical archive: `7181ae4c07997888e1af1dfad41fdb109a489b4ca101e1dd972c9b6d4d13843e`,
875 hashed members. This self-referential status paragraph is post-capture;
proof/source/checker/native/generated executable inputs remain the captured
versions. Independent final archive audit is a publication requirement.
