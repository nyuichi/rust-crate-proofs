# `parse_code` verification checkpoint

## Result

The exact runtime body of `src/parse_code.rs` is proved against an independent
model of its result and `Bytes` cursor effects. The selected proof contains 33
valid Z3 obligations across 11 targets: 27 for the byte-reading dependency
path and 6 for the digit helpers, exact model, and parser body. The generated
Coma, per-target proof JSON, Why3 sessions, and hashes are frozen under
[`evidence/current-after-next-contract`](evidence/current-after-next-contract/).

`parse_code` accepts all three-digit decimal values from 000 through 999; it
does not impose the HTTP status range 100 through 599. It consumes exactly
three digits on success and leaves any following byte untouched. EOF before a
bad byte returns `Partial` after consuming each available valid digit. A
non-digit is consumed before `Error::Status` is returned. The model records
these exact branches, including the unchanged mark and absolute cursor.

This proves the helper boundary. The outer `Response::parse_with_config_and_uninit_headers`
caller and the other parser routines remain separate proof work; this
checkpoint does not claim an integrated parser proof.

## Source and model

`lib.rs` includes `src/parse_code.rs` at the original crate-root location. A
literal comparison of the extracted runtime function with the original
`parse_code` body at base commit
`99d87ae3485a56509e007523e8b5a19e2f0fb9a3` confirmed the body is unchanged.
The harness path-includes the
actual `Bytes`, `Status`, `Error`, and parser macro source files. It defines no
substitute result enum or duplicate parser body.

`verification/code.rs` independently models EOF or a bad byte at each of the
three read positions, plus the complete three-digit path. The proof checks the model helper
bodies (`is_ascii_digit`, `decimal_digit_value`, and `parse_code_model`) and
the actual parser body against that model.

The old `Iterator::next` contract exposed only the `IteratorSpec::produces`
relation, which did not preserve `Bytes.mark` or state the returned byte and
cursor transition. The current contract adds those exact frame and result
facts. Its implementation is body-checked through `peek`, `bump`, `advance`,
and the byte-permission helper; the existing `IteratorSpec` relation remains
and its refinement target passes.

## Proof results

All 33 entries in `TARGETS.tsv` are distinct `(target path, VC leaf)` pairs,
and every leaf is recorded as valid with Z3 4.15.3.

| Target | VCs | Result |
| --- | ---: | --- |
| `bytes_subsequence_head` | 1 | Valid |
| `Bytes::byte_permission` | 7 | Valid |
| `Bytes::peek` | 4 | Valid |
| `Bytes::advance` | 6 | Valid |
| `Bytes::bump` | 2 | Valid |
| `Bytes::next` | 6 | Valid |
| `Bytes::next` trait refinement | 1 | Valid |
| `is_ascii_digit` | 1 | Valid |
| `decimal_digit_value` | 1 | Valid |
| `parse_code_model` | 1 | Valid |
| actual `parse_code` | 3 | Valid |

The proof first checked the iterator closure, then the model helpers and actual
parser target. The previous attempt, before the stronger `next` contract,
stalled on `parse_code` and was interrupted without a final Why3 status. It
does not count as `Unknown`, `Timeout`, or `Invalid`; its nine earlier green
targets are preserved separately in
[`evidence/first-attempt-before-next-frame`](evidence/first-attempt-before-next-frame/).

## Reproduction environment

Translation used the isolated string-model compiler because the ordinary
nightly compiler ICEs on existing string literals in `src/error.rs` before it
reaches `parse_code.rs`. The actual parser and actual error source still pass
through translation with the isolated compiler.

The fresh translation used `nightly-2026-02-27`, Rust 1.95.0-nightly commit
`6a979b3e32522049d0acb4a47f7ae44b7c8abfd5`, and `cargo-creusot 0.11.0-dev`.
The isolated compiler SHA-256 was
`a1ea923760d0225f828e90e2b405c4f7d3de4177868aad6f04f158472778f293` both
before and after translation. Why3 was 1.8.2+git, why3find was 1.2.0+dev, and
the selected prover was Z3 4.15.3. Proofs ran from the exact `string/` harness
directory using direct `why3find prove --no-cache -s -j 1` over the targets in
`TARGETS.tsv`, through the repository wrapper at
`httparse/1.10.1/run-proof.bash`, with one prover and a 1000 MiB limit. The
Why3 config hash was
`e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`; the
harness `why3find.json` hash was
`a5609701e7760be9f89cada433de5274372ea89856f2e01803c619586e808aca`.

The selected isolated manifest is
`verification/probes/code-harness/string/Cargo.toml`; its hash is
`8d09c9c45e2c15c09a974c91cd6d5fd787901b549f808a377b859e8138ae0f9e`. It
uses the string-model copy of `creusot-std` under
`/workspace/scratch/httparse-string-model/creusot-libs`.

## Native checks and trust boundary

Before the proof-only `Iterator::next` contract additions, native library
checks passed with default features and with `--no-default-features`. The
focused `cargo test --lib test_response_` runs passed all 18 response tests in
each configuration. The subsequent source edits add only Creusot contracts;
the runtime `parse_code`, `Bytes::next`, and iterator bodies did not change.

No algorithm body in this selected crate scope is marked trusted. The proof
uses the contracts for raw-pointer and permission primitives supplied by the
Creusot standard library; those standard-library implementations remain in
the trusted base. The result is a source-level proof of the helper and the
listed `Bytes` contracts, not a proof of Rust's pointer implementation or of
the full HTTP parser.

## Source hashes

| File | SHA-256 |
| --- | --- |
| `src/parse_code.rs` | `19c688a14893da0a51bbc626a49e825b5807b7a68499d7e93b77fb46f6150fb1` |
| `src/verification/code.rs` | `7e199a638766d49c45f969f589f465ecd891cd8f246e915d5d4c4103e9b723db` |
| `src/iter.rs` | `369662bbf36c68ba814750ce3a91c4aa78ab40e9453af0f7e07f30d869103625` |
| `src/lib.rs` | `d2ce3b18454b17efd578550bd439319cb9e6707a00b16f3962a7444fb5a13307` |
| `src/verification/model.rs` | `cf3a6f426eb55e28f61a2f5b46277639db9ff86ec0b584bf026735c59aa25cb1` |
| harness `src/lib.rs` | `ac66086d45912485e3ec57a22dc42bf85421c75168d01a985373bfddb6f51338` |
| harness `Cargo.toml` | `a3d0a9fbe8e002e615ea80a35b23a2fc00b277d55306f36cc16f6248fe3e73da` |
| string harness `Cargo.toml` | `8d09c9c45e2c15c09a974c91cd6d5fd787901b549f808a377b859e8138ae0f9e` |
| `why3find.json` | `a5609701e7760be9f89cada433de5274372ea89856f2e01803c619586e808aca` |
