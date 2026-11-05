#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/proofs"
source /workspace/aeneas-tools/activate.sh
lake build
lake env lean Nonzero.lean 2>&1 | tee ../evidence/nonzero-axioms.log
python3 - <<'PY2'
from pathlib import Path
import re
s=Path('../evidence/nonzero-axioms.log').read_text()
rows=re.findall(r"'Nonzero\.(\w+)' depends on axioms: \[(.*?)\]",s,re.S)
assert len(rows)==2,rows
for name,axioms in rows:assert set(a.strip() for a in axioms.split(',')) <= {'propext','Classical.choice','Quot.sound'},(name,axioms)
print('Two nonzero preservation theorem axiom audits passed')
PY2
