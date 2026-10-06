# Architecture decisions — 2026-10-06 (Asia/Tokyo)

Status: adopted work policy, following the user's explicit agreement to freeze
rejected methods and reopen them only when their premises change. Baseline:
`1ae61aabb82b4d777067eb1f83c5fb2ecad5ca77`. This document does not claim a new
proof, silently reduce the verification target, or assert mathematical
impossibility. The previous eight-phase API-expansion proposal is superseded by
architecture admission first.

## Target and authorized changes

The goal remains complete verification of the bytes 1.11.1 implementation under
explicitly documented primitive/tool assumptions, with actual byte semantics,
memory safety, ownership, concurrency, and destruction. Existing scope-limited
gates remain reusable component evidence, not the full target.

Small bytes code changes and explicit consuming cleanup are already authorized.
A cleanup-based implementation may be evaluated as a named modified variant;
proving that variant does not prove original automatic Drop. Do not silently
remove sharing, Clone, downstream trait use, or concurrency to obtain success.
Large Creusot changes and trusted bytes-specific ownership/refcount protocol are
outside the currently authorized approach. Generic physical/atomic primitive
assumptions must stay explicit, reviewed, and distinguishable from protocol
contracts; preserving an existing TCB does not establish its formal adequacy.

The initial architecture witness uses std on x86_64, normal-return execution,
and an explicitly recorded allocator/atomic boundary. Passing it is necessary,
not sufficient, for the full target. Remaining configuration obligations include
no_std, serde, extra-platforms/portable-atomic, platform width/alignment, and
panic/unwind/allocation-failure behavior. No unsupported configuration is
silently marked covered. Loom and native tests are observations, not proofs.

## Frozen methods

| ID | Decision / reason | Evidence | Reopening condition |
|---|---|---|---|
| D01 | Reject ghost classification of mutable physical B4 access: ghost erasure can change the result while the proof still succeeds. | `probes/vtable-leaf-integration/ghost-b4-erased-write.rs`, its manifest and native/proof logs | A different, sound mutable-access encoding with an erasure/initialization argument and the preserved counterexample rejected. An annotation change alone is insufficient. |
| D02 | Reject applying the stock SC invariant-opening rule to weak Release/Relaxed operations: it permits physical permission extraction without Acquire. | `probes/verus-refcount/native_bridge_boundary.rs`, `artifacts/manifest.json`, `CONCURRENT_VERIFICATION_PROGRESS.md` | A generic weak-memory resource rule with a soundness/adequacy justification and missing-Acquire/release-sequence negatives. Numerical postconditions or relabeling a primitive atomic are insufficient. |
| D03 | Reject trusted universal laws for open public Buf/BufMut traits. | `probes/actual-public-buf-default/`, `REMAINING_API_MATRIX.md` | A proved implementation refinement plus an explicit, enforceable implementer contract; a sealed local interface only proves its stated subset. This is a scope/soundness policy, not a failed VC. |
| D04 | Reject trusted bytes-specific ownership, refcount, last-owner, or retirement theorems. | User constraint; `RAW_VEC_TRUSTED_BOUNDARY.md`, `CONCURRENT_VERIFICATION_PROGRESS.md` | Bodies must be proved under an adequate generic primitive boundary. No reopening through renaming a protocol axiom as a primitive. |
| D05 | Freeze stock-tool retries of unchanged callback dispatch/static-pointer materialization and Clone shared-reference ticket splitting. | `probes/native-integration-frontier/logs/actual-clone.log`, `logs/shared-receiver-fraction.log`; `probes/bytes-observer-subset/evidence/constructor-frontier.tar.gz` | Verified relevant translator support or a genuinely different runtime representation/interface with a source correspondence argument. More wrappers, removing const again, or moving declarations without addressing the remaining blocker are insufficient. A finite enum alone addresses dispatch, not ownership. |
| D06 | Reject aggregation of independent models/configurations as integrated verification. | `STATUS.md`, `CLOUD_RESUME_RESULTS_2026-10-05.md`, gate source/configuration manifests | One compatible representation, invariant, configuration and caller composition, with exact code correspondence and all dependencies proved or explicitly primitive TCB. Proof-file counts are not API completion counts. |

Also freeze repeated unchanged whole-crate runs merely to rediscover the same
four frontend errors. The current evidence is
`probes/native-integration-frontier/logs/production-current-frontend.log` and
`native-latest-manifest.json`. Rerun after a relevant source/tool/configuration
change or an evidence-integrity failure, not as architecture progress.

## T01 follow-up decision D07

Freeze the unchanged real-spawn attempt with `State<T>.expected: Snapshot<(T,T)>`
when T carries non-Objective physical ownership. Both spawn sites fail E0277;
no VC phase is reached. Evidence is in `probes/architecture-thread-transport/`.
Native threaded success is not a reason to repeat the proof or assert Sync.

Reopen only with an actually objective metadata representation and a body-proved
relation to the affine AtView payloads, or relevant independently justified
new tool support. A blanket Objective/Sync implementation, an erased payload,
or another wrapper around the same snapshot does not meet that condition.
This freezes one concrete representation, not every possible concurrency proof.

## Reopening procedure

Before an experiment reopening D01–D07, record its decision ID, exact changed
premise (source/tool/contract/configuration), evidence that it changed, the one
question the experiment distinguishes, positive and negative acceptance checks,
and a bounded stop condition. Preserve old failures and scope. A different
agent, larger budget, elapsed time, or renewed optimism is not a changed premise.
An integrity replay of old evidence must be labelled audit, not a new approach.
No extra user confirmation is required for work within existing authorization.
Actual changes to the final target or permitted tool/TCB scope need an explicit
choice; do not infer them from silence.

## Architecture admission gate

Before broad API implementation, require one coherent witness for:

1. Allocate and initialize actual storage; establish its representation relation.
2. Create a second live handle while conserving real affine access authority.
3. Transfer/share the required handles through the chosen real thread boundary.
4. Read the specified bytes while other handles can retire; preserve lifetime.
5. Connect actual refcount RMW/orderings to live handles and resource retirement.
6. Recover and deallocate exactly once at the last retirement, including an
   empty/zero-payload handle. Name automatic Drop versus explicit cleanup.

The witness must include the selected actual implementation bodies, callable
contracts, one ownership model, allocation identity, and at least both retirement
orders. A fixed two-handle witness is only an admission test; arbitrary live
handle counts and subsequent API closure remain full-target obligations.

Negative controls must reject missing Acquire, an outstanding empty ticket,
uninitialized publication, and duplicate/early reclamation at the intended
obligation. Sequential native harnesses or event-label models alone do not meet
thread-transfer or physical-resource requirements. A body with requires(false)
never establishes a reachable lifecycle. Document every standard/tool primitive
assumption; add no bytes-specific trusted theorem.

Admission outcomes: PASS (then lock the architecture and migrate API proofs),
FAIL (record the exact obstruction), or BLOCKED (a prerequisite cannot be
expressed/proved under current constraints). FAIL/BLOCKED does not authorize a
smaller completion claim. First use existing distinguishing evidence; rerun only
where a new premise or missing decisive test makes it necessary.
