# bytes 1.11.1 architecture decisions

The target is the original public API and representation, with the retained
runtime fixes. The retired `bytes::verified` variant is historical evidence.
Full original-crate verification remains **NOT ADMITTED**.

The current bounded result is [AZ](probes/original-root-phase-clone-2026-10-09/README.md):
nonempty Box input, arbitrary finite capped cursor steps, common Raw/Shared
Clone, lexical peer retirement, exact suffix reads and final normal Root Drop.
Its complete positive has 191 functions / 2,207 prover leaves / zero null or
structural leaves. This does not establish arbitrary escaping/concurrent owners,
other representation paths, unwind, BytesMut, or complete API/configuration coverage.

[The archived decision record][history] retains milestone details, exact failure
identities and their original evidence references. Keep the actual counterexamples
and immutable evidence; deleting repetitive prose does not retire their constraints.

## Frozen methods and conditions for reopening

- **D01 — mutable physical access:** do not classify mutable B4 access as ghost
  when erasure can change runtime results. Reopen with a sound access encoding
  and erasure/initialization argument. Preserve `vtable-leaf-integration` evidence.
- **D02 — weak atomics:** SC invariant opening does not justify permission
  extraction at Release/Relaxed operations. Require a justified generic weak-memory
  resource rule; preserve missing-Acquire and release-sequence counterexamples in
  `verus-refcount`. Relabeling the primitive or adding numeric facts is insufficient.
- **D03 — open Buf/BufMut traits:** no trusted universal implementer laws.
  Require an enforceable implementer contract and proved refinement. A sealed
  local interface establishes only its stated subset (`actual-public-buf-default`).
- **D04 — bytes protocol trust:** ownership, refcount, last-owner and retirement
  laws remain body-proof obligations. Temporary local assumptions allowed by the
  current user policy must be explicit, strong and have a removal path; they are
  open obligations, never completed verification or generic primitives by renaming.
- **D05 — unchanged tool frontiers:** no wrapper-only retries of static/callback
  dispatch or shared-reference affine-ticket splitting. Require relevant supported
  tool behavior or a genuinely changed sound interface with source correspondence.
  Preserve `native-integration-frontier` and `bytes-observer-subset` failures.
- **D06 — composition:** independent models/configurations do not add up to an
  integrated proof. Require compatible representation, invariants, configuration,
  callers and source correspondence. Proof counts are not API coverage counts.
- **D07 — thread transport:** a non-Objective physical payload inside
  `Snapshot<(T,T)>` cannot be made transportable by wrappers or blanket Objective/
  Sync implementations. Require objective metadata body-related to affine AtView
  payloads or justified tool support (`architecture-thread-transport`).
- **D08 — floating-point conversion:** bit-pattern proofs do not prove native
  float materialization. Require generic bitcast contracts preserving relevant
  IEEE/NaN distinctions or supported translation. Preserve the to_bits frontend
  and contractless from_bits failures recorded in the archived D08 section.
- **D09 — Serde:** unconstrained Serializer/Deserializer/SeqAccess calls lack
  enforceable semantic contracts. Trusted ensures(true) cannot establish format,
  visitor or error behavior. Reopen with implementer contracts/refinements or
  tool support; preserve the archived `verified-serde-*` failures.
- **D10 — callback panic:** normal-return FnOnce contracts cannot frame resources
  across unwind. Require payload translation and a reviewed exceptional-effect
  interface, or another enforceable failure interface. Preserve
  `callback-panic-catch-unwind-ice-229`; native success is not exceptional proof.
- **D11 — fallible spawning:** unchanged raw scope/private Scope projection is
  frozen. Dropping an unstarted closure can mutate aliased state, so reject an
  unrestricted error frame. The reviewed Copy-slot Option-return boundary is
  restricted to its ordinary slot; it proves no unrelated-global frame, totality
  or exceptional cleanup. Preserve `builder-error-drop-alias-copy-negative-229`.
- **D12 — Hash/Formatter:** opaque generic Hasher/Formatter calls do not prove
  output effects. Require enforceable contracts or tool support. Explicit proved
  digest/hex operations are distinct interfaces, not Hash/Debug compatibility.
  Preserve `hash-only-unproved` and `debug-only-contractless` evidence.
- **D13 — totality:** arbitrary FnOnce/IteratorSpec and allocator/OS interfaces
  do not guarantee termination. Require checked callback/iterator bounds and
  separately justified allocator/scheduler progress. Preserve `termination-support`;
  normal-return cleanup and finite checked computation are separate claims.

## Ownership, overflow and trusted primitives

**AB/AC:** post-fetch_add abort checks plus a modulo count cannot establish
last-owner status under unbounded pending increments. The archived toy scheduler
is an abstract counterexample, not a native OS use-after-free demonstration.
Do not invent a bytes-specific concurrency bound. The changed native algorithm
uses guarded Relaxed fetch_update in all three increment paths: only successful
updates commit registration; refusal/retry creates no ticket. Preserve
`refcount-overflow-review-2026-10-08` and the actual guarded implementation.

The guard admits old_count <= MAX_REF_COUNT; attempting from MAX_REF_COUNT+1
aborts without increment. Common Clone contracts describe normal returns, not
guaranteed return at exhaustion. AZ's singleton loop starts Shared Clone at
count one and retires its lexical peer each iteration; it needs no lifetime
step quota. Old post-fetch_add source bridges do not prove the changed code.

**AD and closed history:** sparse protocol safety does not establish that a
caller's handles exhaust all issued tickets. Preserve the failed sparse-hole
driver. No asserted exact IDs, fixed quota, trusted no-unknown-owner law or mutable
parent registry repairs missing issuance information. Require a body-proved
complete issuance ledger or a sound generic history/closure capability with a
native interpretation. The scoped observation-cursor results are this changed
premise; they do not establish unrestricted escaping-owner completion.

Unrestricted From<Vec> cannot require len<capacity: Vec::new is a counterexample.
Keep branch-aware strong contracts for the actual representation alternatives;
do not change the native len==capacity optimization merely to satisfy refinement.
Tokenless immutable GhostShared access cannot mint reclaimable ownership.
Alternate-entry invariant reentrancy and the rejected open_at extension remain
unsound; preserve those counterexamples and Objective/AtView discipline.

The user permits reviewed **generic** trusted synchronization/resource primitives.
Consult shipped creusot-std contracts and relevant upstream examples when blocked
(e.g. Mutex/guard, AtomicInvariant and Committer); absence of a shipped adapter
alone is not a reason to stop. Record strong contracts, native interpretation,
assumptions and an interface-preserving replacement path. Caller proof does not
prove primitive adequacy. Relaxed release-sequence carry neither Acquires a
predecessor nor publishes the caller's current view; final recovery still needs
its actual Release/Acquire protocol. Bind ghost effects to the actual atomic event.

Normal-edge Drop elaboration and registered erased callbacks retain explicit
compiler/correspondence TCB. Sequential normal-return witnesses do not prove
unwind, concurrency, general destructor semantics or arbitrary Rust callgraphs.
Never infer missing membership, allocation identity or ownership facts from a
weaker summary: export and body-prove the required contract.

## Working budget and evidence

Default: strong contracts, implementation proof, next coverage gap. Reuse unchanged
audited controls. Add a targeted negative only for a changed trusted boundary,
ownership mechanism, correspondence check or concrete suspected omission; state
the gap and affected targets, and label diagnostic exclusions honestly.

Run the complete applicable positive proof and correspondence for a validated
proof increment; repeat only after a relevant change or unresolved failure.
Editorial cleanup does not require rerunning proofs. Preserve exact proof/source
identities, immutable closures and failures; distinguish frontend, VC and tool
failures. Negative rejection does not establish soundness.

To reopen a frozen method, record one changed premise, supporting evidence and
one bounded distinguishing question in the affected probe. New assertions,
timeouts, agents or cosmetic wrappers are not changed premises. Stop repeated
unchanged whole-crate frontend failures; review an interface after repeated
same-shaped failures instead of growing proof scaffolding. Existing authorized
work needs no extra approval or duplicate milestone updates across documents.

[history]: https://github.com/nyuichi/rust-crate-proofs/blob/3d48f465a227aef113e39ce16f6a1bd170443867/bytes/1.11.1/verification/ARCHITECTURE_DECISIONS.md
