# httparse 1.10.1 API and callable inventory

This inventory is based on the imported published tree. It includes the
doc-hidden `_benchable` API, private executable helpers, trait bodies, and
target-gated SIMD paths. A public value can be constructed directly whenever
its fields are public; its invariant therefore cannot assume that it came from
a successful parse. Verification status is recorded separately in
[`VERIFICATION_STATUS.md`](VERIFICATION_STATUS.md).

## Public and doc-hidden API

| Item | Exact behavior to specify |
|---|---|
| `Error` variants: `HeaderName`, `HeaderValue`, `NewLine`, `Status`, `Token`, `TooManyHeaders`, `Version` | Exact variant for invalid input, including precedence when a line is malformed and header capacity is exhausted. `Display` and `std::error::Error::description` return the exact fixed strings in `description_str`. |
| `InvalidChunkSize` | Separate chunk-size error type; `Display` writes `invalid chunk size`. |
| `Result<T>` | Alias for `core::result::Result<Status<T>, Error>`. |
| `Status<T>::Complete(T)`, `Status<T>::Partial`; `is_complete`, `is_partial`, `unwrap` | Exact discriminant predicates. `unwrap` returns the value for `Complete` and panics with `Tried to unwrap Status::Partial` for `Partial`. |
| `ParserConfig::default` and seven builder setters | All seven flags default false. Each setter changes only its own flag and returns the same mutable config: response whitespace after header name; multiple request-line delimiter spaces; multiple response status delimiter spaces; obsolete response header folding; whitespace before first header name; ignore invalid response header lines; ignore invalid request header lines. Only the multiple-space, obsolete-folding, and leading-header-space flags have public getters; all flag effects on parsing must still be specified. |
| `ParserConfig::parse_request`, `parse_request_with_uninit_headers` | Delegate to request parsing with this config. Return `Complete(body_offset)` only when the complete request and header terminator are present; return `Partial` or the exact parse error otherwise, while preserving mutations made by stages already parsed. Initialized versus uninitialized output slice effects differ. |
| `ParserConfig::parse_response`, `parse_response_with_uninit_headers` | Corresponding response behavior, including reason-phrase, seven config effects, result offset, and partial/error mutation behavior. |
| `Request<'headers, 'buf>` public fields `method`, `path`, `version`, `headers`; `new`, `parse`, `parse_with_uninit_headers` | `new` initializes scalar options to `None` and stores the supplied header slice. Fields are publicly mutable and can contain arbitrary valid values. Parsing accepts leading empty lines, stores fields as each start-line stage completes, and on header failure/partial preserves prior stage updates and already-written header slots; initialized wrapper restoration semantics differ from uninitialized-header call. `parse` returns the body offset on complete parse. |
| `Response<'headers, 'buf>` public fields `version`, `code`, `reason`, `headers`; `new`, `parse` | `new` initializes options to `None`. Fields are publicly mutable; do not impose parsed-only invariants. Parsing stores completed status fields before header parsing, permits empty/missing reason, maps accepted obs-text reason to `""`, and preserves earlier mutations on later failure or partial input. |
| `Header<'a>` public `name: &'a str`, `value: &'a [u8]`; `Debug` | Publicly constructible with any valid Rust name and byte value. `Debug` formats valid UTF-8 values as strings and invalid UTF-8 as bytes. Parsed header names are ASCII token strings; parsed value bytes may include obs-text. |
| `EMPTY_HEADER` | Exact constant: `Header { name: "", value: b"" }`. |
| `parse_headers` | Parses a header block with default `HeaderParserConfig`; returns exact consumed offset and parsed prefix of the caller-provided storage, or `Partial`/error with exact initialized and overwritten prefix. End-of-headers accepts CRLF and bare LF. Capacity/error precedence is observable. |
| `parse_chunk_size` | Parses the crate's actual permissive grammar, not a stronger RFC grammar. At most 16 hex digits, including leading zeroes; spaces/tabs end digit collection; semicolon starts ignored extension bytes; CR must be followed by LF; the code accepts CRLF even before any digit. Returns consumed offset and `u64` size, `Partial`, or `InvalidChunkSize` exactly. |
| `_benchable::{parse_method, parse_uri, parse_version, Bytes}` | Doc-hidden but externally reachable. All cursor transitions, result values, errors and panic/unsafe preconditions are in scope. In particular the 8-byte `parse_version` fast path consumes eight bytes even for an invalid version; the short-input path can return `Partial` after consuming a valid prefix. |

## All crate-owned executable callables

The following lists include the internal helpers called by public APIs and
specialized scanners that are only compiled for selected target cfgs.

| Source / component | Callable bodies |
|---|---|
| `src/lib.rs`: byte classification | `is_method_token`, `is_uri_token`, `is_header_name_token`, `is_header_value_token`; static maps `URI_MAP`, `TOKEN_MAP`, `HEADER_VALUE_MAP`; `byte_map!` expansion. Specify each accepted byte set exactly. |
| `src/lib.rs`: errors and status | `Error::description_str`, `Error::fmt`, `Error::description` (`std` only), `InvalidChunkSize::fmt`, `Status::is_complete`, `Status::is_partial`, `Status::unwrap`. |
| `src/lib.rs`: configuration | `ParserConfig`'s seven builder setters, its three getters, `parse_request`, `parse_request_with_uninit_headers`, `parse_response`, `parse_response_with_uninit_headers`, manual `Default` and derived `Clone`. |
| `src/lib.rs`: requests | `Request::new`, `Request::parse`, `Request::parse_with_uninit_headers`, private `parse_with_config`, private `parse_with_config_and_uninit_headers`, `skip_empty_lines`, `skip_spaces`. |
| `src/lib.rs`: responses / values | `Response::new`, `Response::parse`, private `parse_with_config`, private `parse_with_config_and_uninit_headers`, `Header::fmt`. |
| `src/lib.rs`: start-line and scalar parsing | `parse_version`, `parse_method`, `parse_reason`, `parse_token`, `parse_uri`, `parse_code`. Preserve exact `Complete`/`Partial`/error variants, consumed offsets and borrowed byte spans. |
| `src/lib.rs`: headers and unsafe storage | `parse_headers`, `parse_headers_iter`, `deinit_slice_mut`, `assume_init_slice`, `parse_headers_iter_uninit`, `ShrinkOnDrop::drop`; private `HeaderParserConfig` and its default construction. This includes exact slot writes, shrinking on every exit, config behavior, trim semantics, skipped-line behavior, and errors. |
| `src/lib.rs`: chunk parser | `parse_chunk_size`, including termination, hex recurrence/overflow bounds and permissive extension handling. |
| `src/iter.rs`: `Bytes` API | `Bytes::new`, `pos`, `peek`, `peek_ahead`, generic `peek_n`, `bump`, `advance`, `len`, `is_empty`, `slice`, `slice_skip`, `commit`, `advance_and_commit`, `as_ptr`, `start`, `end`, `set_cursor`; `AsRef<[u8]>::as_ref`; `Iterator::next`; private `slice_from_ptr_range`. State the pointer provenance/allocation and lifetime relation in addition to numeric bounds. |
| `src/simd/swar.rs` | `match_uri_vectored`, `match_header_value_vectored`, `match_header_name_vectored`, `match_tail`, `match_block`, `match_uri_char_8_swar`, `match_header_value_char_8_swar`, `offsetnz`. Prove safe advancement, conservative block prefix semantics, scalar-tail correction, and exact byte sets. |
| `src/simd/sse42.rs` (`x86`/`x86_64`) | `match_uri_vectored`, `match_url_char_16_sse`, `match_header_value_vectored`, `match_header_value_char_16_sse`, `byte_is_allowed`; test-only table comparison bodies. Include target-feature and unaligned-load safety. |
| `src/simd/avx2.rs` (`x86`/`x86_64`) | `match_uri_vectored`, `match_url_char_32_avx`, `match_header_value_vectored`, `match_header_value_char_32_avx`, `byte_is_allowed`; test-only table comparison bodies. Include target-feature and unaligned-load safety. |
| `src/simd/neon.rs` (`aarch64`) | `match_header_name_vectored`, `match_header_value_vectored`, `match_uri_vectored`, `match_header_name_char_16_neon`, `match_url_char_16_neon`, `match_header_value_char_16_neon`, `offsetz`, `offsetnz`, local `clz`, `byte_is_allowed`; test-only table comparison bodies. Include load alignment/bounds and target-feature assumptions. |
| `src/simd/runtime.rs` (`x86`/`x86_64`) | `detect_runtime_feature`, `get_runtime_feature`, and three scanner dispatchers `match_header_name_vectored`, `match_uri_vectored`, `match_header_value_vectored`. Specify atomic/cache states and feature-detection justification. |
| `src/simd/mod.rs` | cfg-selected reexports; inline compile-time SSE4.2 and AVX2 wrappers for header name, URI, and header value scanners. Prove each selected branch's ISA precondition and scanner contract. |
| `src/macros.rs` | `next!`, `expect!`, `complete!`, `space!`, `newline!` macro expansions. Treat their early returns and cursor movement as part of each expanded caller's behavior. |
| `src/lib.rs`: proof specification | `parser_config_state` (logic-only; maps all seven config flags in a fixed tuple). Its intended contract is exact observation of the seven stored booleans. |
| `src/verification/model.rs`: proof-only logical functions | `accepts`, `maximal_prefix_end`, `accepted_prefix_span`, `TokenOutcome::deep_model`, `parse_token_model`, `Span::deep_model`, `valid_span`; model types `ByteClass`, `TokenOutcome`, `TokenResult`, `Span`. These definitions are independent of the runtime tables and remain model-only until tied to concrete executable parser bodies. |
| `src/verification/chunk.rs`: proof-only logical functions | `hex_capacity`, `hex_value`, `scan`, `parse_chunk_size_model`; model types `ChunkOutcome`, `ChunkState`. This describes the permissive chunk state machine but is not a runtime refinement proof until connected to `parse_chunk_size`. |
| Compiler-generated trait implementations | `Error`: `Copy`, `Clone`, `PartialEq`, `Eq`, `Debug`; `InvalidChunkSize`: `Debug`, `PartialEq`, `Eq`; `Status<T>`: `Copy`, `Clone`, `Eq`, `PartialEq`, `Debug`; `ParserConfig`: `Clone`, `Debug`, `Default`; `Request` and `Response`: `Debug`, `Eq`, `PartialEq`; `Header`: `Copy`, `Clone`, `Eq`, `PartialEq`; `build.rs::Version`: `Debug`, `Clone`, `Copy`, `PartialEq`, `PartialOrd`. Their structural behavior is generated by Rust, but they are public callable trait implementations where their types are public. |
| `build.rs` | `main`, `enable_new_features`, `enable_simd`, `Version::parse`, `var_is`; this controls the compiled backend through compiler version, target architecture/features, and `CARGO_CFG_HTTPARSE_DISABLE_SIMD` / `CARGO_CFG_HTTPARSE_DISABLE_SIMD_COMPILETIME`. Exact cfg output and target matrix are part of backend coverage. |
| `benches/parse.rs` | Criterion benchmark functions `req`, `req_short`, `resp`, `resp_short`, `uri`, nested `_uri`, `header`, nested `_header`, `version`, nested `_version`, `method`, nested `_method`, `many_requests`; these are benchmark harness code, not parser implementation. |
| cfg(test) only | Unit/property tests and SIMD truth-table tests are validation code, not production parser bodies. Their no-default compilation and runtime execution remain part of the supported validation matrix. |

## Crate-owned unsafe operation inventory

Every item below needs a local precondition, same-allocation provenance where
raw pointers are involved, and a postcondition about state and exposed bytes.

| Location | Unsafe operation / required proof |
|---|---|
| `src/lib.rs`, `Request::parse_with_config` and `Response::parse_with_config` | Raw-slice cast from `&mut [Header]` to `&mut [MaybeUninit<Header>]` and the reverse cast restoring the original slice on `Partial` or `Err`. Prove same slice allocation, aliasing, initialized-prefix exposure, and mutation behavior. |
| `src/lib.rs`, request/response uninitialized adapters | Call `assume_init_slice` only after `parse_headers_iter_uninit` has established that every exposed element is initialized. |
| `src/lib.rs`, `parse_version` | `Bytes::advance(8)` after `peek_n::<[u8; 8]>(8)` succeeds. |
| `src/lib.rs`, `parse_method` | `Bytes::advance(4/5)`, `Bytes::slice_skip(1)`, and `str::from_utf8_unchecked` on the matched ASCII `GET `/`POST ` fast paths. |
| `src/lib.rs`, `parse_reason` | `slice_skip(1/2)` and `from_utf8_unchecked` after validation of HTAB/SP/VCHAR; obs-text path must expose only the empty string. |
| `src/lib.rs`, `parse_token` | `slice_skip(1)` and `from_utf8_unchecked` after every byte was accepted by the method-token predicate. |
| `src/lib.rs`, `parse_uri` | `slice_skip(1)` after delimiter observation; UTF-8 conversion is checked and maps invalid UTF-8 to `Error::Token`. |
| `src/lib.rs`, `parse_headers_iter` / storage helpers | `deinit_slice_mut` casts mutable references to slices of `T` into slices of `MaybeUninit<T>`; `assume_init_slice` casts back; `ShrinkOnDrop::drop` uses `get_unchecked_mut(..num_headers)`. Prove exact initialized prefix and `num_headers <= original_capacity` on every exit. |
| `src/lib.rs`, header parser | `slice_skip(skip)` after checking CRLF or LF. Header name uses `from_utf8_unchecked` only after scanner/token validation. |
| `src/iter.rs` | `Bytes::new` uses pointer `.add(slice.len())`; `peek`, `peek_ahead`, `advance`, `bump`, `slice`, `slice_skip`, `as_ref`, `set_cursor`, iterator `next`, and `slice_from_ptr_range` dereference or create slices from raw pointers. The invariant `start <= cursor <= end` is necessary but not sufficient without the shared input allocation and live borrow. |
| `src/simd/sse42.rs` | SSE4.2 target-feature functions, 16-byte unaligned loads, vector comparisons and mask extraction. Callers require runtime feature detection or compile-time target-feature evidence. |
| `src/simd/avx2.rs` | AVX2 target-feature functions, 32-byte unaligned loads, vector comparisons and mask extraction. Callers require runtime feature detection or compile-time target-feature evidence. |
| `src/simd/neon.rs` | NEON intrinsic loads/comparisons/reductions and target-feature scope on `aarch64`; prove each 16-byte load has at least 16 remaining bytes and matching architecture support. |
| `src/simd/runtime.rs` | Atomic feature cache loads/stores and runtime feature detection; prove cache encoding cannot select an unsupported backend under any interleaving. |

There is no claim that the unsafe operations above are discharged by passing
tests or by a `cfg(creusot)` replacement. Any proof-only replacement needs a
mechanical refinement bridge to the exact executable implementation.
