#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
python3 check-source.py | tee source-check.log
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
export CARGO_NET_OFFLINE=true
timeout --signal=TERM --kill-after=5s 900s cargo kani --manifest-path source/Cargo.toml 2>&1 | tee default-kani.log
timeout --signal=TERM --kill-after=5s 900s cargo kani --manifest-path source/Cargo.toml --features serde 2>&1 | tee serde-kani.log
python3 checkpoint.py
