# `http` 1.5.0 provenance and verification status

**Overall status: in progress; no full-crate proof is claimed.** The API
inventory is generated from the modified verification tree, records each
declaration's official-archive source line where it matches, and marks added
verification-support declarations separately from the upstream runtime API.
The integrated runtime suite passes for the supported configurations.
Creusot has proved small leaves through actual-source harnesses so far; those
runs do not prove the crate as a whole.

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

The full runtime suite passed with the pinned toolchain and locked dependencies:

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
| `HeaderValue::from(u16)` / `hex_digit` | yes | yes, 3 targets / 12 own leaves (From body 7, refinement 1, helper 4) | no HTTP body trusted; Bytes remains a stated dependency premise | no; source-linked named leaf snapshot only; the other integer conversions remain open |
| `VacantEntry::{key,into_key}` and `OccupiedEntry::{key,get,get_mut,into_mut}` | yes, with conditional in-range premise for occupied accessors | yes, 6 own leaves and 4 Vec-index support leaves | no | no; actual-source named leaf harness only; does not prove occupancy or reachability |
| URI Parts/default/conversion partial batch | yes, selected contracts | 48 of 55 expected own terminal leaves passed; 7 remain unresolved | no | no; named URI leaf harness only |
| `uri::Port<T>::as_u16`, conversion, and three `PartialEq` bodies | yes | yes; 5 bodies/7 VCs, plus 4 refinements/8 VCs | no | no; actual-source leaf harness only |
| URI HTTP/HTTPS prefix helpers | yes | yes; 2 bodies, 4 VCs | no | no; actual-source leaf harness only |
| `header::map_capacity::{checked_raw_capacity,usable_capacity}` | yes | yes, 3 VCs | no | no; actual-source leaf harness only |
| `Version::default` | yes | yes, 1 VC, and its refinement, 1 VC | no | no; actual-source leaf harness only |
| `StatusCode::{from_u16,from_bytes,as_u16}` and five classifications | yes | yes; 8 + 6 + 12 = 26 VCs | no HTTP body trusted; explicit std `NonZero<T>` TCB | no; actual-source leaf harness with documented partial exclusions |
| public `http` API | in progress | no | no | no |

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
