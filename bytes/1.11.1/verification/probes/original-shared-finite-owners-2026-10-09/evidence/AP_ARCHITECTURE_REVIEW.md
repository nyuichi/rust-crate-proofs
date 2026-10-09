# AP architecture review

Full original bytes 1.11.1 verification remains **NOT ADMITTED**. This increment
concerns normal completion of a closed sequential runtime-count client; it does
not admit concurrency, arbitrary external owner escape, unwind cleanup, or all
remaining original API/configuration behavior.

The parent reports diagnostic v2 at 130 targets / 1040 actual prover goals /
zero nulls, archive SHA256
`cd5d0bee4424579a8dc1c1235b6333de95bdbbca52df44945475e55a4ff068ec`.
This is not this reviewer's independent final archive audit. The first failure
129/1075/1 remains archived; its one null was creation-loop inventory preservation.

Reviewed positive active source:
`cf05c10ecfe38af6ec999a4577a5de5bd5a8bca1d4422c8b94737b2349ef428b`.
Extension: `92998cd2ffb7e4fff780c75163394506cf4806a73b23e5f71099adc33bf6cd40`.
Client: `f75b08098021b88d72660357d07607200418dad758781e07c30c38821389d84f`.
The complete AO positive prefix remains byte-exact.

## Snapshot lemma and ownership

No ownership-interface defect was found. The isolated push lemma retains the
complete inventory: every actual element is a strong child, every actual ticket
fraction occurs in the live map, IDs are distinct, map length matches the vector
plus survivor, and conversely every live-map ID names one of those owners.
It proves induction from the actual clone insertion and unchanged cursor
identity. It does not assume completeness, guess IDs, or impose a count quota.

All five lemma arguments are Snapshots. The ordinary `#[check(ghost)]` function
returns unit, has no trusted attribute, and its body consists exclusively of
proof_assert statements. Those statements reason about logical values; they do
not create or extract owners. Pinned private Std snapshot.rs documents that
Snapshot is zero-sized and does not move ownership. Deref and inner are logic
functions; executable ghost extraction via into_ghost requires T: Plain. Bytes
and DetachedScope are not made Plain by this extension. The mode/library
boundary is explicit TCB, not a novel permission-conversion principle.

The first exact null task already contained the old inventory, strong child
content/public identity and final-cursor acceptance, unchanged cursor identities,
fresh exact insertion, and Vec sequence snoc. The fix separated that finite
map/sequence argument into a body-proved interface. It did not weaken an
invariant or add an ownership axiom.

## Native clone and loops

Shared child Clone follows the original Shared-vtable callback: Relaxed load of
the actual readonly-bound data field, then the inherited shallow_clone_arc
body and actual guarded Relaxed refcount increment. It adds no Acquire, root
pointer CAS, or second control allocation. The registration's only trust is the
existing generic exact native-function/pre/post erasure correspondence. Fresh
readonly binding is confined to the new child's initialized atomic field.
Moves into and out of Vec retain the existing generic model interpretation for
exclusively owned movable AtomicPtr fields; no field address escapes this client.

The vector itself contains the affine Bytes values. The creation loop and
pop-Some drain loop are not unrolled. The complete inventory is maintained at
loop headers, and a popped peer's actual terminal effect removes its ticket
before the backedge. The survivor remains live, so peer receipts are nonfinal.
Pop None establishes an empty vector and survivor singleton; its final Drop
establishes the real empty map and payload/control receipts.

The client names the clone result `next` before push to place an erased lemma
between them. Native effect order remains clone then push. Native MIR forms
an unused mutable Vec receiver borrow before the clone; the shadow forms the
push receiver borrow after it. This harmless borrow-timing difference depends
on the closed known clone body not accessing/escaping that Vec or its receiver,
and no observable Vec receiver address. It is within the explicit source/borrow
correspondence premise, not a generic allowance to reorder unknown callbacks.

## Empty Vec and terminal-effect boundary

Actual MIR normal ordering is saved return in bb19, owners Vec Drop bb20,
survivor Drop bb21. The proof follows that order. The generic ordinary
empty_vec_terminal_drop requires actual sequence length zero and returns no
Bytes completion or allocation receipt. It does not assert that Vec<Bytes> has
no destructor. Native Vec still deallocates its own storage. Std's generic Vec
sequence/Resolve contracts and Rust empty-element-drop/storage-destruction
interpretation remain TCB; no new theorem of container allocation reclamation
is claimed. The existing receipts refer to the Bytes payload and Shared control.

The external checker must bind the complete cyclic CFG, creation backedge,
pop-Some move and peer Drop, residual Option drop-flag ladders, pop-None exit,
empty Vec terminal edge and survivor final edge. No finite trace receipt can
substitute for this loop correspondence. Normal-return guarantees preserve
native MAX_REF_COUNT abort and allocation failure/unwind exclusions; they do
not promise successful execution for every runtime count.

## Checker review status

The checker was still being edited at this revision. Final adversarial replay
is pending the checker's stable notification and restored positive generated
inputs. Review must include complete source/macro/trust pins, inherited source
and preimport ancestry, exact Cargo routes/actual compiled-record receipts,
Snapshot-to-affine extraction rejection, and loop/empty-container mutations.
No proof source, ancestor source or prover configuration was changed by this
read-only review. Final restored proof, controls and independent archive audit
are still required before publication.

## Stable checker adversarial review

Reviewed main checker SHA256
`8057cd505d1aafe1e6740c1ffe14edfe3c03fb4b5b810b24e657c0fadb84b79d`,
control script SHA256
`6aef725d91dfa66c83e20ef590605fee631599972843857db2d49cb77a243718`,
and control manifest SHA256
`931807955178b6fbc083b30f5e3c35cd31981d5f3551d1b9c54df9b1d733b56d`.
No concrete accepted mutation was found in this bounded review.

The saved positive fixture's exact extension/client/composition and reconstructed
positive mapping pass their component gates. The independent native bundle
baseline also passes. The live generated source is an intentional parent-run
semantic control and was not presented as a canonical positive baseline.
No full live CLI or full control-suite replay was performed by this reviewer.

Independent in-memory attacks rejected:

- client macro override with recomposed active source and refreshed mapping
  client/active hashes; hash agreement passed before the complete-client pin
  rejected the macro;
- qualified trusted and cfg_attr-trusted injection on the inventory lemma;
- replacing the empty-vector precondition by a nonempty-vector precondition;
- external pointer_event trusted false-postcondition injection;
- the same false-postcondition addition in AP, AO and AN field_event copies;
  the anchored AM manifest rejected the matching inherited corruption;
- changed drain-entry edge, omitted peer Drop, and changed empty-vector Drop
  place with refreshed native capture digests; the independent main checker
  rejected the changed native MIR identity;
- an altered AL checker during fresh AP checker import. Instrumentation saw
  only the already-pinned AO checker load; it rejected before AN or the altered
  AL checker could execute.

To distinguish native body checks from stale-receipt rejection, a second fixture
also overrode read-only resolved-file views in memory alongside the refreshed
capture rows. The native checker then rejected the changed actual drain
backedge at bb16 on its exact executable block, and rejected the empty-vector
Drop place change at bb20 on its exact executable block. No ancestor, native,
Rust or generated input was written for these fixtures.

Inherited source and module routes remain anchored through AO/AN to the
published AL manifest. Exact parsed Cargo manifests exclude added dependency
patches/features/target routes. Actual build/extractor inputs are checked and
four actual Cargo/OUT_DIR artifacts are captured with hashes for the subsequent
archive audit. Whole client and extension pins close macro, extra-item and
qualified/cfg trust injection, in addition to the semantic callback, inventory,
loop, and empty-container checks. The checker's inspected `proof_assert` body
parser treats logical conditionals inside assertions as logical text, not as
runtime owner operations.

The Snapshot-to-Ghost<Bytes> extraction control has been added to the generator
at the parent's request, leaving positive Rust byte-identical; its actual
frontend result is to be recorded by the parent, not assumed by this review.
The final checker/control suite currently includes two additional CLI route
controls beyond the earlier 33-case announcement; the restored replay must
report its actual complete inventory rather than retain the old count.

No further blocker found for AP's bounded correspondence routes. Final restored
all-target proof and independent archive audit remain required. Full original
architecture remains **NOT ADMITTED**.
