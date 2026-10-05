# Current-source StatusCode and Version API audit

The current Status/Version/ASCII emission was reconciled against the accepted scalar proof snapshot: every one of the 80 supported targets has identical full untransformed Why3 stdout, a matching recorded proof JSON, and a complete successful proof tree. The 381 reusable leaves comprise 133 own-goal leaves and 248 callee-contract leaves. The solver-free arity audit matched seven root `split_vc` nodes (60 child tasks); this snapshot has no nested tactic nodes. See [task-stream-reconciliation.json](task-stream-reconciliation.json), [final proof evidence](../final-2026-10-05.json), and [the combined manifest](../../../method/evidence/current-source-reconciliation-20261005/manifest.json).

## `StatusCode`

The emitted runtime bodies cover `from_u16` (accepts exactly 100–999), three-byte decimal `from_bytes` and its `TryFrom`/`FromStr` wrappers, `as_u16`, exact three-digit `as_str` for all 900 valid codes, exact `canonical_reason`, and all five class predicates. Equality/order, `Default`, cloning/conversions, `Debug` and `Display`, `InvalidStatusCode` construction and formatters are included. Display formatter append preservation is proved; a separate runtime regression checks flags, partial writes, and errors. Hash-body claims are limited to preservation of the valid hasher invariant, with protocol compatibility covered by the recording-hasher test.

The 62 public numeric status constants are not emitted in the leaf configuration. Their macro initializer reaches a Creusot translator failure, `unsupported constant expression pattern_type!(u16 is 1..)`, for the `NonZeroU16` niche field; replacing the initializer with checked `NonZero::new` reaches the same unsupported field lowering. This is the known Creusot limitation recorded in [status-constants-pilot-2026-10-05.txt](../status-constants-pilot-2026-10-05.txt). No proof claim is made for those constant initializers. The ghost-only reason-phrase constants are separate from the production numeric constants.

## `Version`

`Version` and its private `Http` representation have exact numeric modeling, equality and ordering, `Default` at model value 11, cloning, Hash callback body/refinement, and `Debug` body/refinement in the accepted proof set. `version.rs` is byte-identical to the recorded source snapshot. Its five public protocol constants are direct enum-valued initializers and have no separate body VCs. The Hash proof establishes hasher-invariant preservation rather than a digest value or general hash coherence; the recorded-hasher regression compares the callback trace with the former derived representation.

These are isolated production-source body proofs, not an integrated proof of all HTTP modules. `NonZeroU16`, standard-library conversion/formatting/hash contracts, and their implementations remain explicit library boundaries.
