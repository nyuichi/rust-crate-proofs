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
cargo kani --harness original_rand32_float_range_only 2>&1 | tee rand32-range-only.log
cargo kani --harness original_rand64_float_range_only 2>&1 | tee rand64-range-only.log
python3 - <<'AUDIT'
from pathlib import Path
for n in [32,64]:
 s=Path(f'rand{n}-range-only.log').read_text()
 assert 'VERIFICATION:- SUCCESSFUL' in s and '0 of 33 failed' in s
print('Both arbitrary-state/increment original rand_float range contracts passed (33 checks each).')
AUDIT
