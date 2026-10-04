# httparse 1.10.1 provenance and verification status

**Status: partial; import, runtime-test baseline, and an isolated model proof
checkpoint are recorded. No executable httparse parser body is yet claimed
proved.**

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
The `std` import under `cfg(test, not(feature = "std"))` enables the unchanged
upstream test macro calls in no-default-features builds. `src/iter.rs` remains
the published implementation; its pointer and slice operations have no
mechanically checked runtime bridge yet.

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
