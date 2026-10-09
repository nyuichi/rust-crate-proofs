# AN architecture review — 2026-10-09

Read-only review of the v3 positive extension, frozen AL core, AM terminal
helpers, native client and selected production MIR. No solver was run for this
review. The active generated negative-control artifact was not used as the
positive source. No material ownership or native-operation mismatch was found
in the reviewed composition. This is not final correspondence admission.

Reviewed source identities:

- `src/reclone_extension.rs`: `44088313f920d23491325973d6833eebb6352606cc6367d8f5c193c4db5227ab`
- `src/promotion.rs`: `2c51555cc0c5eb9da7719609b2b2826e3e354251b802ceb3b440efa2b5f5f3f2`
- `src/terminal_helpers.rs`: `f7e410dcf3c2900464249a3d02ca04deae18cfaf59054ba1f20a9bdf9848d0f7`

## Acquire and pointer ownership

`load_visible_snapshot` is an ordinary body-checked function. Its expected
pointer is a `Snapshot` used only in specifications. Its executable result
comes from `owned_pointer::load_acquire`, whose native operation is the actual
Acquire load. The ghost callback borrows the existing owned pointer permission
and advances its current view through the Std Committer. There is no conversion
of a Ghost pointer into a native value and no replacement pointer allocation.

The Shared phase retains the root's old/new pointer history. Its visible-history
premise and current-view lower bound exclude the old tagged word; they do not
erase that historical entry. `pointer_history_progresses` returns only a
Boolean asserting unchanged owned permission and monotone current. It exposes
no State, Perm or token. This is correctly weaker than equality of current
views and sufficiently strong to retain the Shared-phase validity invariant.
Only the newly constructed child field receives a readonly binding.

## Existing-ARC clone and issuance

Both callback bodies use the loaded native word for the kind test and pointer
passed to `shallow_clone_arc_checked`. Their Shared-phase premises exclude raw
promotion; their explicit original-buffer parity premises choose the matching
native even/odd callback. The native MIR retains Acquire and the ARC branch's
call to `shallow_clone_arc`.

The new ARC helper performs the actual Relaxed increment through the existing
field adapter and calls body-proved `State::on_register`. Production
`shallow_clone_arc` MIR contains the increment and fresh child AtomicPtr
construction; it contains no second promotion CAS or control allocation. The
existing native overflow guard and abort behavior remain present.

The helper exports fresh returned-ticket insertion and an advancing issuance
counter, not assumed ticket IDs. Its v3 additional postcondition states that
the actual child's core accepts the final cursor. This is body-proved: public
metadata equality alone did not expose the child's invariant-model identity.
No invariant-model equality or ownership law was added to the trusted boundary.

The two new trusted registration functions are instances of the existing
generic native/proof callback-erasure boundary. Their correspondence to the
same production vtable targets under the Shared-phase preconditions must be
independently source-checked. They are not proved by this architecture note.

## Automatic cleanup composition

Both native Clone calls borrow `_2` (the original). The normal MIR then drops
second `_7` at bb3, first `_5` at bb4, reads the original at bb5/bb6, assigns the
owned return value at bb7, and drops original `_2` at bb8. Clone borrows end
before the peer Drop edges; read borrows end before the final Drop edge.

The AM consuming adapters retain their complete contracts. Both child effects
remove their actual ticket identities while preserving root core and pointer
ownership. Final root cleanup consumes the updated permission via the existing
generic latest-value `get_mut_finish`; that operation grants no synchronization
view or invented Acquire. The actual final release path performs Acquire and
produces both payload/control free receipts. The terminal interpretation still
requires receiver/data-address nonobservation and absence of independent field
drop glue; no live `&mut Bytes` invariant is restored after deallocation.

## Remaining admission conditions

The parent reports v3 diagnostic success for all 120 targets / 914 prover
leaves / zero nulls, archived as
`af4b16fd872cbcb7ded107dfd13f2a4d033e7022a621e728953ccd69eafbeeb3`.
Final independent source/MIR correspondence, semantic/type controls and the
canonical archive audit remain required. The checker must pin complete
extension/client/helper surfaces, all trusted registration contracts, actual
compiled inputs, and the independently anchored production source inventory.

This result concerns the closed nonempty-input normal-completion path.
Unwinding, allocator abort, overflow termination, concurrent promotion losers,
arbitrary concurrent closure and whole-crate/API coverage remain outside it.
