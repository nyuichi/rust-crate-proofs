#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
tool_home="${AENEAS_HOME:-/workspace/aeneas-tools}"
if [[ -f "$tool_home/activate.sh" ]]; then
  source "$tool_home/activate.sh"
fi
mkdir -p evidence/release
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test 2>&1 | tee evidence/original-tests.log
cargo test --release 2>&1 | tee evidence/release/original-tests.log
sha256sum proofs/Generated/{NohashHasher.lean,translation.json} \
  proofs/ReleaseGenerated/{NohashHasher.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py
python3 tools/extract.py --release
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then
  echo 'Unproved declaration found' >&2
  exit 1
fi
(
  cd proofs
  lake build
  lake env lean Proofs.lean 2>&1 | tee ../evidence/axioms.log
) 2>&1 | tee evidence/verification.log
python3 - <<'PY'
from pathlib import Path
import re
rows = re.findall(r"'Proofs\.(\w+)' depends on axioms: \[(.*?)\]", Path('evidence/axioms.log').read_text())
assert len(rows) == 14, rows
for name, axioms in rows:
    assert set(axioms.split(', ')) <= {'propext','Classical.choice','Quot.sound'}, (name, axioms)
print('All 14 combined debug/release theorem axiom audits passed')
PY
sha256sum src/lib.rs Cargo.toml evidence/upstream.crate \
  proofs/Generated/{NohashHasher.lean,translation.json} \
  proofs/ReleaseGenerated/{NohashHasher.lean,translation.json} \
  proofs/{DebugProofs,ReleaseProofs,Proofs}.lean > evidence/source-generated.sha256
