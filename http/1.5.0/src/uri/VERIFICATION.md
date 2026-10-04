# URI proof map

This file records the URI proof boundary and status for `http` 1.5.0.

## Runtime API and representation

| API | Runtime representation / behavior |
|---|---|
| `Uri` | `Scheme`, `Authority`, and `PathAndQuery`; an absent scheme is `Scheme2::None`, and absent authority/path are empty component values. |
| `Parts` | Optional `Scheme`, `Authority`, and `PathAndQuery`; `Uri::from_parts` rejects scheme without authority or path, and authority plus path without scheme. |
| `Uri::builder`, `Builder::{new,scheme,authority,path_and_query,build}` | Builder carries `Result<Parts, crate::Error>`; parse/conversion errors are retained until `build`. |
| `Uri::{from_maybe_shared,from_static}` and `TryFrom` / `FromStr` | Parse request-target bytes or strings. Empty and too-long inputs are rejected. A one-byte `/` or `*` is special; a leading slash is path-and-query; other forms go through `parse_full`. |
| `Uri::{path_and_query,path,scheme,scheme_str,authority,host,port,port_u16,query}` | Accessors derive optionality from empty/`None` component representations and return slices into component data. |
| `Authority::{from_static,from_maybe_shared,host,port,port_u16,as_str}` | Exact `ByteStr` representation; host and port are parsed from authority text. Validation is in `validate_authority_bytes`. |
| `PathAndQuery::{from_static,from_maybe_shared,path,query,as_str}` | Exact `ByteStr` representation plus a cached query offset (`u16::MAX` means absent). Validation and offset discovery are in `scan_path_and_query`. |
| `Scheme::{HTTP,HTTPS,as_str}` | `Scheme2::Standard(Protocol)` canonicalizes the fixed protocols; `Scheme2::Other(ByteStr)` preserves spelling. Equality/hash are ASCII case-insensitive. |
| `Port<T>::as_u16,as_str` | Numeric `u16` plus the original textual representation `T`. `View` exposes the numeric value; `DeepModel` uses its mathematical integer, matching the `PartialEq` semantics that intentionally ignore the text. Parsing delegates decimal syntax to `u16::from_str`. |
| Component conversions, comparisons, formatting, and hashing | `Authority`/`Scheme` compare and hash ASCII case-insensitively; `PathAndQuery` and `Uri` compare their exposed components; formatting uses component text. |

The intended byte-level relation is that a component's stored `ByteStr` is the
same `Seq<u8>` as the corresponding validated input slice. `PathAndQuery::query`
uses `query == u16::MAX` for no query, otherwise the query begins at
`query + 1`; `Authority` and `Scheme2::Other` retain exact validated byte
sequences. `Uri::parse_full` partitions the input around `scheme://`, the
authority, and the path/query suffix. The current `Bytes` ownership/splitting
representation and `ByteStr` UTF-8 wrapper are separate proof dependencies.

`Port<T>`'s numeric `View` and `DeepModel` deliberately use plain `#[logic]`
definitions, keeping the model abstract at public boundaries while the
definitions remain available to the containing module. Astra identified this
as the Creusot visibility pattern for a public wrapper over a private scalar;
marking the same definitions `#[logic(open)]` causes a private-field
transparency error.

## Parser dependency graph

```text
TryFrom<&[u8]> / from_shared
  -> empty, length, single-byte, and leading-slash dispatch
  -> parse_full
       -> Scheme2::parse
       -> Authority::parse -> validate_authority_bytes
       -> PathAndQuery::from_shared -> scan_path_and_query
```

`Scheme2::parse_exact` validates standalone scheme spellings. Its fixed
HTTP/HTTPS fast paths are ASCII-case-insensitive; the generic scheme path scans
the scheme character table and enforces the 64-byte limit. `validate_authority_bytes`
tracks colon count, bracket state, percent state, the last `@`, and the end of
the authority prefix. `scan_path_and_query` performs separate path and query
passes and returns the cached query offset, fragment offset, and a flag that
indicates whether UTF-8 validation is needed.

## Status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| URI API and representation map | yes | n/a | no | no |
| `Port<T>::as_str` / `AsRef<str>` | pending | no | no | no |
| `Port<T>::as_u16`, numeric conversion, and comparisons | yes | yes; five bodies, 7 VCs, plus four refinements/8 VCs | no | no; actual-source harness only |
| `Port<T>::from_str` | pending | no | no | no |
| ASCII HTTP/HTTPS prefix comparison used by `Scheme2::parse` | yes | yes (`ascii_lowercase`: 1 VC; `eq_ascii_prefix`: 3 VCs) | no | no; actual-source helper harness only |
| remaining scheme classification and parsing | pending | no | no | no |
| authority scanner and constructors | pending | no | no | no |
| path/query scanner and constructors | pending | no | no | no |
| `Uri` parsing, constructors, and accessors | pending | no | no | no |
| `Builder` and component conversions | pending | no | no | no |
| URI equality, formatting, and hashing | pending | no | no | no |

No temporary trusted URI-owned boundary has been introduced. Full parser
verification remains dependent on a model for the downcast/raw-pointer path in
`from_maybe_shared`, and on exact models for the `Bytes` and `ByteStr`
representations. Any temporary boundary, if later required for architecture
experiments, must be removed before claiming full URI verification.

The first whole-module run of the source-inclusion harness passed the
`Port::as_u16` body and its one functional VC, but failed on unrelated
`Debug`/`Display` formatting adapters, generic `AsRef<str>`/`u16::from_str`
calls, and some `PartialEq<u16>` refinement goals. Those failures are retained
as partial status; they are not evidence that the URI component is integrated.
Targeted actual-source proof ran from `verification/uri` with the URI-only
target directory:

```sh
CARGO_TARGET_DIR=/workspace/proof-tools/targets/http-uri ../../scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove \
  verif/http_uri_proof_rlib/port/impl_Port_T/as_u16.coma \
  verif/http_uri_proof_rlib/port/impl_From_for_u16/from.coma \
  verif/http_uri_proof_rlib/port/impl_PartialEq_for_Port_T/eq.coma \
  verif/http_uri_proof_rlib/port/impl_PartialEq_for_Port_T_0/eq.coma \
  verif/http_uri_proof_rlib/port/impl_PartialEq_for_u16/eq.coma \
  -- --locked --offline
```

The first run before adding an output postcondition to `From<Port<T>> for u16`
proved five production-source function bodies with six VCs. A second run
proved the matching `from__refines.coma` and three `eq__refines.coma` files;
those four refinement files passed eight VCs total. The final source adds
`result == port@` to the `From` body and its body/refinement reproof passes two
body VCs and one refinement VC. Counting the latest proof for that method in
place of the earlier one, the current five-body result is seven VCs plus eight
refinement VCs total:

```sh
CARGO_TARGET_DIR=/workspace/proof-tools/targets/http-uri ../../scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove \
  verif/http_uri_proof_rlib/port/impl_From_for_u16/from__refines.coma \
  verif/http_uri_proof_rlib/port/impl_PartialEq_for_Port_T/eq__refines.coma \
  verif/http_uri_proof_rlib/port/impl_PartialEq_for_Port_T_0/eq__refines.coma \
  verif/http_uri_proof_rlib/port/impl_PartialEq_for_u16/eq__refines.coma \
  -- --locked --offline
```

The HTTP crate itself and the full URI module have not had a successful
integrated run.

The harness includes production `port.rs` directly. It defines small
`ErrorKind` and `InvalidUri` shims only because the still-unproved
`Port::from_str` body needs those names to type-check; the proved numeric
accessor, conversion, and comparison bodies do not reference the shims. The
`From<Port<T>> for u16` implementation now has the explicit postcondition
`result == port@`; its body and refinement were replayed with that contract.

```sh
CARGO_TARGET_DIR=/workspace/proof-tools/targets/http-uri ../../scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove \
  verif/http_uri_proof_rlib/port/impl_From_for_u16/from.coma \
  verif/http_uri_proof_rlib/port/impl_From_for_u16/from__refines.coma \
  -- --locked --offline
```

The production `scheme/fixed.rs` helper is included directly in the harness and
is called by the HTTP/HTTPS fast paths in `Scheme2::parse`. Its proof command,
from `verification/uri`, was:

```sh
CARGO_TARGET_DIR=/workspace/proof-tools/targets/http-uri ../../scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove \
  verif/http_uri_proof_rlib/fixed/ascii_lowercase.coma \
  verif/http_uri_proof_rlib/fixed/eq_ascii_prefix.coma \
  -- --locked --offline
```

Both production helper bodies passed (four VCs total). `Scheme2::parse` and
`Scheme2::parse_exact` themselves are still unproved.
