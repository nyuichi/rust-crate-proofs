#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
export CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/bitfield-rangefull24-target
for harness in get_full_u8 set_full_u8 get_full_u16 set_full_u16 get_full_u32 set_full_u32 get_full_u64 set_full_u64 get_full_u128 set_full_u128 get_full_usize set_full_usize get_full_i8 set_full_i8 get_full_i16 set_full_i16 get_full_i32 set_full_i32 get_full_i64 set_full_i64 get_full_i128 set_full_i128 get_full_isize set_full_isize; do
 timeout 60s cargo kani --harness "$harness" --output-format terse > "evidence/$harness.log" 2>&1
done
