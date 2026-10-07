# Architecture decisions — 2026-10-06 (Asia/Tokyo)

Status: adopted work policy, following the user's explicit agreement to freeze
rejected methods and reopen them only when their premises change. Baseline:
`1ae61aabb82b4d777067eb1f83c5fb2ecad5ca77`. This document does not claim a new
proof, silently reduce the verification target, or assert mathematical
impossibility. The previous eight-phase API-expansion proposal is superseded by
architecture admission first.

## Target and authorized changes

The user selected route 1 on 2026-10-06 UTC: the final target is a clearly
named modified bytes 1.11.1 implementation with changed public API and runtime
representation where necessary, under the pinned stock verifier. Complete
verification of the original public API/automatic Drop is no longer the selected
target. Existing proofs remain component evidence until composed into the actual
modified implementation; this decision itself proves no code.

Explicit context/lease operations, consuming sharing or splitting, borrowed read
views and explicit consuming cleanup are authorized. Record a correspondence for
every retained, replaced or excluded legacy API and configuration before claiming
full coverage. Sharing and concurrency must have actual body/resource proofs;
a fixed-two-handle admission witness is not arbitrary-handle completion.
Large Creusot changes and trusted bytes-specific ownership/refcount protocol
remain outside the approach. Generic physical/atomic primitive assumptions must
stay explicit and reviewed; preserving an existing TCB does not establish its
formal adequacy. D01–D07 remain frozen. An actual new representation/interface
is the changed premise for the modified implementation, not permission to retry
unchanged failures.

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

## Modified ordinary-mutation and thaw experiments

Changed premise for D01: ExclusiveBytes holds a genuine ordinary Vec, not a
raw B4 mutable view whose physical writes are classified ghost. Its Vec
mutations are ordinary program effects. Typed DerefMut may reuse the stock
Vec's current/final mutable-borrow relation; it must reject an attempted mutable
borrow of an ordinary owner from ghost code. The old raw B4 erasure witness
remains frozen and preserved. Positive mutation-to-shared composition already
passed in phase2-final-67; the typed-trait positive and separate final-relation
and ghost-borrow controls are a bounded experiment, not an annotation fix for
the old representation. Two equivalent failures require interface review; three
require restructuring under the playbook.

Next thaw experiment retains B1's original sealed RawAllocation in the private
owner. A body-proved borrowed metadata helper may derive a BoundPtr without
copying Recovery or any physical authority. After actual receipts reunite full
lifetime fractions, a private recovery body obtains the full original region.
Existing B2 consumes that descriptor and region to rebuild Vec; B3 consumes
them for cleanup. No new trusted bytes-specific thaw/protocol theorem is
authorized. Formal byte contents and affine recovery are the acceptance checks.
The canonical Vec model lacks pointer/capacity identity, so native identity
checks and the reviewed B2 mapping are recorded separately from formal output
identity. Failure sources must be archived before restructuring.

## D08: native floating-point bit conversion under stock contracts

Freeze the unchanged materialized-float route after two distinguishing actual
Cursor experiments and Astra review. First, an honest `Option<f32>` contract
using `to_bits()` is rejected during translation because `to_bits` is a
program function called in logic. Archive `cursor-f32-to-bits-frontend`:
`4de5ac395388b32e059b0e20ce0efd8739939187a31653e175fd9475803e14b0`.
Second, a runtime `Option<(u32,f32)>` returns the decoded word plus
`f32::from_bits(word)` without float logic. Translation succeeds, but the
contractless `from_bits` call has an impossible precondition; its actual caller
VC fails 1/2. Archive `cursor-f32-from-bits-contractless-vc`:
`2cc70dde091e17d210c4419ba9b6152ddc61c671d989a5147afadce73af2b85b`.
Both native candidates passed 20 tests; native success does not prove bitcast
semantics. Production Cursor was restored byte-for-byte to positive183.

No trusted inverse-bit axiom or bytes-specific float theorem is introduced.
Reopen only with a reviewed generic bitcast contract and a model preserving the
required IEEE/NaN payload distinctions, or relevant supported translation.
This is a current tool/contract boundary, not a mathematical impossibility.
A changed API exposing exact Float32Bits/Float64Bits may replace byte-level
float transport; its reads/writes must be proved through the same production
Cursor/ExclusiveBytes. Native float materialization remains separately uncovered
and must never be implied by proof of the raw bit pattern.

## D09: generic Serde interfaces without implementer contracts

Freeze the attempted stock-tool generic Serialize/Deserialize integration after
actual configured runs and Astra review. Both native candidates pass 27 tests
under verified,std,serde. Serialize's own body proves the exact slice argument,
but fails the unconstrained Serializer::serialize_bytes precondition (2/3).
Deserialize initially has five null obligations. Replacing str::as_bytes and
String::into_bytes with the already-proved copy_from_str/from_string operations
closes the two avoidable conversion failures. The final three nulls are only
Deserializer::deserialize_byte_buf and SeqAccess::{size_hint,next_element}
callable preconditions; the other visitor bodies prove.

Generated Coma exposes opaque preconditions with only false-to-pre axioms,
and no serializer output/effect semantics. An added trusted ensures(true)
would permit delegation without proving serialization behavior; it is not
completion. No universal Serde law, ownership theorem or new TCB was added.
The production source was restored exactly to positive229; candidates remain
reviewable in immutable failed-source archives. The serde configuration and
original trait responsibilities remain uncovered, not silently omitted.

Reopen only with a genuinely enforceable verified Serializer/Deserializer
interface and implementation refinements, or relevant tool support. The existing
verified callback API can transport exact borrowed bytes, but must not be called
proof of Serde's format/error/visitor behavior. Do not repeat these candidates
with assertions, timeouts or a different agent.

Audited exact archives:
- `verified-serde-serialize-229`: `6947ef468ed267ecc4bb0ac4696194bb7696ae8787642cce7b49620aa34decbe`.
- `verified-serde-deserialize-229`: `c2489d452ff2f6191b6d360eed0986a770b978f9b5865313959bb82ff6679ca4`.
- `verified-serde-deserialize-conversions-229`: `2b40df5521b325ad573960639037e4baa58ce2b34a3ef28f2bc2ac2bc34ca8f5`.

## D10: caught callback panic and exceptional resource framing

Freeze the attempted stock-tool catch_unwind route after a distinguishing actual
implementation and Astra review. The candidate retains each ReadHandle, ticket
and worker token outside the user callback's catch boundary, retires after either
Ok or Err, joins workers, and explicitly closes or thaws the parent owner before
returning or resuming a panic. Native tests pass 30/30, including first, second
and both callback panics, empty/spare-capacity allocations and thaw identity.

The actual configured translation crashes in backend/resolve.rs:187 while
resolving the caught Box<dyn Any + Send> payload. No Coma or proof obligations
are generated; zero null leaves here is not a successful proof. Independently,
the stock contracts lack catch_unwind/resume_unwind exceptional-effect framing.
An ordinary FnOnce normal-return postcondition does not establish preservation
of resources on unwind. The exact candidate, native log, translation diagnostics
and compiler crash are archived as callback-panic-catch-unwind-ice-229:
`f4de2c948a77decfbf7b11c43d899d06c24df9fb5aafbb7dd05df4f0e567ca4d`.
Root independently checked all 49 archive members and their hashes.

No exceptional bytes ownership theorem or trusted catch wrapper is introduced.
Reopen only with relevant payload translation support and a reviewed generic
exceptional-effect interface, or a genuinely different enforceable callback
interface whose failure behavior can be proved. Cosmetic payload wrappers,
timeouts and normal-return contracts alone are insufficient. Panic/unwind remains
uncovered; the failed candidate is not retained in production. Thread creation
failure is a distinct pending experiment and is not decided by this result.
