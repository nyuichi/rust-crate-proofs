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

The private `advance_unchecked` body has selected Vec and packet-backed Shared
cursor proofs. The Vec path retains the full allocation-slot frame and updates
the visible prefix in two phases. The Shared path advances the packet cursor
and proves the new pointer, length, capacity, visible-byte suffix and complete
packet allocation-slot frame under its internal layout and initialized-view
preconditions. The selected capsule and packet invariants tie each packet's
ghost allocation identity (namespace, base and capacity) to the protected
Shared storage through its ledger ticket; the ticket's owned interval also
matches the packet's physical region. The identity and interval metadata grant
no access authority, and the proofs do not establish every public producer or
later owner-lifecycle path. They also do not establish a safe public-call chain
that reaches a nonzero cursor; the public `Buf::advance` wrapper is not
translated. `BufMut::advance_mut` has a body proof, but its trait-refinement
obligation remains open.

The selected ghost ownership proofs model each ticket as an exact singleton
fragment carrying a half-open interval within the allocation. The ledger
invariant requires those intervals to be in bounds and pairwise disjoint;
body proofs cover root registration, a two-owner registration, splitting a
parent interval at a valid cut, dropping a registration, and replacing a
ticket's interval while preserving the authority identity. The count-one
packet constructor binds its sole full-range ticket to the packet region and
the initial empty physical pool using the body-proved
`PhysicalPool::empty_from` constructor and unit-normalization helper. The
generic pool retirement and full-range finish operations are also body-proved;
no production path invokes them yet. These generic ledger proofs do not
connect native handle split, drop, conversion, or refcount transitions to the
tickets; pool retirement and recovery transitions also remain unproved in a
production call path. Immutable peers, refcount-to-owner correspondence,
concurrency, unwind behavior, and end-to-end `BytesMut::truncate` across shared
or split states remain outside this checkpoint. See the
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
