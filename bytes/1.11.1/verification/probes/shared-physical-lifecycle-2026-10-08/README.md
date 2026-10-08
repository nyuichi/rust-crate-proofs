# Shared physical lifecycle — bounded component

This isolated probe uses the current physical B1/B4/B3 primitives on a real Vec
allocation. It is not a replacement public buffer API and does not yet prove
original Bytes::clone or automatic Drop.

The latest positive gate proves 77 files. The B caller starts the actual native
counter at one, consumes a constructor-issued affine quota for one Relaxed clone,
and reads the entire original sequence through both lifetime-gated descriptors.
It supports both retirement orders. The Recovery/EndBorrow bundle stays in the
original ticket until that ticket retires; its Release publishes the bundle via
AtView, and the last actual Acquire load makes that same bundle available for
full-token recovery and explicit B3 deallocation. The formal cleanup result is
true. Empty readers, including empty Vecs with allocated spare capacity, retain
affine lifetime fractions.

This is a bounded one-clone source-leaf premise, not arbitrary Clone admission.
The quota proves cloning occurs before retirement; the Relaxed event therefore
does not discard a prior retirement publication. There is no overflow or
wrap-to-zero reclamation inference. Arbitrary counts, concurrent overflow/abort,
original control-block cleanup, escaping handles, and automatic Drop remain
separate obligations. The native B tests cover 12 allocation/order cases; the
historical A test contributes another six cases.

`bounded::State::{initialize,on_clone,on_release,on_acquire}` contain the single
body-proved protocol implementation. They accept the supplied actual atomic
permission and Ghost state/event arguments; an original-source adapter can bind
them to Shared.ref_cnt without allocating a parallel counter. `Retiring` has a
body-proved affine-token split/rejoin interface for scoped control-field access.
The owning Registry in this probe is only the isolated execution harness.

B's final receipt reconciliation is ghost-only and can run after control-block
cleanup: the State owns one exclusive nonlast outcome credit and one exclusive
last outcome credit. Each native Release hands out the credit matching its old
value. Two actual receipts with equal last flags would hold incompatible credits
at the same resource ID, so body-checked RA validity proves XOR. This finite
credit pair belongs only to the one-clone experiment; arbitrary handle-count
accounting remains a separate obligation. Physical recovery still requires the
actual final Acquire and full lifetime fraction; credits grant no permissions.

Historical A41 remains a fixed-two component with parent-held recovery and an
extra diagnostic Acquire, preserved separately in its exact archives. B73's
missing-Acquire control rejects the Pending recovery precondition. The final
current 74-file gate and controls are captured separately; A controls retain
their original scope.

## Reclaimable read authority

GhostShared stores FullBorrow<PhysicalRegion>, not PhysicalRegion. Sharing the
borrow descriptor resolves its final value but never makes the region itself
copyable. Each B4 slice borrows a distinct affine LifetimeToken. EndBorrow yields
the original region only after the full lifetime fraction is reunited and ended.
The descriptor may persist after that event, but no live token permits reading
through it. The current helper is a local adaptation of the historical frozen
region helper, with tokens consistently inside Ghost; current physical sources
are referenced directly and archived, not restored as a production API.

## Synchronization and explicit trust

`event.rs` extends the admitted generic operation-bound EventAtomic interface
with ordinary native Release fetch_sub and Acquire load callbacks. Bind consumes
one affine protocol state. Each operation borrows and restores that SAME hidden
state once at the matching event of its private native AtomicUsize. There is no
alternate ghost opener, invariant export, or fresh-state/authority generation.
The callback is FnGhost and cannot recursively invoke ordinary native operations.
These facts and correspondence between native events and Committer's ward are
explicit generic TCB assumptions; PhantomData does not prove them.

`release.rs` copies the previously reviewed generic Release-RMW publication rule:
publication joins the predecessor release sequence and the local current view,
returning only a Snapshot, never an acquired SyncView. The last Release records
its timestamp and AtView publication marker in an agreement authority. The actual
Acquire load authenticates that marker, proves its read timestamp is the final
modification, and obtains a current SyncView before either sealed payload is
synchronized. This does not turn Relaxed into Acquire or use SC extraction.

The publication agreement contains the logical value of an existing AtView;
it grants no conversion from metadata back to a capability. No Objective marker
was added for physical regions or SyncView. The first attempted
Snapshot<SyncView> agreement failed the actual Objective check and is preserved.

Bytes-specific ticket conservation, count 2→1→0, registration-side uniqueness,
last flags, receipt XOR, full lifetime reunion, and B3 call prerequisites are
body proved. No trusted bytes ownership/refcount/deallocation theorem is added.
The inherited generic TCB also includes physical B1/B4/B3, stock resource
algebras, FullBorrow/lifetime primitives, GhostShared, and AtView/Committer rules.

## Evidence and reproduction

Run `./run-proof.sh` with elevated execution for Why3 sockets. It serializes on
`/tmp/itoa-creusot-proof.lock`, rejects sc-drf, sets 1024 MiB, and uses one prover.
Native: `cargo test --locked`. B controls use `--features` with
`negative_no_acquire`, `negative_b_premature`, `negative_b_duplicate_ticket`,
`negative_b_quota_reuse`, or `negative_b_live_read`. Historical A controls use
`negative_missing_ticket`, `negative_duplicate`, and `negative_live_read`.
Do not execute negative physical-recovery bodies natively.

`positive-conditional-40.tar.gz` is the earlier component with conditional
cleanup only. `positive-exact-one-41.tar.gz` adds receipt-derived completion and
whole-slice equality. The latter captures all path-referenced physical sources
and exact Std boundary snapshots. Earlier frontend diagnostics are preserved
separately and do not count as proof VCs. Control and restored-positive outcomes
are recorded in `RESULTS.json` once completed.

The canonical source snapshot is `positive-b-source-address-77.tar.gz`: engine
exit 0, 77 Coma files and 77 proof trees with zero null leaves, native two tests
passing. The controls below retain their exact B74 source versions.
The final missing-Acquire control fails one B recovery obligation and three
historical A obligations. The corrected premature-recovery control fails only
the full-fraction requirement (32/33); its earlier two-null capture is retained
as an interface diagnostic. Ticket duplication and quota reuse are Rust E0382
frontend rejections. Retiring a ticket while its actual B4 slice remains used is
a Rust E0505 rejection. These three frontend controls produce no Coma/VCs and
are not counted as failed mathematical goals. Exact source versions, member
hashes, and outcomes are recorded by `RESULTS.json` and `audit.py`.

The archived Std files are exact boundary-source snapshots, not a claim that
the complete private patched Std package is byte-identical to the upstream
package. The reviewed generic rules remain explicit assumptions.

The B75 increment adds `State::on_release_with_live_peer`: an actual borrowed
peer ticket is compared against the retiring ticket through stock fragment
compatibility. Their distinct live registrations force count two. The helper
delegates to `on_release` exactly once and proves this event is nonfinal. It
adds no native operation and no trusted lastness law. Its proof-only peer
argument belongs to the bounded diagnostic caller; it is not an extra argument
to the original public bytes API. The initializer also states the exact public
atomic-ward equality needed by a separate source adapter.

B77 additionally exports body-proved token-lifetime observations at the module
boundary, and relates the public atomic ward to the event trait's ward. The
source adapter therefore need not unfold opaque Ticket/Retiring/Pending
validity predicates. These facts expose existing exact capability relationships;
they do not manufacture tokens, synchronization, or permissions.

The final dependency refresh reruns the entire B77 gate after the unchanged
`BoundPtr::as_ptr` body gained its precise numeric-address postcondition.
`positive-b-source-address-77.tar.gz` captures that current canonical helper
source; earlier B77 archives retain the earlier dependency version. The audit
checks the current probe sources, every archived physical dependency, and the
archived Std boundary files byte-for-byte. This remains a boundary-file check,
not an assertion about identity of the complete patched Std package.
