# `parse_code` shared-source harness

This target-local harness includes the actual `Bytes` implementation and its
`IteratorSpec`, the actual `Status` and `Error` source files, parser macros, the
independent code model, and `src/parse_code.rs` at crate root. The function in
`src/parse_code.rs` is the exact `parse_code` body with its refinement
contract, and `lib.rs` now includes that same file at the original root
location. No duplicate parser or substitute `Error` enum is used.

The exact model records absolute input indices and preserves the incoming
mark. The implementation requests three bytes using `Iterator::next`, accepts
only ASCII `0` through `9`, and returns the ordinary decimal value from 000
through 999. It does not impose the HTTP status-code range 100 through 599. If
EOF arrives before three digits, it returns `Status::Partial` after consuming
the available valid digits. If a byte is not an ASCII digit, that byte is
consumed before `Error::Status` is returned. A successful parse consumes
exactly three bytes and leaves later bytes untouched.

The parser contract relates the actual `Result<Status<u16>, Error>` and the
post-call `Bytes` view to `parse_code_model`; it specifies the actual
`Error::Status` variant. The included iterator implementation now has an
explicit `next` postcondition and `IteratorSpec`, with byte production and
cursor transition contracts available for this bridge.

Translation and proof status:

- The actual shared-source translation passed with the isolated string-model
  compiler. It generated the `parse_code` body and model contracts, actual
  `Bytes::next`, and the Error source. The exact selected closure passed 33/33
  proof leaves: 27 for the byte-reading dependency path and 6 for the model,
  digit helpers, and actual parser body. See [`REPORT.md`](REPORT.md) and the
  checksummed target artifacts in `evidence/current-after-next-contract/`.
- Native library checks passed for default and `--no-default-features`.
  Focused `cargo test --lib test_response_` runs also passed in both
  configurations (18 tests each, including success, partial code, and invalid
  code cases). Warnings were limited to the existing unstable-auto-trait
  warning from `creusot-std` and the expected no-std SIMD notice.
- The ordinary activated nightly compiler still stops at `src/error.rs:26` in
  the `description_str` string-literal postcondition (`Unsupported literal`
  ICE) before reaching `parse_code.rs`. The isolated command is
  `./verify-string.sh translate`; it uses the string-model compiler and copied
  `creusot-std` under `/workspace/scratch/httparse-string-model`.
- Replay the selected proof with `./verify.sh prove`. It runs direct
  `why3find prove --no-cache -s -j 1` from the `string/` harness through
  `run-proof.bash`, using the exact target list in
  `evidence/current-after-next-contract/TARGETS.tsv`. `verify-string.sh` only
  translates; it does not start a solver.

The source extraction is wired into the production crate at the original
crate-root position. A literal comparison with the original `parse_code` body
at base commit `99d87ae3485a56509e007523e8b5a19e2f0fb9a3` confirmed that the
runtime body is unchanged. The helper proof does not establish the outer
`Response::parse_with_config_and_uninit_headers` caller or the other parser
routines; those remain open. Neither the code model nor a finite input test is
treated as a runtime proof.
