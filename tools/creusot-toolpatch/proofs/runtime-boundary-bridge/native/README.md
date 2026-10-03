# Native runtime checks

These checks ran for `itoa/1.0.18` on `nightly-2026-02-27`, with Cargo offline and a native target directory separate from the proof target.

Set the common environment from the repository root:

```sh
export PATH=/tmp/cargo-home/bin:$PATH
export RUSTUP_HOME=/tmp/rustup-home
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_HOME=/tmp/cargo-home
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=/workspace/proof-tools/targets/itoa-runtime-bridge-final-native
```

The ordinary debug test run without features exited 0. It passed 12 integration tests and 2 doctests; the library unit-test harness contained 0 tests. See [`default-debug-no-features.log.gz`](default-debug-no-features.log.gz).

```sh
cargo test --offline --locked --manifest-path itoa/1.0.18/Cargo.toml
```

The optimized all-features command initially used fat LTO and one codegen unit, but did not pass matching optimization flags to rustdoc. It exited 101: all 12 integration tests passed, while both doctest-generated binaries failed the `no-panic` link check. The failures came from those separately generated binaries, which did not receive the release optimization needed by the detector. See [`release-fat-cgu1-initial-rustdoc-failure.log.gz`](release-fat-cgu1-initial-rustdoc-failure.log.gz).

Passing the flags explicitly to rustdoc made the doctests pass (2/2, exit 0). The full release command with those flags then passed 12 integration tests and 2 doctests (exit 0; 0 library unit tests). See [`release-doc-fat-cgu1-explicit-rustdocflags.log.gz`](release-doc-fat-cgu1-explicit-rustdocflags.log.gz) and [`release-fat-cgu1-explicit-rustdocflags.log.gz`](release-fat-cgu1-explicit-rustdocflags.log.gz).

```sh
CARGO_PROFILE_RELEASE_LTO=fat \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
cargo test --offline --locked --release --all-features --manifest-path itoa/1.0.18/Cargo.toml
```

```sh
CARGO_PROFILE_RELEASE_LTO=fat \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
RUSTDOCFLAGS='-C opt-level=3 -C lto=fat -C codegen-units=1 -C embed-bitcode=yes' \
cargo test --offline --locked --release --all-features --manifest-path itoa/1.0.18/Cargo.toml
```
