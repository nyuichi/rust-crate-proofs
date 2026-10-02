# Creusot unsigned narrowing-cast tool fix

This patch is isolated from the main crate checkout. It applies to Creusot
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc` (`0.11.0-dev`).

## Behavior

Rust unsigned narrowing casts in MIR are lowered to mathematical remainder
modulo `2^target_width`, then converted from the in-range remainder to the
target type. The lowering emits proof assertions for source nonnegativity and
the remainder range; it adds no assumptions, axioms, or trusted contracts.
Other cast classes retain their original lowering. The backend explicitly
imports `int.ComputerDivision` where the modulo term is emitted.

The associated Why3 driver patch activates the existing total SMT-LIB
`int2bv 64` conversion for BV64. The BV128 shift patch adds a triggered clone
of the unchanged standard `to_uint_lsr` fact and proves it from the original
fact before the witness scripts run.

## Evidence

- `witnesses/narrowcast/`: normal integer and bitwise `u128 as u64` witnesses,
  each one aggregate proof goal, both green.
- `witnesses/high-half/`: actual `(n >> 64i32) as u64` body with only the
  mathematical result postcondition. Its three explicit shift/range
  assertions are themselves proved; the unit has 8/8 green goals.
- `witnesses/ediv-shift/`: unchanged actual `n >> k` body, 1/1 green.
- The pinned Phase 2 itoa source snapshot passes both configurations with the
  new backend and overlay: 70 units / 192 goals per configuration.

The high-half witness demonstrates that the backend emits and proves the
program cast correctly; it does not make cast expressions in logic total. A
contract that needs a narrowing cast's mathematical meaning should avoid
repeating `as u64` in logic and state the result representation directly.

## Reproduction

The complete pins and environment are in `versions.md`. Rebuild the patched
driver and run `scripts/run-high-half-witness.sh` or
`scripts/run-narrowcast-witness.sh`. The baseline replay is
`scripts/run-phase2-baseline.sh /workspace/rust-crate-proofs`.
