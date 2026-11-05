#!/usr/bin/env bash
# Run only after the coordinating agent lifts the current resource hold.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
cargo metadata --offline --format-version 1 > evidence/metadata-debug.json
cargo metadata --offline --format-version 1 --features expect-no-debug > evidence/metadata-release.json
python3 tools/check-config.py
if cargo check --offline --manifest-path config-guard/Cargo.toml > evidence/no-std-export-guard.log 2>&1; then
 echo 'Unexpected std-only IntMap availability: rejecting configuration';exit 1
fi
python3 - <<'PY'
from pathlib import Path
s=Path('evidence/no-std-export-guard.log').read_text()
assert 'E0432' in s and 'nohash_hasher::IntMap' in s
print('Expected unresolved std-only IntMap compile-time guard confirmed')
PY
CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=true cargo kani --harness original_u8_debug > evidence/no-std-debug-kani.log 2>&1
CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=false cargo kani --features expect-no-debug --harness original_u8_debug > evidence/no-std-release-kani.log 2>&1
python3 - <<'PY'
from pathlib import Path
for mode in ['debug','release']:
 s=Path(f'evidence/no-std-{mode}-kani.log').read_text()
 assert 'VERIFICATION:- SUCCESSFUL' in s
 assert '1 successfully verified harnesses, 0 failures, 1 total.' in s
 assert 'Status: FAILURE' not in s
print('Both no_std dependency Kani runs succeeded; fixed std packet remains separate')
PY
python3 tools/check-config.py
