# Unrestricted boxed construction/read/normal automatic Drop — 2026-10-09

D2026-10-09-AK, directed by Astra after audited/published AJ. The immutable native
client takes an unrestricted Box<[u8]>, uses the actual Bytes::from and AsRef,
copies its view and returns. It contains no explicit cleanup/drop, nonempty,
alignment or pointer-parity premise. Target: empty Static and both unpromoted
PromotableRaw tags, exact returned contents, and the sole terminal normal Drop.

## Bodies and correspondence

Pinned after-ElaborateDrops MIR evaluates `_0 = move _4` in bb3, then bb4 drops
owner `_2` (`bytes`) with normal successor bb5 and unwind bb9. External elaboration
before Creusot borrow/liveness saves the Vec, calls an ordinary consuming
bytes_terminal_drop(Bytes, Ghost<&mut Option<BoxedCompletion>>), checks completion,
then returns the saved Vec. No trusted Bytes Drop or live &mut Bytes invariant
restoration after free is introduced. Eleven MIR captures include actual
Bytes::drop, static/even/odd callbacks, their closures, native ptr_map,
free_boxed_slice and the default core AtomicMut::with_mut target.

The AE constructor/read proof component is copied here; its published gate is
unchanged. Local strengthened shape posts on unchanged new/from_static bodies
establish Static for empty construction. The Box helper establishes boxed_only;
there is no Shared case in this client. Raw validity retains the original
BoundPtr, Recovery, complete initialized PhysicalRegion, pointer/length/capacity,
readonly AtomicPtr binding and exact selected table; it also carries the tag's
low-bit fact. Two generic mathematical lemmas are body proved with the shipped
bitwise_proof mode. A single generic tag symbol avoids a second opaque forwarding
symbol hiding the roundtrip relation. No Bytes ownership/branch/capacity theorem
is trusted.

The consuming body selects the original vtable drop slot and uses the unchanged
native three-argument invoke3 interface with an erased bundle. Three closed table
registrations equate only native/shim identities and pre/post descriptors; their
actual callback bodies prove all branch/ownership/effect conclusions. Static
returns NoAllocation. Both raw callbacks retain the KIND_ARC branch explicitly:
readonly data plus the proved low-bit fact excludes it. The selected native
branch still calls release_shared; this is recorded proved-unreachable branch
elaboration, not an arbitrary omitted target.

Even clears the tag, odd casts the stored value, then both retain native
`cap = offset.offset_from(buf) as usize + len`. Exact constructor pointers imply
zero distance; the body proves cap equals the allocation capacity and consumes
Recovery/full PhysicalRegion through the existing generic physical deallocate
boundary. Freed contains a genuine opaque affine FreeReceipt with the original
namespace, pointer, size, alignment 1 and allocated=true. There is no Shared
control allocation/receipt. A ghost flag alone cannot create this effect.

## Explicit generic TCB

The new low-bit clear contract is an **assumed exposed-provenance tag roundtrip**.
Default native ptr_map exposes `ptr as usize`, modifies its address and casts
back; the miri wrapping-add branch is separate and not selected here. Rust's
pinned exposed-provenance documentation says the selected exposed provenance is
not precisely guaranteed. This is an explicit generic physical assumption tied
to the actual forward tag operation and exact base pointer, not a strict
provenance theorem inferred from address equality. The caller independently
retains and checks complete allocation authority before free. The removal path
is a supported exposed-provenance tag interpretation or a provenance-preserving
native operation with its changed source premise separately admitted.

The generic equal-pointer offset_from contract promises only zero distance and
no ownership/liveness. Pinned Rust core documentation explicitly permits pointers
at the same address, including this u8 zero-distance operation. Its actual native
body uses offset_from. Full authority remains mandatory for deallocation. Exact
pinned core ptr/mod.rs and const_ptr.rs are retained in native-stdlib as review
context, not as proof of the exposed-provenance assumption.

Existing generic readonly atomic field, physical read/free, native/source/MIR
interpretation, ghost erasure/reification/invoke3 and terminal-place interpretation
remain TCB with their previously reviewed contracts. The complete selected
closure must not observe/escape the Bytes receiver or data-field address, and no
independent Bytes field-drop glue may execute. Heap allocation pointers remain
real identities. Native tuple needs_drop compilation corroborates the field
profile; borrowed-reference/zero-size input Box interpretation is generic
Rust/Std/compiler TCB. Native execution alone does not prove parity coverage.

## Validation and failures

The first body gate (95/540/8) exposed missing parity/tag/Static shape information.
After an interface review (95/530/3), the remaining tasks showed an opaque tag
forwarder and unprovided bitwise identities. The interface was restructured to
one generic tag symbol and two body-proved bitvector lemmas; the diagnostic gate
then proved 97 files/510 actual prover leaves/zero nulls. These immutable failures
and the positive diagnostic are retained. Earlier frontend type and generated
hyphen-file Coma span-name parse failures are separate tool diagnostics, not
ownership proof evidence. No frozen stock Drop retry or Bytes-specific axiom was
introduced. The same-failure redesign budget was honored.

Semantic development receipts explicitly say not_run; those are not structural
acceptance. Final structural controls, type controls, positive replay and archive
reconstruction are recorded separately after completion.

## Reproduction

`generate-native-mir.sh` pins compiler/cargo, captures the complete selected normal
closure, runs empty/nonempty native examples and the field-profile compile check,
and regenerates the positive elaboration. Its Cargo lib and dependency paths are
relative. `run-proof.sh` uses elevated Why3 sockets, the shared proof lock, one
prover/1024 MiB and sc-drf off. Every generated Coma target is included. Canonical
acceptance checks the explicit current client and mapping, not a frozen positive
copy used by the mutation suite. No defect feature/terminal control is allowed.

`evidence.py` freezes the entire probe, targets/results/run log, production crate,
compatible prior source/checker trees, private Std110 files and exact tool/config
inputs. For archive-only checker replay, unpack the tar to a temporary directory,
copy `inputs/repository/` to the restored repository root, then place `probe/` at
`bytes/1.11.1/verification/probes/original-boxed-automatic-drop-2026-10-09/` there.
All imported sibling checkers/fixtures/support sources must come from the
captured inputs; no live worktree input is needed. Full prover reproduction also
requires restoring the pinned tools/Std and remapping activation/Cargo patch roots
as recorded. No absolute native-harness source path needs patching.

Admission is limited to all-input boxed construction/read/unpromoted normal
automatic Drop under this explicit generic TCB. Clone/promotion, unwind, arbitrary
moves/concurrent closure, other API/configuration coverage and whole-crate
admission remain open. The canonical replay proves **97 files / 510 actual prover leaves / zero nulls**,
all targets included, no defect features/terminal controls, and explicit current
client/mapping correspondence passes. All **115** structural controls reject;
seven semantic and three type controls are recorded in SEMANTIC_CONTROLS.md.
Canonical archive `boxed-positive-final1-2026-10-09.tar.gz`, SHA-256
`04aa0ecfde6d47af4f9b250c39d6fbb74556958752a674d8552688612217acb5`,
contains 2512 hashed members. See its manifest/audit and INDEPENDENT_AUDIT.md.
Final reporting files are outside the immutable input snapshot to avoid self-
reference; executable proof inputs remain frozen. This bounded increment is
admitted under its stated TCB; original architecture remains NOT ADMITTED.
