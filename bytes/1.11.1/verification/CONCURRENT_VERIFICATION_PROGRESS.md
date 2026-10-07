# Concurrent verification progress

## Modified production admission — route 1

The user-selected verified,std entry now passes47 proof files with zero null
leaves in one actual production configuration. Its concurrent and ordered real
thread callers prove input-byte reads, receipt-derived unique final retirement,
and consuming parent B3 cleanup after all affine lifetime fractions return.
Missing Acquire rejects peer synchronization; empty-reader abandonment rejects
full-lifetime end. No new bytes protocol trust or unsafe Send/Sync is introduced.
Native96 lifecycle cases plus the primitive race test pass. See
MODIFIED_VARIANT_ADMISSION.md for independently audited archives and exact TCB.
This closes bounded two-reader std/x86_64 normal-return admission; arbitrary
sharing, the full API/configuration inventory and failure paths remain open.

## Subsequent modified production increments

The current archived integrated entry proves 133 files with zero null leaves:
recursive scoped sharing, ordinary mutation followed by sharing, checked integer
Cursor reads, and two reusable higher-ranked callbacks on actual physical bytes.
See MODIFIED_VARIANT_PROGRESS.md for exact source archives, independent audits
and normal-return scope. Tree nodes use two-child counters; the theorem is
not an escaping Clone implementation or a flat arbitrary-N refcount.

## Architecture thread prerequisite (2026-10-06, Asia/Tokyo)

T01 crosses real stock scoped-thread boundaries and rejects the existing full
payload snapshot because it prevents the shared invariant from being Objective.
T02 uses objective metadata and retains actual affine payloads in AtView. Its
concrete RetiredPart transport, retirement and conditional parent cleanup prove
37 files; native tests exercise84 physical cases. Removing Acquire rejects
retire at11/12. It adds no project trust; existing physical/atomic assumptions
and newly exercised stock thread contracts remain explicit TCB.

This is not actual Bytes Clone, an arbitrary-payload theorem, eventual
exactly-one completion, automatic Drop, or full architecture admission. Read
ARCHITECTURE_DECISIONS.md and ARCHITECTURE_ASSESSMENT.md before further work.
The earlier weak36 sequential gate below remains a distinct component result.

## Cloud resume: stock Creusot weak physical connection

`weak-native-publication` now passes3 proof files with native Relaxed/Release/
Acquire orderings and sc-drf disabled. Release publishes only a snapshot of
SyncView metadata. The deferred stock atomic load is converted to an Acquire
witness by an actual `fence_acquire` before peer AtView synchronization. Missing
Acquire, dropped prior publication and metadata-to-witness controls reject at
the archived intended boundaries; the prior-publication test is sequential and
does not claim all concurrent histories are detected.

`weak-physical-retirement` passes36 files and connects a fixed two-ticket
retirement protocol to actual affine PhysicalRegion and Recovery payloads. Both
retirement orders join the conserved permissions and reach B3 only on the native
final-result branch after Acquire. Removing Acquire rejects retire11/12. Native
matrices exercise endpoints and spare capacity; they are sequential harnesses.

The native atomic wrappers and generic release-RMW publication-history union
remain explicit generic physical/weak-memory TCB. No bytes-specific ticket,
refcount retirement or last-owner theorem is trusted. This is a component
connection, not the actual Bytes Shared control/vtable protocol, arbitrary
refcounts, native concurrent schedule adequacy, or whole-crate proof. Evidence:
`probes/weak-native-publication/evidence` and
`probes/weak-physical-retirement/evidence`.

## Earlier Verus checkpoint (historical)

Astra pursued a runnable Verus proof after the earlier source-only suitability
audit. The new [probe](probes/verus-refcount/README.md) supplies positive and
negative evidence without changing native Relaxed/Release/Acquire orderings,
Creusot, core/std, or the bytes runtime. All commands and source hashes are
recorded in its `artifacts` directory.

- The arbitrary-length release-sequence/publication induction, no-resurrection
  final-zero/read-from lemma, and required A/B/C Relaxed-RMW litmus pass: **5
  verified, 0 errors**.
- Generic tracked retirement escrow and its real PCell permission caller pass:
  **4 verified, 0 errors**. The erased executable also compiles and runs.
- A separate stock PAtomic SeqCst trace proves the exact counter values: **1
  verified, 0 errors**. It holds exclusive permission and is not concurrent.
- Removing Acquire, breaking the release sequence with a plain store, and
  leaving a zero-payload ticket outstanding each reject exactly one intended
  obligation.
- The direct native atomic body attempt reveals the precise stock limitation:
  syntax/control flow passes, but a private `AtomicUsize::new(1)` followed by
  `fetch_sub(1, Release)` and `load(Acquire)` cannot prove `old == 1`. Its single
  failed assertion is archived as `native-value-gap.log`. Stock Verus supplies
  empty native atomic specifications, rather than weak-memory value/view
  contracts. This is an actual failed semantic proof, not an inferred API
  mismatch or native behavioral failure.

These are distinct component results. The model's publications are visibility
labels; the escrow's permissions are actual affine Verus resources. No theorem
connects them yet. No concurrent bytes body, atomic invariant carrying bytes
regions, native synchronization adequacy, cross-tool resource conversion, or
whole-crate proof is claimed.

The remaining design work is specific: supply a reviewed generic weak-memory
atomic boundary, prove the conserved-ticket/retirement protocol around it,
and connect the physical ownership slice wholly in Verus or through a checked
Creusot resource interpretation. A stock SeqCst alternative additionally
requires a coordinated runtime ordering change and the physical ownership
connection. Neither replacing native orderings nor introducing an unreviewed
trusted release contract is necessary to reproduce this checkpoint, and
neither was done.

The earlier `VERUS_REFCOUNT_FEASIBILITY.md` remains historical source-audit
evidence. This document and the new runnable probe supersede its statement
that no Verus proof was run, while preserving its warning that stock PAtomic
alone does not verify unchanged bytes.

## Small primitive bridge attempted

A follow-up did implement the permitted small native numerical primitive
contracts in an isolated experiment. A private Release/Acquire counter caller
then proves (1 body). A separate conserved-ticket/deposited-resource state
machine proves (5 bodies). Consequently, absent stock value specifications
alone are **not** the final reason concurrency remains open.

The actual next integration attempt fails because stock `AtomicInvariant`
rejects the weak primitive. Marking it compatible with the SC invariant rule
makes Release-only PCell permission extraction verify without Acquire; that
paired diagnostic is intentionally rejected as an overstrong rule. Its native
bridge has no bytes-specific contracts, and no production or accepted trusted
boundary is changed. The exact diagnostic and the unadopted primitive clauses
are retained in `probes/verus-refcount/native_bridge_boundary.rs`.

A sound generic weak-memory invariant/view modality, or an independently
validated restricted rule for a sealed library, is still needed before the
native two/three-thread physical escrow proof can be claimed. The existing
release-sequence induction is usable within such a rule but does not itself
justify moving physical resources across weak-memory steps. The README now
records the concrete attempt, the failure, and the rejected tempting shortcut.
