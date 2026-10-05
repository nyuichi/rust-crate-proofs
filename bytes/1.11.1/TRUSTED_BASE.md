# Trusted base and unresolved obligations

## Fixed tool foundation

Rust `nightly-2026-06-22` (rustc `91fe22da8`), vanilla Creusot 0.13.0
`318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`, matching registry creusot-std/proc/
pearlite-syn 0.13.0 with Cargo locks, pinned Why3/Why3find and fixed solver versions.
Their translation, primitive specifications, solver logic, Rust/LLVM and Global
allocator semantics are foundational assumptions, not proved by these probes.

## Current working-tree addition (2026-10-05)

The 103-file in-capacity reserve/resize/extend_from_slice gate adds no TCB.
It calls the body-proved ordinary MaybeUninit fill/copy helpers and publishes
only Known slots through actual set_len. spare_capacity_mut exports its existing
slot interpretation in both slice-relative and buffer-visible index forms;
these additional postconditions are body proved. Native19, ordinary test_bytes118,
and no-default-features checks pass. Reallocation and automatic Drop are excluded.


- Fixed-zero u8 physical bridges: `borrow_empty_bound`,
  `borrow_empty_bound_mut`, and `borrow_empty_bound_uninit_mut` return literal
  zero-length references from sealed non-null BoundPtr metadata. Their lifetime
  borrows immutable descriptor metadata; they preserve the native pointer and
  grant no positive-byte permission, Known evidence, allocation liveness, or
  recovery. One-byte alignment suffices. These are ordinary program operations,
  not ghost-callable physical writes. The unique-only valid-handle-traits gate
  proves actual AsRef/AsMut bodies and refinement VCs (70 files/native1); the
  matching negative rejects one invariant obligation (71 files). Neither the
  strong invariant nor this gate covers Shared promotion/split or automatic Drop.


- `B4-read`, `raw_vec::borrow_bound`, is an additional reviewed initialized
  read-only slice bridge. Shared borrows of the sealed descriptor and matching
  PhysicalRegion bound the returned lifetime; all visible slots must be Known.
  It provides exact length/content, creates no permission, and grants no
  liveness for an empty region. Actual as_slice and the packet read wrapper are
  body proved in the 90-file gate; fourteen native tests pass. The Unknown-read
  negative rejects one intended obligation among 91 files. This bridge does
  not establish a global BytesMut invariant or safe trait integration.


- `B4-uninit`, `raw_vec::borrow_bound_uninit_mut`, is an additional reviewed
  u8-only physical access bridge. Sealed BoundPtr metadata is passed by value;
  the mutable affine region borrow controls the returned MaybeUninit slice
  lifetime. Each slot inner Option maps to standard MaybeUninit<u8>::View, with
  exact prophetic writeback and outside-range frame. Unknown access is allowed
  as MaybeUninit only; publishing/reading bytes still requires Known evidence.
  No generic drop/validity or ownership protocol fact is trusted. Its packet wrapper,
  actual spare_capacity_mut body and initialization/publication callers are
  body proved in the 86-file gate (87-file one-guard negative, native12).
  The earlier frozen 79-file gate does not include this primitive.


- `B4-bound`, `raw_vec::borrow_bound_mut`, adds a trusted physical mutable-slice
  bridge for a sealed bound descriptor and matching affine initialized region.
  The pointer is derived directly, with no arbitrary pointer/address match.
  Region lifetime covers the slice borrow; final values are written into the
  ledger and every other slot is framed. Empty slices assert no liveness. This
  is additional explicit physical TCB, not a trusted bytes ownership theorem.
  The packet borrow helper, as_slice_mut body and simultaneous disjoint mutation
  callers are body proved in the 68-file sequential gate.


- `BOX-ALIGN-01`, `src/ownership_proof/boxed_alignment.rs::into_raw_aligned`,
  consumes an ordinary Box and returns the existing typed Perm ward/value plus
  native pointee alignment. This generic physical bridge fills the missing
  alignment postcondition of standard Perm::from_box. It does not grant bytes
  regions, tickets, refcount, tagging, promotion or finalization facts. The
  address-bit lemma is body-proved. The actual first split gate depends on this
  additional trusted fact (61 positive files, 62-file one-guard negative).


- `RVB-01` and `RVB-02`, in `src/ownership_proof/raw_vec.rs`, are reviewed local
  physical Vec/raw detach and recovery primitives. Their sealed namespace
  interpretation connects native allocation metadata to ghost Recovery and
  PhysicalRegion tokens. B1/B2 bodies are trusted; split/join and the unchanged
  content round-trip caller are proved in isolation. They do not trust a
  BytesMut, Shared, refcount or drop protocol. Exact assumptions and the
  native-pointer comparison restriction are recorded in
  [`verification/RAW_VEC_TRUSTED_BOUNDARY.md`](verification/RAW_VEC_TRUSTED_BOUNDARY.md).

- `STD-PTRWRAP-01`, in `src/provenance_specs.rs`, is an additional assumed
  `extern_spec` for raw-pointer `wrapping_add`, restricted by precondition to
  pointee types whose modeled size is one byte. It specifies only the
  resulting numerical address modulo the address-space size. It does not
  specify preservation/equality of provenance, allocation identity or
  liveness, `Perm`, `PtrLive`, dereferenceability, or any allocation resource.
  The tagged-pointer and null-derived metadata code therefore gets address
  arithmetic facts only; the spec cannot authorize reading through a metadata
  pointer. The isolated positive probe proves 8 files/23 VCs, and the negative
  forged-metadata dereference probe fails at the intended permission VC. This
  is a new TCB assumption and is not part of the historical component replay
  below.

No bytes-specific `#[trusted]` ownership theorem was added. The local B1/B2
bridge supplies the initial owned-region/recovery foundation, but the unresolved
stop-condition-D integration remains outside the address pointer contract: actual
`Vec<u8> -> BytesMut` split/mutation/drop needs independently transferable
owned regions, exact initialization/allocation tracking, separate recovery
authority, a raw allocation descriptor in `Shared`, and view-indexed transfer
through the native Release/Acquire protocol. The available borrowed `Perm`
interfaces do not supply those resources.

## Additional reviewed standard primitive contracts

- `STD-CONVERT-01`: u64 -> usize fallible integer conversion, in `src/std_specs.rs`.
- Existing `Perm::from_box`, `as_ref`, borrowed `split_at`, `to_box`, `live`, and
  `PtrAddExt::add_live`: actual allocation/borrow permission foundations.
- `STD-PTRDIFF-01`: u8 native offset_from, requiring both pointers in the same
  live-allocation range with matching provenance. No ZST case.
- `STD-LAYOUT-01`: from_size_align restricted to positive size <= isize::MAX and
  alignment one. It is not a general complete Layout specification.
- `STD-DEALLOC-01`: native dealloc requiring the base and matching (length,1)
  layout of a full owned Box slice permission. It consumes that unique token and
  is not callable in ghost code. A borrowed/partial slice token is insufficient.

The last three are in the isolated deallocation probe and were reviewed by Astra.
Owned slice permission splitting, if later added, requires re-auditing the
full-allocation token interpretation used by deallocation.

## Not admitted as bytes TCB

Clone/drop/refcount/unique/freeze/promotion/vtable correctness, initialized-capacity
ownership, Send/Sync resource transfer, and unwind cleanup remain proof targets.
No bytes-specific contract for them has been trusted to pass the crate.

## Experimental tool patch

The impure specification-free trait self-bound patch is an experiment, not part
of the standard toolchain used by helper/storage/deallocation proof results.
Astra reviewed its restrictions and identified the external-spec guard. Dedicated
positive and negative regressions are recorded; broader verifier soundness and
invariant/resolve/spec dependency cycles remain review concerns.

## Additional component proofs in the historical 60e81ad checkpoint

The slice cursors, byte codecs, checked reads, initialized/uninitialized writes,
comparison, chain arithmetic and capacity metadata helpers introduce no trusted
contracts or assumed lemmas. Their isolated proofs use the unmodified compiler
and existing standard sequence, integer, slice and `MaybeUninit` contracts.
The Box write-back and borrowed region proofs additionally use standard `Perm`
and pointer-liveness contracts. These results do not establish independently
owned shared intervals or destructor/refcount correctness. Runtime source
connections and ordinary tests do not discharge the callers' ownership premises.

## Local physical access bridge RVB-04

`ownership_proof/raw_vec.rs::borrow_mut` is an additional audited trusted
physical boundary. It derives pointers only from the sealed detached Vec
descriptor and relates initialized slice writes to exclusive region slots,
with full framing outside the borrowed interval. Its exact contract and caller
were reviewed by Astra. The caller proof does not prove this native body.
See `verification/RAW_VEC_TRUSTED_BOUNDARY.md` for bounds and lifetime limits.

## Local physical deallocation bridge RVB-03

`ownership_proof/raw_vec.rs::deallocate_vec` is an audited trusted physical
boundary: exact sealed base/capacity, full region and unique Recovery are
consumed, then a zero-length Vec is actually destroyed. No byte initialization
is required. Native destruction is assumed by this boundary; caller proofs do
not derive it from standard `mem::drop` or establish automatic Drop effects.

## Initial bound-pointer adapter checkpoint

The constructor checkpoint additionally trusts the consuming
`RawAllocation::into_bound_ptr_at_zero` descriptor adapter and B3 variant
`deallocate_bound_vec`. Astra reviewed both exact contracts/native bodies.
Only the converter mints Some binding; unbound construction and nonnull
metadata getters are body proved. B3 still requires full matching affine
authority. The actual extracted constructor and proof-only explicit cleanup
are body proved and introduce no trusted bytes protocol. Removing converter
trust by storing NonNull inside the existing B1 boundary is the next gate.

The later NonNull checkpoint removes converter trust: B1 creates NonNull
inside its existing Vec/raw boundary; conversion merely moves that field and
ghost metadata and is body proved. Current physical trusted functions are
detach_vec, resume_vec, deallocate_vec, borrow_mut, and the deallocate_bound_vec
variant. The exact constructor/explicit-release source gate passes 35 files.


The sealed physical retired-pool core adds no trusted functions. Its seed,
retirement, full-coverage extraction and supporting RA/extractor lemmas are
body proved in the 23-file local gate, over the existing physical primitives.
This does not establish native Shared/refcount or automatic Drop integration.


The sequential registry adds no custom trusted declarations. The vanilla
NonAtomicInvariant/resource framework is used under its generic contracts;
ticket conservation, region retirement, mutable public progress and consuming
finalization are body proved in the 27-file local gate. This does not add a
contract for native AtomicUsize or prove native Shared/automatic Drop.

## Sequential native atomic bridge C1-C4

`src/ownership_proof/sequential_counter.rs` adds four explicit trusted primitive
contracts: construction of a sealed atomic identity and exclusive CounterOwn,
nonwrapping native Relaxed fetch_add, nonwrapping native Release fetch_sub, and
native Acquire load under matching exclusive CounterOwn. These are additional
TCB assumptions, not body-proved atomics. CounterOwn is affine, NotObjective,
and cannot be sent or shared across threads. The cell is private; a token from
another counter cannot authorize an operation. No operation returns byte
permission, tickets, a finalizer, or control-block ownership.

The two positive caller bodies prove exact scalar transitions. Wrong-identity
and underflow configurations each reject one designated precondition. This
bridge has no release-sequence or visibility model and does not verify concurrent
refcount operations. It leaves the existing standard atomic and Vec models
unchanged. The next Shared gate must prove ticket/region conservation separately
and connect the actual native old-count branch to that conservation.

The native SharedBuffer refactor is runtime preparation, not an additional
proved physical bridge. Its reserve, take_vec and automatic Drop bodies have native test coverage
and remain outside the proof gates. The restricted first promotion path is
now body proved in the actual split gate.

## Sealed pointer advance and explicit Shared control gate

The existing B1 trusted invariant additionally guarantees that the allocation's
numeric base address plus capacity fits in usize. A bound descriptor preserves
that fact for its remaining extent. The body-proved advance_within uses the
original pointer's wrapping_add and updates only its sealed offset. Numeric
addresses never create access or recovery permission. The native transformation
is audited as provenance-preserving Rust pointer arithmetic; the numerical
extern spec does not itself prove a provenance relation.

The restricted source control helper joins the physical pool, affine tickets,
exclusive sequential counter ownership and a real typed Shared allocation.
Construction/release/callers are body proved; no protocol theorem is trusted.
Standard Perm::from_box/as_ref/as_mut/drop are the S ownership boundary. Before
B3 frees A, the native SharedBuffer descriptor is disarmed. Perm::drop then
consumes full S permission in ordinary code; its standard contract does not
expose a formal deallocation event or prove automatic Drop effects. The native
allocation-event test separately observes both frees. No new local S-free axiom
is introduced. The earlier control-only gate excludes existing promote/split/release_shared;
the subsequent 61-file actual split gate connects the first promotion and
split_to, with explicit consuming release. Existing release_shared and automatic
Drop remain outside both gates. Unsupported atomic proof adapters have false
preconditions and supply no facts.


The archived 61/62-file first-split gate omits the original whole-type unsafe
Send/Sync impls from extraction and does not establish concurrency safeguards.
The subsequent mutation work gates those impls out of the restricted proof
representation; ordinary native BytesMut remains Send/Sync. This adapter change
is not retroactively included in the frozen first-split proof snapshot.

The readonly-access/lifetime-diagnostic archive records Rust E0505 rejecting
release of a handle while a later read keeps its as_slice borrow live. This is
translation/borrow-check evidence only, not a Why3 VC or automatic Drop proof.

The storage-ops prefix fill/copy loops are body proved with standard
MaybeUninit<u8> contracts (four proof files, nine goals, native four tests). They
add no local trusted primitive. Their separate gate does not prove runtime
resize/extend callers until those callers are connected and verified.

The 93-file unique-access gate reuses B4-read/B4-bound/B4-uninit with the full
constructor region. It adds no physical primitive. The canonical Known clause
on B4-bound follows from its existing Some(Some(final_byte)) clause; it is not
an additional physical assumption. Unique cleanup consumes full Recovery and
capacity coverage at offset zero; deallocation does not read Unknown u8 slots.
Sixteen native tests include exact unique A-free counts with no S allocation.
Global type invariants, traits and unique advance remain subsequent work.

The unique-access/negative-unknown-publication archive records exactly one
failed Known-prefix guard in actual set_len after unique spare storage is
re-uninitialized (94 proof files; caller 13/14). Full allocation ownership and
Some(None) are established before that attempted publication.

The deref-purity-feasibility diagnostic confirms two existing vanilla 0.13
translation restrictions: ordinary Deref must be ghost, and a ghost Deref
cannot call program(terminates) access operations. Native Rust checks the
reduced example. These are compiler diagnostics, not rejected Why3 VCs or
proof of actual BytesMut Deref. Current physical B4 operations remain ordinary
program operations; no ghost writeback or trait-body axiom is introduced.

The completed open-invariant-borrow reduced gate rejects nine mutable
reborrow/resolve invariant leaves across three functions. Even with open_inv
arguments and open_inv_result, an intentionally invalid intermediate value
cannot cross those generated checks in vanilla 0.13. The actual strong
lifecycle-invariant attempt is not claimed proved; valid-handle-only trait
verification is being separated from the explicit-predicate lifecycle gate.
Canonical reduced source/configuration and failed proof leaves are archived
in open-invariant-borrow/rejected, without weakened invariants or new trust.
