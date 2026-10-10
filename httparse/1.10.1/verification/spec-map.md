# httparse 1.10.1 specification map

This map is based on the published source at `/tmp/httparse-1.10.1` and the
imported crate tree. It is a review target for later contracts and proofs, not
a claim that the runtime has been verified.

## Models and representation

- The input model is an immutable byte sequence plus absolute offsets
  `mark <= cursor <= end <= input.len()`. The runtime `Bytes::pos()` is
  `cursor - mark`, and `Bytes::len()` is `end - cursor`. `slice` and
  `slice_skip` return spans in the original input and reset `mark` to the new
  cursor.
- `verification_model::ByteClass` defines the scalar accepted-byte sets
  directly. `maximal_prefix_end(input, start, end, class)` returns the first
  byte outside the class, or `end`; it is independent from lookup tables and
  SIMD/SWAR scanners. `parse_token_model` is the first deterministic parser
  caller: it models exact complete/partial/error outcomes, consumed cursor,
  and returned token span for both delimiters and invalid bytes.
- Parser outcomes must retain the exact status/error, absolute consumed cursor,
  optional parsed scalar values, and observable output-state updates. Public
  Request/Response values permit arbitrary valid Rust fields. A parsed-value
  restriction such as HTTP version `<= 1` or token-shaped header names is a
  parse postcondition, never a type invariant.
- The seven `ParserConfig` booleans are independent inputs. Default sets all
  false; each builder setter changes only its selected field. Proofs must hold
  for all 128 configurations.

## Public surface inventory

| Surface | Required contract/model facts |
| --- | --- |
| `Error`, `InvalidChunkSize`, `Status<T>`, `Result<T>` | Exact variants and equality; exact `Display` text; `Status` queries mirror variants; `unwrap` returns the contained value for `Complete` and intentionally panics for `Partial`. `Error`'s `std::error::Error` implementation is std-only. |
| `ParserConfig` | All seven flags above; getters observe their exact flags; request/response parsing forwards every relevant flag without changing config. |
| `Request`, `Response`, `Header`, `EMPTY_HEADER` | Public fields admit arbitrary valid field values. Constructors set only the documented fields to `None` and retain the supplied header slice. `EMPTY_HEADER` is exactly empty name and empty value. `Header` formatting follows `Debug` behavior for valid UTF-8 and raw byte slices otherwise. |
| `ParserConfig::{parse_request,parse_request_with_uninit_headers,parse_response,parse_response_with_uninit_headers}` | Delegate to corresponding parse operation with the same config and input; exact returned status/offset and all field/slice effects. |
| `Request::{new,parse,parse_with_uninit_headers}` and `Response::{new,parse}` | Exact stage order, partial/error precedence, mutations after each completed stage, successful body offset, and header-slice behavior described below. Private config variants are implementation proof boundaries. |
| `parse_headers` | Complete result contains the consumed offset and initialized prefix; `Partial`/error preserve the exact initialized prefix and mutable destination length effects. Header capacity zero/exact/short cases and syntax-vs-capacity precedence are explicit. |
| `_benchable::{Bytes,parse_method,parse_uri,parse_version}` | These are public reachable APIs despite doc-hidden status. `Bytes` method contracts include safe observations, exact spans, and preconditions/provenance for each unsafe operation. Parser helper results include cursor effects on every status/error. |
| `parse_chunk_size` | Exact accepted byte language and `(consumed,size)`/partial/error behavior, including the permissive cases in the final section. |
| `Clone`, `Debug`, `PartialEq`, `Eq`, `Display`, `std::error::Error` impls | Preserve their standard derived/explicit semantics for arbitrary constructible values; std-only impls are feature-gated. |

Internal bodies still requiring explicit contracts include byte predicates,
newline/space helpers, token/version/code/reason parsing, header stages,
scanner backends, pointer/slice conversion helpers, and their unsafe call sites.
Build scripts, macros, and SIMD dispatch are also part of runtime refinement.

## Exact state-transition observations

- Request parsing skips any number of leading LF or CRLF lines. It stores
  `method`, then `path`, then `version` as each stage completes. A later
  `Partial`/error does not roll back those fields. The normal initialized-header
  wrapper restores the original `self.headers` slice length on non-complete
  return, although parsed slots can already have been overwritten. The
  uninitialized-header entry point assigns `self.headers` only after all
  headers complete.
- Response parsing stores `version`, then `code`, then `reason`. A failed
  status line or later header parse retains fields from completed stages. Its
  initialized-header wrapper has the same restore-length / retain-overwritten-
  slots behavior. Its uninitialized entry point assigns the initialized header
  prefix only after the header parser completes.
- The common header state machine writes each header only after its whole line
  has parsed, and `ShrinkOnDrop` reduces the visible destination to exactly the
  initialized prefix on every exit. It parses a candidate line before checking
  whether a destination slot exists, so malformed or incomplete input wins
  over `TooManyHeaders`; a valid extra header yields `TooManyHeaders`.
- Ignoring invalid headers resumes after the current line, but NUL and a lone
  CR still fail. Header-name whitespace, initial header whitespace, obsolete
  folding, and ignored-invalid-lines are controlled by distinct config flags.
- The reason phrase is optional. HTAB/SP/VCHAR bytes produce their exact
  UTF-8 string; accepted obs-text makes the returned reason the empty string.
  Bare LF is accepted. CR must be followed by LF.
- URI bytes allow `0x21..=0x7e` and `0x80..=0xff`, then the complete URI span
  must pass UTF-8 validation. Header values remain bytes and do not require
  UTF-8. Methods and names use the same `tchar` set and must be nonempty.
- `parse_version` has an observable fast path: with at least 8 bytes it
  advances by 8 even for a bad version; with fewer bytes it checks the `HTTP/1.`
  prefix incrementally and returns `Partial` after consuming a valid short
  prefix. Preserve both exact behaviors for `_benchable::Bytes` callers.

## Byte and cursor leaf obligations

- Token byte: ASCII alpha/digit or one of `! # $ % & ' * + - . ^ _ ` | ~`.
- URI byte: every byte at least `!` except DEL (`0x7f`), including obs-text.
- Header-value byte: HTAB, every byte from SP upward except DEL, including
  obs-text.
- Scanner contract: source, mark, and end stay fixed; the new cursor is safe;
  all bytes skipped are accepted; and the new cursor is either `end` or the
  first unaccepted byte. A vector block may promise only a conservative
  accepted prefix. The header-value SWAR path deliberately stops at TAB, which
  the scalar tail can then consume. Do not assume borrow-based lane masks are
  exact without proving the relevant property.
- Cursor initialization, slice creation, pointer advance, and `set_cursor`
  require same-allocation provenance in addition to numeric bounds. For a
  returned zero-copy span, prove both exact byte identity and lifetime.

## Parser-family edge cases

- CRLF, bare LF, leading empty lines, truncated CR, and CR followed by a
  non-LF byte have distinct consumed positions and error/partial outcomes.
- Request method and URI are nonempty. Methods use `tchar`; a URI can contain
  high bytes but must be valid UTF-8 once its delimiter is reached.
- Response status consists of exactly three decimal bytes, including values
  outside conventional HTTP status codes. The reason is optional and may
  become empty on obs-text.
- Header capacity zero, exact capacity, and one extra valid header are
  observable. `TooManyHeaders` follows parsing the candidate line.
- Values trim trailing SP, HTAB, CR and LF after the line. Empty values,
  continuation/folding, first-header whitespace, invalid-line skipping, NUL,
  and lone-CR behavior depend on the relevant flags.
- Repeated parsing starts from arbitrary existing field values. Fields whose
  stage has not completed can retain old values; completed stages overwrite
  them. Initialized and uninitialized APIs expose different header-slice state
  after failure and require separate contracts.

## Chunk-size runtime quirks

- Zero through sixteen hex digits are accepted before CRLF; zero digits yield
  size zero. A seventeenth digit fails.
- Spaces/tabs after a digit end the size digits. A semicolon starts an
  extension once, and arbitrary extension octets are ignored. Whitespace
  before the semicolon is accepted, but no more size digits are accepted.
- CR must be immediately followed by LF even while reading an extension.
  Successful `pos` includes both line-ending bytes. Do not strengthen this
  behavior to the stricter RFC extension grammar.

## Initial proof order and status

1. Prove the direct byte-class formulas and the `maximal_prefix_end` bounds,
   accepted-prefix, and first-unaccepted properties.
2. Prove `accepted_prefix_span`, then `parse_token_model` as a representative
   exact caller, without reopening the recursive definition in a large VC.
3. Prove cursor/byte-operation contracts and bridge the token model to
   `parse_token` and its actual byte predicate.
4. Add stage outcome/state models for requests, responses, headers, and chunk
   parsing, then prove thin orchestration against them.
5. Establish scalar implementation correspondence before vector/SIMD
   refinement.

## Model-only proof checkpoint

The independent scalar model translates and proves in the target-local harness
`verification/probes/model-harness`. It includes `src/verification/model.rs`
directly and avoids resolving httparse's upstream dev-dependencies. The normal
Creusot translator passed with:

```sh
CARGO_NET_OFFLINE=true ./verify.sh translate
```

The isolated proof passed with:

```sh
CARGO_NET_OFFLINE=true ./verify.sh prove
```

`verify.sh prove` calls the crate-local `run-proof.bash`, which serialized this
run with the shared queue and checked the configured one-prover, 1000 MiB
profile. Creusot 0.11.0-dev translated the harness; Why3 1.8.2 and Z3 4.15.3
were configured for proof. The output contained 7 proof files and 7 successful
obligations: `maximal_prefix_end` (1), `accepted_prefix_span` (1),
`parse_token_model` (1), and `Clone` for ByteClass, Span, TokenOutcome, and
TokenResult (1 each). There were no failed obligations in this run.

| Component | Contract reviewed | Model body proved | Runtime bridge | Integrated run | Trusted/excluded |
| --- | ---: | ---: | ---: | ---: | --- |
| `maximal_prefix_end` | yes | yes, 1 VC | no | no | model-only |
| `accepted_prefix_span` | yes | yes, 1 VC | no | no | model-only |
| `parse_token_model` | yes | yes, 1 VC | no | no | model-only |
| `accepts` and outcome selectors | yes | open logical definitions; no body VC | no | no | model-only |

The generated `PartialEq` implementations previously failed their refinement
and equality obligations. They were removed because parser semantics need no
structural equality on these model values; outcome selection now uses explicit
match definitions and token spans are inspected by a match helper. After this
interface change and `cargo creusot clean --force`, all model obligations
passed. This is an isolated model checkpoint only: no runtime byte-table,
cursor, scanner, or parser body has been bridged or integrated.

Current status: specification map reviewed; the three named logical parser
functions are independently body-proved. Runtime implementations and all
public parser surfaces remain unproved. No trusted model contracts are present.
