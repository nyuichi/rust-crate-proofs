#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
for manifest in Cargo.toml std-release/Cargo.toml nostd-debug/Cargo.toml nostd-release/Cargo.toml; do
 timeout 60s cargo kani --manifest-path "$manifest" --harness all_enabled_initial_prefix
done
