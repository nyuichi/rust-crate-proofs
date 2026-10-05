# Coordinator-carrier repeated split probe

This probe extracts the actual `BytesMut` and `Shared` declarations and the
selected method bodies directly from `src/bytes_mut.rs` at build time. The
fragment manifest records source offsets and hashes. Its dedicated
`bytes_proof_repeated_split` configuration connects those bodies to
`carrier_protocol.rs` and the scalable affine ticket registry.

The target lifecycle constructs from one Vec, calls actual `split_to` twice on
the handle that retains the exclusive coordinator, mutates and reads the three
visible regions, and explicitly releases all three in each of six orders. The
public harness clamps requested positions to the current visible length; the
actual method contract requires the precise position bound. Ordinary automatic
Drop, concurrent operations, and splitting a sibling without the coordinator
are outside this gate.

The second split takes the actual ARC `shallow_clone`/`increment_shared` path.
Its adapter uses the existing native Relaxed scalar-counter bridge. Promotion
transfers the original physical allocation capabilities into the registry;
it does not detach another Vec. Stable per-ticket creation metadata is separate
from the evolving coordinator pending map. Final native counter result one
selects recovery, disarming the buffer before allocation deallocation and typed
control-block destruction.

No new trusted ownership or refcount protocol is introduced. The existing Vec,
physical byte access, boxed alignment, and exclusive scalar atomic bridges and
standard permission operations remain the trusted boundary.

Status: the integrated gate passes 106 proof files with zero unproved leaves.
Five native tests cover boundary positions/all six release orders and exact
original-buffer/control-block allocation/free counts. Native execution does not
by itself establish the ghost protocol or a whole-crate proof.

The proof configuration factors the original register/advance/descriptor-update
tail into a finishing helper; the same native pointer/length/capacity operations
are retained. A body-proved coordinate lemma relates the original relative-slot
initialization predicate to absolute physical slots. Pending-map cardinality,
fresh-ID and matching frame facts are body proved at their defining boundaries.
These proof interfaces add no trusted ownership rule.

The `negative_carrier_missing_ticket` feature constructs an empty Vec, performs
two actual split_to(0) calls, forgets one empty sibling and explicitly releases
the other two. Claiming the second release is final is rejected at exactly one
assertion (17/18), in a 95-file negative run. This feature is proof-only and is
not executed natively. Empty physical coverage cannot replace the missing ticket.

Final source regressions pass legacy split/storage110, safe traits72 and public
constructors61 files, all with zero unproved leaves. The existing spare access
now factors prefix preservation into a body-proved access operation; the
initialization postcondition is decomposed into its equivalent ownership and
Known-prefix clauses. Existing B4 remains the only physical spare-access axiom.
Evidence: `prefix-final-positive`, `prefix-final-negative`, and `regressions/`
under `verification/artifacts/evidence/coordinator-carrier-split`.

The second operation may instead be actual `split_off`, bounded by remaining
capacity. Its returned right handle retains the coordinator; left length is
min(old_len, at), right length saturates at zero, and the full capacity slot
state is preserved, including Unknown spare bytes when at > old_len.
The `negative_carrier_unknown_publication` proof-only feature rejects exactly
set_len's Known-prefix guard (16/17, 95 files) after this repeated split path.
Do not execute negative features natively.
Final split_off evidence is `split-off-final-positive`,
`split-off-final-ticket-negative`, `split-off-unknown-negative`; the 110-file
legacy regression is `regressions/split-off-final`.

The final gate also extracts actual spare_capacity_mut/set_len, advance_unchecked,
truncate/clear, resize/extend_from_slice and reserve. The reserve contract
restricts its unchanged fast path; resize/append are bounded by current capacity.
A storage harness reinitializes/writes/publishes one spare byte, advances all
three descriptors, shrinks/clears, and recovers the original full allocation.
A capacity harness fills/appends in the disjoint regions. Ticket identity and
region endpoints are body-proved frames. Native5 observes no reallocations.
Final evidence: capacity-positive (106 files), capacity-unknown-negative and
capacity-ticket-negative (108 files, one intended leaf each), with the legacy
110-file regression under regressions/capacity-final. Historical split-only
counts above refer to their archived checkpoints. No growth or general reclaim
claim is made.
