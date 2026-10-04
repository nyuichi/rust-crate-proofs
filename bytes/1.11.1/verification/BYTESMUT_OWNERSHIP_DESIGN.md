# Actual BytesMut ownership frontier (Creusot 0.13)

This is a resource-design checkpoint, not a proof of BytesMut. The relevant
source is `src/bytes_mut.rs`; standard-library references below are relative to
`/workspace/bytes-proof-tools/creusot-source/creusot-std/src`.
No trusted contracts or runtime source changes were added by this investigation.

## The first requested path already uses Shared

The exact runtime dependency is:

```
BytesMut::from_vec(Vec<u8>)
  ManuallyDrop::new(vec); vec.as_mut_ptr(); vec.len(); vec.capacity()
  ptr := NonNull(base); len := original len; cap := original capacity
  data := provenance-less packed KIND_VEC metadata, offset zero
split_to(at)                         [split_off has the same promotion]
  shallow_clone()
    promote_to_shared(2)
      rebuild_vec(ptr, len, cap, decoded offset)
      Box::new(Shared { vec, original_capacity_repr,
                        ref_count: AtomicUsize::new(2) })
      Box::into_raw(shared); self.data := shared
    ptr::read(self)
  self.advance_unchecked(at)
  other.cap := at; other.len := at
as_slice_mut()/DerefMut on each resulting handle
  slice::from_raw_parts_mut(handle.ptr, handle.len)
Drop(first handle)
  release_shared -> fetch_sub(1, Release) != 1 -> return
Drop(last handle)
  release_shared -> fetch_sub(1, Release) == 1
                 -> load(Acquire) -> Box::from_raw(shared) -> drop Shared/Vec
```

There is no path through either actual split API that yields two KIND_VEC
handles. Thus milestone 7 depends on single-threaded promotion and Shared
resource accounting, even though the initial handle is Vec-backed. Concurrent
CAS promotion belongs to `Bytes`, not this `&mut BytesMut` promotion path.
A design that defers *all* Shared ownership until after split/drop cannot match
the actual runtime. Atomic `Ordering` must remain unchanged.

## Proposed representation invariant

Use the actual runtime fields, plus erased resources, rather than replacing the
runtime type. For a nonzero allocation describe its identity/provenance `A`, base
`b`, capacity `C`, and allocator/layout. Each handle has offset `o`, length `l`,
capacity `c`, and an affine writable-region resource `R(A,o,o+c)`:

- `0 <= l <= c`, `0 <= o`, and `o+c <= C`.
- Its actual pointer equals the provenance-preserving offset `b+o`, not just the
  same integer address. Its actual `len == l` and `cap == c`.
- `R` gives unique access to `[o,o+c)` and describes each slot as
  `None` (uninitialized) or `Some(byte)`. Every slot in `[o,o+l)` is `Some`.
- The logical byte sequence is exactly these initialized first `l` slots.
- In KIND_VEC, decoded position equals `o`, `C == o+c`, and the unique allocation
  recovery authority plus the detached prefix `[0,o)` remain owned by this
  handle. That prefix remains initialized, as required by the rebuilt Vec
  length `o+l`. Advancing does not lose the prefix needed by `rebuild_vec`.
- In KIND_ARC, `data` denotes the actual live Shared control block and the handle
  owns one affine reference token for that block. The control block's Vec raw
  descriptor describes `b,C`; it does not separately authorize access to bytes
  whose region permissions reside in handles.

Truncate/clear can reduce the logical initialized prefix without forgetting
already initialized slots. `split_off(at)` allows `at > len`: the returned
handle then has length zero and can contain uninitialized capacity. A model of
only `Perm<*const [u8]>` for the entire capacity would falsely assert that spare
capacity is initialized. `MaybeUninit` slots or equivalent initialization state
are necessary.

For `split_to(at)`, transfer the former `R(A,o,o+c)` into disjoint resources
`R(A,o,o+at)` and `R(A,o+at,o+c)`. Return prefix length/capacity `at`; retain suffix
length `l-at`, capacity `c-at`. Pointer arithmetic must preserve `A`.

`shallow_clone`'s `ptr::read(self)` temporarily copies overlapping descriptors.
The helper must return a pending descriptor that has no independent permission
or public BytesMut invariant yet. The split caller distributes the resource and
seals both public handle invariants after adjusting fields. It must not assign
full writable ownership to both intermediate values. Either inline this private
operation in the proof boundary or specify a pending construction state;
asserting an invariant for the raw duplicate is unsound.

The Shared protocol must partition the allocation into live-handle regions and
retired/detached regions held for eventual recovery. It owns the unique recovery
authority. The count corresponds to live/pending reference tokens, not merely an
integer. Dropping a handle retires its region, consumes its reference token, and
performs the actual decrement. Only the last decrement, followed by the actual
Acquire operation, can collect all regions, restore the owning Vec resource,
and recover the Box of Shared. Empty regions require distinct reference tokens:
zero capacity is not permission to decrement an arbitrary block.

## Exact missing resource and why existing APIs do not supply it

1. **Vec allocation extraction with initialization and recovery.**
   `std/vec.rs` models a Vec as `Seq<T>`; `capacity` only promises `capacity >=
   len`. It has no permission-aware `as_mut_ptr`, `into_raw_parts`, or
   `from_raw_parts` bridge and no allocation identity/layout/spare-capacity
   resource. `from_vec` intentionally suppresses the Vec destructor, so an
   ordinary sequence postcondition does not transfer its allocation authority.
   A `Vec::into_boxed_slice` detour has a contract, but can shrink/reallocate and
   removes spare capacity; it is not the actual runtime path.

2. **Owned byte-region separation with a separate recovery authority.**
   `std/ptr.rs::Perm::split_at_mut` (currently line 739) takes `&mut self` and
   returns `(&mut Perm, &mut Perm)`. The owner remains one `Box<Perm>`; the child
   lifetimes are bounded by its borrow. `elements_mut` and `index_mut` also
   return borrows. The initialized-storage and region-permissions probes use
   exactly this pattern and reunite through Rust borrow expiry. They do not
   return two independently droppable owning handles. There is no owned
   split/join or region-to-allocation recovery API.

   This distinction is essential to soundness: `Perm::to_box` takes a
   `Box<Perm<*const T>>` and requires only pointer equality with its ward.
   Giving both halves of an interior split that same owned permission type
   would permit recovery of a Box at an interior address, and independently
   deallocating both halves with incorrect layouts. An owned split that returns
   two ordinary `Box<Perm>` is therefore not a sound missing convenience method.

3. **Liveness is not access or recovery authority.**
   `PtrLive` is copyable, is bounded by a borrow of `Perm`, and allows valid
   pointer arithmetic within the specified provenance/range (including one-past).
   It cannot yield `&mut Perm`, extend its own lifetime, or supply deallocation
   authority. A suffix witness also cannot establish a preceding allocation
   base: rebuilding an advanced Vec requires the full-allocation witness.

4. **Existing ghost accounting cannot manufacture physical permission.**
   `ghost::Resource` and its resource algebras can encode affine reference and
   interval accounting; their split/join operations do not produce a raw-pointer
   `Perm`. A mathematical map of bytes plus tokens would be a disconnected model
   unless an actual allocation bridge links it to physical resources.
   A `Box::leak`/static borrow workaround permanently loses recovery authority.
   A non-atomic invariant around a whole owner imposes a common scoped access
   token and does not give two independently transferable handle resources.
   Opening an atomic invariant cannot lend an interior `&mut Perm` beyond its
   ghost closure. Keeping the owner in such an invariant does not fix owned
   region transfer or concurrent ordinary reads/writes by itself.

5. **Vec stored in Shared needs a suspended ownership interface.**
   `promote_to_shared` reconstructs an actual `Vec<u8>` and stores it in Shared
   while handle pointers mutate its bytes. One cannot give that Vec an independent
   ordinary exclusive byte resource and also give the same resource to the
   handles. The model must suspend/rejoin its contents ownership while preserving
   its raw descriptor and eventual destructor entitlement. Raw parts must agree
   with the unique recovery token. A stronger numeric contract on `rebuild_vec`
   cannot discharge this requirement.

6. **Weak-memory resource recovery remains a theorem obligation.**
   Pointer `Perm` is explicitly non-objective. Shared atomic invariants require
   resources transported with `AtView` and a dominating `SyncView` before use.
   Vanilla atomic wrappers support ordering-parametric fetch_add/load but no
   fetch_sub. A native subtraction wrapper is a narrow primitive extension;
   reference-token conservation, release-sequence accumulation, and last-owner
   recovery must then be proved. Enabling `sc-drf` or changing Release/Acquire
   into SeqCst is not an acceptable shortcut. These existing frameworks are
   useful, but do not remove missing items 1, 2, and 5.

## Minimal sound extension to design before continuing ownership proofs

Introduce an affine **owned region** distinct from `Box<Perm>` and a unique
**allocation recovery authority**. A Vec/raw-buffer bridge consumes the original
Vec ownership and produces its exact runtime raw descriptor, the authority, and
one full-capacity region with a faithful initialization mask and initialized
prefix. No pointer/integer input alone may create either resource.

An owned split consumes one region and produces exactly two same-provenance,
disjoint adjacent regions. An owned join consumes adjacent regions of the same
allocation and concatenates their current contents/initialization states. Access
methods borrow only that region, and typed `[u8]` access requires initialization.
Neither region alone can call `Perm::to_box`, `Vec::from_raw_parts`, or deallocate.
The authority plus complete recovered coverage, no outstanding borrows/readers,
correct base/layout/allocator, and the required initialized prefix enable Vec
recovery. Zero capacity and one-past empty regions need explicit rules.

Keep the existing full `Box<Perm>` interfaces unchanged. For fully initialized
boxed buffers, provide a bridge that consumes the full owned permission into the
new authority/region form and recovers the exact full `Box<Perm>` after joining;
this preserves reuse of established `free_boxed_slice`/explicit deallocation
proofs. For Vec spare capacity, preserve its actual layout, including bytes not
currently initialized, rather than pretending it is a fully initialized Box.

This can be a carefully specified creusot-std memory-resource extension, not a
new BytesMut theorem axiom. Existing Resource algebra/protocol machinery should
prove the BytesMut-specific reference-count and range-accounting rules. A formal
soundness argument must cover identity/provenance, exclusivity, linear transfer,
layout, initialization, borrow escape, suspended Vec ownership, and view-indexed
thread transfer before these primitives are trusted.

## Diagnostic evidence and acceptance probes

The adjacent `probes/ownership-frontier` crate is deliberately not a replacement
BytesMut. Its default function reuses the already proved Box-region helper.
Optional failing features demonstrate separate tool/model boundaries:

| Probe | Meaning | Observed result |
|---|---|---|
| default `borrowed_split_positive` | Existing borrowed region split, two mutations, full Box recovery | Body proof passes; helper only |
| `owned_split` | Use current split result as two owned permissions | Rust E0308: pair of mutable references is not pair of Boxes |
| `live_recovery` | Recover a live allocation using only copyable PtrLive | Translates; Box::from_raw ownership precondition fails |
| `vec_raw_prefix` | Actual constructor raw-parts prefix | Translated; `vc_from_vec_raw_prefix` fails; ManuallyDrop::new / Vec::as_mut_ptr have impossible preconditions |
| `rebuild_vec` | Exact actual helper body, no invented resource contract | Translated; `vc_rebuild_vec` fails; raw-pointer sub / Vec::from_raw_parts have impossible preconditions |

The full actual runtime path above is established by source review, not claimed
proved by these probes. No negative failure is counted as a successful body proof.
An unsupported function or a Rust type rejection is not reported as a failed VC.

Before enabling the extension, require positive probes for: fully initialized
Box -> owned regions returned from a helper -> independent mutations -> join ->
full Box recovery/deallocation; Vec with `len < capacity` preserving its exact
base/capacity; initialized-prefix conversion; advanced offset recovery; actual
BytesMut split with both drop orders; and cross-thread Release/Acquire collection.

Require negative probes rejecting: overlapping writable splits; duplicate owned
region; use after region transfer/drop; recovery with only one region; interior
pointer deallocation; wrong allocation/allocator/layout join; same address with
wrong provenance; read of an uninitialized slot; stale contents after mutation;
mutable access after freeze; double reference-token consumption; last-owner
recovery while one token remains; omitted Acquire or inadequate SyncView; and
arbitrary integer metadata converted into dereference permission.

`std::mem::drop` has a Resolve contract rather than an allocation-deallocated
postcondition. A successful `drop(handle)` VC alone is not evidence that the
actual Drop/release/last-owner deallocation body has been connected. The target
requires separate body evidence for that path and conservation/recovery of its
resources.

The decision boundary is stop condition D for the ownership target: the precise
resource interface above needs soundness design. Translation cleanup and body
proofs that do not require it can continue independently. There is no integrated
crate proof or actual split/mutation/drop ownership proof at this checkpoint.

The ownership-frontier probe's positive Coma/proof files are snapshotted under
`verification/artifacts/component-evidence/ownership-frontier/`. Failing
constructor/recovery Coma tasks and proof JSON are retained under
`verification/artifacts/evidence/ownership-frontier/{vec-raw-prefix,rebuild-vec,live-recovery}/`;
the probe-local `evidence` directory is ignored and is not the retained evidence
location. These diagnostics show the current contracts do not provide the
required allocation resources; they are not body proofs of the actual runtime
constructor or drop path.

Reproduction from the bytes crate directory (proof invocations require elevated
execution for Why3 sockets; the wrapper serializes proofs and keeps sc-drf off):

```sh
./scripts/verify-bytes.sh ownership-frontier
BYTES_TRANSLATE_ONLY=1 ./scripts/verify-bytes.sh ownership-frontier --features owned_split
./scripts/verify-bytes.sh ownership-frontier --features live_recovery
./scripts/verify-bytes.sh ownership-frontier --features vec_raw_prefix
./scripts/verify-bytes.sh ownership-frontier --features rebuild_vec
```

Logs are retained under `probes/ownership-frontier/logs`; generated failing Coma
tasks and proof JSON are retained centrally under
`verification/artifacts/evidence/ownership-frontier/`. The probe-local
`evidence` directory is ignored. `source-correspondence.json` records an exact
body hash/match for rebuild_vec and the explicit limitations of the adapted
constructor-prefix diagnostic. Default proof: **Proved (2 files)**.
This includes the existing borrowed-region helper and its diagnostic caller,
not actual BytesMut callers.

## Comparison normalization is a separate structural boundary

The vanilla `std/cmp.rs` extern specifications require Self and Rhs to have the
same DeepModel type. Specializing generic reference implementations does not
supply Bytes/BytesMut's missing byte representation. It also does not reconcile
`str`/`String`'s `Seq<char>` with byte buffers' intended `Seq<Int>`.

A structural workaround now present in the `cfg(creusot)` source uses named
functions accepting actual handle references, converting through the
actual `as_slice`/`as_bytes` operations, and calling
`comparison_ops::equal` or `compare`. The ordinary comparison trait impls are
retained for the normal build; the proof configuration may use adapters. This
avoids a handle `DeepModel` projection in principle, but actual-handle
byte-semantics still needs a justified slice permission/representation
contract. The latest full-crate translation stops on the `static_clone` /
`STATIC_VTABLE` and `owned_clone` / `Owned::VTABLE` cycles before generating
Coma tasks. Therefore no claim is made that the current adapters or constructor
bodies have translated or been body-proved, and no runtime caller connection is
established. This translation issue is separate from stop condition D; the
unsupplied representation resource at `as_slice` remains the ownership
boundary.
