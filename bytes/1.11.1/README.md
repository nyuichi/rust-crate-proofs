# bytes 1.11.1 runtime verification

The selected direct-source constructor proofs cover `BytesMut::from_vec` and
the public `with_capacity`, `new`, `zeroed`, `From<&[u8]>`, `From<&str>`, and
`Default` paths. They establish an offset-zero Vec-backed `unique_valid` state
and the stronger `unique_vec_rebuildable` condition: the visible initialized
bytes and any hidden prefix needed to reconstruct the original Vec are known,
with the specified length and contents. Capacity is at least the requested or
copied length (`from_vec` retains the modeled Vec capacity), and spare-capacity
slots remain unknown. `zeroed` establishes the requested zero-filled length;
slice and string conversions establish the copied contents. `From<&str>` uses
the generic Std model `Seq<char>::to_bytes()`; the concrete native UTF-8 encoder
is not proved by this crate run (see the [Std overlay notes](verification/std-support/README.md)).

`BytesMut::len` and `is_empty` are proved without a `unique_valid`
precondition. For a `unique_valid` state, `clear` sets the length to zero and
preserves the pointer, capacity, storage tag, authority identity and all
capacity slots. `truncate(n)` sets the length to `min(n, old_len)` and preserves
the initialized prefix, pointer, capacity, storage tag and all capacity slots.
Unsafe `set_len(n)` requires `n <= capacity` and known initialized slots for
any growth; it changes only the logical length and preserves the pointer,
capacity, storage tag and all capacity slots. The original-source proofs
include the local ownership, provenance and capacity-representation helper
bodies in their call cone.

The private Vec advance is proved in two phases: `advance_vec_layout` moves the
cursor while retaining the full allocation-slot frame, then `set_visible_len`
restores the visible-prefix invariant. `advance_unchecked` is proved only for
the Vec no-promotion branch, with a rebuildable state and `count <= len` as
preconditions. The initial constructors establish the rebuildable state at
offset zero, but the selected proofs do not establish a safe public-call chain
that reaches a nonzero offset. The public `Buf::advance` wrapper is not
translated, and `BufMut::advance_mut` has a body proof but its trait-refinement
obligation remains open. Shared-backed transitions, immutable peers,
refcount-to-owner correspondence, lifecycle operations, concurrency and
unwind behavior are outside this checkpoint. See the
[selected TCB](verification/results/direct-bytesmut-len-api/TCB.md) and
[generic Std overlay](verification/std-support/README.md) assumptions.

This is a verification fork based on the crates.io 1.11.1 source. It includes
branch runtime fixes and verification-only annotations/refactors; this evidence
applies to the fork and does not establish equivalence to the unmodified
published release. The added proof state is erased from native builds. The run
uses default `std` features, x86_64 native atomic orderings, and disables
sc-drf. Results describe the selected sequential, normal-return proof scope,
not concurrent `Bytes` behavior.

`./verify-all.bash` translates the live crate source and runs fresh, no-cache
proofs with the pinned bytes Creusot toolchain, default `std` features, native
atomic orderings, and one prover under the shared lock. Why3 needs elevated
execution. `./verify-all.bash --check` checks the saved COMA and prover results,
source/configuration identities, target inventory and translation diagnostics
without invoking a solver. A changed input is a reuse miss and requires a fresh
proof run.

The Root lifecycle result in `verification/results/current.zip` predates the
current source changes. It and `components.zip` remain historical provenance;
neither archive is used by the runner or checker, or represents current
implementation coverage. See the [saved-results notes](verification/results/README.md).

Source lineage: crates.io bytes 1.11.1, archive SHA-256
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`, revision
`417dccdeff249e0c011327de7d92e0d6fbe7cc43`. [MIT license](LICENSE).
