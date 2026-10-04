# httparse 1.10.1 verification status ledger

This is a partial P1 checkpoint. No request/response/chunk parser body has a
proved Creusot body or runtime-refinement bridge yet. `Status` operations and
the seven `ParserConfig` flags have exact contracts and direct shared-source
proofs; exact commands, VC counts, and native test counts are in
`PROVENANCE.md`. The independent byte/token model passes in isolation, but its
logical byte predicates have not been connected to runtime table or scanner
bodies. The separate chunk model remains model-only and unproved.

**Full verification gate: OPEN.** `verify-all.bash` refuses to run until this
gate is explicitly closed and every ledger row reports reviewed contracts,
proved bodies, a runtime bridge, all integrated configurations, and no trusted
or excluded gap.

| Component | Contract reviewed | Body proved | Runtime bridge | Integrated configuration | Trusted / excluded gap and removal condition |
|---|---|---|---|---|---|
| `Status<T>` variants and inherent methods | yes: exact variants, predicates, and `unwrap` precondition/payload | yes: 3 methods; Clone body/refinement also proved | direct shared-source harness | isolated contracts harness | Derived Debug/Eq/PartialEq are excluded under `cfg(creusot)`; close after generic payload model, equality refinement, and formatter refinement are proved. Intentional `unwrap(Partial)` panic is precluded by its contract. |
| Seven `ParserConfig` flags, default, setters and getters | yes: exact seven-field view, all-false default, single-field setter updates/frames, and four getters | yes: default, all 7 setters, all 4 getters, and a 3-call chained builder caller; 26 VCs total across the Status/config checkpoint | direct shared-source harness imports `config.rs` | isolated contracts harness | The four `parse_*` adapters in `lib.rs` and every flag's effect on parser behavior remain open. |
| `Error`, `Result`, `InvalidChunkSize`, formatting | no | no | no | none | Error variants and formatting are open; Error is outside the contracts harness. |
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
| All public/trait/derived adapters and std/no_std surfaces | partial: Status/config inherent surface only | partial: Status/config Clone; config Default | partial: direct harness covers extracted modules only | native default and no-default tests pass; parser adapters remain open | Debug derivations and Status Eq/PartialEq are excluded under `cfg(creusot)`; Error formatting, generic `TryFrom`, other trait adapters, reexports and full cfg matrix remain open. |
| `build.rs` backend cfg generation and supported target selection | no | no | no | none | Compiler-version, architecture, feature and environment cfg outcomes must be checked against each reachable backend. |
| Independent byte, token and span models | yes | yes: `maximal_prefix_end`, `accepted_prefix_span`, and `parse_token_model`; 7/7 total VCs including four derived Clone bodies | no | isolated harness only | Model-only; byte-table, cursor and executable scanner bridges remain open. |
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
