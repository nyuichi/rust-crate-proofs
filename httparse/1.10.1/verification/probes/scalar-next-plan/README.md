# Scalar parser boundary checkpoint: `skip_spaces`

**Status: the extracted helper and selected dependency closure are independently
proved. The helper is not yet part of a crate-integrated proof.** This planning
note is not a verification ledger entry and does not close the crate gate.

## Selection

The lowest-dependency next scalar helper is `skip_spaces` in `src/lib.rs`.
Its loop reads only `Bytes::peek`, `Bytes::bump`, and `Bytes::slice`; it does
not call the byte-class tables, SIMD scanners, UTF-8 conversion, parser config,
or another parser. The `bump` unsafe call is guarded by the immediately
preceding `peek() == Some(b' ')` branch, which establishes `cursor < end`.
There is no unsafe string or pointer conversion in the helper.

`skip_empty_lines` is a slightly larger follow-up: it distinguishes CRLF from
bare LF and returns `Error::NewLine` after consuming a CR and a bad following
byte. `parse_reason`, `parse_method`, `parse_uri`, and the header state machine
also introduce byte-class, UTF-8, or scanner dependencies described below.

## Exact behavior to model

Let the entry `Bytes` view be `(input, mark, cursor, end)`, with its normal
valid-cursor conditions. Define `p` as the first index at or after `cursor`
whose byte is not ASCII space (`0x20`), or `end` if the remaining suffix
contains only spaces. The runtime result and exit view are:

| Condition | Result | Exit cursor | Exit mark |
|---|---|---:|---:|
| `p < end` | `Ok(Status::Complete(()))` | `p` | `p` |
| `p == end` | `Ok(Status::Partial)` | `end` | entry `mark` |

Both paths preserve `input` and `end`; no error is possible. On completion,
the byte at `p` is observed but not consumed. In particular, a tab is a
non-space terminator, so `b" \t"` completes at the tab. Empty input and an
input made entirely of spaces are partial. When all-space input is partial,
the cursor has advanced to EOF but the committed mark has not moved: there is
no `slice()` on that path. The completed path calls `slice()` even if zero
spaces were skipped, so it commits the mark to the current cursor.

The independent model should express `p` as the maximal initial `0x20` prefix
endpoint, with a termination variant `end - cursor`. Its postconditions should
state the result category above, `cursor <= p <= end`, that every byte in
`input[cursor..p]` is `0x20`, and that `p == end` or `input[p] != 0x20`.
Include the mark equation explicitly; using only the returned `Status` would
miss the observable difference between complete and partial cursor commits.
The entry mark need only satisfy the existing cursor invariant
(`mark <= cursor`); do not strengthen the helper precondition to `mark ==
cursor`.

Useful loop facts are a fixed `input`, `mark`, and `end`; `cursor` stays within
`[entry.cursor, end]`; and every byte between the entry cursor and current
cursor is `0x20`. The loop variant is `end - cursor`. At a non-space byte,
`slice()` establishes the completed mark equation. At EOF, the implementation
returns before slicing and must preserve the entry mark.

## Isolated harness boundary

`skip_spaces` remains inline in the production `src/lib.rs` for this
checkpoint. Its runtime body was extracted byte-for-byte to `src/skip_spaces.rs`
for an isolated harness; the harness path-includes that file with the actual
`src/iter.rs`, `src/macros.rs`, `Status`, and `Error` definitions, plus the
independent model in `src/verification/spaces.rs`. No production `src/lib.rs`
edit was made for this helper checkpoint. The exact-source comparison checked
the runtime `peek`/match/`bump`/`slice` body.

The existing `code-harness` is the closest scaffold for the actual `Bytes`
path and absolute cursor model. Reuse its shared-source `Bytes` closure and
string-model compiler environment: the active compiler has an existing ICE
on string-literal contracts in `src/error.rs`, even though this helper itself
never returns an error. Keep the model outcome separate from the runtime
`Status<()>` and prove that the actual helper body refines it. A selected run
should include the necessary `Bytes::peek`, `Bytes::bump`, and
`Bytes::slice` dependencies, while recording shared `Bytes` targets only once
rather than adding their prior proof totals again.

There is no feature-dependent branch in `skip_spaces` or `Bytes`; the helper
uses neither std nor a generated backend configuration. The harness uses the
default feature and isolated string-model compiler. The selected closure has
69 `Valid` split-subgoal results across 11 unique named conditions; 26 split
results belong to the extracted helper body. The full run bundle and raw logs
are preserved under
`../spaces-harness/evidence/direct-why3-20261005T052959319229043Z/`. This does
not establish a native test result or crate-integrated parser proof.

## Remaining scalar/parser inventory

| Routine | Runtime behavior and proof dependencies | Reason to defer behind `skip_spaces` |
|---|---|---|
| `parse_method` / `parse_token` | Fast paths recognize exactly `GET ` and `POST `; otherwise scan nonempty `tchar` bytes until a literal space. Generic fallback consumes the first invalid byte before `Error::Token`, and consumes the delimiter before completing. EOF after a valid prefix is partial. Fast paths use fixed-array peeks, unsafe advances, and `slice_skip(1)`. | Needs token-class/table refinement, two fast-path equivalence cases, and exact span/mark handling. |
| `parse_uri` | SIMD scans the runtime URI byte class (`0x21..=0x7e` except DEL, plus `0x80..=0xff`), then consumes one byte. It completes only when that byte is SP and the URI is nonempty. EOF gives partial; another terminator gives `Error::Token` after consuming it. SP is consumed before empty-span or UTF-8 errors. It uses checked UTF-8 conversion; invalid UTF-8 yields `Error::Token`. | Requires scanner prefix refinement, URI byte predicate, nonempty span, and UTF-8 validity/rejection at the exact boundary. |
| `parse_version` | With at least eight bytes, compares the first eight bytes against `HTTP/1.0` and `HTTP/1.1`, advances all eight even on mismatch, then returns version or `Error::Version`. With fewer than eight, consumes each matching prefix byte, errors after consuming a mismatching byte, and returns partial only for a matching prefix ending at EOF. | A separate worker is already preparing its model and owns this source area. |
| `parse_reason` | Consumes bytes until LF or CRLF. Accepts HTAB, SP, visible ASCII, and obs-text. Invalid control bytes are consumed before `Error::Status`; CR followed by a non-LF consumes that byte before the error; EOF, including after a CR, is partial. If any obs-text occurs, success returns `""`; otherwise it returns the ASCII slice excluding the terminator via unsafe `from_utf8_unchecked`. | Requires an exact byte-scan model and a proof that every byte returned through unchecked UTF-8 is ASCII; obs-text must take the empty-string branch. |
| `skip_empty_lines` | Repeats over bare LF and CRLF. A CR with missing LF is partial if input ends, or `Error::NewLine` after consuming a non-LF byte. At the first other byte it commits with `slice()`; at EOF it returns partial without committing. | The CRLF branch adds an error/result distinction and `next` consumption precedence absent from `skip_spaces`. |
| `space!` / `newline!` macros | `space!` consumes exactly one SP and commits; EOF is partial and a wrong byte is consumed before the supplied error. `newline!` consumes LF or CRLF and commits; a non-newline byte is consumed before `Error::NewLine`, and CR plus a wrong next byte consumes both. | Small, but their inline macro result/error behavior is best specified alongside their parser callers; they are not standalone functions. |
| Header name/value parsing | Header names use the token class and optional spaces-before-colon config; checked/unchecked name construction follows validated ASCII. Values skip leading SP/HTAB, accept the header-value byte class, handle LF/CRLF, trim trailing SP/HTAB/CR/LF, and may unfold lines under config. Invalid-line ignoring and capacity alter consumption and initialization behavior. | This is a configurable state machine with scanner, UTF-8, error recovery, folding, trim, and uninitialized-memory obligations. |

All behavior in this inventory was read from the httparse 1.10.1 runtime
source; RFC text alone is not a substitute for these consumption and fallback
rules.
