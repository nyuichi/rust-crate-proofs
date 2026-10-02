# `mulhi` residual composition attempt

Base commit: `51090b4`. Candidate source is `u128_ext.rs`; its one-file patch
against the base is `MULHI_CANDIDATE.patch`. Runtime arithmetic is unchanged.
The compact residual helper is opaque to its caller but its body is explicit
and independently proves 8/8 goals. The latest attempt binds the returned
residual and exact-floor result with snapshots. The `mulhi_limb_identity`
helper separately passes 7/7 goals but is not called by this attempt.

The bounded 90-second run used:

```sh
cargo creusot --simple-triggers=false prove --no-cache \
  verif/itoa_rlib/arithmetic/u128_ext/mulhi.coma
```

The run timed out with `vc_mulhi` at 49/51. In the generated COMA the two
unresolved formulas are:

```text
x * y == result * 2^128 + residual
quotient == result
```

The second formula is a caller obligation after
`exact_floor_from_split(x * y, result, residual, 2^128)`. Its adjacent goal
`quotient == x * y / 2^128` passes. The residual helper contract and range facts
also pass. This is an unfinished caller-side composition, not evidence that the
formulas are unprovable. No trusted declaration or runtime algorithm change was
added.

The replay artifacts are colocated here: `u128_ext.rs`, `MULHI_CANDIDATE.patch`,
`mulhi.coma`, `proof.json`, and `bounded-run.log`.
