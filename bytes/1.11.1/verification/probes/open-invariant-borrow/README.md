# Open mutable invariant boundary diagnostic

This no-trust diagnostic attempts to return an open mutable borrow after writing
an odd value into a type whose invariant requires an even value. It also tests
relaying that borrow and calling the helper on a locally opened value. Callers
restore the even value before their own exit.

Creusot 0.13 nevertheless inserts current-invariant obligations at mutable
reborrow creation and resolution. The failed run records `leave_open` 3/6,
`relay_open` 4/8, and `restore_already_open` 3/5. `open_inv_result` does not suppress
these internal reborrow checks. This is a representation boundary diagnostic,
not a proof that open mutable handles can cross private helper boundaries.

Run only through `./scripts/verify-bytes.sh open-invariant-borrow`.
