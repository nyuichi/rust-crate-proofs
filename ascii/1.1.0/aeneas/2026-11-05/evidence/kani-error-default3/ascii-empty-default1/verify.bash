#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
export CARGO_NET_OFFLINE=true RUSTFLAGS='-A bindings_with_variant_name'
export CARGO_TARGET_DIR=/tmp/ascii-empty-default1-target
for harness in original_default_static_ascii_str_empty; do
 timeout 60s cargo kani --harness "$harness" --output-format terse > "evidence/$harness.log" 2>&1
done
