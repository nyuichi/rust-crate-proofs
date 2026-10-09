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
zero nulls, plus native default1011 and portable cleanup2 tests, plus no-default library build.
The common-declaration archive and root receipt are in the original-source probe.
The external affine CloneQuota still prevents integrating Clone(&self); moving
that quota to a receiver cannot make it movable through &self. A new
registration-only experiment must issue fresh affine tickets repeatedly using
one shared source ticket and a residual lifetime-token pool, proving bytes
registration bodies rather than assuming them. Full retirement/publication
coexistence and overflow/interleaving admission remain separate obligations.

## D2026-10-08-Z — Checked ghost effects inside registered erased callbacks

Changed premise from D-Y's external forwarding: a closed macro skeleton emits
one native target and a checked shim with exactly one call to that same target
and all extra work inside a checked, terminating ghost block. The shim consumes
an affine Resource<Excl<Int>>, applies the shipped ExclUpdate, preserves its
resource identity and returns the native-result-related value. Native and shim
bodies are checked. Generic erasure registration/invocation assume replay of
that exact checked ghost execution; arbitrary pointer/shim pairs are not admitted.

The final isolated gate proves six files/19 actual prover leaves, zero nulls.
A wrong native/shim certificate fails exactly one admission guard; consumed-input
duplication fails E0382 and writing a native result in ghost code is rejected.
Complete archives, exact printed negative task and root source/hash audits are
in unsafe-ghost-transform-dispatch-2026-10-08. Generic correspondence remains
TCB and may be replaced by supported erased-call contracts while preserving
the client interface; no bytes registration/refcount law is assumed.

This result does not synchronize a ghost update with a native atomic event.
Actual bytes integration must put its body-proved transition at the same single
RMW through the operation-bound generic atomic interface and a checked field
projection. Calling the native RMW and then performing another model adapter
RMW would be a double increment and is rejected as an integration strategy.
The actual five-field Vtable binding, Ghost source-ticket transport through
Clone(&self), dynamic registration/retirement coexistence and suppression of
a later automatic Drop after explicit cleanup remain obligations.

## D2026-10-08-AA — Quota-free registration accepted in registration-only scope

The changed premise is stock LifetimeToken residual splitting plus a checked
fresh-key authority update, instead of the external one-use CloneQuota. The
constructor and each registration body preserve the full fraction accounting,
source fragment and actual Relaxed increment relation. One caller registers
twice from the same borrowed source; a body-proved stock Fragment/Excl
composition lemma proves the two new ticket IDs distinct. The public wrapper
contract includes new_id > source_id. Final23 files/113 actual prover leaves
close with zero nulls; native one-test observes1,2. Forgotten-map insertion
leaves one state-preservation and three dependent callback goals unproved;
owned-ticket duplication fails E0382. Root receipts verify all171 final archive
members and seven current source/configuration inputs, plus the negative inputs.

The registration invariant has exact live prefix and count modulo usize range.
No removals, payload recovery or release-sequence carry are claimed. Replacing
C's existing bounded State with it is invalid: that State has retirement,
AtView recovery and completion credits absent from this model. The next protocol
needs sparse live keys bounded by monotone next, pool/fraction conservation,
matching nonwrapping native count and inherited release publication through
Relaxed RMW, with actual Acquire before typed recovery. No bytes law may be
assumed to bridge this mismatch. Exact IDs/old values1,2 are native observations,
not a generic concurrent theorem.

An intermediate positive archive (SHA prefix7ef78f00) was overwritten before root
audit and is unrecoverable. It is explicitly excluded from evidence. The current
immutable final archive is c707fc4f2e807fbeb3b3ad1635977528b86838cb52f22fa31b82a0f637a26994,
independently audited against current source. Negative snapshots remain available.
Future input changes require distinct evidence labels and must not overwrite
existing captures.

## D2026-10-08-AB — Post-increment overflow checks do not admit unbounded lastness

Pinned Rust Arc's own source says its post-fetch_add check is not100%
water-proof. Shipped Creusot Arc::clone assumes its functional value relation
and does not body prove its native refcount. Copying that contract is not
permission to assume bytes refcounts under the user policy.

The executed toy scheduler demonstrates an abstract unbounded-concurrency
wrap before pending abort checks run; a later normal-return clone can see0,
then a release sees1 while other owners remain. This is not a native OS
use-after-free demonstration. The current model has no justified physical bound
on pending clones. Count modulo word range therefore cannot justify lastness.
Do not retry the same inference or introduce an arbitrary bytes-specific bound.
Reopen only with a justified execution-model bound, or a changed native algorithm.

A guarded Relaxed CAS/fetch_update is the concrete small-code alternative:
check the actual expected count before a successful increment; only success
commits one affine registration transition, and failed/retried attempts do not
create tickets. Retain predecessor release-sequence publication without Acquire
or current-view publication; retain final Release decrement and actual Acquire.
The proposal has explicit retry/starvation/hot-path/abort-timing differences.
It was applied in D2026-10-08-AC below. All three native increment paths must be audited,
not only selected Shared. Required generic adapter and controls are recorded in
refcount-overflow-review-2026-10-08. This source review/toy experiment is not
a formal protocol proof or complete original API verification.

## D2026-10-08-AC — Guard before the actual increment

The user authorized implementing AB's three-step plan. The changed premise is
now native code: all three increment paths call `ref_count_ops::increment`,
which uses Relaxed/Relaxed fetch_update with production `next_ref_count`.
Its callback is pure arithmetic; refusal makes no store by this operation.
Successful updates are bounded by MAX_REF_COUNT+1, strictly below usize::MAX.
Owned decrement's debug bound now admits that already-allowed last success.
Public API, native representation, Release decrements and final Acquire remain.

The exact production arithmetic and a once-only model-atomic CAS loop are body
proved in the new original-public-shared gate. The loop follows upstream
logically_atomic_faa's retained affine callback across CAS failures. That
upstream SC example does not justify Relaxed synchronization. Generic native
operation/invariant alignment and precise Relaxed release-sequence carry are
separate trusted boundaries; no bytes ownership law is thereby assumed.
Relaxed carry must neither Acquire the predecessor nor publish the caller's
current view. Removal requires supported operation-bound weak atomic/invariant
contracts preserving these interfaces, not changing bytes protocol clients.

Default native tests, no-default library build, portable atomic boundary tests
and existing Loom clone models pass. The retained atomic microbenchmark shows
additional cost; it is not a public-API throughput measurement and makes no
fairness/termination claim. Evidence is in refcount-guard-performance-2026-10-08.

Historical C44 source bridge still targets the previous post-fetch_add code;
do not count it as proof of this changed native increment. New sparse
registration/retirement and actual public Bytes/Vtable integration remain
separate body-proof obligations until their exact-source evidence is audited.
The post-increment counterexample and earlier failed routes remain frozen.


## D2026-10-08-AD — Separate sparse protocol safety from closed-client completion

The guarded sparse lifecycle default core body proves quota-free registration,
count = live-map cardinality, arbitrary-order retirement, live-peer nonlastness,
and conditional typed recovery only after Release plus actual Acquire. Its
current v24 capture proves 37 files / 207 actual prover leaves / zero nulls.
No bytes ownership, refcount, lastness or recovery law is trusted.

A distinguishing multi-clone/sparse-hole driver was translated and tried.
Its exact failed VCs and source remain archived. The event boundary quantifies
over every protocol state with the same immutable public descriptor; this
permits additional live IDs. A local caller consequently cannot establish that
its handles exhaust all issued tickets. Astra reviewed this information loss.
Do not retry unconditional final recovery with assertions, exact expected IDs,
a fixed quota, a trusted no-unknown-owner law, or a mutable parent registry.
Reopen only with a body-proved complete issuance ledger, or a sound generic
history/closure capability compatible with actual shared Clone and backed by
an explicit native interpretation and standard-library analogue. Conditional
final-branch safety and native allocator tests do not establish completion.

The selected actual-source public integration also demonstrates a separate
trait-domain obstruction: From<Vec> is unrestricted, so requiring len<capacity
on its implementation violates refinement (Vec::new is a counterexample).
Keep that failed refinement and the earlier mutual Fn-spec cycle immutable.
Do not hide the failure by conditional postconditions or alter the native
len==capacity allocation optimization solely to pass the proof. Reopen with
a branch-aware Bytes model and strong contracts for Shared plus promotable,
static and owned alternatives. Selected Shared constructor/Clone/AsRef/cleanup
bodies are reusable components, not whole From, whole crate, or admission.

Generic native-field event alignment, weak publication, pointer/region access,
and closed Vtable ghost-erasure correspondence remain explicit TCB with strong
interfaces and removal paths. Their adequacy is not proved by these clients.

Next bounded work should introduce a representation-sum proof sidecar and a
uniform content/view contract, then prove the existing len==capacity
promotable/empty From branch. Preserve the existing Shared lifecycle interface
and its strong constructor/Clone/read contracts. Do not migrate broad APIs
before the branch-aware admission and complete-history capability are validated.


## D2026-10-09-AE — Full-domain constructor representation sum

Changed premise from AD's Shared-only From refinement: a proof-only sum carries
the actual Shared, raw promotable, or static representation and one unconditional
content/view contract. From<Vec<u8>> has no input precondition in this gate.
The production len==capacity optimization and both pointer-parity branches
remain intact. This is a bounded constructor/read experiment; it does not
assert promotable Clone, issuance completeness, or architecture admission.

The Shared constructor consumes the existing strong sparse protocol unchanged.
The equal-capacity branch calls the actual Vec::into_boxed_slice, whose pinned
Std contract preserves contents, before the actual Box::into_raw operation.
B1-BOX in ownership_proof/raw_vec.rs supplies only the missing generic physical
interpretation: exact consumed allocation, capacity equal to slice length, sealed
affine Recovery and full initialized PhysicalRegion. Vtable choice, tag binding
and representation validity remain body-proof obligations.

Pinned Std analogue: std/ptr.rs Perm::from_box (lines557–565 in canonical
public final6's private Std inputs) is trusted, check(terminates), erased to
Box::into_raw, and returns a pointer paired with an owning Perm. Box::into_raw
itself has no usable ownership postcondition; Box::from_raw requires false.
B1-BOX may be replaced by supported boxed raw-parts contracts preserving its
resource interface. Its allocator/provenance/layout/initialization adequacy
remains an explicit physical TCB assumption, independent of caller proof.

Astra's pre-proof review caught and removed a pure Box-to-pointer observer.
The exact pinned Creusot backend/ty.rs lowers Box<T> to T, so equal logical
contents would otherwise imply equal native addresses. The revised boundary
pairs its returned native pointer with a fresh affine namespace instead.
Namespace freshness never implies fresh numerical addresses. Empty boxes have
no allocated block; capacity-zero physical cleanup must be a no-op. Preserve
this counterexample and the distinction from opaque Vec's pointer observer.

Acceptance: full From trait refinement is included; arbitrary-input and
branch-specific representative callers prove exact reads, including empty
Vec and empty spare-capacity inputs. Both parity-dependent constructors are
proved symbolically. Missing ownership, wrong table/tag or wrong contents must
be rejected by semantic controls. Capture exact source/tasks/configuration and
private Std before reporting success. Positive constructor proof is 21 files / 153 prover leaves / zero nulls,
including both unrestricted From refinements. Semantic controls and independent
archive audit are recorded in the probe README; full architecture admission
remains blocked. Use the existing two/three-failure restructuring budget.

## D2026-10-09-AF — Closed-client capability review

Read-only review of the actual pinned private Std and upstream examples supplies
no changed premise for reopening AD's complete-client theorem. Auth fragments
prove inclusion, not exhaustive issuance. LifetimeToken::end needs the full
fraction; FullBorrow/EndBorrow do not derive that fraction from opaque events.
Atomic::into_inner requires full Perm and bounds all timestamps, rather than
recovering exclusive access from the caller's handle list. logically_atomic_faa
retains one callback over CAS retries, not all intervening calls. Arc::clone
assumes functional equality and does not prove native reclamation history.

A generic closure/history boundary needs a concrete scope/effect or accessor
discipline connecting every completed call to the complete event log, compatible
with actual Clone(&self). A log lower bound or trusted close() claiming no unseen
owners is insufficient. Before reopening, require a native interpretation, a
Std/tool analogue or separately justified capability, and distinguishing escaped
clone, unfinished callback and forgotten live-handle controls. No such primitive
is implemented or admitted by this review; AD remains frozen. Automatic Drop
effect lowering and the other full-target obligations also remain open.

## D2026-10-09-AG — External normal-edge destructor effect elaboration

The user authorized continued Astra-guided work until complete verification.
Astra recommends a small generic normal-return Drop elaboration gate before
complete issuance tracking. This is a changed premise from stock Drop-to-Goto,
not a rerun of the frozen automatic-drop source with a stronger summary alone.
No Creusot source change is required: exact pinned native MIR after ElaborateDrops
is mapped to a generated proof-only shadow before borrow/liveness analysis.

The shadow keeps guard fields/lifetimes, removes its Drop impl, copies the exact
native destructor body into a normal &mut helper, and inserts one call at each
supported normal Drop edge. Removing the trait implementation prevents double
native execution in the shadow. An independent checker must establish source,
destructor, place/type/target/order and exact-once correspondence; hashes alone
are insufficient. First scope: fresh local guards, straight-line normal return,
known monomorphic destructors, fields without independent drop glue, checked
scalar operations/callees. Unknown calls, escapes, raw/unsafe access, threads,
forgotten or unsupported moves/drop flags and additional field glue are rejected.
Unwind edges are captured and explicitly outside the normal-return theorem.

This source/MIR/shadow interpretation is NEW generic tool TCB; it is not a trusted
destructor postcondition, not a bytes last-owner law, and not whole-crate admission.
Actual destructor effects and caller postconditions remain body-proved. Std's
mem::drop only ensures resolve(t), so it is not an existing effect analogue.
Std's erased Ghost-channel thread contracts and the accepted closed-vtable
checked shim are interface precedents, not proofs of this elaborator's adequacy.
Removal condition: supported native MIR Drop/liveness effect translation that
preserves the same body/caller contracts and passes the retained counterexamples.

Initial nested-borrow diagnostic: updating *(^guard).0 alone does not preserve
the old field's loan; caller propagation fails despite the helper proving. The
body-proved frame `^(guard.0) == ^((^guard).0)` connects the original nested loan
future to the returned one. Together the exact true/toggle value effect and frame
prove the shadow helper and caller. An unconditional future value assertion is
rejected because a caller may legally modify the guard after the helper returns.

Acceptance requires SetTrue and non-idempotent Toggle normal-scope witnesses;
omitted/duplicate/wrong-target/early Drop, mutated body and false summary controls;
checker rejection of unsupported native effects. Preserve existing stock failures.
Only after this gate is audited should the same correspondence discipline be
extended to a fresh actual Bytes client and an erased generic ScopeCursor. AD's
lost issuance information is not solved by this preliminary generic Drop result.
Positive source/MIR/shadow checks cover all three supported scopes. The restored
gate proves7files/14actualproverleaves/0nulls (historicalv1was7/12/0); omission,
duplicate, wrong-target, early-order and false-summary controls reject3/2/2/1/5
semantic leaves respectively. Twenty-two checker controls reject unknown effects,
hidden trust and extra normal/cleanup successor statements. Exact set_true
signature/attributes and shadow-wide trust/unsupported annotation rejection were
added following Astra review. Accepted only in this bounded normal-return scope,
with independent matching-source archive audit; full Bytes admission remains open.

## D2026-10-09-AH — Scoped observation cursor distinguishing experiment

Astra recommends reopening AD only with an erased affine scope cursor tied to
all completed events in a fresh closed client. The changed premise is an explicit
generic history/closure tool boundary, not an assertion that the caller owns all
handles. `ScopedProtocol::Observation` contains only resource-free logical data;
the sparse State projection is `(live_map, next_id)`. The cursor never returns a
State, Perm, lifetime token, recovery payload, AtView or physical authority. The
same initialized State is consumed once when binding the invariant and seeding
the cursor. Event contracts retain the native model/field, weak ordering,
Committer, protocol and callback obligations and add pre/post observation
alignment. Bytes-specific map updates and final recovery remain body proved.

The bounded experiment is `probes/scoped-issuance-cursor-2026-10-09`. First prove
fresh initialization, shared-source registration, sparse retirement and complete
final Acquire/recovery in the existing protocol, without quotas or guessed IDs.
An independent closed-source checker must reject untracked events, escaped or
forgotten handles, unknown/unfinished callbacks, threads and raw aliases. This
protocol prerequisite is not actual Bytes admission. Actual-source follow-up
must independently check constructor extra erased result and Clone/cleanup extra
erased argument correspondence: native `From` returns Bytes and native Clone
still takes `&self`; cursor stays in the shadow caller, never inside Bytes.

This introduces generic closed-scope/history correspondence TCB. Std erased
Ghost channels and operation-bound invariant callbacks are interface precedents,
not a shipped complete-history theorem. Adequacy requires the closed-call and
noninterference interpretation plus source/effect checker; no trusted bytes
last-owner/destructor theorem or arbitrary global concurrent closure is allowed.
Removal path: supported effect/history translation preserving these interfaces
and all retained controls. AD/AF counterexamples remain frozen and preserved.
Before any positive claim, capture exact tasks/sources/tool pins and distinguish
checker rejection, type rejection and failed semantic VC. Apply the existing
failure/restructuring budget; no unchanged retry after structural obstruction.

AH protocol result: the final restored gate proves 38 files / 219
actual prover leaves / zero nulls, without excluded targets. Singleton, fresh
insert/removal, old native count/cardinality and full final recovery are body
proved. Fifteen driver/interface structural controls pass. False summary,
missing Acquire and an extra unretired owner each fail one semantic leaf; pure
observation extraction as State or Perm is an E0308 type rejection. Native
protocol client passes separately. Actual Bytes ghost-channel source mapping
remain separate prerequisites; independent final2 archive audit passes all 762
regular members, 38 tasks and 110 private Std inputs; do not turn
this closed protocol witness into arbitrary concurrent/full-crate admission.

## D2026-10-09-AI — Actual Shared closed-client ghost channel

After independently audited/published AH (4d809085), Astra recommends applying
that cursor to one actual Shared Bytes normal-return client. Native From and
Clone(&self) remain unchanged. The checked constructor shadow returns Bytes plus
an erased cursor, consuming its single initialized State at a distinct scoped
field binding. Only the invariant descriptor remains inside the proof sidecar.
Clone and cleanup thread the separate cursor through the existing three-native-
argument vtable erasure interface, bundling extra Ghost inputs; no new native
callback result ABI is introduced. Existing unscoped adapters cannot accept the
new scoped descriptor. The client alone requires len<capacity; the unrestricted
From refinements and representation-sum constructor gate remain untouched.

A cleanup output channel starts None and returns KeptAlive or Reclaimed. A genuine
deallocation receipt cannot be a ghost flag that survives omission of a free.
Copied generic physical/typed-free boundaries return opaque affine receipts with
precise allocation/pointer/layout identification; typed zero-sized disposal is
distinct from a real allocation free. Their primitive adequacy remains physical
TCB. Body-proved free_recovered calls both boundaries and returns their pair;
Reclaimed contains that pair. No bytes-specific last-owner, completion or
recovery law is trusted. Release, actual Acquire, full token recovery and both
frees remain body/source obligations.

Experiment `probes/original-shared-scoped-client-2026-10-09` must prove content,
exhaustive returned-ticket updates, sparse parent-early retirement and exactly
one reclamation receipt. Independently reconstruct actual constructor/trait/
vtable targets, cursor/result channels, native operation order and free/drop
suppression correspondence. Reject escape, forgotten/unretired owners, unknown
callbacks, threads/raw aliases and unscoped events. Retain wrong cursor, omitted
event/Acquire/free and duplicate cleanup controls, distinguishing typing,
correspondence and VC failures. Actual automatic Bytes Drop is outside this
increment and must later connect to AG's native MIR effect elaboration. No broad
API expansion before this actual-source admission succeeds. Preserve failed
artifacts and the existing two/three-failure restructuring budget.

AI result: restored canonical replay proves 75 files / 433 actual prover leaves / zero nulls, no excluded targets/features. Exact native public client passes five inputs.
The independently reconstructed source/helper/native-harness correspondence and
72 structural controls pass, including exact selected constructor branch,
Shared destructor/no-drop field profile, full native/helper operation bodies,
wrong/omitted cursor and effects, extra owner/events, hidden trusted false summary
and false driver premise. Missing Acquire, each missing free, and an unretired
owner each reject one semantic leaf; exact failed captures and normalized tasks
are retained. The buffer-free control normalizes its false receipt-construction
precondition to not inv_Atomic_usize; record rejection sensitivity rather than a
separate buffer-receipt theorem. Generic typed receipt means raw storage ownership
consumption, not execution of T::drop. Native helper signatures and production
source remain unchanged; historical narrowed From refinement is not used here.
Canonical evidence/audit in the probe directory records immutable source/task/
private-Std/config/native-log inputs. Automatic Drop and the original full target
remain open. Next ask Astra and continue; do not relabel this as full admission.

## D2026-10-09-AJ — Actual normal-edge terminal Bytes Drop

After AI is audited and published at 9313e190, Astra directs a new exact two-owner
normal-return automatic-Drop witness. The native client constructs first, clones
second, copies second's AsRef view into an owned Vec and returns it; it contains
no explicit cleanup/drop call. Pinned after-ElaborateDrops MIR has terminal
normal edges dropping second then first, with unwind successors separately
recorded. Production Bytes::drop remains unchanged.

Changed premise for the frozen stock Drop-to-Goto route: AG's external normal
edge elaboration now joins AI's body-proved actual Shared cleanup/cursor/free
receipts. Before Creusot borrow/liveness, a proof-only artifact with no Bytes Drop
impl consumes each certified terminal place through an ordinary body-proved
bytes_terminal_drop(Bytes, Ghost<&mut Cursor>, Ghost<&mut Option<Completion>>).
It can reuse AI's checked implementation/contract but cannot be trusted or demand
restoration of a live &mut Bytes invariant after freeing its allocation. Return
expression is evaluated first, then effects in native MIR order, then the saved
output is returned. Prove contents, second KeptAlive, first valid Reclaimed and
empty cursor map; retain per-owner removal facts to distinguish swapped places.

This consuming shadow requires a NEW explicit generic terminal-place premise:
selected destructor/target closure must not observe or escape the address of
Bytes or its data field. Only stored pointer/length/vtable values and get_mut
value copying are relevant. Check the actual callback closure, normal core
AtomicMut alias/body, and native MIR projection/calls. No independent native
Bytes field-drop glue is allowed; keep AI's Shared field/destructor checks.
Neither this premise nor a helper named cleanup establishes arbitrary destructor
move equivalence. Source/MIR/shadow/loan adequacy remains generic tool TCB with a
removal path through native destructor-effect support; bytes laws stay body proved.

Probe original-shared-automatic-drop-2026-10-09 joins immutable native client,
actual production destructor/target MIR, generated helper/client, exact normal
places/successors/order/count and recorded excluded unwind edges. Controls omit
either effect, duplicate/swapped/wrong place, early effect before last read,
changed/missing native Drop edge, changed destructor/vtable, field glue,
receiver/data-address observation/escape and hidden calls. Keep missing Acquire
and both missing-free controls. Native execution is corroboration only. No
production source/API change, native cursor or whole-crate admission. Start with
this straight-line normal path; apply the existing two/three-failure redesign
budget, archive exact failed inputs and consult Astra after the audited increment.

AJ result: canonical replay proves all77 files/445 actual prover leaves/zero
nulls, checker passes explicit active source/mapping, and independent structural
replay rejects all62 controls. Six semantic development defects leave one null
each; three ordinary affine/loan controls reject before VC generation. Native
five-case execution and field-profile compile assertion pass. Canonical archive
SHA256 a550b2564160823fe425157ed45dba90471577d3124e6677e7d101b2bacb55c9
has1717 hashed members. Support-module literal-path checking was strengthened
after independent audit found a concrete acceptance gap, with ten new mutations.
No production API/source change or whole-crate admission follows.

## D2026-10-09-AK — All-domain boxed normal automatic Drop

After audited/published AJ f66edbd1, Astra directs unrestricted actual
From<Box<[u8]>> -> AsRef/to_vec -> normal-return automatic Bytes Drop, empty
Static and both unpromoted PromotableRaw branches. Native witness has no input
or pointer parity preconditions and no explicit Drop. Reuse AE actual constructor
ownership and AJ terminal-place interpretation, preserving return evaluation
before the native Drop. Shared and original all-domain constructor gates stay
unchanged. Proof must consume actual Recovery/full PhysicalRegion and return
NoAllocation for static, exact namespace/pointer/size/alignment FreeReceipt for
nonempty. Retain native free_boxed_slice offset_from+len computation: capacity
and branch unreachability are body proved, not Bytes-specific trusted laws.
If needed, only generic tag provenance roundtrip/equal-pointer distance boundaries
may be added, with exact native interpretation/Std analogue/removal path. Source
checker records original ARC branch and a proved unreachable no-promotion branch;
readonly binding does not extend through Clone/promotion. Normal-only terminal
receiver/data-address non-observation and no field-drop glue remain required.
Controls cover missing Drop/free, duplicate free, missing tag removal, wrong
pointer/layout, raw-as-static/ARC, early Drop, native target drift. Retain both
parities symbolically even if native allocation observes only one. After two
same failures reassess boundary; no third assertion-tuning repetition. Admission
is only all-input boxed constructor/read/unpromoted normal automatic Drop.

AK result: canonical no-feature/restored replay proves all97 files/510 actual
prover leaves/zero nulls, explicit current client+mapping correspondence passes,
and all115 independent structural controls reject. Seven semantic controls
reject with recorded null counts1/1/1/2/6/1/1; duplicate free/Drop and early Drop
reject before VC generation (E0382/E0382/E0505). Native11 selected MIR/five-case
execution and field-profile compile check pass. Canonical SHA256
04aa0ecfde6d47af4f9b250c39d6fbb74556958752a674d8552688612217acb5
has2512 hashed members. Two body failures prompted interface restructuring to
one generic tag symbol and body-proved bitvector lemmas, not a Bytes axiom.
Astra reviewed default ptr_map only as an assumed generic exposed-provenance
roundtrip; pinned native documentation context and removal path are explicit.
Independent review exposed absolute harness path dependence, fixed with relative
paths and regenerated native evidence. Imported sibling checker/support trees
are captured and the archive restore layout is documented. No production/API
change or full-crate admission follows. Consult Astra after audited publication.

## D2026-10-09-AL — First promotable Clone and closed phase lifecycle

After audited/published AK9fcd2954, Astra directs a nonempty-Box, both-parity,
single-threaded first promotion witness: construct original, clone child,
child.cleanup(), read/copy original, original.cleanup(). Scope is explicit
cleanup, not promotion automatic Drop or concurrent CAS losers. The new affine
PromotionScope is separate from observation-only ScopeCursor: Raw directly owns
constructor Recovery/fullPhysicalRegion and unconsumed root AtomicPtr Perm;
Shared owns root Shared core/cursor/updated root permission; Finished marks actual
final cleanup. Bytes stores an immutable root descriptor; refinement is on the
Bytes/scope pair, never a standalone root API using a weak Bytes invariant.
Source checker rejects root/scope escape, unregistered writer/access, stale
readonly binding and unmatched borrowed root access.

First prove a generic actual-core-AtomicPtr/ownedPerm/Committer boundary with
native Acquire load and strong CAS(AcqRel,Acquire), exact load==expected success
and !=expected failure callbacks. Initial singleton history and closed owned
permission body-prove no failure; no Bytes-specific success axiom. Keep the
native failure branch and its proved-unreachable correspondence. Separately
body-prove State::initialize_pair from the actual singleton2 counter permission,
payload/fullLifetime to State/two distinct actual tickets, exact ledger and
fraction/pool/cardinality preservation. Never initialize1+invented increment.
Refactor only the new probe ownership core to separate root updated pointer
permission from child immutable readonly binding. Both drop adapters reuse the
same body-proved release/Acquire recovery/two physical frees; original retains
promotable vtable and reaches ARC branch after publication. Only generic field/
CAS/ghost-channel correspondence adds TCB; all phase/resource/count/effect laws
remain body proved. Controls: count1, missing ticket, double raw transfer, wrong
expected CAS, stale readonly history, root raw-drop after promotion, missing
Acquire/free, root access withoutscope, extra writer. Two same VC failures trigger
scope/ownership redesign, not logical State getter, trusted CAS success or weak
conditional content post. Audit/publish full bounded AL then ask Astra again.

AL development prerequisites on 2026-10-09: actual count-2 pair initialization
passed diagnostic 76/450/0; owned strong AtomicPtr CAS passed 79/482/0; its
post-store visible/latest history consequences and terminal owned-permission
get_mut projection passed 83/507/0. All targets were translated and included.
Correspondence status is deliberately not_run (2), so none is AL lifecycle
admission. Root native AsRef contains no atomic operation; post-CAS Acquire
helpers may be used only at actual Acquire sites. Terminal get_mut consumes
the unique permission and returns only an exact history value/maximal timestamp
projection of Std into_inner, without manufacturing a SyncView/Acquire event.
The root retains the two-entry history, never a read-only re-sealing.
Archives are immutable under original-promotable-first-clone-2026-10-09/evidence;
independent pair/CAS prerequisite reports record the current bounded scope.


AL canonical gate now passes112/808/0 with all112 targets, no features/exclusions,
and source/native-shadow correspondence status0. Main controls25/25 and native
MIR/source controls24/24 reject. The checker fail-closes on literal include
redirects, unreviewed contracts/trust/effects/getters, build/extractor/Cargo route
changes, and requires actual compiled record bytes plus exact Cargo receipts.
Seven semantic controls leave1/1/2/1/2/2/2 real null leaves; omission features also
exercise imported Shared functions. Five frontend snapshots distinguish initial
E0507, owned-permission/physical/child E0382 and read-across-cleanup E0505/E0502.
Missing free negative nulls arise at resource-resolution/false-conjure premises,
not an independent receipt-absence theorem. All failed tasks/input snapshots
are immutable and hash-bound. Canonical archive53a500c22c725a259f20377e4da56c9e82bb4fc695400397c55940bf28e8eed1
captures4428 members including privateStd110 and exact compiled-record/build
receipts. Independent audit gates publication. This bounded AL completion does
not admit the full original architecture. After publication ask Astra exactly
「次何するのがいい？」 and execute the next concrete recommendation.


## D2026-10-09-AM — First promotion, lexical and return automatic Drop

After audited/published AL361c7cd2, asked Astra「次何するのがいい？」.
Astra directs actual native nonempty-Box first promotion with inner-scope child
Drop, root read/to_vec, return evaluation and terminal root Drop. Changed
premise is AL's closed actual Raw-to-Shared ownership transition combined with
AJ/AG's normal terminal-MIR elaboration, not an unchanged public Drop attempt.
Freeze AL ownership helpers/contracts byte-for-byte; add two nontrusted consuming
child/root terminal adapters that forward existing cleanup with erased scope and
completion loans. Preserve child root/pointer frame and KeptAlive completion;
client establishes actual singleton ledger before final root adapter, paired
receipts and real zero-ledger completion. Generate calls only at certified native
normal Drop edges; never execute native Bytes::drop as well in proof artifact.
Check actual places/normal successors and inner Drop before root last read;
return evaluation precedes root Drop. Exact callback mapping, no receiver/data
field address observation/escape and no independent field-drop glue remain
explicit generic terminal/compiler/erasure TCB. Root consumes updated owned
history, never reseals readonly. Controls omit either effect, swap adapters/
places, duplicate consumption, early root drop across read, mutate lexical MIR
edge, introduce receiver escape/field glue, omit Acquire or either free.
All-target feature-free restored gate, independent source/MIR/generated/compiled
record correspondence, native witness and archive-only audit gate publication.
Unwind remains excluded. Two same semantic interface failures trigger redesign,
not weakened validity/receipts/content or new Bytes destructor trust. Full
original architecture remains NOT ADMITTED; after AM publication consult again.


AM final canonical gate passes115/824/0, complete115 targets, no defect features,
no excluded targets, full source/native/generated/actualcompiled correspondence0.
Two body-forward wrappers preserve AL contracts and sourcebyteexact core, while
actual native lexical/return Drop places/order are independently checked. Native
field-profile compile assertions and anchoredAL361c7cd production manifest bind
61source files+Cargo/lock and exclude hidden record/import/drop-glue changes.
Review found concrete accepted proof_assert macro shadowing and production
record/import redirects; complete client-token and native-baseline input gates
now reject the exact defects with refreshed receipts. Main45/45 and native32/32
controls pass. Six semantic controls produce1/1/3/2/2/2 real nulls; four type
snapshots give threeE0382 and E0505/E0502 with noComa. Exact nulls are retained;
omitted effects yield resource/content resolution sensitivity, and missing free
GhostConjure paths are not direct native receipt-absence theorems. Feature
omissions also fail retained public_shared bodies; no doublecountedAMcoverage.
Canonicalarchive7db16b230e56cd75c9cffb34620a2a4593af23ff68bbc03ed8d5913050b4424e
has6281 members including actualcompiledrecords/buildreceipts/privateStd110 and
all proof/source/config inputs. Independentarchive audit gates publication.
Normal completion only; allocator/to_vec unwind, abort, generaltermination,
concurrent losers, escaping/generalClone and otherAPIs/configurations remain
open. Full original architecture is NOT ADMITTED. Asked Astra again exactly
「次何するのがいい？」 after canonical work; nextimplementation followspublication.

## D2026-10-09-AN — Reclone the already-promoted original handle

After AM publication at ae7b7e5124c064d2e686ad1a9fb20d52e5c2b795,
Astra directs a separate original-promotable-reclone-2026-10-09 probe. AL and
AM remain unchanged. The selected nonempty Box client creates the original,
first clones it through Raw-to-Shared promotion, then clones the same original
through the native promotable callback's existing-ARC branch. Lexical scope
exit drops the second then first child, reads the surviving original, saves
the return Vec, and normally drops the original.

The changed premise is a paired Shared phase retaining the root's actual
AtomicPtr owned history and post-publication view. A body-proved Acquire helper
takes an erased Snapshot expected value; no Ghost pointer is converted to a
native argument. Its visible-history precondition rules out reading the old
tagged word. The subsequent actual Relaxed increment uses the existing scoped
field event and body-proved State::on_register. The contract exports fresh
returned-ticket insertion into the complete ledger, without fixed IDs or a
ticket quota. The second clone performs no control allocation or pointer CAS.
Only its new child pointer field receives a readonly binding. The original
retains the promotable vtable and its owned old/new pointer history.

The root's current view may advance at Acquire. A new Boolean frame preserves
the owned history and expresses monotone current; it does not reuse the old
same_pointer_owner predicate requiring current equality, or return a resource.
Root core, descriptor, strong content/physical invariant and cursor identities
remain preserved. The two Shared-phase callback registrations are new instances
of the existing source-checked generic erasure boundary; no bytes ownership,
successful increment or last-owner law becomes trusted.

Required controls distinguish raw-branch misuse, discarded publication view,
missing registration/store or ledger insertion, stale readonly resealing,
omitted child retirement, duplicate/early terminal consumption, final Acquire
and both frees. The main checker must close complete source surfaces, literal
routes and compiled input receipts, including the reviewed production input
manifest established in AM. Normal completion only; overflow abort, unwind,
concurrent promotion losers, arbitrary concurrent closure and the rest of the
API remain outside this increment. Two failures of the same semantic interface
require reassessment, not weakened contracts. Full original architecture is
NOT ADMITTED; no proof claim is made by this pre-experiment decision.


AN restored canonical gate passes120/914/0 with complete120 target inventory,
no features/exclusions/diagnostic and independent correspondence0. Initial
body failure120/988/2 exposed missing child-cursor model identity export;
adding body-proved child.accepts(finalcursor) fixed it without weaker posts or
new trust. Wrong branch/reset view/omitted registration/three omitted Drops/
swapped child identity/missing Acquire/two missing frees produce real nulls
2/4/2/1/1/1/1/2/2/2; paired retained Shared targets are not independent defects.
Five compiler controls give threeE0382, E0505/E0502 and owned extractionE0507.
All inputs/exact tasks are archived. Main49/native52 controls reject, including
external pointer-event false contracts and macro override with refreshed hashes.
Ten support imports and reviewed checker source digests are pinned;
actual generated record, fingerprint, output and root-output bytes are captured.
Frontendv1 cached-status metadata flaw inherited fromAM is preserved/disclosed;
newv2 archives exclude actual.why3find caches and have no stale solver artifacts.
Canonicalarchive7181ae4c07997888e1af1dfad41fdb109a489b4ca101e1dd972c9b6d4d13843e
has875 members inclprivateStd110; independent archive audit gates publication.
Normal-only closed3owner scope, explicit genericTCB and fullNOTADMITTED remain.
Asked Astra again exactly「次何するのがいい？」 after canonical completion;
nextimplementation follows independent audit and publication, never a silent
reduction of original API/configuration/concurrency/unwind obligations.

## D2026-10-09-AO — Original retires before its surviving child

After AN publication at 5f3d20a3005b727ace827685d0601d791d180f78,
Astra directs original-promotable-surviving-child-2026-10-09. AN's complete
positive source is the byte-exact inherited prefix; AL/AM/AN remain unchanged.
The nonempty Box native client moves its first clone out of an inner scope,
normally drops the original there, reads the surviving child, saves the return
Vec, then normally drops the child. This changes recovery ownership, rather
than merely adding another bounded owner.

The root terminal adapter consumes PromotionScope, including its updated owned
pointer history and root core. Its erased output contains only the affine
ScopeCursor in DetachedScope, plus a nonfinal completion. No root ticket,
physical recovery, pointer Perm or State is retained in that detached object.
Existing body-proved State::on_release publishes the original ticket's sealed
recovery into private protocol State; the final child's real release and
Acquire obtain Pending and recover it. The surviving child reads through its
own ticket and physical lease. No resource-returning logical getter or new
Bytes-specific trusted recovery/destructor law is introduced.

New callback registrations are exact instances of the existing generic erased
vtable-call boundary. Native terminal place/normal-edge mapping, address
nonobservation, absence of independent field drop glue, source/compiled-record
identity and closed outer-scope completeness remain explicit tool TCB. The
survivor escapes an inner lexical scope only, not the checked outer client.
The new root callback consumes its input, so no destroyed root invariant is
restored. Existing strong payload/control receipts and map-removal contracts
remain required.

Controls omit root recovery publication, either Drop, cursor handoff, final
Acquire or either free; attempt duplicate root/scope consumption and early
child Drop across its read; and reject retaining root resources in DetachedScope
or changing actual normal edges/places/source routes. Publication requires a
restored feature-free all-target gate, complete source/MIR/compiled-input
correspondence, native witness and independently audited immutable evidence.
Two failures of the same semantic interface require reassessment. Unwind,
arbitrary concurrent/escaping lifetimes and the remaining original API/config
coverage remain open. Full original architecture is NOT ADMITTED; this decision
precedes the experiment and makes no proof claim.


AO restored canonical125/983/0 includes all125 targets with correspondence0,
no flags/features/exclusions/source controls. Seven actual defects yield nulls
2/1/1/5/2/2/2;15 exact archive-derived tasks are independently printed. Five
frontend defects reject threeE0382, live-borrowE0502/E0505 and wrong-adapterE0061.
Main31/native52 controls reject. Astra found inherited mutable-copy equality,
extraCargo routes and transitive import timing holes; fixed manifest anchoring,
exactCargo and pre-import checker pins reject10 independent attacks.
Canonical5cf0ffb4ae25dd3d8cb757677a5a43032e926c0eac4dbe3822a9c482206238b9
has968members/privateStd110/fouractualCargoinputs. Independent reconstructed
archive replay confirms both checkers/all31+52controls/production63/eighttools;
frontend5exclude caches/tasks. Allpreviousprobes/production remain unchanged.
Strong body ownership/recovery contracts and genericTCB stay explicit;
normal-only closed scope and fullNOTADMITTED persist. Asked Astra again
「次何するのがいい？」; recommendation AP changes premise to inductive
runtime-variable finite owners in actual Vec loops, not bounded unrolling.
Implementation follows this increment's audit and publication.

## D2026-10-09-AP — Runtime-variable finite sequential sharing

After AO publication at 48cf2a0e365dd00b20bf88b93bec8d75a7dde5ca,
Astra directs original-shared-finite-owners-2026-10-09. AO's complete positive
source is the immutable inherited prefix. The selected nonempty Box native
client retires its original owner, clones the surviving Shared-vtable child a
runtime number of times into an actual Vec<Bytes>, drains it by pop and lexical
peer Drop, reads the survivor, saves the return and finally drops the survivor.
No finite unrolling, chosen ticket IDs or owner quota replaces these loops.

The changed premise is a body-proved exact inventory linking every actual
vector element's ticket to the cursor map, plus inductive preservation across
native push/pop and normal terminal edges. Shared child Clone uses the actual
Relaxed pointer load and existing Relaxed refcount increment, not the
promotable original's Acquire path. It has a reusable strong contract exporting
fresh returned-ticket insertion and final-cursor acceptance. The source keeps
its immutable pointer binding; only a new child's own initialized field receives
its fresh readonly binding. No State/Perm/ticket getter, trusted completeness
law or trusted Bytes clone/lastness/destructor law is added.

Pinned private Std Vec push/pop sequence contracts and Resolve over actual
elements support the affine inventory proof. The external effect checker must
preserve the cyclic CFG, pop-Some payload move, one peer Drop per successful
iteration, loop backedges and saved-return/final-Drop ordering. At the actual
Vec destructor edge the proof must establish its contents are empty. The native
Vec destructor still deallocates container storage; generic Std/Rust empty
container destruction remains TCB and supplies no Bytes recovery receipt.
The two Bytes receipts concern the payload and Shared control allocation.

The theorem concerns normal returns. The existing guarded MAX_REF_COUNT
refusal/abort path and Vec allocation failure/unwind remain native and do not
establish normal completion; no arbitrary count quota is imposed. Creation
arithmetic uses made<count to prove increment cannot overflow. Neither total
successful execution for all counts nor unwind cleanup is claimed.

Controls omit registration, a peer effect, complete draining, final Acquire or
one free; attempt forget, duplicate/early consumption or invalid empty-container
cleanup; change a backedge, Some move/drop edge, clone callback or source/import
route. The exact inventory must be proved from body contracts, never assumed.
Two failures of the same semantic interface trigger recorded reassessment,
not weaker contracts or bounded unrolling. Publication requires all feature-free
targets, native loop/effect correspondence, decisive controls, exact compiled
input receipts and independent immutable archive audit. General concurrent or
escaping ownership, unwind and remaining APIs/configurations are still open.
Full original architecture remains NOT ADMITTED. This records the changed
premise before the experiment and makes no proof claim.

### D-AP audited outcome

The restored canonical run proves 130/1040/0, all targets included, with zero
structural leaves and correspondence status zero. Independent reconstruction
checks 1065 members and replays main35/native76 controls, preserving the four
actual Cargo artifacts and location-bound root-output. Nine semantic defects
produce 15 exact null sidecars; four frontend defects reject as recorded.
The Snapshot-only inventory lemma is body proved and extracts no owner resource.
No quota, weaker inventory or native/API change was introduced. Canonical SHA256
a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1. Full original architecture remains NOT ADMITTED.
Astra recommends nested public Range slices, including empty results, next.
