# AN trusted-boundary ledger

AN retains the AL/AM generic physical, pointer-provenance, weak synchronization,
allocation and normal terminal compiler boundaries. Their source and contracts
are captured with this probe; neither body-proof counts nor native tests prove
those primitives adequate for Rust's memory model.

AN adds two instances of the existing closed callback-registration boundary:
`even_reclone_registration` and `odd_reclone_registration`. Each returns the
actual promotable table and a registered three-argument callback whose complete
pre/postconditions equal the corresponding body-proved shared-phase callback.
This is a generic erasure/registration interpretation for a selected native
function and domain. It assumes native callback ABI and ghost argument erasure
preserve that selected body. The independent checker must bind both table
entries, complete callback contracts/bodies, the actual Acquire load and parity
branch, ARC helper, guarded Relaxed increment and exact record construction.
The new instances must not introduce an ownership, count, success-CAS, fresh-ID,
last-owner or destructor law. Those facts are supplied by checked bodies under
the generic primitives. Replace this registration boundary with a verified
native-to-proof translation or native callback contract reification, preserving
the three runtime arguments and erased scope input.

`load_visible_snapshot` is **body-proved**, not trusted. Its expected pointer is
an erased Snapshot used only in pre/postconditions. The native result is produced
by the retained Acquire adapter; it drives the branch and the ARC helper.
Neither an expected runtime pointer nor an allocation capability is conjured.
The exact shipped/private Std analogue is the retained operation committer and
SyncView/Perm load callback interface already reviewed in AL; AN strengthens
its caller export without adding an atomic primitive. In the captured private
Std 0.13.0, `src/std/sync/committer.rs` implements the `Committer<C,T,Acquire,Store>::shoot_load` contract as trusted: the returned
load value is an entry in the owned timestamp map and its published view is
bounded by the advanced current view. `src/snapshot.rs` documents Snapshot as
a zero-sized logical value whose construction does not move ownership and whose
Pearlite expression does not execute natively. AN combines those retained
interfaces; it does not prove their adequacy. The original root keeps
its owned history, and the new Boolean frame requires unchanged ownership and
monotone current view. Only the returned child's fresh atomic receives a
read-only binding.

The guarded increment primitive retains its strong field/lifetime/invariant
and affine cursor contract. `State::on_register` proves actual store completion,
fresh ledger entry and issuance progress. `shallow_clone_arc_checked` exports
that the actual returned child accepts the final cursor, as in the earlier
Shared clone proof; public metadata equality is insufficient for model identity.

The normal Drop compiler boundary maps three exact native places/successors to
body-forward consuming terminal adapters. Saved return evaluation precedes root
Drop. Complete client/helper tokens and pinned production field/import inputs
exclude hidden assertion macros, address observations, escapes and independent
field glue. It does not establish unwind or general Rust destructor adequacy.
Its replacement path remains a verified normal-edge Drop translation with the
same consuming effect interfaces.

Successful AN verification is conditional on these explicit generic assumptions.
The full original crate architecture remains NOT ADMITTED.
