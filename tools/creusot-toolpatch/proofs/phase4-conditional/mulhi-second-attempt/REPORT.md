# `mulhi` exact-result proof attempt 2

The retained `int-proof-assert/` snapshot is an experimental candidate against
repository commit `a0591dd`. It preserves the optimized limb implementation,
adds `mulhi_core`'s exact high-half postcondition, and removes the wrapper's
`trusted` attribute. The candidate is **not proven** and must not be applied
as a passing change.

## Focused result

An early focused `mulhi_core` variant reached 62/64 VCs. The independent
`mulhi_product_congruence` and `mulhi_product_expansion` helper bodies each
passed their single VC. Its two open caller VCs were applications of the
congruence helper's two postconditions at the snapshot call site:

```text
mulhi_product_congruence(x, y, a, b) == x * y
mulhi_product_congruence(x, y, a, b) == a * b
```

Here `a = x_hi * 2^64 + x_lo` and `b = y_hi * 2^64 + y_lo`; the two requires
(`x == a`, `y == b`) are already in the caller context. The helper body proves
the equality directly, but the opaque specification's quantified postcondition
is not being instantiated at the call site. The generated session identifies
these as `vc_mulhi_core.53` and `.54` (both timeout/unknown); neighboring VCs,
including the core ensures, are marked passed. The core ensures result is
conditional on these unproved assertions and is therefore not evidence of a
closed proof.

The `mulhi_product_expansion` helper is an unconditional four-limb polynomial
identity and passed independently. The focused helper proof results are
included for reuse.

## Reproduction

From `itoa/1.0.18`, using the pinned Creusot/Why3 environment and shared proof
runner, run the focused selector:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh cargo creusot \
  --no-check-version --simple-triggers=false prove --no-cache \
  --why3session runtime/u128_ext/mulhi_core -- --offline
```

`int-proof-assert/u128_ext.rs` is the exact final candidate source snapshot.
Its generated Why3 COMA, session status, bounded solver log, and helper proof
results are retained beside this report.

## Follow-up: direct split substitution

One follow-up variant split the two-factor substitution into the direct assertions
`x*y == (x_hi*2^64+x_lo)*y` and then rewrote `y`; the helper itself was also
changed from `#[logic(opaque)]` to `#[logic]`. Creusot still emitted the same
abstract function specification, but an independent COMA inspection found
that the first substitution assertion now discharged. The remaining open
caller obligation was the `mulhi_high_cross_identity` predicate assertion.

One bounded Why3 process for this variant hit repeated internal
`utils/timer.ml:20` assertion errors and did not produce a usable proof session
status. This was an infrastructure failure, not a proof result. A clean later
run produced the result below.

## Final isolated caller VC

The follow-up variants subsequently isolated one different, earlier caller
goal: the second product substitution. The exact open proposition is

```text
(x_hi * 2^64 + x_lo) * y
  == (x_hi * 2^64 + x_lo) * (y_hi * 2^64 + y_lo)
```

The assumption `y == y_hi * 2^64 + y_lo` is already established immediately
before this goal. The preceding substitution of `x` passed. The last
`int-proof-assert/` variant used the independently checked Int-return
congruence helper inside the `proof_assert!` block, following the form used by
`exact_floor_from_split`; the focused core result remained 67/68, with only
`vc_mulhi_core.54` open at the product substitution. Its generated COMA maps
the goal to the assertion above. The subsequent exact-result ensure is marked
passed in that session but depends on this unproved assertion, so the overall
exact-result claim remains unproven.

Other attempted proof shapes included direct factorization of the high-cross
identity and a transparent Boolean logic lemma. Both left the caller's product
substitution open. Only the passing helper-only checkpoint in the source tree
is suitable for integration.
