# Creusot automatic-drop feasibility probe

This probe is pinned to `creusot-std = 0.13.0` and contains no trusted
contracts. `SetTrueOnDrop::drop` calls `set_true`, which changes a borrowed
boolean to `true`.

Both public functions require the initial value to be false and specify the
observable final value as true. `explicit_helper` calls the same `set_true`
function directly; `automatic_drop` relies on Rust's scope-exit destructor.

Run with `./scripts/verify-bytes.sh drop-feasibility`. The recorded result is:

- `set_true` body: proved.
- `explicit_helper` body: proved.
- `automatic_drop` body: unproved, as expected for this compiler limitation.

The translated `automatic_drop.coma` contains guard construction and
`Resolve` assertions but no call to `SetTrueOnDrop::drop`. In Creusot source,
`MaybeLiveExceptDrop` documents that it ignores drops, `MutatingUseContext::Drop`
is classified as `DefUse::None`, and MIR `Drop` terminators translate to a goto.
So `Resolve` only relates the borrow state; it does not account for the
destructor's write. The helper proof is a separate body proof and is not evidence
that the automatic destructor effect is integrated. The Creusot `core::mem::drop`
spec likewise has an empty body and only ensures `resolve(t)`; it does not connect
the destructor body to callers.

The proof command output is saved in `logs/proof.log`.

The `wrong_explicit_helper` feature adds a deliberately false postcondition to
the same helper call. The negative-control run rejects both that function and
`automatic_drop`; the explicit helper remains proved in the default run.
