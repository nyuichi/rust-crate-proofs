# Open mutable invariant borrow feasibility

This reduced ordinary Rust example temporarily makes an Even value odd in a
private transition, returns the mutable borrow with open_inv_result, then
restores it at the caller. Vanilla Creusot 0.13 inserts current/final invariant
checks at the generated mutable reborrow/resolve boundaries. Those obligations
are false in the intended open intermediate state. The argument's open_inv and
the result's open_inv_result do not remove these checks.

The wrapper completed with incomplete functions: leave_open 3/6, relay_open
4/8, restore_already_open 3/5; the other restore caller passed. These are
intentional unsupported intermediate-state VCs, not a successful gate. No
trusted restoration or weakened Even invariant is added. This diagnoses the
BytesMut PendingControl transition issue; it does not prove actual BytesMut
trait or lifecycle code. The byte lifecycle gate keeps explicit predicates,
and ordinary traits use a separate valid-handle-only invariant configuration.
