# `http` 1.5.0 specification map

This map describes the relations required for a complete proof of the published
runtime source. It does not claim those relations have all been implemented or
proved. Every public declaration is tracked in [`API_INVENTORY.json`](API_INVENTORY.json).

## Public values and runtime representation

| Public surface | Runtime representation | Intended logical view and invariant | Main proof boundaries |
|---|---|---|---|
| `StatusCode` | `NonZeroU16` plus numeric constants | Integer in `[100, 999]`; decimal text is exactly three ASCII digits; classifications use decimal ranges | `NonZeroU16` construction/access, decimal decoding, formatting, reason-phrase table |
| `Version` | closed private `Http` enum | One of HTTP/0.9, 1.0, 1.1, 2, or 3; default is HTTP/1.1 | enum observer, `Default`, debug formatting |
| `Method` | known-method enum or inline/allocated extension bytes | exact token byte sequence, valid method-token grammar; known-method safety/idempotence classifications | token validator, inline length bound, UTF-8 view, enum/string equality |
| `ByteStr`, `HeaderName`, `HeaderValue` | `bytes::Bytes`, static strings, and compact tags | exact `Seq<u8>` contents; `ByteStr` is valid UTF-8; the intended RFC `HeaderName` token property is refuted by a current-source counterexample and is blocked; the `HeaderValue` byte predicate is guaranteed by safe checked constructors, not by every value (the unsafe unchecked constructor has a UTF-8 safety precondition only) | planned `bytes` sequence/ownership contracts, conversion, validators, case folding, display/ordering/hash. `from_lowercase(b"foo\"bar")` currently accepts a quote while `from_bytes` rejects it although RFC `tchar` excludes it; see [`HEADER_ANOMALIES.md`](HEADER_ANOMALIES.md). Observed behavior is separate from the intended RFC property. |
| `HeaderMap<T>` and entries/iterators | raw bucket arrays, linked extra-value storage, pointer-based cursors | finite map from normalized header name to ordered value sequences, including repeated values; capacity and ownership invariants for slots and iterators | reserve/grow, insertion/append/removal, duplicate-value chains, entries, iterator/drain/drop, pointer permissions |
| `Authority`, `PathAndQuery`, `Scheme`, `Port`, `Uri` | compact tags plus shared `ByteStr` values | exact component bytes; URI parse/format and parts conversions preserve decomposition; authority/path/scheme grammar and length bounds | byte scanning, split points, percent/path rules, numeric port parsing, shared-byte ownership |
| `Request<T>`, `Response<T>`, builders and `Parts` | concrete generic body plus method/URI/version/headers/extensions | field-wise product view; `into_parts`/`from_parts` and `map` preserve all unchanged fields and body ownership | construction, field access/mutation, builders, error accumulation, parts round trip |
| `Extensions` | `HashMap<TypeId, Box<dyn AnyClone + Send + Sync>>` | typed existential map; insertion/get/remove/clone preserve type identity and downcast correspondence | sound `Any`/`AnyClone` model, allocation ownership, clone witness, downcast frame facts |
| `Error` and conversions | private `ErrorKind` enum containing supported concrete error variants; `&dyn Error` is formed by `get_ref` / `source` | error source and conversion identity; variant projections return the corresponding concrete error reference | `std::error::Error` object model for `get_ref` / `source`, conversion bodies |

## Shared dependency model

For HTTP-owned byte strings, the intended model is exact `Seq<u8>`. The user
authorized a conditional HTTP proof to assume that `bytes` verification has
finished; this tree still lacks the local exact-content observer and narrow
external specifications that let HTTP bodies use that premise. This is an
HTTP modeling task, not a reason to rerun `bytes`. The required `Bytes` and
`BytesMut` contracts cover construction, exact copy, clone, borrowed views,
split/freeze ownership, and mutable append/write effects. See
[`DEPENDENCY_CONTRACTS.md`](DEPENDENCY_CONTRACTS.md) for their status. The
observer must not hide an HTTP runtime implementation, and no local trusted
runtime helper may stand in for missing dependency contracts.

The direct trait-object and raw-pointer cases remain explicit blockers until
Creusot can translate them with a sound representation and ownership model.
No `cfg(creusot)` module may replace the published runtime module in an
integrated result. Small verification harnesses may include actual source files
to isolate a leaf; their results are recorded as leaf proofs, not crate proofs.

## Intended component order

1. Prove scalar leaf facts such as status validation/classification, version
   default, method token validation, and byte validators.
2. Prove exact `ByteStr`, name/value, and URI component transformations against
   the exact byte-sequence observer.
3. Prove public constructors and conversions using those leaf contracts.
4. Prove `HeaderMap` ownership and duplicate-value state transitions, then its
   iterators and drains.
5. Prove `Request`, `Response`, and builder composition field by field.
6. Resolve typed existential contracts for `Extensions` and downcast errors.
7. Run the complete original module set through default and all-features
   Creusot integration; keep the supported runtime test matrix green.

Each loop uses one canonical index/progress measure. Proof status is recorded as
contract reviewed, body proved, trusted, or integrated; a helper proof never
counts as an integrated crate proof.
