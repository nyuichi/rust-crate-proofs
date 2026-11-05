#!/usr/bin/env bash
set -euo pipefail
source /workspace/aeneas-tools/activate.sh
root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:?private regeneration output required}"
mkdir -p "$output/generated" "$output/mir"
export CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/bitfield-range-u8-inline-target
export RUSTFLAGS="-Zmir-enable-passes=+Inline,+DataflowConstProp,+SimplifyLocals-final -Zinline-mir=yes -Zinline-mir-forwarder-threshold=10000 -Zinline-mir-hint-threshold=10000 -Zinline-mir-threshold=10000 -Zdump-mir=all -Zdump-mir-dir=$output/mir"
cd "$root/tools/selector"
charon cargo --preset aeneas --sysroot default --mir optimized --include 'bit_field::_::BIT_LENGTH' --dest "$output" --start-from crate::original_u8_get_range --start-from crate::original_u8_set_range > "$output/charon.log" 2>&1
aeneas -backend lean -split-files -emit-json -abort-on-error -dest "$output/generated" "$output/bitfield_range_u8_inline.llbc" > "$output/aeneas.log" 2>&1
