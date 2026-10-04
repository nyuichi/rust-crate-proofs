# Control block lifetime feasibility

This probe uses a real `Box<Control>` allocation with immutable `metadata`. It
converts that Box to the existing `Perm` pointer permission, moves the boxed
permission into `FullBorrow`, shares that ghost borrow with `GhostShared`, and
borrows the metadata through two half-fraction `LifetimeToken` tickets.

The positive entry points exercise both ticket orders. Each scoped read ends
before the fractions are joined; the joined token ends the synthetic lifetime;
`EndBorrow::get` returns the original boxed permission; and `Perm::to_box`
recovers the allocation using its original pointer. The functions preserve
the metadata value in their postconditions. The probe adds no trusted code.

## Verification

Run the positive proof with:

```sh
./scripts/verify-bytes.sh control-block-lifetime
```

It proved all 6 generated files with the repository's Creusot 0.13 route; the
actual wrapper output is saved in `logs/positive.log`.

The two negative controls are expected to fail at their named goal:

```sh
./scripts/verify-bytes.sh control-block-lifetime --features negative_one_fraction
./scripts/verify-bytes.sh control-block-lifetime --features negative_outstanding_borrow
```

The first tries to end a lifetime with one half fraction. The second keeps a
borrow ticket active and tries to end using only the other half. The recorded
VC results were `vc_negative_one_fraction_cannot_end` (3/4) and
`vc_negative_outstanding_borrow_cannot_recover` (12/13), respectively.
Their captured output is in `logs/negative_one_fraction.log` and
`logs/negative_outstanding_borrow.log`.

There is also a compile-time diagnostic:

```sh
./scripts/verify-bytes.sh control-block-lifetime --features negative_ended_reference
```

It attempts to return a field read through a reference after ending the token,
recovering the Box, and dropping it. Creusot's Rust typecheck rejects the
move of the borrowed token with `E0505` before translation or proof; the
post-deallocation read is not executed. This is compiler evidence, not a
Creusot VC or a proof of deallocation safety. Its output is saved in
`logs/negative_ended_reference.log`.

This establishes feasibility for fractional lifetime accounting and recovery
of one real Box through the existing permission API. It does not establish
Rust reference-count behavior, atomic access, Bytes/BytesMut integration,
concurrent drops, or Drop correctness. The outstanding-fraction VC only
establishes that a ticket holding a fraction prevents ending the lifetime.
The ordinary-reference exclusion was rejected by Rust typechecking, not
proved by Creusot.
