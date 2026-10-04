# Ownership implementation feasibility gates

These results use unmodified Creusot 0.13.0 and native atomic ordering.
They are isolated feasibility checks, not an integrated proof of BytesMut.
No bytes-specific trusted contracts or compiler changes were added.

## Vtable translation

The minimal static and generic owned-vtable patterns reproduce the actual
clone/constant call-graph cycles. An accessor using a static table fails on an
unsupported definition kind. Passing the table through the callback instead
fails on an unsupported indirect function call. Ordinary Rust checking of all
these patterns succeeds; Creusot fails before generating verification conditions.

See [vtable probe](probes/vtable-feasibility/README.md). The current implementation
effort excludes the integrated Bytes vtable/clone path. These experiments do not
justify trusting the callbacks or changing their native behavior.

## Automatic destructor effects

The same write to a borrowed boolean is proved when called explicitly but is
not proved when it relies on scope-exit Drop. The automatic-drop translation
contains Resolve assertions and no call to the destructor body. The separately
emitted destructor Coma is not itself a recorded body proof.

See [Drop probe](probes/drop-feasibility/README.md) and retained generated tasks
under `artifacts/evidence/drop-feasibility/recorded/`. Therefore an explicit
cleanup body may be a useful next target, but proving it will not establish
automatic Rust Drop at callers or actual last-owner destruction.

## Shared control-block ownership

The control-block probe uses a real Box, Perm, FullBorrow, GhostShared and
fractional LifetimeToken values. Both observation orders recover the same Box
and preserve immutable metadata. The positive run proves six generated files.
Ending a half-token and recovering while a separate half-token remains are
rejected at their intended verification conditions.

Retained tasks are under `artifacts/evidence/control-block-lifetime/`.
This establishes a control-block ownership foundation only. It does not prove
atomic/refcount correspondence, release-sequence synchronization, byte-region
recovery, deallocation, or a BytesMut runtime body. A proof-only observation
after recovery is not a runtime use-after-free test.
The separate diagnostic with an actual field read after Box drop is rejected
by Rust typechecking (E0505) at the attempted move of the borrowed token;
translation never begins. This is compiler evidence, separate from the two
fraction-accounting VC rejections.

## Physical byte bridge review constraints

The pure resource ledger must remain separate from physical memory authority.
Only consuming a real Vec may establish the sealed allocation association.
An integer address, model-only ledger constructor, or ghost resource allocation
must not establish permission to access or free native memory.

An empty interval owns the resource-algebra unit. It does not establish
allocation liveness: a surviving empty fragment must not authorize nonzero
pointer arithmetic after full-region recovery. A nonzero offset operation must
borrow a nonempty region and stay within its closed bounds, or separately borrow
the allocation recovery authority. Zero-length slice construction needs an
independent non-null/alignment argument and grants no byte access. Split and
join must not manufacture PtrLive.

## Owned-region algebra and native allocation bodies

The [owned-region probe](probes/owned-region-kernel/README.md) proves seven
generated files, including interval split/join bodies, a helper returning both
regions, and a caller rejoining them with exact known and unknown slots.
The overlap negative rejects the join precondition. These are pure ledger
proofs: the model constructor establishes no physical allocation association.

`src/ownership_proof/raw_buffer.rs` supplies separate native unsafe bodies for
detaching a Vec, recovering it, and explicit zero-length deallocation. Its
isolated native harness checks pointer/capacity preservation, disjoint writes,
zero capacity, spare capacity and cleanup after re-uninitializing a former
initialized prefix byte. The module is not connected to BytesMut,
has no Drop implementation, and adds no Creusot trusted contracts.

The next gate is the reviewed Vec/raw/access bridge. Pointer comparisons need
particular care: native raw-pointer equality is translated to logical pointer
equality, while the address-only comparison specification grants no such
identity. The [paired feasibility probe](probes/pointer-equality-feasibility/README.md)
proves both default files and rejects address-only equality implying logical
pointer identity at the intended VC. This confirms the difference in the
compiler model and specifications; it is not a concrete provenance counterexample.
Before accepting any physical bridge contract, physical identity must come from extraction or
provenance-preserving derivation, with allocation identity established by
resource namespaces rather than address comparison.

The proved access path must audit native raw-pointer `==`/`!=` out of the
identity reasoning. Runtime thin-pointer comparisons can use `addr_eq`, with
ticket/resource identity proved independently. In particular, future
`try_unsplit` integration must examine both buffer-pointer and control-block
pointer comparisons. No such runtime substitution or physical contract has
been added at this checkpoint.

Actual BytesMut split, mutation and cleanup integration remain unproved.
