# Catalogue path compatibility

The Rust source imports `rand_core::TryRng` and implements that trait for
`rand_xorshift::XorShiftRng`. cargo-proofs 0.4.1 resolves the source callable as
`<rand_xorshift::XorShiftRng as rand_core::TryRng>::try_next_u32`.

The service catalogue instead records
`<rand_xorshift::XorShiftRng as TryRng>::try_next_u32`, with API ID
`161f2dfbfba984679a7888b7460c6f0eb6b4365bb35f76cfdfcdb94977f3b357`.
The full path returned no catalogue match; the short path resolved uniquely to
this API. The self type, imported trait, and method identify the same callable.

`cargo-proofs-rand-core-trait-alias.patch` adds only this explicitly confirmed
external-trait alias to the existing allowlist. It retains the full path as
well. It does not generate basename aliases for arbitrary traits or change
the publication resolver's exactly-one-match requirement.

This is a publication-path compatibility change. The source, generated models,
theorem, Lean axiom checks, and Kani evidence are unchanged.
