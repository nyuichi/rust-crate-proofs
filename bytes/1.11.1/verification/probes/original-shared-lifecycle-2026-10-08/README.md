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

Native tests exercise both release orders, actual count and data fields,
empty content and spare-capacity content, and a borrowed slice across peer
cleanup. The latest clean serialized Creusot/Why3 run passes all 43 generated
proof files, with zero recursively audited null VCs. Its exact source inputs,
all Coma files, proof JSON, toolchain/configuration, and native/negative test
logs are archived at `evidence/positive-original-shared-lifecycle-2026-10-08.tar.gz`.
The earlier 15-null and four-null diagnostic snapshots remain archived for
review and are not the current result.
