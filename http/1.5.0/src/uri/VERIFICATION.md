# URI proof map

This file records the URI proof boundary and status for `http` 1.5.0.

> **Checkpoint note (2026-10-05):** The latest bounded URI snapshot is
> [`verification/uri/evidence/parts-authority-comparison-2026-10-05/manifest.json`](../../verification/uri/evidence/parts-authority-comparison-2026-10-05/manifest.json):
> 45 selected targets and 194/194 independently arity-audited own leaves pass
> (Parts/default/conversions 55; Authority fold/case helpers 42; Hash 22;
> PartialEq 41; PartialOrd 34). This is a named source-inclusion harness, not
> a full parser or integrated-crate proof. It supersedes the overlapping older
> 119-leaf Authority and 48/55 Parts snapshots; counts must not be added.
> Earlier rows below describe their individual archived targets and do not
> imply full component closure where they say “pending”.

## Runtime API and representation

| API | Runtime representation / behavior |
|---|---|
| `Uri` | `Scheme`, `Authority`, and `PathAndQuery`; an absent scheme is `Scheme2::None`, and absent authority/path are empty component values. |
| `Parts` | Optional `Scheme`, `Authority`, and `PathAndQuery`; `Uri::from_parts` rejects scheme without authority or path, and authority plus path without scheme. |
| `Uri::builder`, `Builder::{new,scheme,authority,path_and_query,build}` | Builder carries `Result<Parts, crate::Error>`; parse/conversion errors are retained until `build`. |
| `Uri::{from_maybe_shared,from_static}` and `TryFrom` / `FromStr` | Parse request-target bytes or strings. Empty and too-long inputs are rejected. A one-byte `/` or `*` is special; a leading slash is path-and-query; other forms go through `parse_full`. |
| `Uri::{path_and_query,path,scheme,scheme_str,authority,host,port,port_u16,query}` | Accessors derive optionality from empty/`None` component representations and return slices into component data. |
| `Authority::{from_static,from_maybe_shared,host,port,port_u16,as_str}` | Exact `ByteStr` representation; host and port are parsed from authority text. Validation is in `validate_authority_bytes`. |
| `PathAndQuery::{from_static,from_maybe_shared,path,query,as_str}` | Exact `ByteStr` representation plus a cached query offset (`u16::MAX` means absent). The constructors call the shared scanner in `path_scan.rs`; `from_maybe_shared` also has an unverified `dyn Any` sharing optimization. |
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
       -> PathAndQuery::from_shared -> path_scan::scan_path_and_query
```

`Scheme2::parse_exact` validates standalone scheme spellings. Its fixed
HTTP/HTTPS fast paths are ASCII-case-insensitive; the generic scheme path scans
the scheme character table and enforces the 64-byte limit. `validate_authority_bytes`
tracks colon count, bracket state, percent state, the last `@`, and the end of
the authority prefix. `path_scan::scan_path_and_query` performs separate path
and query passes and returns the cached query offset, fragment offset, and a
flag that indicates whether UTF-8 validation is needed.

## Status

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| URI API and representation map | yes | n/a | no | no |
| `Port<T>::as_str` / `AsRef<str>` | yes; generic delegation is conditional on `AsRef<str>` | yes; bodies and refinements passed, plus a concrete `&str` consumer | no | no; actual-source leaf harness only |
| `Port<T>::as_u16`, numeric conversion, and comparisons | yes | yes; five bodies, 7 VCs, plus four refinements/8 VCs | no | no; actual-source harness only |
| `Port<T>::from_str` | yes; exact numeric result, rejection, and preserved representation through the `AsRef`, `str::parse`, and `u16::from_str` models | yes; 15 VCs, plus concrete `&str` caller/relay 8 VCs | standard-library parser contract only | no; actual-source leaf harness only |
| `ErrorKind::{PartialEq,Eq}` | yes; equality agrees with the discriminated integer model | yes; body, refinement, and Eq assertion, 3 VCs | no | no; actual-source error-conversions harness only |
| `InvalidUri::fmt` | yes; preserves the preexisting formatter output as a prefix | yes; 6 VCs | std `Display for str` append contract | no; actual-source error-conversions harness only |
| `ErrorKind` / `InvalidUri` `Debug` | yes; preserves the preexisting formatter output as a prefix | yes; bodies and refinements, 10 VCs total | std formatter/DebugTuple append contracts | no; actual-source error-conversions harness only |
| ASCII HTTP/HTTPS prefix comparison used by `Scheme2::parse` | yes | yes (`ascii_lowercase`: 1 VC; `eq_ascii_prefix`: 3 VCs) | no | no; actual-source helper harness only |
| `Scheme2::parse_exact` and `parse` | yes; exact discriminated result contract | yes; 49 VCs, plus fixed helper bodies/4 VCs | no | no; actual-source leaf harness only |
| `Scheme::try_from(&[u8])` | yes; exact success class, preserved custom bytes, and error class | yes; body/refinement, 23 VCs | Bytes/ByteStr models, ASCII UTF-8 lemma | no; actual-source leaf harness only |
| `From<Scheme2> for Scheme` | yes; exact discriminant and custom-byte View preservation | yes; body/refinement, 2 VCs | ByteStr View | no; actual-source leaf harness only |
| `Scheme::as_str` | yes; requires a non-`None` scheme and returns the exact fixed or preserved spelling | yes; body, refinement, and helper obligations, 17 VCs | ByteStr UTF-8 invariant and ASCII UTF-8 lemma | no; actual-source leaf harness only |
| `Scheme::as_ref`, `Display`, and `Debug` | yes; bodies are conditional on a non-`None` scheme | bodies pass (3, 7, and 7 VCs); trait refinements remain unproved because the trait domain is unconditional | standard formatting/AsRef contracts | no; actual-source leaf harness only |
| `Scheme::try_from(&str)` | exact input byte spelling and error classes | yes; body/refinement, 4 VCs | str-to-byte view, ASCII UTF-8 lemma | no; actual-source leaf harness only |
| `Scheme::from_str` | exact input byte spelling and error classes | yes; body, 2 VCs; no separate refinement goal was emitted | str-to-byte view, ASCII UTF-8 lemma | no; actual-source leaf harness only |
| `Authority::validate_authority_bytes` scanner | yes; successful output is an ASCII URI-character prefix ending at the first `/`, `?`, `#`, or input end | yes; 4 VCs | no | no; actual-source scanner leaf only |
| `Authority::{parse,parse_non_empty}` / constructors and exact validation errors | scanner-prefix preservation only; full bracket, percent, colon, acceptance, and error-priority model pending | no | no | no |
| `Authority::{as_str,AsRef<str>}` | yes; exact stored byte sequence | yes; 2 VCs each | `ByteStr` UTF-8 representation model | no; actual-source scanner leaf only |
| `Authority::host` | exact host-span model pending; direct byte-index loop preserves last-`@`, first-`]`, and first-`:` behavior | body proof pending | ASCII delimiter-to-UTF-8 boundary helpers | no |
| path/query byte classifier and `scan_path_and_query` | yes; exact first-delimiter, offset, high-byte, and error-priority contract | yes; 5 VCs in focused production-source harness | no | no; `PathAndQuery` wrapper remains separate |
| `PathAndQuery` constructors and accessors | yes; `from_static` requires the exact accepted byte domain; `from_shared` returns the exact retained input prefix and first-query cache; accessors expose exact component bytes | yes; from_static 12 VCs, from_shared 39, path 13, query 11, as_str 7, slash_str 2, empty/slash/star 2 each | `Bytes` model, ByteStr UTF-8 model, scanner contract, ASCII UTF-8 lemma, `prefix_at_ascii_byte` / `prefix_through_ascii_byte` and string split contracts | no; actual-source leaf harness only |
| `PathAndQuery` `TryFrom` / `FromStr` relays | yes; successful results preserve the exact prefix and query model; errors are left unspecified (`Err(_) => true`) | yes; five `TryFrom` bodies, 18 VCs total, plus `FromStr` 2 VCs | same constructor and byte/string representation boundaries | no; actual-source leaf harness only |
| `Uri` parsing, constructors, and accessors | pending | no | no | no |
| `Builder` and component conversions | pending | no | no | no |
| URI equality, formatting, and hashing | pending | no | no | no |

No temporary trusted URI-owned boundary has been introduced. Full parser
verification remains dependent on exact models for the `Bytes` and `ByteStr`
representations and on the unverified `dyn Any` optimization in
`from_maybe_shared`. The `path_scan` helper is proved independently of those
dependencies; this does not prove the `PathAndQuery` constructors or accessors.

The first whole-module run of the source-inclusion harness passed the
`Port::as_u16` body and its one functional VC, but failed on unrelated
`Debug`/`Display` formatting adapters, generic `AsRef<str>`/`u16::from_str`
calls, and some `PartialEq<u16>` refinement goals. The generic representation
relay and parser calls now have narrow contracts, and the focused Port targets
below pass. Formatting and complete module integration remain separate.
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

The URI harness includes the production `error.rs`, `port.rs`, `scheme.rs`,
`bytes_model.rs`, `byte_str.rs`, and ASCII helper source. It no longer replaces
`ErrorKind` or `InvalidUri` with shims. The `From<Port<T>> for u16`
implementation has the explicit postcondition `result == port@`; its body and
refinement were replayed with that contract.

The `Port<T>::as_str`/`AsRef<str>` bodies relay the generic trait
precondition/postcondition. A concrete `Port<&str>` consumer proof shows the
returned text view equals the stored representation view. This does not
establish a numeric/text consistency invariant for arbitrary `Port<T>` values.

The latest focused Port proof passed `Port<T>::from_str` (15 VCs),
`borrowed_port_text` (4 VCs), and `parse_borrowed_port` (4 VCs). The parser
result is related to the exact `parse_u16_model` over the text returned by
`AsRef<str>`, and the successful `Port<T>` result retains the exact input
representation value. That model's accepted language matches Rust's `u16` decimal
parser, including a single leading `+`, arbitrary leading zeroes, and overflow
rejection. The `str::parse` and `u16::from_str` contracts are standard-library
specifications, so the HTTP proof is conditional on those boundaries.
The recursive `parse_u16_digits` logic helper has one generated VC,
`vc_parse_u16_digits`, which proves its recursive variant decreases; the pure
non-recursive `parse_u16_model` definition emits no separate body VC. This is
termination evidence for the model helper, not an additional functional proof
of Rust's standard-library parser.
The `InvalidUri::fmt` result says only that formatting preserves the bytes
already in the formatter; it does not specify the emitted message. Its proof
uses the standard-library `Display for str` append contract and the
`Formatter::pad` append-only boundary.
The `Debug` bodies use explicit append postconditions: `ErrorKind` writes the
same single variant-name string as the derive output, and `InvalidUri` uses the
same `DebugTuple` builder path as the derive output. Both body and refinement
targets pass. These contracts establish formatter-prefix preservation, not the
complete text formatting model.

Replay the Port targets from `verification/uri` with the Scheme equality leaf
cfg (ordinary HTTP builds retain those implementations):

```sh
RUSTFLAGS='--cfg http_uri_scheme_leaf' ../../scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove \
  verif/http_uri_proof_rlib/uri/port/impl_Port_T_0/from_str.coma \
  verif/http_uri_proof_rlib/borrowed_port_text.coma \
  verif/http_uri_proof_rlib/parse_borrowed_port.coma \
  -- --locked --offline
```

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

Both production helper bodies passed (four VCs total). The focused scheme run
also proved `Scheme2::parse_exact` (39 VCs) and `Scheme2::parse` (10 VCs);
`TryFrom<&[u8]> for Scheme` still depends on the ASCII UTF-8 helper and its
exact constructor relation. `Scheme::as_str` has remaining `None`-case and
output-relation obligations; `Scheme::empty()` constructs `None`, so no global
non-`None` invariant is assumed.

`Scheme::as_str` is conditional on a non-`None` model: `Scheme::empty()` is a
real internal value, and `as_str`'s `None` arm panics. The `AsRef<str>`,
`Display`, and `Debug` bodies relay that precondition and their bodies pass, but
their unconditional trait refinements do not: the trait declarations do not
carry the non-`None` domain condition. The proof keeps that limitation rather
than excluding the reachable empty representation. Standard scheme text is
returned through the exact static ASCII byte sequences with the shared
ASCII-to-UTF-8 proof; custom text returns the same `ByteStr` dereference used
by the wrapper's UTF-8 invariant.

The scheme model preserves the distinction between standard and custom
variants. A runtime regression records upstream behavior:
`Scheme::HTTP != Scheme::try_from("HTTP").unwrap()`, while each compares equal
to the text `"http"`. The custom spelling is preserved and the standard/custom
`PartialEq<Scheme>` branch returns false; a case-folded text-only model would
misrepresent this behavior.

The authority scanner contract proves a valid ASCII prefix and its first
authority terminator; it does not yet prove the full authority grammar or the
scanner's error variants and priority. The host getter now uses byte-index
loops because `str::split` iterator specifications are unavailable. Its
runtime regression covers repeated userinfo delimiters and bracketed IPv6;
the exact source-level host-span proof remains pending.

The focused PathAndQuery run used freshly emitted Coma after cleaning only the
`http-uri-proof` package. `from_shared` has an exact success-prefix/query
postcondition; its error postcondition remains deliberately broad, so complete
error classification and acceptance are still unproved. The five conversion
body proofs relay that same success model and do not claim exact rejection
behavior. The `from_static` precondition is the public numeric byte-domain
predicate in `path_static_domain.rs`; the source leaf harness includes that
same production definition. These are component proofs only and do not establish
an integrated URI or HTTP run.

## Exact path scanner proof

`src/uri/path_scan.rs` is the production scanner module called by both
`PathAndQuery::from_shared` and `PathAndQuery::from_static`. Its contract
records the exact empty/too-long/single-`*`/initial-character error priority,
the first query and fragment byte offsets, the path/query byte classes, and the
high-byte flag only through the fragment delimiter. Errors after `#` are
ignored, as in the runtime parser. The scanner uses the exact production
`ErrorKind` and the shared `uri/limits.rs::MAX_LEN` value.

The focused harness at `verification/path-scanner` includes those production
files and the nested production byte classifiers directly; it uses no
replacement scanner or error model. The latest elevated proof passed five VCs:
the scanner contract, both classifier bodies, and the slice length/empty
wrappers used in the scanner proof. The `PathAndQuery` wrapper, UTF-8 conversion,
`Bytes` truncation, and public accessors are not included in that target.

The `u16::from_str` parser contract in `creusot-std/src/std/string.rs` is an
explicit standard-library boundary, not a proof of the standard parser body.
Its model allows one leading `+`, then requires one or more ASCII decimal
digits, accepts arbitrary leading zeroes while the value stays at most 65535,
and rejects `-`, whitespace, non-ASCII digits, and overflow. This grammar was
checked against the pinned Rust core source revision
`6a979b3e32522049d0acb4a47f7ae44b7c8abfd5`. Generic `str::parse` and generic
`AsRef<str>` specs relay their callee preconditions/postconditions; they do
not claim unconditional behavior for arbitrary trait implementations.
