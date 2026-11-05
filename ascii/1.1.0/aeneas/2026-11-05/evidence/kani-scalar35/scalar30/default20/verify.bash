#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
export CARGO_NET_OFFLINE=true RUSTFLAGS='-A bindings_with_variant_name'
export CARGO_TARGET_DIR=/tmp/ascii-new-default20-target
for harness in ne_u8_forward lt_u8_forward le_u8_forward gt_u8_forward ge_u8_forward ne_u8_reverse lt_u8_reverse le_u8_reverse gt_u8_reverse ge_u8_reverse ne_char_forward lt_char_forward le_char_forward gt_char_forward ge_char_forward ne_char_reverse lt_char_reverse le_char_reverse gt_char_reverse ge_char_reverse; do
 timeout 60s cargo kani --harness "$harness" --output-format terse > "evidence/$harness.log" 2>&1
done
