#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
export RUSTFLAGS='-A bindings_with_variant_name'
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/kani-conversions/source-provenance.log
cargo kani --manifest-path evidence/kani-conversions/Cargo.toml --output-format terse 2>&1 | tee evidence/kani-conversions/verification.log
python3 - <<'INNER'
from pathlib import Path
s=Path('evidence/kani-conversions/verification.log').read_text()
assert '6 successfully verified harnesses, 0 failures, 6 total' in s
assert s.count('VERIFICATION:- SUCCESSFUL') == 6
print('Five original checked API bodies over full type domains and one valid-discriminant intrinsic check passed')
INNER
