# Original `bytes::Shared` lifecycle source adapter

This probe follows the selected `len < cap` branch of `From<Vec<u8>>`, the
source-shaped `Shared` and `Bytes` fields, `shallow_clone_arc`, and explicit
last-owner cleanup. `source_map.py` pins those exact `bytes.rs` blocks, the
selected `loom.rs` atomic aliases and `AtomicMut::with_mut` implementation,
and the relevant B1/B3 allocation support. The native `Vtable` remains an
opaque input value: `SHARED_VTABLE`, callback implementations, indirect
dispatch, public `Clone`, and automatic `Drop` are outside this adapter.

`source_adapter.rs` carries the three real native `Shared` fields and four
real native `Bytes` fields. The count field returned by the generic atomic
constructor is moved directly into `Shared.ref_cnt`. Clone borrows its source
handle, reads its actual `Bytes.data` field, and performs the actual relaxed
increment on that same `Shared.ref_cnt`; a private affine quota restricts the
bounded protocol to the initial ticket and one clone. The source overflow
guard remains present, and the bounded proof must establish its old count is
one before the guard.

Release consumes the handle and selects the control pointer from the actual
`Bytes.data` field using `AtomicPtr::get_mut`, matching the selected
`AtomicMut::with_mut` body in `loom.rs`. A generic pointer TCB returns only a
pointer copy and states that the field/model is unchanged. The actual count
field then performs Release `fetch_sub`; only the last-owner branch performs
an Acquire load. The recovered payload checks the exact control and B1 base
pointers plus `Shared.buf` and `Shared.cap`, then executes B3 byte cleanup and
typed control-block deallocation. A checked borrowed-slice function and
peer-release scenario keep the B4 byte borrow live across the other handle's
nonfinal release; the negative feature checks that consuming the same handle
while its borrowed slice is still used is rejected by Rust.

The trusted boundaries are generic native/model atomic creation and event
bridges, immutable/mutable reads of the exact atomic pointer field, aligned
Box permission conversion, and deallocation of the exact typed Box using
`Layout::new::<T>()`. These contracts state no bytes-specific count,
last-owner, ownership, or reclamation fact. The `AtomicPtr::get_mut` bridge is
the restricted no-store analogue of Std 0.13's permission-consuming
`AtomicPtr::into_inner`; exact native/model interpretation remains in the
generic TCB.

`source_lifecycle_driver` composes the selected constructor, borrowed-source
clone, live slice across peer release, and final cleanup. Its `reverse` input
selects whether the original or clone supplies the reader, so the native test
exercises both handle orders with empty and nonempty byte prefixes and checks
the complete returned sequence against the input. The separate
`negative_source_no_acquire` feature is Creusot-only: native builds retain the
Acquire operation, while its targeted verification omits that operation and
its callback but still calls `Pending::recover`, which should fail the acquired
guard.

The final default serialized Creusot/Why3 run passes all 44 generated proof
files, with 44 matching proof JSON files, 280 actual prover leaves, and zero
recursively audited null leaves. Root independently verified the source
hashes and Coma/JSON bijection. The native suite passes 2/2, covering both reader/peer orders. The
separate negative borrow check produces Rust error E0505 at cleanup of the
borrowed handle. The targeted no-Acquire negative control is archived at
`evidence/negative-source-no-acquire-2026-10-08.tar.gz`; it discharges 29/30
target goals and leaves only the acquired-view guard for `Pending::recover`
unproved. The exact Why3 leaf formula is in
`evidence/negative-source-no-acquire-leaf-2026-10-08.why`. The feature-mutated
source is never run natively. The full current positive source and output
snapshot is archived at
`evidence/positive-original-shared-lifecycle-driver-2026-10-08.tar.gz`.

The two production files imported by path from the bounded probe are supplied
in `evidence/shared-source-input-completion-2026-10-08.tar.gz`. The installed
Creusot Std overlay sources and crate manifest/lock are supplied separately in
`evidence/shared-source-tool-inputs-2026-10-08.tar.gz`; they are not inside the
positive source archive. Root's byte-audit receipts are
`evidence/root-positive-source-driver-audit.json`,
`evidence/root-source-input-completion-audit.json`, and
`evidence/root-source-no-acquire-audit.json`.

The earlier 43-file positive snapshot and 15-null/four-null diagnostic
snapshots remain immutable historical evidence and are not the current result.
