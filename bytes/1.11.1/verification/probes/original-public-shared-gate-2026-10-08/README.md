# Guarded event boundary and original-public integration staging

The current executable proof is the generic guarded atomic prerequisite. It is
**not yet a proof of public Bytes::clone**, a combined bytes lifecycle, automatic
Drop, or crate-wide verification. The directory is also the staging destination
for the actual public-source integration trial described below.

`src/ref_count_limit.rs` from production is included verbatim. Its body proves
that refusal is exactly old > MAX_REF_COUNT and every success returns old+1 at
most MAX_REF_COUNT+1. `model_cas.rs` body proves the corresponding Relaxed
AtomicUsize CAS loop under stock contracts: the affine FnGhost callback stays
inside an Option across every failed/spurious attempt and is consumed exactly
once only in the successful compare_exchange_weak branch. The loop makes no
termination/fairness claim. This is a stock-model atomic proof, not a proof of
native field/invariant adequacy.

`event.rs` is explicit generic TCB. RawAtomic::new creates an actual core atomic
with its unique model permission; bind consumes the checked protocol. A success
of try_increment provides one matching mutable Committer with old <= limit and
store = old+1, and requires the checked FnGhost transition to restore the same
protocol/public identity and commit once. Refusal exposes only a matching
Relaxed read observation above the limit, with no success callback postcondition.
Its native implementation is fetch_update(Relaxed, Relaxed, next_ref_count),
using the identical production pure guard. The closure contains no ghost/affine
registration. Refusal and internal CAS retries make no store **by this
operation**; other threads may change shared state. The aborting increment
wrapper is body checked and promises only normal-return success.

`relaxed.rs` is the reviewed generic RMW release-sequence carry boundary missing
from the shipped shoot_store postcondition. It records the immediately preceding
history value, appends exactly one store, preserves the atomic identity and
Committer history, and carries the predecessor publication plus the actual
ReleaseSyncView. It returns only Snapshot<SyncView>. The caller's current view
advances monotonically and in this atomic's timestamp, but there is no
publication <= current (Acquire) or current <= publication (Release) clause.
Release retirement must continue to use the existing separate release_rmw rule;
this Relaxed rule is not a replacement for Release publication.

All ticket issuance, sparse map updates, fraction accounting, count equality,
lastness, typed recovery and cleanup remain caller body obligations. None is
assumed by these primitives. Future stock operation-bound invariant/native-field
support and RMW release-sequence contracts can replace these generic boundaries
without changing the clients. Their adequacy is assumed, not proved by passing
client VCs or negative controls.

## Exact standard analogue

The installed creusot-std atomic.rs compare_exchange / compare_exchange_weak
contracts (lines129–230) give mutable success versus read-only failure Committers.
Weak failure may be spurious; it does not promise a mismatching value. Native
bodies select the requested orderings when sc-drf is disabled, despite an
outdated doc sentence saying always sequentially consistent. The run wrapper
rejects sc-drf and fixes one prover / 1024 MiB under the shared proof lock.

The upstream example `examples/logically_atomic_faa.rs` uses an Option<F>
invariant and invokes f.take().unwrap() only on successful CAS. It uses the SC
AtomicI32 API. Our model_cas.rs checks that pattern for guarded Relaxed
AtomicUsize; the example alone supplies no weak-memory publication theorem.
Shipped committer.rs Relaxed shoot_store appends the explicit ReleaseSyncView,
but does not state predecessor release-sequence carry. Its Acquire shoot_load
separately establishes stored publication <= current. Exact consulted sources
are inside each canonical evidence archive under inputs/.

## Evidence

`positive-cas-4.tar.gz` is the first complete four-file positive snapshot:
SHA256 `6c521d34a135a1e9e91288dd5819d492e9b7c9734403f1aa379e10a7a41c231f`.
It contains 29 actual prover leaves and no nulls. Current final replay and
controls are recorded in the adjacent immutable manifests and logs.

`positive-initial-3.tar.gz` is an **incomplete capture**, excluded from evidence:
the capture failed while adding a production input at an incorrect relative path.
It is retained only to avoid silently replacing a used evidence name. No result
or completion claim relies on it. The repaired script forbids overwriting names.

Controls require rejection of (1) gaining Acquire from Relaxed carry, (2)
publishing the entire current thread view with Relaxed, (3) committing twice,
(4) using another atomic field, (5) assuming the success callback issued a result
on refusal, and (6) preserving payload publication after deleting predecessor
carry. Feature mutations are never run as production/native behavior.

## Actual public integration trial

Production keeps four native Bytes fields and five unsafe function-pointer
Vtable fields. Existing source C44 uses a separate OriginalSharedHandle and
opaque Vtable; existing original-freeze has an exclusive cfg-only sidecar. Neither
is the sought public shared Clone proof. The guarded native algorithm also means
C44's old fetch_add snapshot is historical, not current-source coverage.

The next bounded trial will place the combined protocol's live ticket and shared
physical/control descriptors in a cfg-only sidecar of source-extracted Bytes.
Actual From<Vec> (selected len<cap branch), actual Clone trait method, actual
SHARED_VTABLE.clone field, checked shared_clone/shallow_clone_arc callback,
borrowed read and explicit cleanup must share one model. A generic registered
erasure rule must bind the exact clone field to its checked shim and pass the
borrowed sidecar into the same single guarded Shared.ref_cnt event. Calling the
native callback and then a second model atomic is disallowed. Explicit cleanup
must consume the Bytes and suppress later automatic Drop natively.

Whole-crate DeepModel/comparison ICE, direct unsupported FnPtr calls, automatic
Drop and unchanged physical-ownership counterexamples remain frozen. No scalar
stand-in or disconnected API test will be reported as public integration.

## Final primitive-gate result

The restored default `positive-final-4.tar.gz` is canonical for this increment:
SHA256 `bf14dc0734fd1a9765e7d52b58d62ddf8dd36f9e091163e53b9dd6494e965966`.
It proves four files / 29 actual prover leaves / zero nulls. The positive
translation includes the production pure guard, model CAS loop, aborting event
wrapper and publication-preservation client. Trusted primitives are not counted
as body proofs.

The separate Acquire and current-view-publication negatives each reject one
intended goal. `negative-controls-semantic.tar.gz` (SHA256
`136e717692d195c33df1e382e1a7177ab5a28cb17170125eacb152e3b778da0e`)
rejects exactly four intended leaves: wrong field, issuing on refusal, double
commit, and retained payload publication after removing predecessor carry.
`negative-controls-combined.tar.gz` is the earlier frontend diagnostic only:
a Ghost value was extracted outside a ghost block; it never reached semantic
VCs. The corrected semantic control copies its supplied ReleaseSyncView (the
installed type derives Copy) and fails precisely the second store precondition.
