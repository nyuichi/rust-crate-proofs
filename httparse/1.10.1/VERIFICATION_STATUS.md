# httparse 1.10.1 verification status ledger

This is an honest P0 checkpoint. No crate-owned algorithm body has a proved
Creusot body or a runtime-refinement bridge yet. The default and no-default
native test suites pass; exact counts and target/backend conditions are in
`PROVENANCE.md`. The logical byte, token, and chunk models are model-only and
have not yet been proven or connected to executable bodies.

**Full verification gate: OPEN.** `verify-all.bash` refuses to run until this
gate is explicitly closed and every ledger row reports reviewed contracts,
proved bodies, a runtime bridge, all integrated configurations, and no trusted
or excluded gap.

| Component | Contract reviewed | Body proved | Runtime bridge | Integrated configuration | Trusted / excluded gap and removal condition |
|---|---|---|---|---|---|
| Error, `Status`, `Result`, `InvalidChunkSize`, formatting | no | no | no | none | All behavior open; `Status::unwrap(Partial)` panic is specified behavior. |
| Seven `ParserConfig` flags and public setters/getters | no | no | no | none | Exact builder state and every parse interaction remain open. |
| Request and response constructors, fields, parsing, partial/error mutation | no | no | no | none | Exact result offset and state mutation on every exit remain open. |
| Method, URI, version, reason, status code, line-ending and whitespace parsers | no | no | no | none | Exact cursor, UTF-8, byte-span, error precedence and partial behavior remain open. |
| Header line state machine, capacity, trimming, obsolete folding, ignored invalid lines | no | no | no | none | Exact initialized prefix, overwritten slots, and error/partial precedence remain open. |
| Initialized/uninitialized header conversions and `ShrinkOnDrop` | no | no | no | none | Raw-slice view and initialization-prefix proof remains open; no local trust accepted yet. |
| Chunk-size parser | no | no | no | none | Hex recurrence, <=16 digits, termination, malformed input and exact extension behavior remain open. |
| `Bytes` public `_benchable` API and pointer/lifetime invariant | no | no | no | none | All unsafe preconditions, same-allocation provenance, and slice semantics remain open. |
| SWAR scanners | no | no | no | none | Word/lane mask and conservative-prefix proof remains open. |
| SSE4.2 and AVX2 scanners | no | no | no | none | Intrinsic semantics, loads, target features, masks and scanner refinement remain open. |
| NEON scanners | no | no | no | none | Intrinsic semantics, loads, target features, masks and scanner refinement remain open. |
| Runtime SIMD dispatch and cache | no | no | no | none | Atomic-state/feature invariant and all runtime backend paths remain open. |
| All public/trait/derived adapters and std/no_std surfaces | no | no | no | none | Formatting/error trait bodies, generic `TryFrom` adapter, reexports and cfg matrix remain open. |
| `build.rs` backend cfg generation and supported target selection | no | no | no | none | Compiler-version, architecture, feature and environment cfg outcomes must be checked against each reachable backend. |
| Independent byte, token and span models | no | no | no | none | Logic-only model; bodies and representative runtime consumers remain unproved/unlinked. |
| Independent chunk-size state machine | no | no | no | none | Logic-only model; no body proof or runtime refinement connection yet. |

The contract checklist and full callable/unsafe inventory are in
[`API-INVENTORY.md`](API-INVENTORY.md). A crate-level success requires exact
contracts for every public and doc-hidden callable, proof of all executable
crate bodies and unsafe obligations, mechanically checked runtime refinement
for every reachable backend, and integrated target-scoped proof plus runtime
matrix. Tests, finite byte differentials, and proof-model-only code do not close
those gaps. The current trusted local boundary count is zero; this does not mean
the crate is verified or that the toolchain's standard-library assumptions are
closed.
