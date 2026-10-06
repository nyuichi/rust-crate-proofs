# Architecture admission assessment — 2026-10-06 (Asia/Tokyo)

Status: full architecture NOT ADMITTED. This is a dependency assessment against
`ARCHITECTURE_DECISIONS.md`, not a proof-impossibility theorem. The selected complete target is now the modified variant authorized by route 1;
the original API is not claimed verified. No broad API expansion should start from this
assessment. T01 found a structural rejection; T02 now proves concrete thread transport (37 files). The full Bytes architecture is still not admitted.

## What the existing evidence decides

| Candidate | Disposition | Decisive evidence / missing connection |
|---|---|---|
| Stock Creusot, ordinary API and automatic Drop | BLOCKED | Actual MIR Drop lowers to Goto. The liveness analysis explicitly assumes no observable destructor effects. The old automatic-drop diagnostic already distinguishes explicit cleanup from scope exit. A destructor body contract alone does not insert its effect at callers. |
| Same, using the already-authorized explicit cleanup variant | NOT ADMITTED | Cleanup avoids the Drop translation obstruction, but default Clone(&self), mutable access, callback/static-pointer translation, and default initialized-authority integration remain unresolved. Changing cleanup alone is not a complete architecture. |
| Finite dispatch tag/enum alone | INSUFFICIENT | It can address selected indirect-call translation; it supplies no affine authority, thread safety, or destructor semantics. Generic from_owner and all conversions also need representation coverage. The successful nonpromotable truncate subset does not prove constructors or other callbacks. |
| Stock Verus with the unchanged native weak orderings | NOT ADMITTED | Numerical primitive contracts were actually tried. They do not establish a sound weak-memory physical invariant; the SC-rule shortcut admits Release-only permission extraction. Creusot physical resources also need a checked interpretation or a complete ownership reimplementation. |
| Stronger runtime atomics / stock SC interface | UNSELECTED, INSUFFICIENT ALONE | Would change implementation orderings and still require actual storage, dynamic ownership, dispatch, mutable access and destruction integration. It is not automatically authorized or a proven solution. |
| Explicit-context, consuming-cleanup implementation using stock Creusot thread tokens | CANDIDATE ONLY | Existing thread wrappers supply fresh per-child tokens. T01 rejected the old subjective snapshot. T02 proves transport of concrete physical payloads using an objective metadata projection and parent cleanup. Broader API/representation migration remains a separate decision. |

## Tool facts checked without replaying frozen failures

The committed `architecture-evidence/` snapshots record the pinned stock source:

- `creusot-drop-translation.rs`: MIR `Drop { target, .. }` becomes `Goto`.
- `creusot-drop-liveness.rs`: the analysis explicitly ignores destructor effects.
- `creusot-tokens-new-validator.rs`: token creation is confined to main, once,
  outside loops.
- `creusot-invariant-contracts.rs`: tokens are neither Send nor Sync; invariant
  opening requires the namespace token. Storing one in a transferable handle or
  freshly minting it inside Clone is not an available route.
- `creusot-thread-contracts.rs`: stock scope/spawn wrappers provide child tokens
  and join contracts. A missing generic spawn API is not the blocker.
- `creusot-marker-validator.rs`: local unsafe Send/Sync impls require trusted
  markers. Accepting that syntax does not discharge any thread-safety theorem.

A tool-required trusted marker is not intrinsically the same as trusting a
bytes-specific refcount theorem. If proposed later, it must remain an explicit
boundary backed by a separately proved actual resource-transfer/protocol
argument. Merely adding the annotation to clear an error is not completion.
No marker or protocol trust is introduced by this assessment.

Astra reviewed these facts and the proposed distinguishing experiment. Luna's
read-only evidence audit distinguishes semantic counterexamples, frontend
limitations, and user/scope policies; they are not all failed proof obligations.
`architecture-evidence/audit.json` hashes the decision evidence, tool snapshots,
existing binary pins and current source correspondence. No unchanged full-crate
proof run is counted as new progress.

## One permitted distinguishing experiment

Experiment T01: `probes/architecture-thread-transport/`.

Changed premise: the existing physical-retirement gate invokes two retirements
sequentially; this experiment crosses actual stock scope/spawn/join boundaries
with the actual PhysicalRegion and Recovery payloads and child Tokens. It asks
whether the present Send/Sync/Objective/AtView interfaces admit that transport
under the unchanged generic primitive TCB. It is a prerequisite experiment,
not the actual Bytes admission witness, arbitrary refcount proof, or proof of
original Clone/Drop. D01–D05 remain frozen.

Use one physical ownership model and the existing weak primitive contracts.
Do not rescue a failure with blanket unsafe Send/Sync, trusted bytes protocol,
SyncView construction from a snapshot, stronger runtime ordering, or a
sequential stand-in. Stop at the first structural obstruction after one interface
review. Save exact sources and diagnostics; frontend rejection is not a failed
VC. A positive native test alone is not proof. Record the outcome before
selecting further architecture work.

## Work order after this assessment

1. T01 is complete and stopped after one interface review. Preserve its rejection; do not rerun that representation unchanged.
2. Decide whether the final implementation retains ordinary interfaces or uses
   an explicit-context variant; explicit cleanup authorization alone does not
   decide all public API changes.
3. If retaining ordinary interfaces, identify and authorize the required generic
   verification-tool work before implementing bytes APIs. Do not promise that
   a small translator patch suffices for Drop, ghost mutable access or tokens.
4. If selecting a modified implementation, document its API/representation
   correspondence and run the complete admission witness before API expansion.
5. Only after admission, migrate existing proofs to that single model and close
   the public API/configuration inventory. Keep normal-return milestones distinct
   from final panic/unwind/allocator and feature coverage.

No remaining API is abandoned automatically. If the user retains all current
constraints and no candidate passes admission, the honest outcome is a blocked
complete target with preserved component proofs, not further disconnected gates.

## T01 result and interface review

Native tests: two test functions passed, including 84 actual threaded physical
cases and the existing primitive race test. Native success is not a proof of
resource transport. Pinned Creusot translation rejects both scoped spawns with
Rust E0277, before any VC/prover phase:

`State<RetiredPart>.expected: Snapshot<(RetiredPart, RetiredPart)>` contains
PhysicalRegion/Recovery carrying NotObjective. That state therefore fails the
`Send + Objective` condition for `AtomicInvariant<State>` to be Sync. A shared
reference to the retirement machine cannot cross the real spawn boundary in
the proof configuration. The previous sequential 36-file gate remains valid
within its scope; it never established this transport condition.

The pointer itself was deliberately retained in the parent; this failure is
not a raw-pointer Send assertion. Children were to retire actual affine payloads
and return recovered capabilities via join; parent cleanup would then consume
those capabilities. Neither this failed construction nor its native success
establishes actual Bytes last-thread deallocation or Clone.

One reviewed structural alternative is to keep the retired capabilities inside
the existing AtView<T> fields, but replace the full-subjective-payload expected
snapshot with an objective metadata projection (identities, capacities, bounds,
and any required initialized-value model). Its relationship to the real affine
payloads must be body-proved. This is not permission to mark PhysicalRegion,
Recovery, State, or their snapshots Objective/Sync by a new trusted rule.
No such repair was implemented or verified in T01. A future test must identify
that actual changed representation, its contracts and negative controls before
reopening. Merely moving/wrapping the same subjective snapshot is frozen.

The choice of final architecture remains pending: preserving ordinary public
interfaces may require verification-tool work beyond the prior scale limit;
an explicit-context variant may require public API changes beyond the already
authorized explicit cleanup. Neither change has been inferred from silence.

## Authorized distinguishing follow-up T02

T01 is committed and immutable at 53974279. The reviewed metadata projection
repair is a local proof-interface change and does not require selecting a new
public API or changing primitive TCB. T02 therefore evaluates it independently
of the pending final-architecture preference, in the new
`probes/architecture-thread-objective/` directory. This is a recorded reopening
of D07 on an actual representation change, not an unchanged retry.

Keep physical authority in affine AtView payloads; objective identity/bounds
metadata cannot mint it. Body-prove the projection and sufficient output
relations for conditional explicit cleanup after real scoped spawn/join.
Preserve native cases, source correspondence, old failures, no-new-trust checks,
and a threaded missing-Acquire control after a positive proof. Admission remains
NOT ADMITTED pending the actual Bytes witness and the independent default
Clone/Deref/Drop issues. T02 positive proof passes 37 files; the matching missing-Acquire control rejects retire at 11/12. The positive archive independently matches all 97 members, 37 Coma files and 37 proof JSON with zero null leaves.

T02 interface correction: the first two semantic runs left only two child-retire
preconditions in thread_roundtrip (39/41). The constructor contract omitted the
returned tickets' left/right roles, and accepts_payload was opaque outside its
module. A body-defined orientation accessor, a truthful constructor postcondition,
and crate-visible unfolding of the body-defined accepts_payload relation close
those obligations. Private-field Coma and visibility diagnostics are retained
separately; they are not failed mathematical obligations. The complete positive
run now proves 37 files. No new primitive or protocol trust was added.

The proved claim is concrete RetiredPart transport through actual scoped threads,
retirement, join and conditional parent cleanup. It does not assert eventual
exactly-one final observer (that remains a native assertion), arbitrary payloads,
actual Bytes Clone, automatic Drop, or architecture admission. Exactly-once and
liveness must be established in the later complete lifecycle witness; never
infer them from the conditional cleanup proof.

The new composition exercises stock trusted scope/spawn/join contracts in
addition to the existing physical/atomic primitive boundary. No new project
trusted contract was introduced; this is not a proof of those standard or
primitive contracts' adequacy. The positive archive SHA256 is
`65e175c8fcf08ace86b59897ae284922bec542f9db233b5308a25040810db62f`.
The same threaded missing-Acquire build rejects retirement at 11/12 obligations;
its exact source/configuration and failure are archived separately.

Decision after T02: retain objective metadata plus actual AtView payloads as a
proved concrete transport component. Do not reopen T01's subjective snapshot,
D01's mutable ghost bridge, D02's weak-SC shortcut or D05's unchanged Clone
attempts. This resolves one prerequisite and leaves the complete architecture
unadmitted. The remaining public-API/tool-scope choice was requested explicitly;
no dependent public API migration or large tool work has started.

## User selection: route 1 — 2026-10-06 UTC

The user explicitly authorized public API/representation changes and selected
the modified bytes variant with no large verifier changes. Earlier references
to a pending public-API choice are historical and superseded by this selection.
Work order items 2 and 3 are resolved: proceed with item 4, a production variant
and a full admission witness, before broad API migration. Architecture admission
remains NOT ADMITTED; this authorization is not a proof result.

The next witness must connect actual initialized bytes, read leases, real scoped
threads, retirement of another handle while a reader stays live, and formally
derived one-time final resource recovery and explicit cleanup. An empty handle
must retain its retirement obligation. Parent cleanup after joined leases is an
authorized interface design; its lifetime semantics must be stated separately
from independent original Bytes::clone lifetimes. Native XOR assertions and
conditional cleanup alone do not close the admission gate.
