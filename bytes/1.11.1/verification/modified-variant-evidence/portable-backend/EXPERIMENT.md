# Actual portable-atomic backend mapping experiment

Scope: an isolated exact current bytes 1.11.1 modified implementation, with
`verified,std,extra-platforms` on the host. Preserve current source hashes before
adapting the native generic atomic boundary. No production or installed std edits.

Changed premise: the existing verified NativeAtomic ignores extra-platforms.
Select portable_atomic::AtomicUsize and its corresponding fence implementation
when the feature is enabled. Keep the exact Release RMW / Acquire fence orderings,
ModelAtomic/Perm/Committer/SyncView sorts, release_rmw publication rule and every
bytes retirement/resource body and contract unchanged. The portable fence adapter
copies only stock fence_acquire's generic view contract; it is explicitly part of
the generic primitive TCB, not a new bytes protocol theorem. No blanket Send/Sync,
SC strengthening, unsafe single-core assumption or critical-section emulation.

Acceptance: full actual library native lifecycle tests and full configured proof,
recorded source/backend/feature correspondence. Then the same actual protocol's
missing-Acquire control must fail at peer-view synchronization. Counts alone do
not prove the primitive's adequacy. Backend implementation is trusted through the
portable-atomic API memory-order guarantee, explicitly recorded and source-pinned.

Stop: first structural obstruction is captured and reviewed; two equivalent VCs
require interface review, three require restructuring. No timeout/prover-count
increase. An unrelated copied frontend typo is captured or corrected before first
proof with an explicit baseline diff.

Limits: host std/scoped normal-return lifecycle only. no_std MSP excludes scoped
atomics altogether, so its alloc proof is not evidence of portable concurrency.
Future target proofs must inspect actual UInt/Slice widths and endian-selected
Coma bodies, not combine a host proof with target metadata.
