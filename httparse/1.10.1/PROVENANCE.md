# httparse 1.10.1 provenance and verification status

**Status: partial; import, runtime-test baseline, shared-source contracts,
isolated byte-class/message proofs, and current-worktree integration runs are
recorded. The extracted chunk-size parser is body-proved; full-crate
verification remains open.**

This crate tree was copied from the published crates.io archive
`httparse-1.10.1.crate`, whose SHA-256 is
`6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87`. The
archive's `.cargo_vcs_info.json` records upstream revision
`9f29e79f9832dbd0ae5220acb17c1866745bdecd` from
<https://github.com/seanmonstar/httparse>.

The complete published file tree, including the build script, optimized
scanner backends, tests, benchmarks, README, and both license files, was copied
from `/tmp/httparse-1.10.1`; the import was checked with `diff -qr` before any
verification files were added. `Cargo.toml` adds the repository-local
`creusot-std` path dependency and registers `cfg(creusot)` alongside the
upstream SIMD cfg names. Cargo regenerated `Cargo.lock` for this local path
dependency. Proof annotations and model files are repository additions. Any
source changes to executable code must be recorded here with their effect on
the published behavior. `ParserConfig`'s derived `Default` was replaced by a
manual implementation that initializes the same seven fields to `false`; this
is behavior-preserving and lets the implementation carry an explicit contract.
`ParserConfig` and `Status` were extracted to `src/config.rs` and
`src/status.rs`; the current worktree includes both at crate root so their
public type names remain unchanged. `Error`, `InvalidChunkSize`, and
Request/Response/Header use root-level shared source includes as well.
`InvalidChunkSize` lives in `src/invalid_chunk_size.rs` because the chunk parser
is a child module, while its runtime type name remains
`httparse::InvalidChunkSize`. The seven configuration flags are crate-visible
only so the parser adapters can continue reading them. The `std` import under
`cfg(test, not(feature = "std"))` enables unchanged upstream test macro calls
in no-default-features builds. Under `cfg(creusot)`, derived `Debug` is omitted
and `Status<T>` also omits derived `Eq`/`PartialEq`, whose generic formatter and
equality refinements are not yet modeled. Ordinary builds retain the upstream
derives. `src/iter.rs` retains the published runtime implementation; its
pointer and slice operations have selected mechanically checked runtime
bridges documented in the memory-pointer report. Private typed array helpers
retain the original fixed-size conversion behavior.

## Runtime baseline

The frozen baseline was run with `./test-baseline.bash` from this directory;
the script loads `/workspace/proof-tools/activate.sh` and scopes Cargo to this
crate directory.

| Configuration | Command | Result |
|---|---|---|
| Default (`std`) | `cargo test --locked` | Passed: 100 unit tests, 263 URI integration tests, 6 doc tests. |
| `no_std` | `cargo test --locked --no-default-features` | Passed: 96 unit tests, 263 URI integration tests, 6 doc tests. Build script reported that no_std disables SIMD. |

The local `creusot-std` path dependency is resolved and compiled during both
native configurations. Its build emits one nightly `auto trait` warning from
`creusot-libs/creusot-std/src/ghost.rs`; the `httparse` no-default build remains
`no_std`, and its full test suite passes.

The recorded compiler is `rustc 1.95.0-nightly (6a979b3e3 2026-02-26)` with
`cargo 1.95.0-nightly (f298b8c82 2026-02-24)`, target/host
`x86_64-unknown-linux-gnu`, little-endian, 64-bit. The baseline target cfg had
`fxsr,sse,sse2,x87` and did not enable compile-time SSE4.2 or AVX2. The default
build script enabled SIMD and selected runtime feature dispatch; the exact
runtime-selected instruction backend was not instrumented. The no-default
build script disabled SIMD, leaving the scalar/SWAR scanner. SIMD truth-table
unit tests for the x86 SSE4.2 and AVX2 implementations passed in the default
test run. The frozen baseline used the behavior-preserving manual
`ParserConfig::default` and the no-default test-only `std` macro import, before
any parser implementation body was claimed proved.

## Shared-source integration checkpoint

The current worktree wires `src/byteclass.rs` as an internal module while
preserving the existing crate paths used by SIMD code. The shared source files
for configuration, status, errors, chunk error, and message types are included
at crate root so Rust's public type identity remains at `httparse::…`. The
chunk parser itself remains in `mod chunk` and is re-exported from the root;
its extracted runtime algorithm is separately verified against the independent
chunk-size model in the isolated shared-source harness.

The latest native runs also cover the `peek_array8`/`peek_array4` callsites in
`parse_version` and `parse_method`. The added `src/iter.rs` contracts are
ghost-only; those helpers use the same fixed-prefix slice and array conversion
as the previous `peek_n::<[u8; N]>(N)` callsites.

| Configuration | Command | Result |
|---|---|---|
| Default (`std`) | `cargo test --locked` | Passed: 101 unit tests, 263 URI integration tests, 6 doc tests. |
| `no_std` | `cargo test --locked --no-default-features` | Passed: 97 unit tests, 263 URI integration tests, 6 doc tests. |

The added `public_type_names_keep_the_crate_root_path` test checks
`ParserConfig`, `Status<()>`, `Error`, `InvalidChunkSize`, `Request`,
`Response`, and `Header`. Lifetime-parameterized types use the compiler's
`'_` placeholders in the expected `type_name` output. After moving the
unchanged `InvalidChunkSize` declaration to its own included file, this test
was rerun and passed.

The contracts harness includes `src/invalid_chunk_size.rs` at crate root to
resolve the actual `src/chunk.rs` body against the same error type. Translation
passes; native offline `cargo check` passes with default and no-default
features. `src/error.rs` is not part of that chunk harness: Creusot ICEs on the
string literals in `Error::description_str`'s postcondition (`Unsupported
literal`). Error descriptions, Display refinement, and full-library Creusot
translation remain open, and this limitation is not treated as proved.

The pinned verification environment provides cargo-creusot `0.11.0-dev`, Why3
`1.8.2+git`, why3find `1.2.0+dev`, Z3 `4.15.3`, CVC5 `1.3.1`, and CVC4 `1.8`.
The Z3, CVC5 and CVC4 versions were queried directly; cargo-creusot's pinned
version comes from the activated proof environment.

An independent scalar model checkpoint passes translation and proof in
`verification/probes/model-harness`. It includes `src/verification/model.rs`
directly because metadata resolution for the upstream test/benchmark graph
needs unavailable registry downloads. `CARGO_NET_OFFLINE=true ./verify.sh
translate` completed successfully. `CARGO_NET_OFFLINE=true ./verify.sh prove`
completed successfully through the target-local `run-proof.bash`, with one Z3
4.15.3 prover and a 1000 MiB limit. Seven generated proof files and seven
obligations passed: `maximal_prefix_end`, `accepted_prefix_span`,
`parse_token_model`, and the four derived `Clone` bodies. The model function
bodies are proved in isolation only; their open byte-class definitions are not
runtime table contracts. The generated Coma files and per-goal `proof.json`
session results are in the harness `verif/` tree for reproduction.

The integrated Creusot proof, runtime-refinement bridge, initialized and
uninitialized header storage, every seven-flag configuration, and SIMD/runtime
dispatch are all outstanding. Baseline tests are behavioral evidence, not a
full runtime proof claim. The complete callable and unsafe boundary inventory is in
[`API-INVENTORY.md`](API-INVENTORY.md); proof status and gaps are tracked in
[`VERIFICATION_STATUS.md`](VERIFICATION_STATUS.md).

## Shared-source `Status` and `ParserConfig` checkpoint

`verification/probes/contracts-harness` includes the actual `src/status.rs`
and `src/config.rs` files in wrapper modules. Its caller invokes those actual builder methods
in sequence, setting one flag to `true`, changing another flag, then changing
the first to `false`; the caller's postcondition records the exact final
seven-Boolean tuple. No replacement parser or `cfg(creusot)` parser body is
used. Contracts state exact `Status` variant behavior, the `unwrap`
precondition/payload result, all-false `ParserConfig::default`, each setter's
single-field update and returned mutable-borrow state, and each of the four
upstream getters.

Reproduction from the harness directory:

```sh
source /workspace/proof-tools/activate.sh
CARGO_NET_OFFLINE=true ./verify.sh translate
/workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash \
  cargo creusot --simple-triggers=false prove --why3session --no-cache
```

The uncached Why3 run used Z3 4.15.3 with the shared one-prover/1000 MiB
profile and completed successfully. All 19 generated Coma files and their 19
`proof.json` records are in the harness `verif/` tree; all 26 verification
conditions passed: chained builder caller (4), seven setters (7), four
getters (4), `Default` (1), `ParserConfig::clone` (2), three `Status` methods
(4), and `Status::clone` body/refinement (4).

This proves the extracted `Status` and seven-flag configuration surface and
that small builder caller. The four parser adapters in `lib.rs`, `Error` and
error formatting, `ParserConfig::Debug`, `Status::Debug`/`Eq`/`PartialEq`, and
request/response parsers remain outside this harness or explicitly excluded
under `cfg(creusot)`.

The current staged chunk proofs are fresh, target-scoped, and use the elevated
`run-proof.bash` wrapper with `--why3session --no-cache` and Z3 4.15.3:
`hex_capacity` (1/1), `append_hex_digit` (1/1), model `step` (1/1),
`hex_nibble` (1/1), runtime `step_chunk_size` (36/36), representative caller
`first_chunk_step` (7/7), and model `scan` (1/1): 48/48 VCs. A fresh no-cache
refresh against the final recurrence-lemma source passed those targets, the
three new lemmas, runtime `ChunkState::clone`, and the outer parser for 61/61
VCs. The three body-checked `scan_initial`, `scan_eof`, and `scan_unfold`
lemmas plus ghost-only `ChunkState::clone` account for 4/4 of those goals.
The outer `parse_chunk_size` proof passed all 9/9 VCs
(`vc_elim_Complete`, `vc_elim_Continue`, `vc_len_u8`, `vc_new_unit`,
`vc_parse_chunk_size`, `vc_scan_eof`, `vc_scan_initial`, `vc_scan_unfold`, and
`vc_step_chunk_size`). All proof runs used the elevated wrapper, no cache, and
Z3 4.15.3. The current chunk proof subtotal is 61/61 VCs. The outer proof
checks the actual shared runtime body against the independent scanner model;
its loop invariant preserves exact result equality, and the three finite
ghost calls connect initial, iterative, and EOF states without trusting the
recursive scan definition globally.

The complementary release-profile proof also passed:
`step_chunk_size` (33/33 VCs) and `parse_chunk_size` (9/9 VCs), with the
command forwarding `--release` to Cargo. Cargo reported the release profile,
which sets `debug_assertions=false` and removes the cfg-selected overflow guard
from the runtime helper's Coma. The exact release Coma and proof JSON files are
snapshotted under `verification/probes/contracts-harness/evidence/chunk-release/`
before any further translation. The release run replaced the active Coma and
proof JSON for those two targets; the preceding debug proof sessions were
preserved from Why3's `.bak` files under
`verification/probes/contracts-harness/evidence/chunk-debug-session-before-release/`.
No debug proof JSON was reconstructed; those two targets can be regenerated in
a later proof slot. Both selected harness configurations cover the executable
cfg branches; neither is a whole-crate integrated Creusot run.

Run `./test-baseline.bash` from this directory for the pinned default and
no-default test commands. `verify-partial.bash` runs one selected proof
configuration through `run-proof.bash`, which sources the active proof
environment, holds the shared `/tmp/itoa-creusot-proof.lock`, and verifies the
configured single-prover, 1000 MiB Why3 profile. It invokes the standard active
`cargo creusot` toolchain and does not depend on the itoa-specific patched
compiler. `verify-all.bash` uses the same wrapper and checks the explicit closed
status gate and every component row before starting, so an incomplete or
model-only ledger cannot produce a full-verification green result. Start the
first proof invocation with elevated sandbox permissions because Why3 uses a
Unix-domain socket.
