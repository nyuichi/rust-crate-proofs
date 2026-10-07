# Inventory of the original bytes 1.11.1 verification work

Audit date: 2026-10-07 (corrected after the initial 06:59 draft). This is a read-only inventory of the pristine-source goal, the isolated proof modules, the exact-source component gates and retained historical diagnostics. It does not add proof results and does not treat route 1's modified API as evidence that the original API is complete.

## Finding and evidence rules

The original full goal is **not proved**: the original public `Bytes`, `BytesMut`, `Buf`, and `BufMut` APIs have not been verified as one integrated implementation, and original shared ownership/refcount/last-owner recovery plus automatic `Drop` are still outside the proof. Many useful source bodies and callers are proved under narrow contracts, extraction, or proof-only cfgs; these are real body proofs for those generated configurations, not proof of all behavior reachable through the ordinary public API.

The pristine source reference for this inventory is the flattened bytes crate baseline identified by the task as `e678b774`. Current proof receipts are historical snapshots, so archived proof-source hashes must be checked against today’s files before calling a proof an exact current-source proof. Root’s read-only parity receipt `historical-component-source-parity.json` reports 33 direct source hash checks, with 28 matches and five mismatches: `src/capacity_ops.rs`, `src/endian_ops.rs`, `src/provenance_specs.rs`, `src/signed_wide_ops.rs`, and `src/variable_read_ops.rs`. The 194-symbol ledger `historical-component-proof-symbols.csv` labels every row as archived isolated body evidence under contract, with `integrated_original_runtime=false`; it also records per-symbol current direct-source hash parity. A mismatch means the archived source hash is not current; semantic equivalence must not be inferred without a separate comparison. The baseline receipt `verification/artifacts/baseline-manifest.json` independently identifies bytes 1.11.1's upstream VCS commit `417dccdeff249e0c011327de7d92e0d6fbe7cc43`, archive SHA-256 `1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`, and the original-import head. Its source hashes show the baseline runtime files correspond to upstream, before the verification annotations and helper modules. The recorded surface is only `std;x86_64-unknown-linux-gnu`, excluding `serde` and `extra-platforms`.

Do not read `PUBLIC_API_COVERAGE.csv`'s 997 rows as 997 verified API methods. `surface-manifest.json` says those are rustdoc compiler items. It separately records 67 isolated exact-source helper rows and warns that contracts condition proofs and do not imply caller integration. The public API rows in the matrix remain `not_started` even where later narrow gates extract some of their bodies. `src/verification.rs` is a logical state-machine model, not the upstream implementation.

Proof-file/VC totals are archive accounting, not method counts or a percentage. At the historical `60e81ad` checkpoint, the record is 19 component targets, 184 Coma/proof files, 540 named VCs, and 38 negative controls. The refreshed component archive has 21 targets, 194 files, 581 named VCs, and 39 negative cases; dependencies and their VCs repeat across configurations. The old checkpoint describes its own revision and must not be blended with the refreshed totals. The refreshed manifest explicitly has `complete_verification: false` and `integrated_runtime: false` for every component.

## What exists, and what kind of evidence it is

### Isolated body-proved component corpus

The 21 component archives prove exact helper bodies against their contracts in isolated crates or source selections. They are useful building blocks; they are not all functions from the upstream public API, and a helper's proof does not prove an enclosing public caller. The counts below are proof/Coma files, not unique methods.

| Component | Files | Main evidence category |
|---|---:|---|
| `helpers` | 5 | `minimum`, min/saturating-subtract arithmetic and representative cases |
| `storage` | 2 | basic storage helpers |
| `deallocation` | 2 | isolated boxed-slice cleanup helpers |
| `bounded-ops` | 5 | bounded lengths/chunks, prefix and budget updates |
| `slice-ops` | 4 | slice cursor/copy arithmetic |
| `cursor-ops` | 10 | position, remaining, chunk and advance arithmetic |
| `byte-codecs` | 7 | u16/u32 encode/decode and round trips |
| `comparison-ops` | 3 | logical sequence equality/order helpers |
| `chain-ops` | 5 | chained lengths and splitting arithmetic |
| `capacity-ops` | 14 | packed metadata/capacity arithmetic, not allocator capacity |
| `slice-read-ops` | 12 | checked fixed-width reads |
| `slice-wide-read-ops` | 18 | checked wide reads |
| `variable-read-ops` | 14 | variable-width read arithmetic/body helpers |
| `initialized-storage` | 13 | initialized byte copy/fill/write callers and frames |
| `uninit-ops` | 4 | local `MaybeUninit` operations |
| `wide-codecs` | 14 | wide integer codecs |
| `endian-ops` | 27 | endian encode/decode/read/write helpers |
| `signed-wide-ops` | 24 | signed wide reads/codecs |
| `region-permissions` | 1 | disjoint Box-region permission caller |
| `provenance-ops` | 8 | pointer address/tag arithmetic helpers |
| `ownership-frontier` | 2 | disjoint borrowed-region writes and a caller |

The authoritative per-target input hashes, actual function names and proof tasks are in `verification/artifacts/component-evidence/<component>/component.json`; the sum and archive status are in `verification/artifacts/checkpoint-manifest.json`. These results range from arithmetic/codec helpers to real disjoint borrowed regions, but none is the original `Bytes`/`BytesMut` protocol. The region algebra `OwnedRegion`/`KernelRA` proof is likewise a pure ledger proof; its model constructor does not identify or authorize a native allocation.

### Exact-source public implementation fragments

Later gates extracted unchanged runtime method bodies or proved cfg-adapted copies with body correspondence checks. Added proof contracts/configuration narrow the accepted input state. Explicit coordinator/ticket parameters and explicit cleanup paths are proof adapters; they are not automatically supplied by the original public Rust API. The most useful API-shaped evidence is:

| Original source API or body | Existing result | Exact claim boundary |
|---|---|---|
| `BytesMut::new`, `zeroed`, `From<&[u8]>`, internal `from_vec` | Constructor extraction: 62 files; the later exact constructor/explicit-release gate is 35 files. `actual-with-capacity` replays 101 files plus a native case. | Exact selected source bodies under their extracted contract; requested capacity uses an added generic capacity model. This does not verify every constructor, promotion, or automatic destructor. |
| `BytesMut::{len,is_empty,capacity}` | Selected exact observer gate: 3 files, native 2. | Checked initialized-unique/canonical-empty invariant. It does not imply `Buf` dispatch or arbitrary handles. |
| `BytesMut::{spare_capacity_mut,AsRef,AsMut}` | Safe-trait gate: 73 files; current carrier variant 116 files. | Limited initialized/registered ownership states; `DerefMut`, public open-trait laws and scope-exit `Drop` remain excluded. |
| `BytesMut::{split,split_to,split_off,advance_unchecked}` and helper transitions | Legacy split/storage: 111 files; coordinator carrier: 114–116 files across captured revisions. | Selected real source bodies connected by a proof-only affine coordinator and explicit cleanup. Repeated splits, endpoints, zero capacity and selected relocation paths are covered. This is not arbitrary unmediated public dispatch or concurrent access. |
| `BytesMut::{truncate,clear,set_len,resize,extend_from_slice}` | The actual split-off/length-change extraction covers `truncate`, `clear`, and `set_len` in its 77-file gate. Separate gates cover in-capacity `resize`/`extend_from_slice` (103 files), unique growth (124), and Shared growth (115). | `set_len` requires every published byte Known; initialization/spare writes have explicit frames. These are different scopes from the 6-file `Bytes::{truncate,clear}` gate below. Growth and no-growth gates have their own capacity/registration conditions; panic, allocation failure and automatic `Drop` are not derived. |
| `BytesMut::{reserve,try_reclaim}` | Exact cfg unique reserve: 120; bounded Shared reserve: 106; bounded Shared no-allocation `try_reclaim`: 108. | Narrow singleton/nonunique/registered states, explicit capability movement and cleanup. Reallocation is mediated through B5; arbitrary allocator-failure semantics and complete public API are not covered. |
| `BytesMut::{freeze,unsplit}` | Restricted freeze/read/recovery: 64; adjacent same-control `unsplit`: 101; same-control fallback: 115; independent-control fallback: 114; mixed Unique/Shared and both-Unique fallback scopes: 121 each. | Separate cfg gates with tickets and explicit retirement. They do not prove ordinary Clone/vtable behavior or automatic `Drop`. |
| `BytesMut::Drop` and release bodies | `release_unique_storage` and selected `release_shared` cleanup bodies pass on explicit consuming call paths (e.g. carrier 114–116 files). | Calling a cleanup function is body-proved; Rust scope-exit destructor dispatch/effects are not. No claim that these gates prove automatic `Drop` or exactly-once last-owner behavior. |
| `Bytes::{len,is_empty}` | Exact metadata-only bodies pass 2 files. |
| `Bytes::{truncate,clear}` | Nonpromotable exact bodies/callers pass 6 files with a table-tag precondition. | Neither the promotable path, all table callback values, nor `Bytes` ownership is covered. |
| `Bytes::{new,from_static,from_owner,Clone}` and shared read traits | No full positive constructor/clone gate. Exact `new`/`from_static` attempts and actual Clone dispatch remain translation failures; helper comparisons and selected cfg read paths exist. | Not integrated. Static/owned vtable callback cycles and indirect function-pointer dispatch fail translation; affine ticket splitting through `Clone::clone(&self)` fails the borrow constraints. `Bytes` Send/Sync and default Deref purity also block full-source translation. `from_owner` has no whole-API proof claim in the matrix. |
| Concrete `impl Buf for &[u8]` | Latest concrete-slice gate proves 82 files: 19 checked integer readers, 19 normal getters, `remaining` and `chunk`; variable unsigned widths 0..=8. Earlier matrix records the 40 core bodies. | Exact concrete slice bodies. It does not verify arbitrary `B: Buf`, trait refinement, `Bytes` or `BytesMut` trait dispatch, floats, all native-endian paths or adapters. |
| Concrete `BufMut` bodies on `&mut [u8]` and `&mut [MaybeUninit<u8>]` | 16 files for selected `remaining_mut/chunk_mut/advance_mut`; native 3. Predicate/default probes are separate. | A local one-implementation interface proves bounds and current/prophetic views; the unsafe “these bytes are initialized” promise and public open-trait law are excluded. |
| `Take`/`Limit`/`Chain`/`Reader`/`Writer` | 28 files for selected constructors, field projections and `into_inner`; native 5. | No generic transfer, adapter iteration, I/O behavior or full `Buf`/`BufMut` composition. |
| `UninitSlice` core/accessors/indexing | The 8-file core gate proves selected view/conversion callers plus `write_byte`, `copy_from_slice`, and `len` under trusted transparent wrappers. Each of six range gates is 12 files with 3 native tests in the recorded configuration: `RangeFull`, `Range`, `RangeFrom`, `RangeTo`, `RangeToInclusive`, and `RangeInclusive`. | `UninitSlice::{new,uninit,uninit_ref}` are `#[trusted]`, not body-proved; `uninit_ref` is included in the expanded accessor configuration. The selected `Index`/`IndexMut` bodies and exact macro correspondence are proved under those wrappers, not as part of the `BytesMut` protocol. Each 12-file total repeats the core/accessor dependencies; do not sum them as unique work or call it a 32-file aggregate. Ordinary `from_raw_parts_mut(ptr,len)` is outside the gate. The separate bound raw constructor passes 42 files only with matching sealed `BoundPtr` and `PhysicalRegion`, and still uses trusted B4. |
| `Vec<u8>`/`BytesMut` cross comparisons | The reversed `PartialOrd<BytesMut> for Vec<u8>` result was fixed in production source with a regression. The exact comparison gate proves 103 files over `PartialEq<Vec<u8>> for BytesMut`, `PartialEq<BytesMut> for Vec<u8>`, `PartialOrd<BytesMut> for Vec<u8>`, and a semantic caller; native tests pass 2/2. | Caller requires a checked initialized unique handle and explicitly closes it. This fixes a real behavior bug and proves those bodies under that gate; it does not prove general `Bytes` comparison, string comparison, full handle ownership or `Drop`. The Oct 5 handoff called the defect “possible”; the later `STATUS.md` and comparison evidence record the actual fix. |

The detailed public-family inventory is `verification/REMAINING_API_MATRIX.md`; gate scope and source receipts are in the linked probe README/evidence manifests. Treat proof counts as counts of generated files only.

### `src/verification.rs` has same-named model methods, not original runtime bodies

Under `cfg(creusot)`, `src/lib.rs` exports the structs and traits in `src/verification.rs` instead of compiling the real runtime `bytes.rs`/`bytes_mut.rs` API. This is a deliberately simplified metadata state machine: `Bytes { len }` models only a length; `BytesMut { len, cap }` models only initialized length and capacity. Its methods and implemented proof-only traits are:

* Model `Bytes`: `new`, `len`, `is_empty`, `split_off`, `split_to`, `truncate`, `clear`; model `Buf::{remaining,advance}`.
* Model `BytesMut`: `capacity_logic`, `new`, `with_capacity`, `len`, `is_empty`, `capacity`, `split_off`, `split_to`, `truncate`, `clear`, `resize`, `freeze`; model `Buf::{remaining,advance}` and `BufMut::remaining_mut`.

Several names match original public methods, but these methods operate only on integer metadata and have no byte array, allocation pointer, `Vec`, `Shared`, refcount, vtable, ownership tickets or destructor. Their proofs were not proof of the same-named original source method. This is the crucial distinction behind the historical `src/verification.rs` proof route.

### Current source is not identical to pristine upstream in the ownership core

The source correspondences above mean the actual current method bodies were extracted/proved in the stated configuration; they do not mean every method body or representation remains byte-for-byte the upstream implementation. `verification/BYTESMUT_OWNERSHIP_IMPLEMENTATION_DESIGN.md` records the intentional representation change, and current `src/bytes_mut.rs` confirms it:

* Upstream `Shared` stored a `Vec<u8>` as the shared buffer. Current ordinary runtime `Shared` stores `SharedBuffer { base: NonNull<u8>, capacity }`; under `creusot`/`bytes_proof_probe`, `base` is the sealed `BoundPtr`. Its promotion path stores a raw allocation descriptor instead of the original Vec value. This changes private runtime representation and is relevant to allocation ownership/release. The `coordinator-carrier-split`, Shared reserve/unsplit and freeze results prove selected **current raw-descriptor source** paths and proof-only registration/coordinator adaptations; they are not proofs of the pristine Vec-backed Shared implementation. They still do not establish original whole-crate correspondence, universal `Bytes` API behavior or automatic `Drop`.
* In ordinary builds, the counter remains native `AtomicUsize` with original Relaxed/Release/Acquire source orderings. Under `creusot` or probe configurations, the `Shared` field is `SequentialCounter`; its exact operations rely on C1–C4 trusted scalar contracts. Thus those sequential protocol VCs are not a proof of the ordinary native atomic implementation or its concurrent weak-memory behavior.
* Current source also introduces proof-only cfg state, proof contracts and helper modules/entrypoints (`BoundPtr`, affine tickets/coordinators and explicit close). These add proof interfaces to generated current-source configurations. Runtime-body extraction and native regression help audit correspondence, but don't remove the storage/counter distinction above.
* The ordinary production `SharedBuffer` pointer descriptor is a runtime change, not only a ghost annotation. Therefore the body proofs for `release_shared`, reserve/unsplit/freeze etc. must be labelled “exact current extracted source under this proof configuration,” not “unchanged upstream body proved.” Any future claim against the pristine API needs an explicit source/refinement argument for this descriptor representation or a proof over the original Vec-backed path.

These representation changes do not themselves constitute bytes-specific trusted axioms: they are changed code. They do change what source the proof establishes, so they belong in the correspondence audit even if public signatures remain unchanged.

### Body helpers, protocol bodies, models and diagnostics

* `verification/probes/scalable-tickets/` proves ticket/map and pending-region resource operations as body-checked logic/resources; `shared-protocol`, `sequential-retirement-registry`, `retired-region-pool`, and carrier probes prove restricted retirement, split/join, registration and cleanup sequences. The verified protocol code is not a trusted bytes/refcount theorem. It composes above the primitive boundaries described below, and much of it is a coordinator/client abstraction rather than ordinary public dispatch.
* The `shared-control`/control-block lifetime result (six files) proves a real `Box`, `Perm`, `FullBorrow`, `GhostShared` and fractional lifetime-token exercise. It does not prove bytes' atomic refcount, Release-sequence visibility, byte-region recovery or automatic destructor integration.
* `verification/probes/raw-buffer-native/` tests native unsafe Vec detachment/recovery/free behavior. This harness has no Creusot trusted contract and is not connected to the public `BytesMut` source or protocol.
* `verification/probes/drop-feasibility/` is a diagnostic: an explicit cleanup call proves, while the same effect through scope-exit `Drop` is absent from the generated caller. This is not a positive destructor proof.
* `verification/probes/vtable-feasibility/` and `native-integration-frontier/` are translation diagnostics, not failed Why3 proof obligations: recursive vtable call graphs, unsupported indirect function-pointer calls, `Send`/`Sync` marker constraints, `Deref`/`DerefMut` purity and `&self` affine-splitting are preserved as concrete limits.
* `verification/probes/handle-comparison/` adds a total sequence view only in generated probe copies for checked unique handles; the view's zero fallback for Unknown slots is only totalization and grants no initialization or ownership.
* Public `Buf`/`BufMut` default predicate probes extract exact defaults and prove selected concrete implementations through sealed local interfaces. They explicitly do not create contracts for downstream implementations of the open public traits. `D03` rejects trusting such universal laws.
* Native `cargo test`/`no_std` passes (including later 1,256 ordinary tests at the recorded checkpoint) demonstrate build/behavior. They do not fill a missing formal source/API proof.

## Current trusted boundary inventory

The audited local ownership boundaries are enumerated in `verification/RAW_VEC_TRUSTED_BOUNDARY.md`, `verification/OWNERSHIP_FEASIBILITY_RESULTS.md`, and source `#[trusted]` declarations. The current inventory contains generic physical/scalar trusted bridges; the bytes-specific ownership, ticket, retirement and refcount protocol bodies are not trusted. This is a factual description of the current code, not a rule that such trust is permanently forbidden.

| Boundary | Trusted item(s) | What the contract supplies | What it does not supply |
|---|---|---|---|
| B1 — detach | `src/ownership_proof/raw_vec.rs::detach_vec` | Consumes the actual `Vec<u8>`, suppresses its destructor, binds native base/capacity and original initialized prefix to affine Recovery/full `PhysicalRegion`; spare slots become `Unknown`. | No alias/refcount/Bytes handle fact; not an ordinary source-body proof. |
| B2 — resume | `raw_vec::resume_vec` | Consumes matching sealed descriptor, Recovery, exact full `[0,capacity)` coverage and Known requested prefix; rebuilds Vec whose sequence is current Known slots. | No arbitrary pointer substitution, sharing or protocol fact. |
| B3 — explicit deallocation | `raw_vec::{deallocate_vec,deallocate_bound_vec}` and reviewed allocator leaf | Consumes matching Recovery and exact full coverage; deallocates the byte allocation via the audited native allocator/layout path; no initialized-prefix requirement. | Does not establish formal automatic `Drop`, final-owner status, control-block cleanup or native `drop` semantics. |
| B4 — physical byte access | `raw_vec::{borrow_mut,borrow_bound_mut,borrow_bound_uninit_mut,borrow_bound,borrow_empty_bound,borrow_empty_bound_mut,borrow_empty_bound_uninit_mut}` | Contracts relate raw slice lifetime, sealed pointer/region identity, exact range, Known reads or `MaybeUninit`, writeback and outside-region frame. Zero-sized paths make no liveness claim. Immutable read classification is separately checked; mutable operations remain real program effects. | The physical slice creation/body is not proved; no handle/refcount law. Marking mutable B4 ghost is rejected by a native erasure counterexample. |
| B5 — reallocate | `raw_vec::reallocate_bound` | Consumes original full-capacity authority, requires offset-zero base and strict growth bounded by `isize::MAX`; preserves old Known/Unknown slots and gives new tail `Unknown`. | No pointer freshness theorem, Bytes ownership/refcount, sharing or failure-path proof. |
| B6 — bound offset | `BoundPtr::offset_from_bound_base` | For sealed descriptors in one namespace/capacity with offset-zero base, returns the represented offset via native numerical subtraction. | No provenance/liveness/permission or allocation identity from address equality. |
| C1–C4 — sequential scalar counter | `SequentialCounter::{new,fetch_add_relaxed,fetch_sub_release,load_acquire}` in `sequential_counter.rs` | Exact exclusive scalar value/resource binding for those native operations under a `CounterOwn` ghost authority and nonwrapping premises. | No release-sequence, cross-thread visibility, atomic adequacy or byte-region/reclamation rule. This is explicitly a sequential bridge. |
| Vec allocation/capacity | `vec_capacity::with_capacity_bound`; private std `capacity_model<T,A>` plus Vec `with_capacity/capacity/reserve/reserve_exact` contracts in `verification/std-support/alloc-capacity.patch` | Opaque generic capacity observation keyed to that actual Vec; requested capacity fact and capacity monotonic/lower bounds. | No Seq-to-allocation identity, pointer permission, provenance/injectivity or allocator totality. |
| Vec base observation | private std `base_model<T,A>`/`as_ptr`/`as_mut_ptr` contracts in `address-model.patch` | Opaque numerical base-address observation for that Vec and pointer/address correspondence. | No provenance identity, allocation liveness or permission. |
| Vec growth/allocation externs | generic std contracts in private `alloc-capacity.patch` | Capacity and unchanged sequence on reserve; relevant `Vec`/`Box`/slice/String/convert/slice-spec exposure in the private package. | No bytes-specific ownership fact; existing infallible extern termination assumptions are inherited. |
| Pointer arithmetic | `STD-PTRWRAP-01` in `src/provenance_specs.rs` | For one-byte pointees, numerical wrapping address relation. | No logical pointer identity, provenance, dereference permission, liveness or `PtrLive`. |
| Integer conversion | `STD-CONVERT-01` in `src/std_specs.rs` | `u64 -> usize` TryFrom result relation and overflow branch. | No memory or protocol premise. |
| Box representation | `boxed_alignment::into_raw_aligned<T>` | `Perm::from_box` result preserves the value/permission and native pointee alignment. | No refcount, tagged-handle, byte-region or recovery theorem. |
| `UninitSlice` reinterpretation | `UninitSlice::{new,uninit,uninit_ref}` in `src/buf/uninit_slice.rs` | Trusted transparent slice reinterpretation with length and element-view correspondence. | These wrappers are assumptions, not body-proved casts. They grant no provenance from the ordinary raw-pointer constructor, no ownership capability and no blanket initialized-state publication. |

`RawAllocation::into_bound_ptr_at_zero` was trusted in an earlier checkpoint, but the current boundary record says B1 now stores `NonNull` directly and this constructor is body-proved; do not list it as current TCB. Likewise B4's `#[check(ghost)]` classification is allowed only for immutable read bridges; mutable B4 is never trusted as ghost-observable.

Root’s current trust ledger `/tmp/bytes-current-trusted-functions.json` lists 27 source `#[trusted]` declarations: B1–B6, four sequential C1–C4 methods, the three `UninitSlice` casts, `into_raw_aligned`, `with_capacity_bound`, and five later route-1 atomic items above. The private std overlay adds separate generic trusted contracts for opaque Vec `capacity_model`, opaque `base_model`, and `try_spawn_with_slot`; do not mix those with bytes-owned protocol trust.

### Later route-1 TCB that must not be confused with the original checkpoint

The current workspace contains new modified-variant code and private standard contracts. Keep it separately classified from the pre-handoff/original API work. `src/verified/atomic.rs` has five later route-1 generic atomic boundaries: `NativeAtomic::{new,decrement,acquire}`, `release_rmw`, and (under `extra-platforms`) `fence_acquire`. They are not bytes-specific ownership theorems, but are later route-1 TCB and do not make the original implementation verified. The private std overlay also has generic `try_spawn_with_slot` (`F: Copy`) in `copy-slot.patch`, which is a generic thread-slot frame contract; it does not prove unrestricted callbacks or Bytes ownership. These rules are reviewed/modified-target premises, not evidence of original `Bytes` API completion.

## Generic future replacements for the local physical/scalar rules

No such replacement is currently proved. The following are local trust-removal directions, not new rules to add now. A verified generic primitive could reduce the TCB while leaving the bytes protocol code untouched; it must retain a real native implementation and prove the stated resource/pointer relation. Removing these generic assumptions would still not solve open-trait refinement, vtable translation, source integration or automatic `Drop`.

| Local trust | Future generic verified rule/API needed |
|---|---|
| B1/B2 | A verified owned-allocation/Vec raw-parts transfer and reconstruction interface connecting actual allocator, pointer provenance, capacity, initialized prefix, exact full resource and destructor suppression/restoration. A pure sequence or integer-address model is insufficient. |
| B3 | A generic allocator deallocation primitive consuming the matching allocation/layout and ownership authority, with exact normal return/free behavior. This is distinct from Rust automatic `Drop`. |
| B4 | Verified raw slice construction and access primitives preserving provenance, lifetime/aliasing, permission, byte-value ledger and outside-range frame for shared, mutable and `MaybeUninit` views, including sound empty references. |
| B5 | A generic allocator reallocation contract consuming old allocation authority, retaining old slot/init facts, describing new unknown bytes and matching the native old/new layout; allocator failure remains a separate exceptional behavior. |
| B6 plus `STD-PTRWRAP-01` | A provenance-aware pointer model for deriving/bounding offsets and advancing the original allocation pointer. Numerical address equations must never mint allocation identity or access permission. |
| C1–C4 | Generic contracts/body proofs for native atomic new, Relaxed add, Release sub and Acquire load, including a sound weak-memory/release-sequence resource rule if used concurrently. Scalar arithmetic alone cannot prove last-owner resource recovery. |
| `with_capacity_bound`, `capacity_model` | Standard `Vec::with_capacity/capacity/reserve` contracts proved from the standard allocator/Vec implementation or an accepted generic allocation abstraction. Preserve that the observed capacity is for this exact Vec. |
| `base_model` | A verified standard Vec base-pointer observer relation with provenance/pointer authority; an integer address observer alone does not replace B1 identity. |
| `into_raw_aligned<T>` | Verified generic Box-to-raw-permission conversion contract including alignment and exact ownership transfer. |
| `UninitSlice::{new,uninit,uninit_ref}` | Verifier support or verified generic repr(transparent) slice-cast rule with exact length/element view and lifetime, so the current trusted wrappers can be removed without granting raw-pointer authority. |
| `STD-PTRWRAP-01`, `STD-CONVERT-01` | Stock verified contracts for one-byte raw-pointer wrapping's numerical result and checked integer conversion, with pointer provenance/access facts kept separate. |

The raw-descriptor representation, proof-only `BoundPtr`, explicit coordinator/tickets and proof-only sequential counter are source/configuration changes rather than trust items. To claim the pristine Vec-backed path, either verify that path directly using generic Vec ownership operations or prove complete source refinement from original Vec semantics to `SharedBuffer`, including allocation identity, spare initialization, reserve/reclaim, conversions, destructor suppression and every public caller. Separately, the proof-only scalar counter must be replaced for a concurrency claim by a sound generic atomic model for the native operations. This is a source-correspondence requirement, not a new trusted rule to add.

The latest user instruction supersedes the older D04 blanket policy on whether a bytes-specific protocol boundary may ever be temporarily trusted: temporary local protocol trust is authorized only if it is minimal and has an explicit future proof/discharge path that permits local removal. No bytes-specific ownership/refcount/retirement contract has been added in the source inventory audited here. This report makes no new trust proposal; it identifies current generic TCB and future removal interfaces. Underlying Creusot resource/permission/pointer/invariant model assumptions and Rust/allocator foundations remain tool/library TCB.

## Original full-goal remainder

The original target includes all supported public methods/trait implementations, conversions, comparisons, iterators, adapters, `Bytes`/`BytesMut` ownership and clone/refcount paths, ordinary cleanup, and actual configuration behavior. Current evidence does not discharge:

* complete `Bytes` construction, callback/vtable clone, shared reads and reference-counted ownership; the vtable calls fail translation before VCs;
* `Bytes`/`BytesMut` public `Buf` and `BufMut` dispatch or universal laws for downstream implementors; open-trait trust is rejected;
* fully unmediated `BytesMut` split/promotion/reserve/reclaim/unsplit lifecycles; existing real-body results require restricted cfg/ticket/coordinator contexts and explicit consuming release;
* original scope-exit/automatic `Drop`, including user-visible final-owner reclamation. The explicit cleanup bodies remain valuable but are a changed call pattern;
* concurrent refcount/resource conservation, publication and Release-sequence synchronization under the original atomics/handles;
* full `serde`, `extra-platforms`, `no_std` API proof, all target widths/alignments, unwinding, allocation failure and panic behavior. Native tests or build checks are not proofs of these cases.

The later route-1 target is a changed `bytes::verified` API/representation with its own source correspondence and configuration gates. `STATUS.md` explicitly says it does not claim the original public API or automatic `Drop` fully verified. Those route-1 positives must not be added to this original-target inventory as if they close the items above.

## Primary receipts

* `verification/REMAINING_API_MATRIX.md` — original public API families, exact body scopes and remaining boundaries.
* `verification/THREAD_HANDOFF_2026-10-05.md` — source checkpoints, prior exact-body gates and explicit cleanup distinctions.
* `verification/RAW_VEC_TRUSTED_BOUNDARY.md` — B1–B6 physical bridge contracts and proof scopes.
* `verification/OWNERSHIP_FEASIBILITY_RESULTS.md` — vtable, automatic-Drop, control-block and physical allocation findings.
* `verification/probes/actual-uninit-slice/README.md` and its archived evidence — core/accessor/range gates, raw-constructor boundary and deinitialization negative.
* `verification/probes/handle-comparison/evidence/vec-traits-refinement/README.md` — actual Vec/BytesMut comparison bodies, fix and native regression.
* `verification/probes/native-integration-frontier/README.md`, `verification/probes/drop-feasibility/README.md`, `verification/probes/vtable-feasibility/README.md` — exact failure classifications.
* `verification/artifacts/baseline-manifest.json`, `surface-manifest.json`, `checkpoint-manifest.json`, and `artifacts/component-evidence/*/component.json` — source/configuration identity, inventory semantics and component receipts.
* `verification/ARCHITECTURE_DECISIONS.md` — records historical D04 and still-relevant D03/D06/D07 boundaries. D04’s blanket prohibition on temporary bytes-protocol trust is superseded by the latest user authorization; the factual TCB inventory above records that no such new contract has been added. The open-trait and integrated-proof cautions remain relevant.
