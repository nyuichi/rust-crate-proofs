#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
# This activation path is specific to the recorded environment; elsewhere put
# rustup/rustc on PATH before invoking this script.
if [[ -f /workspace/proof-tools/activate.sh ]]; then
  source /workspace/proof-tools/activate.sh
fi
kani_home="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}"
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="$kani_home/bin:$PATH"
python3 prepare.py
kani-driver harness.rs --harness one_step_equivalence 2>&1 | tee result.log
python3 - <<'PY'
from pathlib import Path
log = Path('result.log').read_text()
assert 'VERIFICATION:- SUCCESSFUL' in log
assert '1 successfully verified harnesses, 0 failures, 1 total.' in log
print('Source-body relational harness passed')
PY
