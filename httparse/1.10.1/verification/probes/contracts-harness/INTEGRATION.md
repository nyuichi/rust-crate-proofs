# Shared-source integration checkpoint

This note records the shared-source `lib.rs` integration checkpoint. The chunk
parser remains in `mod chunk`; its actual body is proved against the shared
independent model in the isolated contracts harness.

`lib.rs` includes the actual `config.rs`, `status.rs`, `invalid_chunk_size.rs`,
`error.rs`, and `message.rs` files at crate root. This preserves public paths
such as `httparse::ParserConfig`, `httparse::Request`, and
`httparse::InvalidChunkSize` in Rust's `type_name` output. The extracted
`byteclass.rs` is an internal module, and its lookup constants and predicate
functions are re-exported crate-privately so the existing SIMD paths continue
to resolve. The original inline declarations have been removed from `lib.rs`.

The `public_type_names_keep_the_crate_root_path` test covers
`ParserConfig`, `Status<()>`, `Error`, `InvalidChunkSize`, `Request`,
`Response`, and `Header`. Request, response, and header type names contain the
compiler-rendered anonymous lifetime placeholders. The test passed after
moving `InvalidChunkSize` into its own included source file.

The latest native checks include the root callsite changes to the shared
`peek_array8` and `peek_array4` implementations. Their `iter.rs` contracts are
ghost-only and the methods preserve the previous fixed-size peek behavior.

| Command | Result |
|---|---|
| `cargo test --locked` | Passed: 101 unit tests, 263 URI integration tests, 6 doc tests. |
| `cargo test --locked --no-default-features` | Passed: 97 unit tests, 263 URI integration tests, 6 doc tests. |
| Contracts-harness `./verify.sh translate` | Passed after the chunk error type was root-included separately. |
| Contracts-harness `cargo check --locked --offline` | Passed with default and no-default features. |

Including `error.rs` in the contracts harness caused a Creusot compiler ICE on
the string literals in `Error::description_str`'s postcondition (`Unsupported
literal`). The chunk harness therefore includes `invalid_chunk_size.rs` only;
Error variant-description and Display proofs remain open. This compiler
limitation is not treated as evidence that the Error contract passes.

The current staged proof results are all fresh and use the elevated
`run-proof.bash` wrapper with `--why3session --no-cache` and Z3 4.15.3:

| Target | VCs |
|---|---:|
| `verification_chunk::hex_capacity` | 1/1 |
| `chunk::append_hex_digit` | 1/1 |
| `verification_chunk::step` | 1/1 |
| `chunk::hex_nibble` | 1/1 |
| `chunk::step_chunk_size` | 36/36 |
| `first_chunk_step` caller | 7/7 |
| `verification_chunk::scan` | 1/1 |
| `verification_chunk::scan_initial`, `scan_eof`, `scan_unfold` | 3/3 |
| ghost-only runtime `ChunkState::clone` | 1/1 |
| actual `chunk::parse_chunk_size` body | 9/9 |

The total staged result is 61/61 VCs. The outer parser proof uses three
body-checked ghost lemmas to connect its initial state, one-step loop
transitions, and EOF to the recursive model; no global trusted recurrence was
added. This establishes the actual extracted `parse_chunk_size` body against
the exact independent model, including complete consumed offset, partial, and
invalid outcomes. Z3 4.15.3 ran through the elevated wrapper with
`--why3session --no-cache`. See `../../../PROVENANCE.md` and
`../../../VERIFICATION_STATUS.md` for proof scope and remaining crate gaps.

The release-profile executable branch also passed a fresh selected proof:
`chunk::step_chunk_size` 33/33 and `chunk::parse_chunk_size` 9/9, using
`-- --offline --release` and the same elevated wrapper. Cargo reported the
release profile; the translated helper Coma omits the `cfg!(debug_assertions)`
guard. Snapshots of both release Coma files and their proof JSON are stored in
`evidence/chunk-release/`.

The release run replaced active Coma and proof JSON outputs for those two
targets. The preceding debug-profile Why3 sessions are preserved from their
generated `.bak` files under `evidence/chunk-debug-session-before-release/`;
their XML marks every goal valid. No debug proof JSON was reconstructed, so
the two affected debug targets can be regenerated in a later proof slot.

The isolated byte-class/message checkpoint can be reviewed separately; this
report does not claim that all bodies in the integrated crate are proved.
