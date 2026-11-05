#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
export PATH=/workspace/kani-tools/kani-0.68.0/bin:$PATH
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
CARGO_NET_OFFLINE=true cargo kani --solver z3
