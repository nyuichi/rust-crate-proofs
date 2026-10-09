# AT boundary interpretation

AT adds no trusted Bytes ownership, refcount, lastness, content or destructor
law. The new shallow/shared Clone helpers and public-operation shadow are
ordinary body-proved functions. Resource-bearing State/Perm/ticket objects are
never returned by a logical getter. Allocation comparisons are Boolean only.

The only new trusted registration instance is check(ghost)
owned_view_clone_registration: it returns Ghost<ViewCloneSpec>, registers the
actual Shared vtable callback, and equates its pre/postconditions exactly with
the ordinary shared_owned_view_clone_checked helper. No native function pointer
or branch decision is produced from Ghost code. The native three-argument
vtable invocation is unchanged. This uses the inherited generic erased-call
interpretation and requires independent exact callback/body correspondence.

The native shared callback loads Relaxed. The existing owned-field event and
State::on_register retain the actual guarded Relaxed RMW and weak-memory
committer obligations. Fresh ticket identity and fraction, exact map insertion,
source preservation and final recovery are body-proved. Fractions come from the
State pool; no fraction constancy, hidden-owner exclusion or last-owner axiom is
introduced. The affine external scope and closed-source effect discipline remain
inherited generic tool TCB and do not establish arbitrary escaping clients.

The replacement client extends normal-edge Drop correspondence to assignment:
next is evaluated and retained; the actual old value place is dropped; next is
moved into value. The proof-only consuming terminal effect must correspond to
that one actual edge, with no duplicate native destructor. Address-nonobserving
terminal interpretation, exact field profiles and absence of independent owning
field glue remain required. Cleanup/unwind edges are captured but not admitted.
Final return evaluation precedes the final value Drop.

AS's explicit nonnull boundaries and all other inherited physical, allocation,
free, readonly-pointer, atomic, scoped-history, Std and compiler interpretations
remain. Caller proofs do not independently prove universal native adequacy of
every generic boundary. Remove individual boundaries only when equivalent
verified Std/compiler interfaces are available. Exact source/module/actual Cargo
input correspondence and immutable independent archive audit gate admission.
Full original architecture remains NOT ADMITTED.

The singleton insert/remove lemma is ordinary body-proved algebra using existing
Std ext_eq; no new trusted lemma or Bytes resource law is introduced. The sole
new trusted function remains the exact ghost-only callback registration instance.
Canonical v2 proves155 targets/1417 prover leaves/zero null and structural leaves;
its independent1544-member audit replays main52/native45 and checks production63,
privateStd110, four actual Cargo inputs, six configs and eight tool rows. These
caller/correspondence results do not prove universal native adequacy of generic
boundaries or admit modular returned owners, concurrency or cleanup/unwind.
Canonical SHA256958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d.
See evidence/AT_CANONICAL_AUDIT.md for the immutable reconstruction and scope.
