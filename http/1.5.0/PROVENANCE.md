# `http` 1.5.0 provenance and verification status

**Overall status: in progress; no full-crate proof is claimed.** The API
inventory is generated from the modified verification tree, records each
declaration's official-archive source line where it matches, and marks added
verification-support declarations separately from the upstream runtime API.
The recorded integrated runtime matrix passes on its recorded source snapshot.
The current worktree contains later Map, Name, Value, and URI WIP beyond the
older whole-production frontend snapshot; each newer proof is limited to its
named source/target scope below. Creusot has proved selected actual-source
leaves and bounded API groups, not the crate as a whole.

## Published source

The source is the official crates.io archive for `http` 1.5.0. The crates.io
sparse index marks 1.5.0 non-yanked and records it as the latest 1.x release at
the time of selection. The archive SHA-256 is
`918d3568bebf352712bc2ef3d46a8bcf1a75b373be6539de198e9105cbbf9ce0`, matching
the index checksum. The archive's `.cargo_vcs_info.json` names upstream commit
`e559023f67e3fad6ecc3ee91307be178e0f13626` and marks the release tree dirty;
the archive checksum is the source identity used here.
The checked-in archive is `provenance/http-1.5.0.crate`; the inventory
generator verifies its checksum before comparing source lines.

`Cargo.toml.orig` preserves the published dependency declarations. The local
verification manifest pins `bytes` to the registry release `=1.11.1` and uses
the repository's proof-annotated `itoa` source under its local package alias
`itoa-proof` (the library name remains `itoa`). The lockfile captures this
verification graph. The official archive ignores lockfiles by default; this
crate's `.gitignore` was adjusted so root and harness lockfiles remain visible
for reproducibility. See [`itoa`'s independent provenance record](../../itoa/1.0.18/PROVENANCE.md)
for that dependency's evidence and trust boundary. Runtime APIs and the
upstream `std` requirement are kept.

## Explicit dependency premise for `bytes`

The user authorized treating the in-progress `bytes` validation as complete
for planning this HTTP verification. That instruction is a task premise, not
proof evidence present in this workspace: the local `bytes/1.11.1` copy still
has a length-only Creusot model, and this HTTP tree currently has no
`bytes_seq` observer or exact-content external specifications. Therefore no
HTTP byte-content claim is yet proved or mechanically connected to the
assumed result. The intended dependency contracts are:

- `Bytes` and `BytesMut` expose exact initialized byte contents as `Seq<u8>`;
- constructors, copies, clones, splits, conversions, and `freeze` preserve the
  corresponding sequence and ownership relation;
- borrowed views (`AsRef<[u8]>`, `Deref<[u8]>`) equal the model sequence;
- mutable writes extend or update that sequence exactly and remain within the
  initialized/capacity invariant.

The user-authorized premise permits a **conditional HTTP proof** to use those
contracts without restarting the `bytes` validation. The workspace copy must
not be used as evidence for exact contents, and this tree still needs a named
opaque observer plus narrow external specifications before byte-dependent
HTTP bodies can consume the premise. That is an HTTP modeling gap, not a
request to rerun `bytes`. No HTTP runtime method may be marked trusted to
compensate for a missing dependency interface. The `bytes` source tree has not
been edited or reverified as part of this task. The exact assumed contracts
and their current implementation status are recorded in
[`DEPENDENCY_CONTRACTS.md`](DEPENDENCY_CONTRACTS.md).

## Feature and runtime matrix

The published crate has only the `std` feature, enabled by default. Its source
contains an upstream `compile_error!` when `std` is disabled, so `--no-default-
features` is unsupported and is not a proof configuration. The supported proof
matrix is default `std` and `--all-features`; these currently select the same
feature set.

At the recorded runtime-check source snapshot, the full suite passed with the
pinned toolchain and locked dependencies; this is not a rerun of the later WIP
sources described in the checkpoint:

| Configuration | Unit and integration tests | Doctests | Result |
|---|---:|---:|---|
| default `std` | 181 | 228 | pass |
| `--all-features` | 181 | 228 | pass |
| `--no-default-features` | compile check | n/a | expected upstream `std`-required compile error |

Commands, run from this directory:

```sh
scripts/run-proof.sh cargo test --locked --offline
scripts/run-proof.sh cargo test --all-features --locked --offline
scripts/run-proof.sh cargo check --no-default-features --locked --offline
```

The no-default command is expected to fail at the source's explicit
`compile_error!`; it does not indicate a regression. The exact diagnostic is
preserved in [`verification/no-default-features.expected.log`](verification/no-default-features.expected.log).

## Full-verification requirements

The target is complete only when every upstream runtime declaration in
[`API_INVENTORY.json`](API_INVENTORY.json) has a reviewed contract or
invariant, every reachable implementation body
(including private helpers, unsafe blocks, iterator and `Drop` code) is proved,
dependency contracts are backed by evidence or explicitly taken as the
user-authorized `bytes` premise, and the target crate's default and
all-features integrated Creusot runs both succeed. Reviewed standard-library
and tool TCB boundaries are recorded separately; proving the standard-library
implementations themselves is not part of this HTTP target. The public API
ledger alone does not cover all reachable
implementation bodies; see [`IMPLEMENTATION_INVENTORY.md`](IMPLEMENTATION_INVENTORY.md)
for the source-component and unsafe/ownership checklist. HTTP-owned trusted
bodies, substitute modules that remove runtime implementations from
translation, and unlisted API exclusions do not satisfy this criterion.
`API_INVENTORY.md` summarizes upstream declarations separately from
verification-support additions; unproved statuses remain false until evidence
is recorded here. Trait impl records include the actual `Self` type and
generic trait arguments so methods such as `From<X> for Error` cannot be
misreported as methods on `X`.

Current proof evidence:

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| `header::value::is_valid` | yes | yes, 1 VC | no | no; actual-source leaf harness only |
| `header::value::is_visible_ascii` | yes | yes, 1 VC | no | no; actual-source leaf harness only |
| `HeaderValue` numeric/TryFrom group | yes, for the selected numeric conversions | yes; the latest accepted source-linked snapshot proves all 18 selected targets with 55/55 own leaves at `value.rs` SHA `5420fc76…`; the full profile emitted and archived 106 COMAs | no HTTP body trusted; the HTTP `bytes` interface remains an explicit user-authorized premise | no; this closes only the named numeric/TryFrom scope, not all HeaderValue APIs. The later formatter WIP snapshot at SHA `197fe107…` is outside this proof |
| `HeaderValue` formatters | yes, for partial formatter attempts | Partial: the archived attempt proves the four Error formatter bodies (15 own leaves total), while their refinements each retain one open leaf; the separate HeaderValue Debug body attempt did not close. A later WIP snapshot records 38 focused native tests passing. | no | no; the WIP manifest explicitly records native parity only, with no fresh Creusot emission or proof; formatter closure remains local open work |
| `HeaderName` / `HdrName` | yes, selected parsing, representation, equality/hash and formatting scopes | Base snapshot: 34 targets / 280 leaves; source-650 reconciliation: 28 targets / 336 own leaves with exact task and arity audits; frozen source SHA `8c463390…`: a fresh 17-target batch / 58 own leaves, all complete | no | no; `StandardHeader::from_bytes` on that same freeze is partial at 81/82 leaves. Its helper split and caller are not proved. Later Name structure WIP at recorded SHA `966107ba…` has no accepted proof. The 17-target proof and partial proof are recorded under `name_value_wip_current_2026-10-05`; this is not a 195-target closure |
| `VacantEntry::{key,into_key}` and `OccupiedEntry::{key,get,get_mut,into_mut}` | yes, with conditional in-range premise for occupied accessors | yes, 6 own leaves and 4 Vec-index support leaves | no | no; actual-source named leaf harness only; does not prove occupancy or reachability |
| `HeaderMap` lookup/getter resume | yes, conditional on `header_map_find_ready`; getter contracts have no functional result post, and sealed `find` only bounds a returned index | The accepted 13-target snapshot at map source freeze SHA `ca22bfb5…` has 10 HTTP body roots + 3 refinements and 30 literal-`true` imported support roots; all 43 direct roots pass. An independent direct-root arity audit is archived. | no HTTP body trusted | no; conditional modular bodies/refinements only. `find`/`find_with_hash` have meaningful `Some` slot/hash/key posts but unconstrained `None`; the getters have no functional result posts. No absence theorem, key/hash coherence, or constructor/mutation establishment is proved. The unpublished functional-lookup WIP's first frozen attempt failed in the Rust frontend before COMA generation; a later frontend check passed but has no proof result |
| URI Parts/default/conversions + Authority helpers/Hash/PartialEq/PartialOrd | yes, selected actual-source contracts | yes; fresh 45-target checkpoint, 194 own terminal leaves (Parts/default/conversions 55, Authority helpers 42, Hash 22, PartialEq 41, PartialOrd 34) | no | no; named URI leaf harness only, no parser/full-crate claim |
| `Debug for Uri::fmt` | yes, success text and formatter-error prefix/append post | yes; body and refinement, 7 own leaves, independently arity-checked | no | no; the selected Debug proof consumes an opaque imported `Display::fmt` support contract. `Uri::Display::fmt` is separate and remains unproved (48 body + 3 refinement tasks emitted) |
| `Hash for Uri` | yes; `Hasher` invariant preservation | yes; body and refinement, 25 own leaves (22 body + 3 refinement), with a solver-free arity audit | no | no; [published proof manifest](verification/uri/evidence/uri-hash-proof-2026-10-05/manifest.json) is bound to the pre-Eq-fix source freeze. It proves invariant preservation only under modular getter, component Hash, and library Hash contracts; it does not establish a digest value, Eq/Hash coherence, or the `Hash<Authority>` / `Hash<Scheme>` bodies |
| Same-type `Uri` equality follow-up | yes, selected same-type equality relation | Partial failed run: 16/19 body leaves and 4/4 refinement leaves pass; 3 body leaves remain open (20/23 successful leaves). | no | no; the [published partial manifest](verification/uri/evidence/uri-eq-proof-2026-10-05/manifest.json) is not an Eq proof. `Uri`-to-`str` equality remains unproved |
| URI Builder API | yes, selected state, validation and parts-conversion posts | yes; 15 targets / 46 own + 36 support leaves. Fresh untransformed task streams match all 15 previously proved targets byte-for-byte; native Builder tests pass 8/8 | no | no; named actual-source leaf run only; no parser/full URI claim |
| `From<Authority>` / `From<PathAndQuery>` for `Uri` | yes; exact component-preservation and empty-component posts | yes; 2 bodies + 2 refinements, 6 own leaves; all four proof COMAs match fresh clean emission and independent split arity prints the six tasks | no | no; named URI leaf harness only. `Uri::Display`, same-type/`str` equality, parser and other conversions remain separate open work |
| Request/Response selected constructors/defaults | yes, selected body/postcondition scopes | yes; 15 targets / 41 leaves (15 own, 26 call/spec support) with exact run-log and direct-goal arities | no HTTP body trusted; imported HeaderMap/Extensions defaults remain support contracts | no; named composition profile only; distinct source snapshot, not additive to the 49-target archive |
| Request/Response Builder relays and fixed-method shortcuts | yes, selected conditional callback/error relays and exact fixed-method result models | yes; 35 targets / 280 terminal leaves (116 own, 164 call/spec support); all target roots match the proof trees and nested splits are audited | no HTTP body trusted; imported URI/Status/Version contracts and callback support remain explicit premises | no; named composition profile only. The post-`Error::is_empty_uri` independent v2 audit at [`report.json`](verification/composition/evidence/run-2026-10-05-current-builder-relays-and-shortcuts/current-source-reemit-after-error-core-v2/independent-astra-task-streams/report.json) (SHA `9d6f8721…`) confirms 9 current full Why3 stdout streams byte-identical and 26 other raw COMAs byte-identical to the archived proved inputs. This is not a claim that all 35 streams were identical; counts are not additive to the 49-target or 15-target snapshots |
| `HeaderMap::try_insert_entry` private helper | yes, exact success/error branch plus explicit field frame | yes; 1 own leaf and 3 Vec/error-constructor support leaves, with no-preprocess printer roots matching all four proof JSON keys | no | no; named actual-source map leaf profile; caller table-index update/lookup not proved |
| `uri::Port<T>::as_u16`, conversion, and three `PartialEq` bodies | yes | yes; 5 bodies/7 VCs, plus 4 refinements/8 VCs | no | no; actual-source leaf harness only |
| URI HTTP/HTTPS prefix helpers | yes | yes; 2 bodies, 4 VCs | no | no; actual-source leaf harness only |
| `header::map_capacity::{checked_raw_capacity,usable_capacity}` | yes | yes, 3 VCs | no | no; actual-source leaf harness only |
| `Method` | yes, selected production body and refinement contracts | yes; all 103 current-source Why3 task streams match the proved snapshot byte-for-byte, and all complete proof trees are reused: 877 successful leaves (737 own, 140 support). Independent audits cover 28 root nodes / 682 tasks and 7 nested nodes / 15 tasks | no HTTP body trusted; hasher invariant callbacks remain explicit std contracts | no; isolated Method/ASCII leaf profile; not integrated HTTP |
| `StatusCode`, `Version`, and private `Http` | yes, selected scalar and formatting contracts | yes; 80 current-source task streams and proof trees reused, 381 leaves (133 own, 248 support). Includes Status conversions, predicates, exact reason/text, equality/order/hash/formatting and selected Version/Http methods | no HTTP body trusted; std `NonZero` and hasher contracts are explicit TCB | no; 62 numeric `StatusCode` constants remain excluded after the recorded constant translation failure |
| public `http` API | in progress | no | no | no |

The generated API ledger currently has 21 explicit overrides in
[`API_EVIDENCE.json`](API_EVIDENCE.json). It does not yet map every newer
accepted proof group above to its public declarations, notably Method, Name,
Value numeric conversions, Map lookup, URI Builder/conversion targets,
`PathAndQuery::Display`, `Debug for Uri`, and `Hash for Uri`.
Therefore a `false` `body_proved` field in `API_INVENTORY.json` can mean that a
proved leaf has not yet been mapped to that declaration; it is not evidence
that every such body lacks any bounded proof. The declaration ledger remains
conservative, and these component snapshots still do not establish full API or
integrated coverage.

The captured whole-production frontend checks are not integrated proof runs:
the archived default and all-features profiles each stop at 19 dyn/experimental
diagnostics, with zero other errors, on stable 159-path source map SHA
`cc2009b2192b237b893e0ecae63eed6395e89aba13ab28321e002cfbaefce433`. They
emit no COMA and run no Why3 prover. Later Map lookup source work is outside
that captured map.

These are production-source leaf results through isolated harnesses; none is
an integrated `http` proof. Their evidence and commands are recorded in the
component verification notes: [`verification/headers/README.md`](verification/headers/README.md),
[`src/uri/VERIFICATION.md`](src/uri/VERIFICATION.md),
[`verification/map-capacity/README.md`](verification/map-capacity/README.md),
and [`verification/scalars/README.md`](verification/scalars/README.md). The runtime check in
[`HEADER_ANOMALIES.md`](HEADER_ANOMALIES.md) records an upstream header-name
parsing inconsistency; this work preserves the published behavior and does
not claim that the observed parser satisfies the intended RFC token property.

## Translation and representation boundaries

The published source must remain selected for both runtime and proof builds.
The current translation investigation has these compiler-sensitive areas:

- `Extensions` stores `dyn AnyClone + Send + Sync` and uses `Any` downcasts.
  The pinned Creusot clone-map elaborator only admits `Debug` and `Write` trait
  objects. Supporting `Any` requires a sound tagged-existential model, cast and
  downcast round-trip facts, and frame conditions; removing the allowlist is
  not a safe workaround.
- `HeaderMap` uses raw pointers, pointer arithmetic, `MaybeUninit`, and unsafe
  iterator/drop code. The current function translator rejects raw-pointer
  dereferences. A possible later design is an ownership-aware `Perm` model with
  the standard slice permission specifications, while retaining the exact
  upstream runtime operations. The required extensional and ownership
  invariants are detailed in [`RUNTIME_MODEL.md`](RUNTIME_MODEL.md).
- UTF-8 unchecked constructors and `NonZeroU16` constructors/accessors require
  reviewed standard-library preconditions and postconditions. They are not
  public API exclusions and cannot be treated as proved from a successful
  helper-only run.

These compiler limitations are active investigation gates, not accepted
boundaries for a claim of full verification. `SPECIFICATION_MAP.md` records the
runtime-to-model relations and intended proof components. The default and
all-features full-crate translation attempts both stop before VC generation;
the first production diagnostic and subsequent compiler ICE are recorded in
[`INTEGRATED_TRANSLATION.md`](INTEGRATED_TRANSLATION.md), with both full logs
linked there.

## Reproduction

`verify-all.bash` runs the supported default and all-features proof
configurations, stopping on its first failing command. The separate all-features
translation attempt is recorded in [`INTEGRATED_TRANSLATION.md`](INTEGRATED_TRANSLATION.md).
All proof commands must go through
[`scripts/run-proof.sh`](scripts/run-proof.sh), which serializes Why3 across
agents with `/tmp/http-creusot-proof.lock` and fixes resources at one prover
with 1024 MiB. Run the proof wrapper outside the filesystem sandbox, as required
by the repository instructions.
