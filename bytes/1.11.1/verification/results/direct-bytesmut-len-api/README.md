# Direct-source `BytesMut` proof evidence

On this verification fork based on crates.io `bytes` 1.11.1, selected
constructor bodies establish that `with_capacity(n)` returns an empty buffer
with capacity at least `n`, and `zeroed(n)` returns `n` zero bytes with
capacity at least `n`. The selected crate-private `from_vec(v)` conversion
preserves `v`'s length and bytes; `new` and `Default` establish an empty
buffer. These results apply to this fork; its native source refactors are not
shown equivalent to the unmodified release.

`proof.json` records the fresh proof run from the crate's live source. The
`coma/` and `results/` trees retain the exact selected COMA bodies and
successful prover results, together with the feature tree and translation log.
All selected targets translate from this crate's original source; no separate
Std-source proof targets are included. [`TCB.md`](TCB.md) describes the selected
guarantees and their assumptions.

For the generic Vec-storage path, `VecRawParts::take` and
`VecStorage::hold_owned` now have body proofs composing the B1 capture with
the body-proved recovery-token and physical-region construction. The
`VecStorage::capture_vec_owner` primitive that binds the consumed native Vec's
pointer, length, capacity, and contents to a sealed witness is trusted; it
does not create those affine resources or establish any `Bytes` owner,
refcount, or retirement rule. The private promotion helper calls `hold_owned`
to build its initial Shared storage state; later Shared lifecycle operations
remain outside this proof.

The native Shared reserve-growth path now compacts its initialized visible
bytes to the Vec base before calling `Vec::reserve`, so a hidden prefix left
uninitialized by another disjoint handle is not included in the Vec length.
The existing regression test covers that sequence. Reserve remains
`no_translate`; this is a tested native behavior change, not a proof of Shared
ownership or lifecycle.

The selected source targets also include body proofs for cursor-relative
prefix helpers, Vec metadata packing and position updates, `BoundPtr`
projections, `set_visible_len`, `advance_vec_layout`, `advance_unchecked`, and
`BufMut::advance_mut`. The selected constructors establish an offset-zero
`unique_vec_rebuildable` Vec state, including the known initialized prefix
needed to reconstruct the original Vec. `advance_unchecked` is proved only
under its internal rebuildable-state, `count <= len`, capacity, and
representable-position preconditions; this keeps the proof on the Vec
no-promotion path and preserves the allocation-slot frame.
The `BufMut::advance_mut` implementation body has a proof under its
cursor-validity and initialized-slot preconditions, but the generated trait
refinement obligation is not proved because those implementation preconditions
are stronger than the trait prestate.
No selected proof establishes a safe public-call chain that produces a
nonzero cursor. The public `Buf::advance` wrapper is not translated, and the
`advance_unchecked` body proof still covers only the Vec no-promotion branch.
A separate body proof now covers the private
`promote_to_shared_count_one` helper: it moves a rebuildable Vec into a Shared
header and binds the nonzero cursor, whole allocation region, count-one native
counter permission, and root owner ticket into the capsule. The native test
`count_one_promotion_preserves_nonzero_offset_view` exercises this helper at
offset 3 with zero additional advance and checks the visible view. Neither
result proves a safe public call chain reaches the promotion branch.

The selected source targets include body proofs for the generic `OwnerLedger`
algebra and the current-value refcount operation wrappers and increment loop.
They also prove the one-shot `RefCountCell` bootstrap extraction body, which
moves its event permission and recorded scalar value out of the cell and leaves
the proof-only sidecar detached. No production `Bytes` or `BytesMut` path calls
that extraction yet; the selected scalar adapters still require the sidecar to
contain its invariant. The proof-only `new_with_owner` constructor also has a
body proof for creating one native counter and returning its exact permission
and initial value while leaving the sidecar empty. The proof branch of
`promote_to_shared_count_one` uses it; native builds instead use the ordinary
`RefCountCell::new`. Outside that one initial promotion proof, the ledger and
refcount proof state are not connected to later production owner operations.
These results do not establish that the native refcount equals the number of
registered handles across transitions, that a count of one proves exclusivity,
or that Shared storage may be reclaimed. They are a partial direct-source
checkpoint, not complete
`BytesMut::truncate` verification across all safe states.

The current increment also body-proves `PhaseAuthority`'s one-shot close
action, `shared_capsule::make_count_one_packet`, and
`promote_to_shared_count_one`. For this private promotion helper, the proof
constructs a count-one Shared capsule from the actual Vec storage and exact
native counter permission, with one root ticket and the allocation's full
physical region. It does not prove every safe call path reaches the helper or
connect later clone, split, drop, and recovery operations to that registration.
It therefore gives no general count-to-owner, uniqueness, or reclamation
guarantee.

Use `./verify-all.bash` to regenerate the translation and run fresh proofs, or
`./verify-all.bash --check` to validate this saved evidence against current
inputs without invoking a solver.
