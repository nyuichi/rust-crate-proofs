# Modified bytes 1.11.1: selected architecture and coverage contract

Selection: user route 1, 2026-10-06 UTC. Public entry: `bytes::verified`,
selected by `--no-default-features --features verified,std`. Native and proof
builds select the same production entry and bodies; `cfg(creusot)` adds the
existing physical/atomic specifications. The original default API remains a
separate implementation and is not the complete verification target.

Status: bounded two-reader architecture ADMITTED; complete variant NOT COMPLETE. Names below describe
required responsibilities; the final public-method inventory must be reconciled
with the actual compiled source. This specification introduces no trusted axiom.

## Admission contract

The scoped owner detaches a real initialized `Vec<u8>` through B1. No live Vec
co-owner remains. Its pointer and capacity are descriptors; actual Recovery and
PhysicalRegion capabilities govern access and deallocation. One frozen physical
model is used throughout construction, immutable reading and final reclamation.

A parent lifetime fraction anchors the allocation. Explicit sharing creates two
read handles with their own affine lifetime fractions and retirement tickets,
including empty views. Their runtime borrowed slices are created through the
existing B4 read boundary. Safe slices and affine fractions cross real scoped
thread boundaries; raw pointers receive no blanket Send/Sync assertion.

A handle reads while retaining its own lease and closes by consuming itself.
Closing one handle must not invalidate another reader. Actual native Release
RMWs and the final Acquire fence govern concurrent ticket return; numerical
metadata alone grants no physical authority. Completion receipts must derive
from those actual operations. Consuming both matching receipts must formally
establish exactly one final return, then reunite all lifetime fractions. Only
then may parent cleanup consume full Recovery/PhysicalRegion through B3 once.
A missing receipt, live reader or empty outstanding handle prevents this close.

The first witness has two scoped handles, std, x86_64 and normal return. It is an
architecture admission witness, not arbitrary-count or independent-Clone proof.
The generic primitive TCB remains explicit; no bytes-specific protocol, receipt,
last-owner theorem or Send/Sync theorem may be trusted to discharge this gate.

## Legacy correspondence and subsequent phases

| Legacy responsibility | Modified route | Coverage required before full completion |
|---|---|---|
| Vec-backed construction, length, empty, immutable bytes | Actual scoped owner/read views | Constructor and byte-model correspondence, including empty input and spare capacity |
| Clone and implicit shared slice/split creation | Explicit consuming/context-aware sharing and splitting | Arbitrary counts and bounds; every returned handle has a real lease/ticket |
| Drop and last-owner reclamation | Explicit close of each handle and final owner cleanup | Conservation, exactly-once cleanup; explicit normal-return/unwind limitations |
| Deref/AsRef | Borrowed read views tied to a live handle/lease | Actual byte semantics and borrow lifetime through peer retirement |
| BytesMut, mutable views, freeze, reserve, reclaim, unsplit, extend | A sound exclusive-storage phase joined to this same model | Reject the preserved D01 ghost-erasure counterexample; no disconnected mutable model |
| Buf reads and adapters | Concrete proved implementations or a sealed verified interface | No trusted laws for arbitrary open downstream implementations |
| BufMut writes and IO adapters | Explicit exclusive/mutation operations | Initialized-byte model and actual effects; adapter bodies composed into this entry |
| Box/String/iterator/static/generic-owner conversions | Explicit supported conversion inventory | Real ownership and allocator correspondence for each admitted conversion |
| serde, no_std, portable atomics, widths/alignment, panic/OOM | Explicit configuration/behavior inventory | Each retained configuration proved; exclusions stated as target changes, never silently counted |

Work sequence: (1) full scoped admission above; (2) arbitrary-handle sharing and
bounds; (3) exclusive mutation/freeze with the same physical representation;
(4) concrete read/write/adapter API composition; (5) conversions and retained
configuration coverage. Previously proved helpers are reused only where their
contracts and source match this implementation. Each validated increment is
committed and pushed; admission failure triggers interface review and a recorded
stop decision rather than repeated frozen experiments.

Admission passed in MODIFIED_VARIANT_ADMISSION.md. Validated subsequent
increments are recorded with exact source archives in MODIFIED_VARIANT_PROGRESS.md.
The current proved public surface includes scoped_roundtrip,
scoped_after_peer_close, scoped_tree, scoped_set_and_read, scoped_numeric_read,
with_shared_read, callback_length_and_first, ExclusiveBytes and Cursor. The
low-level owner, handles and receipt/context operations remain private.

Tree sharing admits every finite requested count >= 2 through hierarchical
two-child counters, with explicit cleanup for rejected counts 0/1. It does not
provide escaping handles or a flat-N Clone counter. ExclusiveBytes uses ordinary
Vec mutation and explicit close; capacity-growth quantities and same-allocation
thaw are still open. Cursor has concrete checked integer readers and
advance/copy, not an open Buf law. Higher-ranked callbacks cannot return a borrow
of their input, and their own contracts govern their behavior.

Remaining responsibilities in the table are still obligations until an actual
connected implementation is proved or a concrete blocked attempt and decision
is recorded. These normal-return std/x86_64 results do not establish no_std,
serde, alternate atomics/platforms, unwind cleanup or total termination.
