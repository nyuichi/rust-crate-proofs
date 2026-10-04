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

Native raw-pointer Eq/Ne is translated as logical pointer equality/inequality.
The paired diagnostic demonstrates that address-only `addr_eq` does not grant
that identity. All proved caller paths must therefore exclude native pointer
Eq/Ne from identity reasoning. The actual try_unsplit comparisons now use an
address-only helper, but that helper is not a proof of try_unsplit.

Sealing the native descriptor removes direct pointer substitution at this bridge.
It does not repair the upstream translation for arbitrary caller control flow.
Logical allocation identity must come from the sealed namespace and resource
ownership, independently of runtime address comparisons.
