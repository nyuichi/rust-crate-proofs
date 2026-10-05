# httparse 1.10.1 verification status ledger

This is a partial P1 checkpoint. Request/Response constructors have proved
Creusot bodies and representative callers, but request/response parsing and
chunk parsing do not yet have proved bodies or runtime-refinement bridges.
`Status` operations and the seven `ParserConfig` flags have exact contracts and
direct shared-source proofs; exact commands, VC counts, and native test counts
are in `PROVENANCE.md`. The extracted byte-class source and table constructors
pass an isolated shared-source harness (29/29 VCs, including the four scalar
helper bodies and a small contract-composition caller). The latest committed
checkpoint retains the inline byte tables and predicates; the current worktree
has the extracted-module wiring, pending native crate tests and an integrated
runtime proof. The separate chunk model and runtime parser proof remain open.

**Full verification gate: OPEN.** `verify-all.bash` refuses to run until this
gate is explicitly closed and every ledger row reports reviewed contracts,
proved bodies, a runtime bridge, all integrated configurations, and no trusted
or excluded gap.

| Component | Contract reviewed | Body proved | Runtime bridge | Integrated configuration | Trusted / excluded gap and removal condition |
|---|---|---|---|---|---|
| Byte-class tables and scalar predicates (`src/byteclass.rs`, `byte_map!`) | yes: each helper is specified by the independent token, URI, or header-value predicate; table constructors specify every `u8` entry | yes: 3 table-constructor VCs, 4 helper bodies (3 VCs each, including initializer setters), and `classify_byte` caller (5 VCs); 20/20 total | direct shared-source proof in the isolated byteclass harness | isolated byteclass harness only; current worktree wires the extracted module, but native crate tests and integrated runtime proof remain pending | No trusted table-content axiom. The committed checkpoint keeps inline definitions; scanner and parser callers remain open. |
| `Status<T>` variants and inherent methods | yes: exact variants, predicates, and `unwrap` precondition/payload | yes: 3 methods (4 VCs); Clone body/refinement (4 VCs) | direct shared-source harness | isolated contracts harness | Derived Debug/Eq/PartialEq are excluded under `cfg(creusot)`; close after generic payload model, equality refinement, and formatter refinement are proved. Intentional `unwrap(Partial)` panic is precluded by its contract. |
| Seven `ParserConfig` flags, default, setters and getters | yes: exact seven-field view, all-false default, single-field setter updates/frames, and four getters | yes: default, 7 setters, 4 getters (12 VCs); Clone (2 VCs); 3-call chained builder caller (4 VCs) | direct shared-source harness imports `config.rs` | isolated contracts harness | The four `parse_*` adapters in `lib.rs` and every flag's effect on parser behavior remain open. |
| `Error`, `Result`, `InvalidChunkSize`, formatting | no | no | no | none | Error variants and formatting are open; Error is outside the contracts harness. |
| Request/Response constructors, fields, and representative callers (`src/message.rs`) | yes: all parsed scalar fields start as `None`, returned header slice matches the input sequence; `EMPTY_HEADER.value` is empty | yes: 9 VCs covering Request/Response constructors, two callers each, Header Clone, and the constant value accessor/initializer | direct shared-source proof in isolated message-core harness; no parser-body refinement | isolated message-core harness only; native default/no-default checks pass | Empty header name model remains open (`str::view` opaque; direct string literal contract ICEs); runtime Header traits and proof-excluded Request/Response derives remain open. |
| Request/response parsing, partial/error mutation, and public-field refinement | no | no | no | none | Exact parser result, cursor offset, field initialization, and mutation on every exit remain open. |
| Method, URI, version, reason, status code, line-ending and whitespace parsers | no | no | no | none | Exact cursor, UTF-8, byte-span, error precedence and partial behavior remain open. `parse_version` has an independent model in preparation; no shared runtime bridge or proof is claimed. |
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
| Independent byte, token, span, and cursor models | yes | yes: `maximal_prefix_end`, `accepted_prefix_span`, `parse_token_model`, `cursor_slice_skip`, and five Clone bodies; 9/9 VCs in the byteclass harness | no direct parser-body bridge | included in the isolated byteclass harness | Other cursor logic definitions and `Bytes` runtime contracts remain open; parser-stage models and executable scanner bridges remain open. |
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
