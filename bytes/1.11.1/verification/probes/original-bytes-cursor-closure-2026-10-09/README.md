# AR ownership-preserving cursor closure

Canonical v2 proof and independent final archive replay are complete.
Full original bytes 1.11.1 verification remains **NOT ADMITTED**.

This probe keeps the complete AQ positive source and all inherited targets,
with one explicitly pinned proof-only enum extension (`Vacant`). It adds
ordinary body-proved contracts for actual Bytes Buf::advance/inc_start,
remaining, chunk and read over Shared and Static slice-created views.
`advance_api(&mut Bytes, count)` takes no external ScopeCursor: its strong frame
preserves the actual lifetime ticket, allocation, binding and data/vtable.
Owned Shared views may reach zero length without becoming ticket-free Static.

A runtime-variable native client slices an original promoted Box, drops its
original and owner, performs any finite sequence of valid advances, observes
exact suffix bytes and forces a final drain. Shared zero-length final Drop
retires its real ticket and performs final Acquire and both frees. Static empty
views retain no ticket; the earlier owner Drop recovers the allocation. These
are normal valid-range sequential paths; other representations, public methods,
concurrency, unwind and configuration coverage remain open.

The native smoke gate passed 140 cases. Source/MIR correspondence captures 30
MIR bodies (29 production plus the client) and exact normal Drop places.
`native_cursor_bindings.rs` and `native_view_bindings.rs` are extracted review
artifacts, not compiled input. The actual Cargo input gate separately captures
OUT_DIR/public_records.rs and its build fingerprint/output/rootoutput join.

Diagnostic history is immutable under evidence/: first full run150/1373/1/0,
whose sole task exposes the missing generic slice::is_empty contract; an
intermediate frontend-only E0433 macro-path error; corrected diagnostic
150/1328/0/0. `AR_DIAGNOSTIC_AUDIT` independently validates the archives and
inputs. Diagnostic correspondence was skipped (exit2), so these counts alone
are not canonical admission. The full source/Cargo correspondence check and45 main controls pass. The native
checker76 controls reject as expected, including independent native omission
of each of the two actual dealloc calls. All16 semantic controls retain61
failed tasks, independently reprinted; allthree affine/frontend controls
reject (E0382, E0502, E0277). The canonical feature-free run proves all150
targets with1328 prover leaves, zero null/structural leaves, no exclusion and
correspondence exit0. Independent canonical replay passes1312 members, main45/native76 controls and
archived Cargo inputs without a live target directory. Canonical v2 SHA256 is
`b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7`.
The historical v1 lacks two required AQ evidence documents; v2 supplies them
with unchanged150 COMA/proof hashes. See evidence/AR_CANONICAL_AUDIT.md.

Generic native pointer metadata, erased callback registration and the narrow
slice extern contract are explicit TCB in TCB.md. The inherited empty-borrow
adapter needs its own explicit nonnull precondition before full admission;
AR callers already provide nonnull. All archives are location-bound captures
with an external pinned toolchain, not portable proof bundles.
