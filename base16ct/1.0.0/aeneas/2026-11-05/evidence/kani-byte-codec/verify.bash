#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py
cargo metadata --offline --format-version 1 > evidence/dependency-metadata.json
for harness in lower_pair_decode_all_bytes upper_pair_decode_all_bytes mixed_pair_decode_all_bytes lower_byte_encode_and_roundtrip upper_byte_encode_and_roundtrip; do
 cargo kani -Z unstable-options --harness-timeout 60s --harness "$harness" > "evidence/$harness.log" 2>&1
done
python3 tools/check-source.py
python3 tools/check-results.py
