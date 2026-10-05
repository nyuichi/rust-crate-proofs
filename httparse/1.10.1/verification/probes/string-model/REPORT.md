# Isolated string-model translation checkpoint

This probe includes the actual src/message.rs and src/error.rs files and
exercises decoded Rust string literals, the str sequence model, exact UTF-8
examples, EMPTY_HEADER.name, and Error::description_str. Intentionally false
string and byte claims use a separate negative/Cargo.toml manifest, so their
expected failure cannot mask positive translation or proof results. A smaller
error/Cargo.toml harness remains available to isolate only the actual error
body and its formatting trait translations.

## Implementation boundary

- Compiler source is pinned to 437d3d8d00b8114d7a3b4f7b8738d594a395f5bc.
- Apply the existing narrowcast patch first, then
  tools/creusot-toolpatch/patches/httparse-string-model.patch.
- The string compiler patch adds THIR string-literal handling, maps str to
  seq.Seq.seq Char.t, lowers each Rust Unicode scalar through Char.of_int,
  and tracks the seq.Seq dependency. It updates all four Char upper bounds to
  0x110000 while retaining surrogate exclusion.
- The full local creusot-libs tree is copied to
  /workspace/scratch/httparse-string-model/creusot-libs. Only its copied
  creusot-std/src/std/string.rs gets the builtin identity View patch.
  Independent convert.rs additions are synchronized from the current local
  library tree and retained.
- prelude-generator writes into the isolated compiler source tree; the
  generated package is copied to the isolated data home at
  /workspace/scratch/httparse-string-model/creusot-data/share/why3find/packages/creusot/creusot.
  The active installed package is not edited.
- The compiler build and probe use the fresh target directory
  /workspace/proof-tools/targets/httparse-string-model.

## Current status

The isolated compiler build and reproducible build wrapper passed. Positive
native checks passed with default and no-default features. Negative and error
native checks passed. Each native check emitted only the existing creusot-std
unstable-auto-trait warning.

Positive translation passed for actual src/error.rs and src/message.rs.
Generated Coma shows:

- the empty literal and actual EMPTY_HEADER.name initializer as a typed
  `Seq.empty: Seq.seq Char.t`;
- "A\0" as scalar values 65 and 0;
- é and € as scalar values 233 and 8364 with byte sequences [195,169] and
  [226,130,172];
- U+D7FF, U+E000 and U+10FFFF as scalar values 55295, 57344 and 1114111, with
  exact UTF-8 bytes ending in [244,143,191,191] for U+10FFFF.

The error translation emits the actual description_str, caller, Display,
std::error::Error::description, and derived Debug COMA bodies. The actual
empty-header constant initializer is visible in the generated message module.
No view_str or primitive-string lowering appears in these targets. The
separate negative translation succeeds and emits contradictory expected claims
(16 returned characters versus a 24-character string claim, and returned byte
66 versus claimed byte 65); this is translation evidence only, not a solver
counterexample result.

The first positive proof invocation reached Why3 but failed while parsing the
actual `actual_empty_header_name.coma`, before generating any VC or starting a
prover. Why3 rejected the earlier empty-string representation `Seq.create 0
[||]`. The isolated compiler now lowers zero-character strings to the existing
`seq.Seq.empty` with explicit `Seq Char.t` type; no new string axiom or default
character was introduced. The updated compiler rebuilt successfully and
positive, negative, and error translations all passed. `why3 prove --type-only`
then parsed and typechecked all 52 emitted COMA files (20 positive, 22
negative, 10 error); it does not run a prover and supplies no proof counts.
The positive, negative, and error COMA manifests hash to
`cd30403d6087a55e572d471b347b388dad722f852370b166459c2738d6182e2d`,
`9e76e93ccd055305c8b6ac5f608cc7a568117c708be3a404c28817ae2dc34cd9`, and
`dae13642318847492b6fc63ad2baf9c74b49a67b5bf105ad1a58c26a61b7a0e5`,
respectively.

The proof runner now calls `why3find prove` directly from each exact harness
directory with the isolated Why3 config and package path, bypassing the Cargo
proof frontend's auto-refresh behavior. Proof modes save Why3 sessions and set
`-j 1`; negative/error modes create their evidence directory before logging
prover results. Its isolated XDG config/cache paths are also set for
translation. An initial Cargo proof-frontend invocation
refreshed the global Why3 config from 1 to 4 parallel provers; this setting was
restored to 1, with the 1000 MiB limit and active tool paths retained. The
global config hash is `e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`.

## Solver checkpoint

The positive run was completed with the isolated `why3find` package and the
one-prover, 1000 MiB profile. It accepted 21 of 27 positive VC obligations.
The six unproved obligations are:

- `scalar_boundaries`, `two_byte_scalar`, and `three_byte_scalar`: exact
  non-ASCII UTF-8 byte claims remain unproved. The lowered scalar and expected
  byte constants were inspected and match Rust UTF-8 encodings; the saved
  Why3 proof JSON records null for these goals, which does not distinguish
  timeout/unknown from invalid.
- `impl_Debug_for_Error::fmt__refines`: exact derived Debug refinement remains
  unproved; its separate `fmt` body VC passed. A passing formatting body VC
  does not establish exact rendered text.
- `impl_PartialEq_for_Error::eq` and `eq__refines`: both remain unproved.

The other positive obligations passed, including the actual
`EMPTY_HEADER.name` empty-string result, Request and Response constructors,
empty/ASCII-with-NUL/raw string literals, and actual Error description calls.
The positive run's `actual_error_description`, `description_str`, and
`std::error::Error::description` results establish those exact description
strings under their stated contracts. Clone and formatting body-safety goals
also passed. The `Display::fmt` goals certify its generated body obligations,
not exact observable formatting semantics.

The separate Error run was interrupted after its selected-target prefix and
must not be counted as a complete Error manifest result. It had reached the
actual description, description caller, Error trait description, Clone,
`Display::fmt` body, and `Debug::fmt` body targets; the known Debug refinement
failure was observed before interruption. Duplicate Eq and Debug refinement
targets were not rerun after that failure. These partial results do not close
the Error harness.

The deliberately false string and UTF-8 claims have been translated and
typechecked. No negative solver run has been made yet. The current
`prove-negative` mode selects only the two contradictory claim COMA files and
logs prover results to `evidence/negative-prover-results.jsonl`; it is ready
for a later authorized run. Therefore there is no negative counterexample or
rejection result to report at this checkpoint.

No new httparse-specific string axiom was added; UTF-8 calls still refer to the
copied standard library's existing `CharExt::to_utf8` encoding model. The
positive scalar-byte obligations remain open and require body-checked standard
library lemmas before claiming full Unicode coverage.

Patch hashes at this checkpoint:

- narrowcast patch: c3ac3e596b822f483d2f0f782169aacc5c752449cf66771ee7268ee2feb8eb24
- string compiler patch: 8b3205493c45b909c8fb980b3e6cb3c4f63d52511060f12ec227c4e105d4f2ea
- isolated standard-library View patch: 7b92f2d54dc845a982246c220bc37004ae14d6ee5e51ff0ff4bbc87eb90b5ab9
- generated and installed isolated prelude.coma: cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b
- synchronized shared creusot-std/src/std/convert.rs: 5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348
- isolated compiler binary: a1ea923760d0225f828e90e2b405c4f7d3de4177868aad6f04f158472778f293
- global Why3 config restored to memlimit=1000, running_provers_max=1: e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07
- string-model proof runner (`verify.sh`): fb0198eae713ac691a939b7be88a014cb1fb5faa696ad730d8f4cd744b033178
