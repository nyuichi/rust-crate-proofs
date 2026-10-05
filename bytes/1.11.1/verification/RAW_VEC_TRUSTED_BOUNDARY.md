# Local Vec/raw physical bridge

This bridge keeps Creusot 0.13's ordinary `Vec<u8> -> Seq<u8>` model unchanged.
It is an isolated, single-threaded target, not a BytesMut ownership proof.
The compiler and creusot-std are unmodified.

## Physical interpretation

`RawAllocation` contains the actual allocation base and capacity extracted from
the consumed global-allocator `Vec<u8>`, plus a private ghost namespace. It has
no Copy, Clone, Deref or Drop implementation and no public constructor. The
namespace identifies this detached ownership instance; it is not an address.
Pure slot ledgers and integer metadata cannot construct a physical capability.

The bridge assumes the original pointer's provenance, non-nullness and byte
alignment, and the original allocator/layout pair. For positive capacity the
layout is the original Vec's byte capacity with alignment one; capacity zero
has a valid dangling pointer and no allocation to free. These are audited Rust
representation facts, not consequences of namespace equality or the map algebra.

Recovery owns the unique exclusive recovery marker and no byte slots. A
PhysicalRegion owns its exclusive interval of slots in that same resource
namespace. Known(x) means that the physical byte is initialized and has value x;
Unknown means that no initialization/value claim is available. Both physical
capabilities are non-objective. A Recovery or nonempty region protects the
allocation from recovery/free; an empty map fragment grants no allocation
liveness or byte access. Forgetting ghost tokens can leak resources and does
not count as deallocation.

## Reviewed B1/B2 contracts

- **RVB-01 / B1, `detach_vec`:** consumes the ordinary Vec, disables its
  destructor, retains its actual pointer/capacity and returns runtime length
  plus ghost Recovery/full-region capabilities. The old initialized prefix
  preserves exactly the Vec sequence; spare capacity becomes Unknown.
- **RVB-02 / B2, `resume_vec`:** consumes the sealed native descriptor and both
  capabilities. Namespace and capacity must match, coverage must be exactly
  `[0, capacity)`, and every requested prefix byte must be Known. The restored
  Vec sequence equals those current slot values, not a saved initial sequence.

Astra reviewed the saved declarations and native bodies. B1/B2 remain trusted
physical primitives; their body contracts are not solver-proved. Their eventual
replacement would require equivalent owned raw-allocation primitives supplied
by the standard model. There is no temporary trusted bytes-specific protocol.

The B1/B2 probe proves ten generated files, including the physical split/join
bodies and a helper-returned pair rejoined and resumed with unchanged contents.
Its exact source snapshot and generated tasks are retained under
`artifacts/evidence/raw-vec-bridge/b1-b2-*`. This gate establishes no mutation,
deallocation, automatic Drop effect, Shared protocol or BytesMut integration.

## Reviewed B4 access gate

**RVB-04 / B4, `borrow_mut`:** derives its pointer from the sealed native
descriptor, accepts no caller-supplied pointer, and ties the returned reference
to the raw and mutable ghost borrows. Its contract preserves identity/bounds
metadata and every unborrowed slot. Astra reviewed the exact declaration and
caller. B4 remains a trusted physical access primitive.

The nonempty initialized-slice caller holds both disjoint mutable references,
writes different bytes, joins the regions and resumes a Vec with the changed
contents and unchanged remaining bytes. Eleven generated files pass; two native
probe tests pass. Exact source and tasks are retained in `b1-b2-b4-source` and
`b1-b2-b4-positive`, with a separate hash manifest. This is contract composition
and caller body proof, not physical primitive body proof or BytesMut integration.

## Reviewed B3 explicit cleanup gate

**RVB-03 / B3, `deallocate_vec`:** consumes the sealed descriptor, matching
Recovery and exact full-region coverage. It requires no initialized prefix.
Its native length-zero Vec destruction is part of the audited physical TCB,
not a consequence of the weak `mem::drop` contract. Astra reviewed the exact
declaration and body.

Fourteen generated files pass with the cleanup callers. Four native probe
tests cover identity recovery, disjoint mutation, and explicit cleanup in normal
and reversed fragment tuple return order. Cleanup includes empty Vecs with zero
or reserved capacity and nonempty split Vecs. Reordering returned fragments is
not a proof of either automatic Drop order or refcount retirement. Exact source
and tasks are retained under `b1-b2-b4-b3-*` with a separate manifest.

## Compiler-model restriction

### B5 physical reallocation (unique-growth gate)

`reallocate_bound` adds a local physical allocator contract. It consumes
matching Recovery and full capacity coverage, requires an offset-zero base and
strictly greater capacity bounded by `isize::MAX`, and returns the replacement
allocation's complete authority. Existing slots preserve their current Known
or Unknown state; newly allocated slots are Unknown. The native leaf uses the
global byte allocator's `alloc`/`realloc` and matching old layout. Failure follows
Rust's allocation-error path and is outside normal-return proof.

No bytes handle, refcount, sharing, or final-owner rule is trusted by B5. It
does not promise distinct resource IDs or pointer addresses. Strict growth
ensures old descriptor capacity cannot match the returned capabilities. Equal
capacity reallocation is deliberately excluded. See the exact body gate and
negative controls in `probes/unique-growing-reserve`.

Native raw-pointer Eq/Ne is translated as logical pointer equality/inequality.
The paired diagnostic demonstrates that address-only `addr_eq` does not grant
that identity. All proved caller paths must therefore exclude native pointer
Eq/Ne from identity reasoning. The actual try_unsplit comparisons now use an
address-only helper, but that helper is not a proof of try_unsplit.

Sealing the native descriptor removes direct pointer substitution at this bridge.
It does not repair the upstream translation for arbitrary caller control flow.
Logical allocation identity must come from the sealed namespace and resource
ownership, independently of runtime address comparisons.

## Bound-pointer constructor body gate

`BoundPtr` preserves a single native NonNull word; its private proof binding
records namespace, allocation capacity and absolute offset. Arbitrary pointer
metadata can create only an unbound descriptor. Copying the descriptor grants
no memory authority. The native wrapper has transparent representation and
compile-time size/alignment checks; the instrumented proof layout is different.

The initial consuming `RawAllocation::into_bound_ptr_at_zero` adapter is trusted
and reviewed. `deallocate_bound_vec` is the reviewed B3 offset-zero variant:
its sealed binding must match native capacity and full affine capabilities.
Erasing the binding while retaining pointer bits rejects its intended VC.
The initial constructor-helper gate proves sixteen generated files.

A build-time extraction includes the exact BytesMut and Shared declarations,
actual annotated from_vec, ownership predicate, slot projection, explicit
proof-only release, vptr/invalid_ptr and metadata constants. It imports the
actual arithmetic/provenance/resource modules and adds no trusted stubs.
Thirty-four generated files pass, including the source constructor body, its
KIND_VEC/offset-zero and byte-prefix contracts, explicit release and caller.
Release moves out the witness, frees through B3 and forgets the handle,
preventing a second native destructor on this terminal path. Native extraction
tests preserve the allocation/content and the four-word normal BytesMut layout.

This extraction omits other impls and automatic Drop. Actual mutation, reserve
and promotion paths conservatively discard the temporary ghost witness;
unadapted pointer changes become unbound. The predicate is not an invariant
for arbitrary BytesMut handles. Full-runtime translation still fails on the
two vtable cycles; Shared promotion/split/refcount and normal Drop remain open.

The current gate reduces that initial trust: B1 stores NonNull directly and
`into_bound_ptr_at_zero` is now body proved. The helper gate proves seventeen
files and the exact constructor/explicit-release gate proves thirty-five.
B3-bound remains a physical deallocation variant, with no protocol trust.
The initial thirty-four-file snapshot is retained separately as historical
evidence; the current source/task hashes are in
`artifacts/evidence/bound-constructor-body-manifest.json`.

## Next integration gate and limits

The next local gate collects genuine physical region resources into a sealed
retired pool. An empty seed owns only the RA unit; it grants neither byte access
nor allocation liveness. Retirement must consume the original affine resource.
Finishing requires exact full-capacity coverage and preserves slot values.
Allocation metadata or a zero reference count must never mint those resources.

A bounded sequential registry gate may store Recovery and the pool in a
NonAtomicInvariant, with explicit caller-supplied namespace Tokens. Both
retirement orders must recover full resources before explicit B3 cleanup.
Tokens must not be duplicated or recreated by a trusted helper. This gate
cannot establish concurrent use or the native reference-count branch.

Astra's source review found that AtomicInvariant's Objective bound applies to
its Sync implementation, not to local construction/opening. Non-objective
physical state is therefore usable locally. Vanilla 0.13 nevertheless has no
modeled native core AtomicUsize::fetch_sub, and switching invariant kinds does
not supply that missing scalar-value contract.

Actual Shared integration will require replacing its ordinary Vec ownership
with one sealed raw allocation descriptor and connecting every counter access.
A possible local generic sequential atomic bridge would be additional trusted
primitive code, requiring exclusive sealed counter authority and retaining the
actual Relaxed/Release/Acquire operations. It must specify scalar changes only;
tickets, last-owner uniqueness and physical resource recovery must be proved
above it. No such bridge is implemented at this checkpoint. Cross-thread
publication/view transfer and release-sequence reasoning remain separate open
obligations, as do automatic Drop effects and the full-crate vtable cycles.


## Sealed physical pool core gate

The retired-region-pool core snapshot passes 23 proof files. PhysicalPool stores
the actual Resource<KernelRA>, a sealed allocation descriptor, and NotObjective.
empty_from owns only the RA unit. retire consumes PhysicalRegion and transfers
its existing resource; finish(self, zero) requires zero == 0 and exact full
capacity coverage before rewrapping that same resource as PhysicalRegion.
B3 still requires the separate unique Recovery. No physical primitive or bytes
protocol was newly trusted.

A proved product-unit lemma exposes its empty-map value, and map_op_get exposes
pointwise composition. The model extractor explicitly exports marker-None and
interval-domain facts, so the physical caller can use them across the opaque
module boundary. These are proved interface facts, not additional assumptions.

Both retirement orders invoke explicit deallocation. This core gate does not
prove native Shared, mutable access followed by retirement, refcount, tickets,
concurrency or automatic Drop. Capacity-zero completion may use the empty unit,
which grants no byte access; unique Recovery remains necessary to deallocate.

The final extension proves 25 files and passes two native tests. It adds split
at capacity and independent disjoint mutation followed by retirement and B2
recovery, preserving both changed bytes and all other contents. Each negative
feature emits 26 files: half-finish rejects one full-coverage leaf; the assumed
sealed-input namespace API check rejects only the two identity-matching leaves.
Neither negative executes invalid native operations. Per-configuration sources
and full logs are archived separately from the core checkpoint.

A final fresh constructor replay over the current pool and cap-class source
passes 41 files and three native tests. Its one-source snapshot is archived as
`bound-vec-constructor/current-pool-cap-class/`; it replaces the mixed-kernel
limitation for this component gate. Pool helpers are included body proofs, but
actual BytesMut split/Shared/refcount/Drop remain outside this extraction.


## Sequential registry gate

The 27-file gate uses real exclusive registration RA fragments and physical
regions. It returns both kinds of resource before consuming the unique NAI
coordinator and invoking B3. open_mut exposes progress; each operation proves
allocation/registration identity remains fixed. A split-at-zero missing ticket
rejects the finalize guard even though the right region covers all capacity.
Two native tests check erasure and explicit cleanup. No protocol theorem or
new primitive is trusted.

The standard NAI/resource framework remains part of the existing proof TCB.
This gate is neither native Shared nor an atomic counter or cross-thread proof.
Actual Shared raw-storage wiring and native atomic contracts remain integration
work; vtable translation and automatic Drop effects remain upstream blockers.


## Bound-descriptor mutable access

`B4-bound`, `raw_vec::borrow_bound_mut`, is an additional explicitly trusted
physical access primitive. It derives the native pointer directly from sealed
BoundPtr, whose namespace/capacity/absolute offset originate at B1 and are
preserved by bounded pointer advancement. It accepts no caller-supplied pointer
or address-equality substitute. The matching affine PhysicalRegion must cover
the requested interval and every accessed slot must be Known. A nonempty
interval supplies allocation liveness through that owned region. A zero-length
slice requires only the already nonnull u8-aligned pointer; it asserts no
allocation-liveness fact and performs no pointer arithmetic.

The mutable region borrow lasts as long as the returned slice. Its prophetic
postcondition writes back the final slice values, preserves region geometry,
namespace and resource identity, and frames all other slots including Unknown
spare capacity. Split, join, retirement and recovery must wait until this borrow
ends. The bridge relies on the audited provenance-preserving construction and
advance discipline, not the numeric wrapping-add spec alone.

This contract is reviewed. The actual mutable-view, packet-borrow helper and
simultaneous disjoint-write callers pass in the 68-file gate; native execution
passes four tests. The earlier frozen 61-file split checkpoint excludes mutation.
The canonical slot_known structure is body-proved equivalent to existence of
a Known byte; this normalization preserves the physical access requirement.


## u8 MaybeUninit access

The additional reviewed `B4-uninit` physical boundary
`raw_vec::borrow_bound_uninit_mut` derives its pointer from
a sealed BoundPtr passed by value. A matching affine region must own the entire
requested interval. The returned lifetime is bounded by the mutable region
borrow, rather than by a temporary pointer descriptor. The byte ledger's inner
Option maps to standard MaybeUninit<u8>::View, with prophetic final Option
writeback and an exact outside-interval frame. Empty access claims no allocation
liveness. This bridge is deliberately restricted to u8; it assumes no generic
destructor, drop or initialized-type validity rule.

No Known precondition is required to obtain MaybeUninit access. Publishing bytes
through set_len or reading through as_slice_mut still requires every visible
slot Known. MaybeUninit::uninit may remove Known evidence, and that change must
return to the ledger when the borrow ends. Standard MaybeUninit new/uninit/write
contracts are reused; ownership splitting, registration and recovery are not
trusted. The actual spare_capacity_mut, packet adapter and initialization/
publication callers pass the 86-file gate. Native twelve tests pass; the
87-file re-uninitialized-growth negative rejects exactly one Known-prefix
guard. The earlier 79-file advance checkpoint excludes this additional bridge.

## Read-only bound access checkpoint

`B4-read`, `borrow_bound`, requires sealed allocation identity, capacity and
range correspondence, a shared PhysicalRegion borrow, and Known visible slots.
The returned shared slice has exact recorded contents and cannot outlive its
shared descriptor/region borrows. Overlapping shared reads are permitted;
mutation, retirement and recovery of that same resource wait for the borrow to
end. The primitive uses the bound native pointer directly and creates no
permission from integer metadata. Empty access grants no allocation liveness.
The body-proved packet wrapper and actual as_slice callers pass 90 files;
Unknown reading is rejected by one obligation among 91 files. This is local
physical-access trust, not a trusted handle protocol.

The unique-access checkpoint reuses the same sealed physical boundaries for
constructor-owned full regions. Returning the full unique region to B3 requires
no Known-prefix condition: the native recovery descriptor has length zero and
frees the allocation without reading bytes. Normal slice access and set_len
retain their Known requirements. The canonical slot_known clause on B4-bound
only restates its exact initialized-byte writeback, adding no physical premise.

## Fixed-zero physical references

The unique-only safe-trait gate adds `borrow_empty_bound`,
`borrow_empty_bound_mut`, and `borrow_empty_bound_uninit_mut`. They share-borrow
sealed non-null descriptor metadata and return only literal zero-length u8 or
MaybeUninit<u8> references at that native pointer. Empty mutable footprints may
coexist. They neither establish allocation liveness nor create byte permissions
or recovery authority. These remain explicitly trusted ordinary program
bridges. A nonzero full buffer must use its shifted spare endpoint; the empty
shortcut applies only when capacity is zero. Actual trait bodies and refinement
are proved in a separate unique-only invariant context, not the split context.

## B6 sealed pointer offset metadata

`BoundPtr::offset_from_bound_base` is a trusted physical representation primitive
introduced for exact cfg Shared reserve. It requires valid sealed descriptors
with the same namespace and capacity and an offset-zero base, and returns the
view's existing logical offset. Its native body subtracts numeric addresses.
It neither dereferences nor creates/reconstructs a pointer, grants no byte or
liveness authority, and proves no singleton/refcount/recovery rule. Those rules
remain body-proved affine protocol code. The relational representation fact
between sealed pointer addresses and their ghost offsets remains in the TCB.
The unrelated-allocation negative feature is proof-only.

## Immutable B4 ghost classification

Only `borrow_bound` and literal-zero immutable `borrow_empty_bound` are now
classified as ghost-observable physical reads. Shared borrows retain existing
affine authority until the returned references end; they cannot initialize or
write bytes. The exact source gate checks ghost and native reads before and
after real ordinary writes (34 files, zero unproved). Erased ghost mutation is
rejected by the non-ghost mutable bridge, and a stale-read claim rejects1/2.
All mutable and uninitialized B4 access remains ordinary program operations.
The read bridge itself remains trusted physical TCB, not a body-proved
raw-slice construction and not a trusted bytes ownership law. The earlier
mutable-B4 ghost diagnostic remains a counterexample and is not repaired by
this immutable-only classification. See `probes/readonly-b4-purity/evidence`.
