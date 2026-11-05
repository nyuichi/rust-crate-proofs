#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
python3 tools/check-evidence.py --source-only
CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true cargo kani --harness original_u8_debug > evidence/debug-u8-final.log 2>&1
CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=false cargo kani --features expect-no-debug --harness original_u8_debug > evidence/release-u8-final.log 2>&1
cargo run --example native > evidence/native-debug.log 2>&1
cargo run --release --example native > evidence/native-release.log 2>&1
python3 tools/check-evidence.py
