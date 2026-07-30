#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)

export CARGO_NET_OFFLINE=true
rust_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
verus_target_dir="${VERUS_CARGO_TARGET_DIR:-$repo_root/target/verus/tokio-1.52.3}"
oneshot_probe_target_dir="${VERUS_ONESHOT_PROBE_TARGET_DIR:-$repo_root/target/verus/tokio-1.52.3-oneshot-poll-probe}"

# Keep the runtime regression focused on SetOnce. The upstream integration
# test is gated on `full`, so its exact test target still requires that feature.
CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full \
  --test sync_set_once

expected_verus="0.2026.07.27.31579f0"
actual_verus=$(verus --version)
if [[ "$actual_verus" != *"$expected_verus"* ]]; then
  echo "expected Verus $expected_verus" >&2
  echo "$actual_verus" >&2
  exit 1
fi

CARGO_TARGET_DIR="$verus_target_dir" cargo verus verify \
  --manifest-path "$script_dir/verification/Cargo.toml" \
  --locked \
  --offline

# This is a connection/translation probe for the production-shaped
# `Future::poll` signature. Its body is an explicit external boundary, so
# `0 verified, 0 errors` means the signature connected successfully; it is not
# a proof of the oneshot poll body or its wake-up behavior.
CARGO_TARGET_DIR="$oneshot_probe_target_dir" cargo verus verify \
  --manifest-path "$script_dir/verification-probes/oneshot-poll/Cargo.toml" \
  --locked \
  --offline
