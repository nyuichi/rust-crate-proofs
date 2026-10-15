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
interval lender, retirement, and full-range recovery helpers also have body
proofs. The lender transfers the actual affine resource for a fully present
interval and preserves unknown slots; it grants no byte-read authority by
itself. A checked
`freeze_into_readonly` path moves an actual Mutable packet's ticket to a
Readonly ticket and returns its actual physical region to the pool, while
preserving the reference count, storage, lease/release authority, caller
tokens, and Context entries. The selected private freeze helper connects this
transfer to a `Bytes` sidecar; the public `BytesMut::freeze` body, its native
static vtable identity, and readonly byte reads remain unproved. The payload
protocol also proves that every active Readonly ticket's visible interval
remains in the physical pool with known slots; a checked helper derives this
fact from a live ticket and the actual ledger. This pool fact grants no slice
borrow authority. The selected
`Bytes::dispose_shared` path uses that actual Readonly packet and pointer-cell
permission to enter the same common release body, whose final-owner branch
retains the original release, acquire, and storage-free order. Automatic
`Bytes`/`BytesMut` Drop remains unchanged and unverified, so the selected
operation sequence explicitly disposes each related handle and makes no
Context-consistency claim after an untracked automatic Drop. The original
`release_shared_common` body now passes its actual stored authority and full
physical region to the generic `retire_stored_vec` boundary on the final
release, explicitly destroys the empty replacement VecStorage, and consumes
the actual Shared root permission before freeing the Box. The crate-private
unsafe `BytesMut::dispose_shared` path supplies its actual packet and caller
context to this same native release body and has a body proof. The ordinary
`release_shared`/automatic `Drop` caller remains untranslated, so this does not
prove post-Drop context consistency or cover every native ARC clone, freeze,
or recovery transition. Concurrency, unwind behavior and safe-state
reachability of the common view precondition across all Shared lifecycle paths
remain outside this checkpoint. Thus these body proofs do not yet establish
an end-to-end public `BytesMut::truncate` guarantee for every safely produced
state. See the
[selected TCB](verification/results/direct-bytesmut-len-api/TCB.md) and
[generic Std overlay](verification/std-support/README.md) assumptions.

The crate-private unsafe `BytesMut::dispose_unique` body also consumes a
rebuildable Vec-backed handle: it reconstructs the original allocation through
the checked signed-offset `rebuild_vec` path, captures that actual Vec with
`VecStorage::hold_owned`, then consumes the resulting authority and full region
through `retire_stored_vec`. Its native branch performs the same reconstruction
and drops that Vec once. This narrow explicit path does not prove public freeze,
unselected `rebuild_vec` callers, or automatic `BytesMut::drop`.

The verification-only Shared capsule also has body-proved affine header-loan
transitions. A token-aware `EventBoundInvariant::open_with_tokens` exposes the
protocol, counter-owner agreement and callback frame; packet take/restore
helpers move the exact boxed header permission into and out of a lease bound to
its generation and header/storage metadata. Acquisition remains conditional:
the packet invariant and namespace token do not imply that the header root is
present, so `try_take_header_read_loan` returns `Option`. The selected explicit
disposal path uses these helpers around the original native decrement, acquire,
and retirement sequence; its actual packet, matching context rights, terminal
action, storage authority and full physical region are preserved through that
source body. Automatic `Drop` remains in place, but its context-free wrapper is
untranslated and this proof does not claim a post-Drop context state. The
result does not close every safe producer or lifecycle path for `truncate`.

A generic append-only `ReadyRegistry<R>` now has body proofs for moving the
producer's actual affine `Fragment<R>` into a fresh permanent slot, taking that
same capability while leaving a tombstone, and returning it to the slot. Its
contracts preserve the complete resource id and RA state of every other slot;
`ReadyCap::update` applies the real authority update to the stored fragment and
preserves its resource id and key. The selected private count-one promotion
and original-Vec split producers pass a caller-supplied context into packet
construction, which registers the actual idle header-lease fragment. The
packet's take/restore helpers use that context to move the same capability into
and back out of a header read loan. Body proofs cover both Vec-backed and
packet-backed ARC branches of public `BytesMut::split_off`, `split_to`, and
`split` for valid, normal-return inputs. The Vec route requires a rebuildable
unique allocation and registers its idle capability. The ARC route requires
the existing matching idle entry, performs the source relaxed refcount
increment once, splits the actual owner ticket and physical region, and
restores the same capability to its Context entry. Both routes export strict
Shared layout and initialized-prefix facts, byte/length/capacity/pointer
relations, distinct owner tickets for one allocation and lease key, and the
matching idle Context entry for both handles. The crate-private explicit
disposal method can consume either resulting handle with this actual Context
and reaches the proven native release body. These split paths do not close
every safe producer or lifecycle path needed for `truncate`.
The verified operation sequences explicitly dispose each related Shared
handle with its caller-supplied context; they do not assume that an untracked
automatic `Drop` leaves that context consistent.

The selected targets also body-prove an affine `OwnedSharedContext` and its
borrowed view. Construction consumes a caller-supplied token set and starts
with an empty registry; reborrows preserve the actual token namespaces and
frame the registry's exact final entries. Registration, take, and return move
the producer's actual fragment through that registry. The selected private
count-one promotion and `split_unique_to_shared` paths thread this context into
packet construction, as do both Vec-backed and packet-backed ARC branches of
public split methods. The ARC branch borrows the existing matching idle
capability and returns it to the same registry entry after splitting actual
affine resources. Packet loan take/restore returns the exact capability to the
Context. Other public producer call chains and ordinary automatic lifecycle
paths remain unproved; this context does not establish that every safely
reachable state satisfies the `buffer_view_valid` precondition for `truncate`.

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
