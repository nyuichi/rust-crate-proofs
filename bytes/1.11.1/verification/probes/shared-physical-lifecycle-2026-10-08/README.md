# Shared physical lifecycle — bounded component

This isolated probe uses the current physical B1/B4/B3 primitives on a real Vec
allocation. It is not a replacement public buffer API and does not yet prove
original Bytes::clone or automatic Drop.

The latest positive gate proves 41 files. The caller reads the entire original
sequence through two overlapping borrow descriptors, retires one reader, reads
again through the other, and explicitly deallocates the same allocation after
recovering its full physical authority. Empty readers retain lifetime tokens,
including empty Vecs with allocated spare capacity. Per-side completion
agreements connect each last flag to the actual Release RMW's old value; their
body-proved invariant yields XOR. Consequently the caller's cleanup-completed
result is formally true on normal return, not merely a native assertion.

The current bounded constructor starts the native count at two. Dynamic clone
from count one, arbitrary live counts, original control-block allocation and
its cleanup, independent escaping handles, and automatic Drop remain separate
obligations. Recovery/EndBorrow stay in the parent in this first component;
transport of those subjective capabilities to an arbitrary last thread is not
claimed. The caller makes an extra diagnostic Acquire observation to combine
the two completion receipts. Original release integration must use its existing
last-owner Acquire load for the final observation, or separately justify its
proof interface; this extra observation is not silently attributed to Bytes.

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
Native: `cargo test --locked`. Controls use features `negative_no_acquire`,
`negative_missing_ticket`, `negative_duplicate`, and `negative_live_read`; do not execute negative
physical-recovery bodies natively.

`positive-conditional-40.tar.gz` is the earlier component with conditional
cleanup only. `positive-exact-one-41.tar.gz` adds receipt-derived completion and
whole-slice equality. The latter captures all path-referenced physical sources
and exact Std boundary snapshots. Earlier frontend diagnostics are preserved
separately and do not count as proof VCs. Control and restored-positive outcomes
are recorded in `RESULTS.json` once completed.
