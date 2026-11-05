#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
export CARGO_NET_OFFLINE=true
timeout --signal=TERM --kill-after=5s 60s cargo kani --harness original_deserialize_fixed_protocol
