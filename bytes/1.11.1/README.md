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

`BytesMut::len` and `is_empty` are proved without a representation
precondition. From a `buffer_view_valid` prestate, `clear` sets the length to
zero and `truncate(n)` sets it to `min(n, old_len)`. Both preserve the visible
byte prefix, native pointer, capacity, header word, exact proof sidecars and
every allocation slot. Unsafe `set_len(n)` requires `n <= capacity` and known
initialized slots for any growth; it changes only the logical length and
preserves the same full allocation frame. The common view precondition covers
either a Vec-backed unique layout or a packet-backed Shared view with known
visible bytes. These are modular body proofs: the Shared form carries the
`SharedOwnerPacket` type invariant, and the proofs do not establish that every
safe producer or lifecycle transition supplies this precondition. The
original-source proofs include the local ownership, provenance and
capacity-representation helper bodies in their call cone.

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

The selected ghost ownership proofs model each native strong reference as an
exact singleton ticket in one ledger. Mutable tickets own disjoint physical
intervals; readonly tickets record visible intervals and may overlap one
another. The physical pool is the in-bounds complement of Mutable intervals.
Body proofs cover readonly registration and reclassification, root and
two-owner registration, splitting a Mutable interval, dropping a registration,
and replacing a ticket binding while preserving ticket and authority
identities. A generic event payload ties the ledger's reference count to the
counter permission model; the initial count-one promotion and original Vec
split producer construct matching payloads and packets. The generic pool
retirement and full-range recovery helpers also have body proofs. A generic
stored-Vec destructor boundary has an exact `drop(storage.take_for_retirement())`
native body, but no selected release path calls it yet. Native ARC split, clone,
drop, freeze and recovery transitions are not yet connected to the ledger.
Concurrency, unwind behavior and safe-state reachability of the
common view precondition across Shared lifecycle paths remain outside this
checkpoint. Thus these body proofs do not yet establish an end-to-end public
`BytesMut::truncate` guarantee for every safely produced state. See the
[selected TCB](verification/results/direct-bytesmut-len-api/TCB.md) and
[generic Std overlay](verification/std-support/README.md) assumptions.

The verification-only Shared capsule also has body-proved affine header-loan
transitions. A token-aware `EventBoundInvariant::open_with_tokens` exposes the
protocol, counter-owner agreement and callback frame; packet take/restore
helpers move the exact boxed header permission into and out of a lease bound to
its generation and header/storage metadata. Acquisition remains conditional:
the packet invariant and namespace token do not imply that the header root is
present, so `try_take_header_read_loan` returns `Option`. No selected public
Shared lifecycle body yet uses these helpers around native counter operations.
They establish proof plumbing, not safe producer closure for `truncate`.

A generic append-only `ReadyRegistry<R>` now has body proofs for moving the
producer's actual affine `Fragment<R>` into a fresh permanent slot, taking that
same capability while leaving a tombstone, and returning it to the slot. Its
contracts preserve the complete resource id and RA state of every other slot;
`ReadyCap::update` applies the real authority update to the stored fragment and
preserves its resource id and key. The selected private count-one promotion
and original-Vec split producers pass a caller-supplied context into packet
construction, which registers the actual idle header-lease fragment. The
packet's take/restore helpers use that context to move the same capability into
and back out of a header read loan. Public producer wrappers and native
lifecycle transitions are not yet proved to supply and thread this context, so
these internal paths do not close safe public reachability for `truncate`.

The selected targets also body-prove an affine `OwnedSharedContext` and its
borrowed view. Construction consumes a caller-supplied token set and starts
with an empty registry; reborrows preserve the actual token namespaces and
frame the registry's exact final entries. Registration, take, and return move
the producer's actual fragment through that registry. The selected private
count-one promotion and `split_unique_to_shared` paths thread this context into
packet construction, and packet loan take/restore returns the exact capability
to it. The public safe producer call chains and native lifecycle paths remain
unproved; this context does not establish that every safely reachable state
satisfies the `buffer_view_valid` precondition for `truncate` or provide a
verified explicit release path.

This is a verification fork based on the crates.io 1.11.1 source. It includes
branch runtime fixes and verification-only annotations/refactors; this evidence
applies to the fork and does not establish equivalence to the unmodified
published release. The added proof state is erased from native builds. The run
uses default `std` features, x86_64 native atomic orderings, and disables
sc-drf. Results describe the selected sequential, normal-return proof scope,
not concurrent `Bytes` behavior.

`./verify-all.bash` translates the live crate source and runs fresh, no-cache
proofs with the pristine pinned bytes Creusot toolchain, default `std` features,
native atomic orderings, and one prover under the shared lock. Why3 needs elevated
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
