#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
export RUSTFLAGS='-A bindings_with_variant_name'
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/kani-case-operations/source-provenance.log
cargo kani --manifest-path evidence/kani-case-operations/Cargo.toml --output-format terse 2>&1 | tee evidence/kani-case-operations/verification.log
python3 - <<'INNER'
from pathlib import Path
s=Path('evidence/kani-case-operations/verification.log').read_text()
assert '7 successfully verified harnesses, 0 failures, 7 total' in s
assert s.count('VERIFICATION:- SUCCESSFUL') == 7
print('Seven original case/character API bodies over complete AsciiChar domains passed')
INNER
