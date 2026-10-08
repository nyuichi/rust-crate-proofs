# Architecture decisions — 2026-10-06 (Asia/Tokyo)

> Current policy (2026-10-07): the user authorized removal of the alternative API and high-rework representation. The original bytes API is the target; see `REMOVAL_2026-10-07_JA.md` and crate `AGENTS.md`. Earlier route-1 selection and its admission are historical. Local temporary trusted contracts must meet the stated removal condition; D04's blanket prohibition is superseded, while counterexamples remain valid.

Historical status: adopted work policy, following the user's explicit agreement to freeze
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

## D11: unchanged raw/std scope projection for fallible spawning

Freeze the raw std::thread::scope and private Scope.inner projection routes
after actual experiments and Astra review. A changed parent-slot representation
keeps the ReadHandle in Option until the worker starts. Spawn Err leaves it
available for fallback read and explicit cleanup; successful workers return it
before parent retirement. Both targeted native tests pass. Rust's pinned thread
implementation drops the startup closure on creation Err before calling it.

The actual raw-scope proof generates 230 Coma/proof JSON files and fails exactly
one callable-precondition obligation (candidate 3/4): std::thread::scope has no
stock contract. A separate typecheck confirms that the supported Creusot Scope
is a different type and its inner std scope is private (E0308/E0616). These are
interface limitations, not counterexamples to the runtime parent-slot design.
Root independently audited all 508 and 47 members of the respective archives:

- builder-spawn-scoped-parent-retained-229:
  `61e5fd51b56d73865fe3768a37332968bed0bcb1839ba2491f1f2ce04b059773`.
- builder-creusot-scope-projection-typecheck-229:
  `15d26e3201292ef8317367da53e216edb0a4ed15e43ba68fd04fd6cb3dbb7388`.

Reopen these unchanged routes only with relevant supported scope/spawn contracts
or a genuinely changed interface. No projection bypass or trusted bytes
retirement theorem is permitted. A small generic fallible-spawn library
extension is still under adequacy review; this decision does not reject it or
mark thread-creation failure covered. In particular, failure must preserve the
unexecuted closure's borrowed resources without assuming its normal FnOnce
postcondition. No production source was changed by these experiments.

Adequacy follow-up: reject the proposed unrestricted F error frame. A concrete
native startup failure drops F, whose owned Drop guard mutates an Arc<AtomicUsize>
also held by the parent slot. The callback body never runs, yet the aliased state
changes. A Copy bound rejects this owned destructor capture with Rust E0277.
Both observations, pinned Rust/std sources and exact logs are preserved in
builder-error-drop-alias-copy-negative-229,
`64e3b77915dbb2d3fca108b735511695361962774ca36c2a46b9e7f525ad8795`;
root independently checks all 14 members. This is native/typechecking evidence,
not a formal proof. The restricted F: Copy, explicit mutable-slot, Option-return
standard wrapper is a new bounded experiment still pending its actual caller
proof and mutated-slot-then-failure negative. It frames only the slot's ordinary
value/prophecy and does not assert that independent synchronized state or all
globals stay unchanged. No bytes-specific resource theorem is authorized.

Adequacy review rejected an unrestricted generic error-frame proposal with a
concrete native counterexample: the unstarted closure captures a Drop guard
holding an Arc clone of state also present in the parent slot. Creation Err
drops the closure, the guard changes that atomic state, and the closure body
never runs. Exclusive borrowing of the outer slot does not exclude such aliases.
Freeze the unrestricted error-frame claim; normal FnOnce postconditions cannot
repair it. An enforceable F: Copy restriction rejects that Drop capture (E0277)
and removes this particular destructor effect. It remains a new, unproved
generic standard-boundary experiment, with no claim about unrelated threads or
global observations. Both native witnesses and pinned source are preserved in
builder-error-drop-alias-copy-negative-229:
`64e3b77915dbb2d3fca108b735511695361962774ca36c2a46b9e7f525ad8795`.
Root audited all 14 file members. No Creusot proof is claimed by this archive.

## D12: unchanged generic Hash and standard Formatter boundaries

Actual isolated Hash-only implementation and concrete byte-log caller complete
translation but leave two unresolved results among 259 proof files. The generic
Hasher::write call lacks a callable precondition/effect model; the concrete
recorder caller also cannot recover the desired feed relation from the Hash
implementation's interface. Archive hash-only-unproved is
`afea2920ab829e336eded57144404b6916abca7bbb5dce76afc12330aa0cb0ea`.

Actual Debug-only formatting, after repairing pure-model visibility, leaves
four unresolved results among 259 files. Formatter::debug_tuple, field, finish
and the resulting output have no modeled standard output effect. Two separate
decimal-list model goals also lack byte-range prerequisites; those are repairable
model issues and are not evidence that decimal encoding cannot be proved. Archive
debug-only-contractless is
`ebf2cb6270b9b2c6f50e16c96dafda2b850fa6987677ed514c0fffa5b1580fa4`.
Both actual semantic archives and their member hashes are independently audited.
The initial frontend visibility error and missing Why3find setup are retained
separately as diagnostics, not additional semantic counterexamples.

Astra reviewed the exact failures. Freeze the unchanged generic Hasher and
standard Formatter interface routes: new assertions, timeout increases or opaque
wrappers do not supply their missing contracts. Reopen only for enforceable
standard-library/implementer contracts or relevant tool support, with a bounded
distinguishing experiment. Do not freeze deterministic byte encodings or checksum
algorithms. Route 1 may instead provide explicit body-proved byte_digest and
hex_bytes operations with exact recurrence/Seq output and consuming cleanup;
these are pending experiments, not Rust Hash/Debug compatibility or completed
trait verification. No bytes ownership/refcount law is trusted.

### D11 / D12 changed-premise admission, 2026-10-07

The unchanged and unrestricted routes above remain frozen. The restricted generic Copy-slot Option-return interface now has a positive body-checked bytes caller (256 files, zero null, `56d8767eb6117043bc14008288a0be6c3a64bc5c6b29a333605d835b7705502d`), native success/forced creation failure, and targeted mutated-slot rejection (`8dd93014c04786580f39db0d2c187d1dae114f1e11fe56b4705b8a4df1c4f435`). Astra reviewed the generic ordinary-slot frame, F:Copy enforcement, and normal-return scope. This is sufficient to attempt production integration of the explicit reviewed Std trusted method and the untrusted/body-proved bytes caller. It makes no totality, exceptional cleanup or unrelated-global frame claim.

The explicit polynomial digest/lowercase hex interface has passed its candidate whole-source body gate (266 files, zero null, `a4f04f92ce61c3a9ddd43f3ab930d9ee62beba4c08cacc375df321981ab3995d`) after structural sequence lemmas. It is admitted for production integration as a changed functional API, with exact models and explicit source cleanup. Generic Hasher/Formatter compatibility stays blocked. Neither changed premise trusts a bytes-specific ownership/refcount theorem. Production completion requires its own source/configuration capture.

## D13: unconditional totality under unchanged callback/iterator/allocator/OS interfaces

Actual check(terminates) attempts fail on FnOnce::call_once, Vec::from_iter and its caller's missing classification. The interface-failure bundle and exact original inputs are retained inside termination-support. Marking a finite cursor plus close as terminating reaches the real B1/B3 classification gaps; allocator cleanup is not assumed terminating to bypass them. A separate-module client then fails generated Coma field binding, not arithmetic; moving the client into the existing iteration module resolves that bounded visibility issue and proves nine targeted bodies. The integrated production-finite-cursor-std-287 gate subsequently proves all 287 bodies, including the finite cursor return client and separate normal-return explicit close client.

A small checked callback trait is an enforceable changed interface: its finite implementation, generic dispatcher and closed client prove. A self-recursive implementation carrying check(terminates) and variant(0) reaches Why3; its functional normal-return VC proves, while the actual 0-to-0 variant-decrease VC fails. Both results and native checks are preserved in bundle `fec80ce45ba32c3fe1d2ab1e3058f9e1a35542388e601f22b77234676f63bbd3`. The probe has no value/result contract and is interface feasibility evidence, not completion of the scoped borrowed callback API. General diagnostics and finite cursor evidence are in `2dfef832222326a5f4a00dcab24217fc14ced3cf17fa2e4ae3a249feeaacc4bf`.

Astra reviewed the exact recursive VC and cursor shape. Do not claim unconditional totality for existing arbitrary FnOnce/IteratorSpec APIs or allocator/OS-dependent cleanup and threading. Their interfaces permit divergence or omit termination guarantees. Freeze retries under those unchanged premises; wrappers, ghost Fn facts, timeout changes and new trusted B3 progress assumptions do not repair them. Reopen only with enforceable checked callback/iterator bounds and an explicit separately justified allocator/scheduler progress scope. This is not a claim that every closed byte computation is blocked: the finite checked cursor read terminates and returns its live owner; normal-return close is body proved without a totality claim. No bytes ownership/refcount theorem is trusted.

## D2026-10-08 — Original bounded path and fixed tool frontiers

The original unique/off0 len<cap construct/append/freeze/actual-read gate passes
61 files, with archived exact source correspondence. This admits continued work
on original native representation, not the retired alternative API. It does not
admit a complete crate, Clone, Drop or refcount theorem.

Keep the missing-model generic-comparison ICE and static/atomic materialization
frontiers fixed until premises change. Comparison reopening requires an actual
memory-linked model and contracts retaining the original generic API. The
materialization leaves may be replaced locally when frontend/Std support gives
sound native contracts; their present true-only contracts establish no atomic
values, data pointer relation, refcount or callback facts. Do not repeat the same
frontier with assertions or larger timeouts. Protocol proofs remain separate
open obligations. Details and receipts: ORIGINAL_PATH_2026-10-08_JA.md and
probes/original-freeze-read-2026-10-08/README.md.

## D2026-10-08-S — Strong original contract admission before further ownership work

Stages 1–2 define the final original ownership/refcount contract and audit
freeze/read61 as a reusable physical/content component, not an already-embedded
singleton of a full shared protocol. See ownership-design-2026-10-08/STRONG_SPEC_JA.md.
The exclusive per-handle PhysicalRegion/Recovery/Box Perm storage cannot be
duplicated for Clone; shared authority/access encoding needs redesign. Small
removability of that change has not been established.

Do not grow singleton-only ownership scaffolding or claim a final shared API
architecture before a sound generic interface supports original Clone(&self),
readonly sharing and relevant weak-memory resource transfer. Original signature
and native ordering must be preserved. New state summaries cannot trust bytes
registration/refcount/finalizer laws. D01/D02/D05 remain fixed pending their
specified changed premises. Automatic Drop is still blocked by the existing
Drop-to-Goto semantics; explicit cleanup is not a proof of original scope-exit
effects. Numerical atomic constructor contracts alone do not admit the protocol.

Revisit only on concrete new generic/tool support with adequacy and bounded
negative controls. No new failure replay was needed for this design decision.
This fixes the architecture gate without abandoning the original target or
discarding useful historical proofs. Full API generalization must prove the
strong contract, with bounded branch lemmas called and not relabeled as full APIs.

## D2026-10-08-T — Tokenless immutable sharing is not reclaimable ownership

A changed-premise stock GhostShared diagnostic consumes the actual Vec via B1
and duplicates readonly witnesses through &self. Two actual B4 byte reads
prove34 files. Attempted consuming extraction for B3 is rejected E0507 before
translation. Its immutable arbitrary-lifetime access has no lastness or owned
extraction. Do not adopt this interface as complete original Clone/cleanup
architecture or add an extraction axiom conflicting with its shared lifetimes.

Luna audited the stock shared mutation surfaces; none removes the absent
Tokens/affine permission from original Clone(&self). Astra reviewed the result
and rejected a simple open_at(existing AtomicInvariant, &mut Committer, f)
addition: f can reopen the same invariant via independent existing Tokens, and
commit permission integration remains missing. No such trusted opening was added.

Reopen only with an adequate generic atomic-event-bound opening/update/recovery
interface preserving original API and native ordering, rejecting alternate-entry
reentrancy, duplicate event use, missing Acquire and premature recovery. A new
restricted invariant type is a research candidate, not a proved small wrapper
or a temporary trusted contract with demonstrated local removal. Existing
D01/D02/D05 and original Drop boundaries remain fixed.

Evidence/source audit and detailed reasoning:
probes/tokenless-sharing-2026-10-08/RESULTS_JA.md and audit.json. No bytes-specific
protocol trust, production representation change or original API proof added.

## D2026-10-08-U — User-authorized generic trusted synchronization boundary

After the RefCell/Mutex explanation the user explicitly selected the trusted
boundary direction and required consulting/imitating creusot-std analogues when
blocked. This supersedes treating lack of a presently body-proved new generic
primitive as a reason by itself to stop. Strong reviewed generic contracts may
be adopted as explicit TCB while bytes protocol bodies remain proof obligations.
The pinned upstream tests/should_succeed/mutex.rs itself trusts lock/guard
operations; shipped AtomicInvariant and Committer likewise trust primitive
resource effects. Native interpretation and model scope must be recorded.

D2026-10-08-T's immutable GhostShared recovery rejection and alternate-Tokens
reentry counterexample remain valid. A distinct restricted interface with no
alternate ghost opening entry is a changed premise for a bounded experiment.
Do not copy the rejected open_at extension onto existing AtomicInvariant.
Preserve Objective/AtView weak-memory discipline and actual atomic event identity.
No bytes-specific ownership/refcount law is made trusted by this authorization.

## D2026-10-08-V — Restricted registration caller passes under explicit generic TCB

The D-U experiment uses a distinct EventAtomic, ordinary native Relaxed increments
and FnGhost callbacks, with no alternate ghost-open/stock-invariant path. Bind
consumes one affine protocol state; each event operates on that SAME persistent
state and restores its successor. This native/model/resource correspondence and
Send/Sync discipline are explicit new generic trusted assumptions, not proved
from PhantomData or implied by positive tests.

The bounded registration caller passes 8 proof files and 2 native tests, preserving
a source ticket and producing a distinct ticket through a no-extra-argument &self
method. Six misuse controls reject their intended invalid operations. Admit this
interface only for the reviewed Relaxed registration experiment; do not reopen the
reentrant old open_at rule. No bytes-specific ownership/refcount law was trusted.

Original Bytes integration, physical read sharing, ordering-sensitive recovery,
overflow-abort, last-owner and automatic Drop remain obligations. This does not
pass the full lifecycle admission gate. Next investigate physical sharing and
Release/Acquire using actual Std AtView/Committer rules with corresponding negative
controls, before broad original API integration. Preserve this interface as the
replacement boundary for future generic tool support; local trust removal has not
been demonstrated. See TRUSTED_ATOMIC_EVENT_2026-10-08_JA.md for exact evidence scope.

## D2026-10-08-W — Token-gated immutable borrowing is recoverable

Changed premise from D-T: permanently share FullBorrow<PhysicalRegion>, not the
region itself. FullBorrow.borrow requires a live LifetimeToken. The separate
EndBorrow recovers the region only after the full fraction ends the lifetime.
GhostShared resolves the FullBorrow value, freezing its physical contents.
Copies of the descriptor cannot obtain a region after the token lifetime ends.
No new physical Objective/Sync law or bytes-specific protocol trust was added.

Fixed-two A proves41 files with full input-byte equality at both overlapping
reads, including after peer retirement; native Release decrements, actual
Acquire load, affine token join/end and actual B3 cleanup. Per-side agreement
receipts plus a diagnostic extra Acquire observation body-prove exactly-one
normal-return final result. Missing Acquire fails3 VCs, premature half-token end
fails1; duplicate tokens and still-live B4 read retirements are rejected by
Rust E0382/E0505. The generic operation-bound native/model state rule and RMW
release-sequence rule remain explicit trusted assumptions.

Admit this reclaimable physical-sharing component only in its fixed2 scope.
Subjective EndBorrow/Recovery stay in parent; final ownership transport to any
last thread, original constructor1/dynamic Clone, control/data/vtable binding,
overflow-abort and automatic Drop remain obligations. Preserve D-T direct-region
GhostShared extraction rejection and D02 missing-Acquire counterexamples.
Evidence: probes/shared-physical-lifecycle-2026-10-08/README.md, RESULTS.json and
positive-final-41.tar.gz with independent root receipt.

## D2026-10-08-X — Bind the reclaimable component to actual native fields

Changed premise from D-W: construct the original Shared three-field record and
the original Bytes four-field record, including actual core atomic fields,
rather than retaining an independent EventAtomic in the caller. The body-proved
constructor connects B1 buffer/capacity and the typed Box permission to that
Shared, its ref_cnt permission to its moved actual field, and Bytes.data to the
same control pointer. It proves34 files/114 leaves (selected constructor13) under
explicit generic native/model atomic TCB. Vtable is an explicit input; the actual
SHARED_VTABLE constant/indirect public call is not thereby verified.

The same-number wrong-ref_cnt and wrong-Shared.buf isolated controls each fail
exactly one constructor leaf; field identity is not inferred from count equality.
Archive/root audit receipts are under original-shared-lifecycle-2026-10-08.

The bounded next experiment is count1->one affine-quota clone->both retire orders
with a single compatible token/map protocol and actual control/payload cleanup.
Recovery is owned by the first ticket until its Release publication, then by the
last observer only after actual Acquire. A body-proved ghost transition interface
must be shared by the component and the source leaf; no duplicate shadow counter
or bytes-specific trusted registration/lastness law is allowed.

A scoped owned-lease generic atomic boundary is authorized for the control loan
obstruction: a typed FullBorrow plus live token protects the same control block;
a body-verified field projection selects its actual atomic; no native reference
escapes the event; retirement receives the token after the last field access.
The primitive models access/order/event state, not count-to-handle or lastness.
Its adequacy remains explicit TCB, with replacement by tool/Std support preserving
the same interface. Ordinary byte reads keep token-tied lifetimes.

Positive acceptance requires genuine byte contents, both release orders, actual
field/data identity, full authority recovery and matching explicit cleanup.
Misuse acceptance requires missing-Acquire, live-read, wrong-field/pointer and
duplicate-resource rejection. Review interfaces after two same-shaped failures;
restructure after three. Preserve failed attempts and the D-T/D02 counterexamples.
Sequential bounded leaves do not pass full architecture admission: arbitrary
Clone, overflow/abort interleavings, real-thread transport, public vtable dispatch
and automatic Drop remain separate obligations. Do not expand disconnected APIs
or relabel these bounded results as the original public crate's verification.

## D2026-10-08-X result — selected native-field lifecycle accepted

The bounded acceptance gate now has a matching-source positive: protocol77 and
original-source leaf44 including an end-to-end driver, both with zero nulls and root
archive/source audits. Recovery is transported by the original ticket or its
Release publication, not retained in a parent. Actual field identity, physical
reads across peer retirement, both release orders, Acquire recovery and exact
buffer/control cleanup are connected in body-checked source-correspondent code.
There is no shadow refcount, trusted bytes registration/lastness law or extra
post-cleanup atomic observation. Source-interface repairs export only body-proved
token/atomic/address observations; numerical addresses never grant provenance.

This accepts the selected initial-owner/one-clone sequential leaf only. Full
architecture admission remains pending: arbitrary Clone and overflow/interleaving
behavior, real-thread Send/Sync transport, actual vtable/public dispatch and
automatic Drop are outside the result. Existing D-T, D02 and frontend failures
remain frozen unless their premises change. Preserve all failed source snapshots
and distinguish semantic negative controls from frontend resource errors.

The final driver44 run restores Acquire and proves both reader/release orders
with exact input-sequence preservation. A fresh targeted source mutant omits
Acquire and its callback: only Pending::recover's acquired-view guard fails,
while its validity guard succeeds. Current borrowed-slice cleanup is rejected
with E0505. These close the selected-path source controls, not the full original
architecture admission gate; earlier socket/daemon interruptions are recorded
as execution diagnostics and never counted as mathematical failures.

## D2026-10-08-Y — Shared declaration and registered indirect calls

Changed premise from the frozen direct MIR FnPtr call: a safe erased pointer
can invoke the shipped FnExt contract through explicit Fn::call. Named function
reification remains unsupported, and unsafe pointers do not implement Fn.
The isolated unsafe-affine-dispatch probe therefore generates a native target,
a body-checked safe shim calling that exact target, and a private registration
getter together. Generic registration and indirect invocation remain explicit
trusted correspondence rules, with no arbitrary pointer/spec pairing or
function-address identity assumption. Both native and shim bodies are checked.
The accepted restricted result is six proof files/12 actual prover leaves, zero
nulls; a mismatched certificate leaves exactly the registration guard unproved,
and resource duplication fails E0382. Root source/archive audit is in the probe.

Resource forwarding outside the callback is body checked, but callback-owned
ghost transformations, actual SHARED_VTABLE selection and public Clone are
not established. The next distinguishing experiment must transport ghost
input/output inside the checked shim while preserving its native erasure.
Do not count scalar dispatch as bytes ownership/refcount verification. Existing
FnPtr, Drop, reentrancy and Acquire counterexamples remain frozen.

Original Shared and the selected lifecycle leaf now include one maintained
three-field declaration from src/bytes/shared_record.rs. These remain distinct
compiled types in distinct crates; the public Bytes layout/API is unchanged.
The current source-correspondent 44-file gate has 280 actual prover leaves and
zero nulls, plus native default1011/no-default1011/portable cleanup2 tests.
The common-declaration archive and root receipt are in the original-source probe.
The external affine CloneQuota still prevents integrating Clone(&self); moving
that quota to a receiver cannot make it movable through &self. A new
registration-only experiment must issue fresh affine tickets repeatedly using
one shared source ticket and a residual lifetime-token pool, proving bytes
registration bodies rather than assuming them. Full retirement/publication
coexistence and overflow/interleaving admission remain separate obligations.
