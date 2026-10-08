# Guarded event boundary and original-public integration staging

The default feature proves the generic guarded atomic prerequisite. The optional
`public_shared` feature is a selected actual-source integration experiment,
with body-proved construction, Clone dispatch, reads and cleanup branch safety.
It does **not** prove all-domain `From<Vec>`, unconditional eventual final cleanup,
automatic Drop, other representations, or the complete crate.

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

The current trial stores the combined protocol's live ticket and shared
physical/control descriptors in a cfg-only `Ghost<OriginalSharedProof>` sidecar
of source-extracted Bytes. The four native fields and five actual unsafe Vtable
fields are unchanged. The mutable Vtable return type is extracted into its own
namespace with the exact `bytes_mut::Shared` pointee (Vec, capacity repr, atomic),
not substituted with the immutable Shared record.

Actual marked From<Vec> and Clone trait methods dispatch to body-checked
instrumentation. The selected constructor preserves the original len<capacity
Shared branch; the native len==capacity optimization is untouched. Clone borrows
the input ticket and calls the actual Shared.ref_cnt guarded operation once
through `field_event::increment_owned`, whose native branch includes production
`ref_count_ops::try_increment`. No second native counter is updated. The actual
public `Bytes::cleanup(self)` method consumes a ManuallyDrop natively and calls
its stored drop callback once; the selected shim proves Release, conditional
last-owner Acquire, recovery, and physical/control deallocation.

`shared_registration` and `shared_drop_registration` are explicit generic
closed-table/ghost-erasure TCB. Its
contract connects a stored unsafe callback with the checked shim's exact FnExt
pre/postconditions. The native interpretation names the production getter;
`generated/native_bindings.rs` records exact source declarations in their
production namespace. The extractor additionally checks all five static target
identifiers, Clone's common guarded helper, and Drop's Release/Acquire/free chain.
These checks and source hashes are **not** an automatically checked equivalence
proof between the instrumented body and its native erasure. The proof module is
Creusot-only; the real production crate owns the executable native table. No
bytes ownership or refcount theorem is assumed by this reification boundary.

`impl Invariant for Bytes` selects the Shared-backed representation for this
experiment and makes the actual Clone trait's unrestricted precondition valid
within that type domain. It does not cover promotable/static/owned variants.
The strong actual From<Vec> precondition fails trait refinement for Vec::new()
(len==capacity==0): narrowing an unrestricted trait input is invalid. This exact
counterexample is retained, and run-public-proof.sh explicitly excludes only
that refinement from the **selected-body** result. The future full model needs
branch-aware representation alternatives and strong contracts on every branch.

The client driver makes three clones from the same source, retires a peer and
the original source, clones a surviving peer, reads through actual AsRef, and
cleans up the remaining handles in mixed order. It proves the observed bytes. It does not prove that one invocation necessarily takes the final branch
or that physical cleanup occurs exactly once. The opaque event interface
quantifies over states with the same public descriptor, including states with
additional issued tickets. A complete client history/closure theorem remains
open; no assumed observer, fixed quota, or conditional-cleanup result substitutes
for it. Protocol lastness and reclamation inside the final branch are separately
body checked.

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

## Physical projection boundary and exact standard comparison

`physical_projection.rs` moves only metadata access across the existing B3/B4
boundary: the native pointer/length/capacity remain ordinary source fields while
BoundPtr and recovery/region capabilities remain Ghost. Preconditions require
raw-pointer equality (not just address equality), namespace/capacity/range
agreement, and Known initialized slot values. Returned slice contents match those
slots, and its lifetime borrows both descriptors. Deallocation consumes Recovery
and the complete physical region at the exact base/capacity/layout. Empty slices
still require the sealed non-null metadata invariant.

This matches production `raw_vec::{borrow_bound,borrow_empty_bound,
deallocate_bound_vec}` except that the native pointer is passed explicitly and
checked against the Ghost descriptor. The installed Std `std/ptr.rs:628` exposes
trusted `Perm::as_ref` with pointer==permission.ward and exact returned value;
`Perm::to_box` at line676 consumes the Box permission under the same equality.
Stock pointer slice constructors specify pointer metadata, but do not establish
this region's initialized byte content or deallocation authority. The adapter
therefore remains explicit generic physical TCB. `deallocate_typed_box` consumes
the typed Box permission, uses Layout::new<T>, and skips zero-sized deallocation;
for this source Shared all fields are non-owning and production checks the
atomic field has no destructor. Generic allocation support can replace these
bridges without changing the lifecycle model.

The `negative_physical_projection` controls reach semantic VCs and reject a wrong
raw pointer (one null) and an out-of-region read (two nulls: extent and Known
slots). Archive `public-physical-controls.tar.gz`, SHA256
`5b88816aafff26d84ddc86660955f08edadd00b943af72ece7fa728251cf3631`.
These are diagnostic negative results, not passed proof files.

## Public integration diagnostics

- `public-trait-cycle-frontend.tar.gz`: frontend-only mutual Fn-spec cycle from
  storing the callback's own postcondition in the Bytes invariant. Replaced by
  closed table identity and a freshly obtained generic erasure certificate.
- `public-trait-refines-rejected.tar.gz`: exact From and initial Clone refinement
  null leaves, retained with exported Why3 tasks. The Clone issue is addressed
  by the explicit selected Bytes invariant; From narrowing remains invalid.
- `public-selected-semantic1.tar.gz`: first body attempt, SHA256
  `846c67c656a0c5c107040ee8b43c86ed190fc8c001dabddce44cca16b7bc202f`.
  Constructor/read passed; three Clone FnPtr specialization leaves and four
  cross-module lifetime/atomic-identity leaves remained. Diagnostics predate the
  corrected mutable Shared namespace and actual cleanup method extraction.
- `public-selected-semantic2.tar.gz`: targeted retry, SHA256
  `30a300c50ae88c25d21e5d09d9e150709c1e64a2250196a7c11dcee1dedfe72c`.
  All three previously failing body files pass. Getter lifetime specialization
  removes the higher-ranked FnPtr model mismatch; body-proved lifecycle
  postconditions expose unchanged atomic identity and the retirement lifetime.
  This is a targeted retry, not an all-file result.

`capture_public.py` archives the complete probe source, translated proof inputs,
production source, imported lifecycle/physical probe source, and the installed
Std source with an immutable SHA256 member manifest. Each selected body result
must use its explicit target list; unrelated/known failed From refinement files
in the diagnostic snapshot must not be counted as passed.

## Canonical selected-source body result

The final multi-clone/mixed-retirement source replay (`public-selected-final6.log`)
passes 17 explicitly selected files / 267 actual prover leaves / zero nulls.
This includes both actual public Clone and AsRef trait refinements, the selected
From method body, public cleanup, stored clone/drop callback dispatch, and the
body-checked constructor/read/retirement/recovery implementations.
`generated/public-proof-targets.json` lists the precise scope and the excluded
invalid full From refinement. `generated/public-proof-summary.json` pins the
source and proof hashes. Neither full constructor coverage nor unconditional
eventual final cleanup is claimed. `audit_public.py` checks every archive member
and counts only the explicit selected proof targets.

`public-selected-final5.tar.gz` is explicitly **excluded**: the optional marker
uniqueness assertion caught one remaining duplicate free_shared begin marker;
a shell command continued to capture despite failed extraction validation. Its
source summary does not match the changed extractor, so it is not a coherent
positive snapshot. The adjacent exclusion receipt preserves the reason.

`public-diagnostic-tasks.tar.gz` preserves the exported null tasks and their
printer source (SHA256 `c7ddaac52e1ea9e0fca7681a8fdb87ce564e2146b0be178cd46219da5bc598f2`).
Expanded diagnostic task directories and run logs are ignored by Git; immutable
archives contain them. Canonical positive capture now uses `--positive` to check
all recorded source/Coma/proof hashes before creating an archive. The independent
audit repeats these coherence checks as well as verifying every archived member.

Canonical current-source archive: `public-selected-final6.tar.gz`, SHA256
`4dc978e6f5a7e9cd94e0ca2c9a93eb34dbacc319245b2390ee579b76bdc6fb4a`. The corresponding audit verifies all 1230
members and the 17-file / 267-prover-leaf / zero-null selected scope.
