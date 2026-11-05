#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/evidence/kani-model"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
cargo kani 2>&1 | tee verification.log
