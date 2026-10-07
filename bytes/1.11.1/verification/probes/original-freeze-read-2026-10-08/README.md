# Original offset-zero freeze and read experiment

Plan recorded before this gate's source edits. Extend original-unique-write's
actual BytesMut sidecar with the retained B1 RawAllocation. Select the original
Vec/off0 freeze branch and the original Bytes::from(Vec) len<cap Shared branch.
Interpret their raw reconstruction/detachment boundaries with existing B2/B1;
prove original control flow, constructor fields, and immutable read connected to
actual byte allocation capabilities. Native four-field representations remain.
No replacement buffer and no trusted public freeze/constructor contract.

The Shared Box is actually allocated with original buf/cap/AtomicUsize fields.
A cfg-only Box Perm records its native pointer and fields. The ordinary native
atomic constructors and static Vtable reference may use local true-only
translation leaves; they establish no atomic value, data pointer contents,
refcount, ordering, vtable identity or callback behavior. These remain explicit
open obligations. Read validity comes from retained actual B1 capabilities and
B4, not atomics or a fictitious ghost refcount. Return live Bytes; no Drop claim.

Strict len<cap excludes boxed/promotable/static paths; original KIND_ARC freeze
is excluded. Calls to omitted branches have false-precondition interfaces in the
probe and must be unreachable. Preserve exact source and generated span hashes.
Two equivalent failures require interface review; third requires restructuring.
No timeout increase. Positive requires complete nonempty proof tree/engine exit0.
Native tests use the actual main crate, not proof-only vtable callback stubs.


## Completed bounded result

The complete configured gate exits 0 with **61 proved files**. The connected
caller constructs the original `BytesMut`, appends actual input bytes, freezes
through the unique/off0 branch, constructs the actual `Bytes` Shared record, and
reads an actual byte through its slice. It returns the live `Bytes` and proves
its entire byte sequence equals the input and the observed first byte is correct
(or None for empty input). Input length is below `usize::MAX`; capacity is
requested as length+1. Growth, nonzero offsets, the len==cap boxed path, and the
KIND_ARC freeze arm are outside this result.

`positive-byte-load-61.tar.gz` preserves the proof-time source, generated source,
Coma, full JSON trees and logs. The earlier 60-file positive established the
slice and length before the byte-load client was added. The pointer-retarget
negative completed with exactly one null leaf: after replacing the actual
`Bytes.ptr` with null while retaining its sidecar, `as_slice` cannot establish
`original_frozen_valid`. Its generated task is archived separately. This is not
a test of arbitrary equal-address pointers with different provenance. The
unique-write regression under the same pointer overlay proves 51 files; its
archive is in the preceding original-unique-write probe.

The actual main-crate native test covers empty and nonempty values, pointer
continuity across append/freeze, and byte contents. The original BufMut and Bytes
native suites also passed (23 and 118 tests). Native tests run actual Drop;
these tests do not establish a formal destructor theorem.

## Exact source correspondence and assumptions

Extraction copies marked spans from original `src/bytes_mut.rs` and
`src/bytes.rs`, including their actual field definitions. It selects concrete
Buf::advance and From<Vec>::from bodies as inherent proof surfaces; it does not
prove the open trait laws or compile the complete original crate frontend.
Native Shared/vtable callbacks are not replaced by test stubs: native tests
compile the real main crate. The proof excludes static-vtable materialization,
using a local ordinary trusted getter with only `ensures(true)`; the atomic
constructor translation leaves also have only `ensures(true)`. These are three
explicit temporary tool boundaries, not atomic or vtable specifications.

The physical interpretation uses existing B1 detach, B2 resume and B4 borrowing
contracts. The retained RawAllocation is consumed by B2 and reissued by B1 on
the original Vec-to-Bytes route. Namespace renewal consumes the old affine
capabilities; no allocation is duplicated. The selected proof branch uses a
body-checked off0 reconstruction helper; native `rebuild_vec` remains original.
The proof also omits native pointer.add(0), under the proved zero advance
precondition. Sidecar extraction uses `Option::take`, avoiding moving a field
out of the Drop-bearing original BytesMut. The extracted gate omits Drop.

Exact pointer-model metadata links the original fields to the sealed BoundPtr;
Box Perm links the actual Shared pointer to its buf/cap fields. The result does
not derive ownership, liveness or provenance from a numeric address or arbitrary
raw-pointer comparison. The covered path copies the original pointer through
Vec/B1, the retained descriptor/B2, then Vec/B1/Bytes.ptr, without reconstructing
it from an address. Provenance correspondence remains part of the inherited
B1/B2/B4 pointer interpretation; Opaque.ptr equality alone is not claimed to
prove provenance for forged equal-address pointers. The separate numeric tag
pointer never supplies byte-access authority.

All byte-control-flow, initialization, field correspondence, Box permission,
and selected freeze/read contracts are body proved. Generic physical primitives,
typed raw memcpy, and the reviewed Std capacity/address/pointer observers remain
explicit TCB assumptions. No new bytes-specific ownership/refcount law is
trusted. No atomic initial-value/data-field relation, vtable identity/callback,
Clone, shared aliases, automatic Drop, unwind cleanup, or full freeze API claim
is made. The temporary atomic/getter leaves can be replaced locally by adequate
tool contracts, but that alone would not prove those remaining protocols.

The source-map generator now distinguishes `source/` and `generated/` keys;
older proof-time maps used the same names and overwrote the whole-source map
entries. Those archives still contain exact source files and their full hashes.
`evidence/audit.json` records byte-identical regenerated Rust after the map-only
repair, as well as archive hash/count checks.
