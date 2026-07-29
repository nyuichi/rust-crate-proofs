#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
cd "$script_dir"

export CARGO_NET_OFFLINE=true
creusot_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
verus_target_dir="${VERUS_CARGO_TARGET_DIR:-$repo_root/target/verus/heapless-0.9.2}"

CARGO_TARGET_DIR="$creusot_target_dir" cargo creusot --simple-triggers=false prove

expected_verus="0.2026.07.27.31579f0"
actual_verus=$(verus --version)
if [[ "$actual_verus" != *"$expected_verus"* ]]; then
  echo "expected Verus $expected_verus" >&2
  echo "$actual_verus" >&2
  exit 1
fi

CARGO_TARGET_DIR="$verus_target_dir" cargo verus verify --locked --features verus
