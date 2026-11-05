#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
python3 check-source.py | tee source-check.log
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
export CARGO_NET_OFFLINE=true
cargo kani 2>&1 | tee default-kani.log
cargo kani --features serde 2>&1 | tee serde-kani.log
python3 - <<'AUDIT'
from pathlib import Path
for mode in ['default','serde']:
 s=Path(mode+'-kani.log').read_text()
 assert 'VERIFICATION:- SUCCESSFUL' in s
 assert 'Complete - 1 successfully verified harnesses, 0 failures, 1 total.' in s
print('MCG original all-seed public-state contract successful under default and serde')
AUDIT
