# Official PointerGenerator investigation (source-only, no run)

Pinned official Kani0.68.0 sources saved as kani-pointer.rs (upstream
library/kani_core/src/arbitrary/pointer.rs) and kani-mem.rs
(library/kani_core/src/mem.rs), read through GitHub connector.

PointerGenerator<const BYTES> owns MaybeUninit<[u8;BYTES]>. any_in_bounds<T>
chooses an offset <=BYTES-sizeofT and itself uses buf.add(offset). It can generate
multiple pointers sharing this one provenance or different provenance through
multiple generators. The generator is stack storage, not arbitrary heap history.
Docs explicitly exclude whole address space and cap sizes by the solver maximum
object size (default2^48) as well as isize::MAX. Thus it does not remove the
pointer-encoding limit or quantify arbitrary allocation sizes/provenance.

A useful distinct finite-domain candidate is PointerGenerator<64>, generating
any_in_bounds<[u8;size_of::<Rand32>()]>. Using a byte array avoids requiring a
new Arbitrary impl for original Rand32 (or violating orphan rules). Cast to
*mut Rand32, retain live generator, assume pointer aligned, use separate live
public from_state source and actual clone_to_uninit, observe copied/source
states. Uninitialized destination is permitted. This covers all aligned
placements in that64-byte internal object; it is broader in placement than the
typed exact slot but still a finite buffer contract. A second generator would
supply a different bounded provenance, not all arbitrary heaps. No implementation
or solver run has been made for this candidate.

Generating kani::any::<usize>() as *mut Rand32 is full address nondeterminism,
but can_write/is_inbounds predicates are not a sound shortcut to selecting all
valid allocated states: docs say nonnull unallocated/deallocated inputs panic
and Kani cannot reason about their allocation status beyond asserting validity.
is_inbounds includes unsupported-construct guards for such cases. An assumption
can_write(raw) does not automatically make those prerequisite guards unreachable.
It would also not conjure every possible allocation into the finite execution.
No direct arbitrary-heap pointer API was found in these official APIs.

Recommendation: retain current fixed-slot/full-state Kani contract and exact
copy_nonoverlapping one-Self manual footprint argument separately. A finite
PointerGenerator placement contract may be useful but cannot close an
arbitrary-valid-region machine claim. Do not use invalid pointer tests, stubs,
model replacement, safety-check suppression, or relabel finite buffers as allheap.
