#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
python3 tools/check-source.py | tee evidence/source-provenance.log
CARGO_NET_OFFLINE=true cargo test 2>&1 | tee evidence/original-tests.log
sha256sum proofs/Generated/{EndianType.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py
sha256sum --check evidence/generated-before.sha256
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'CHECK'
from pathlib import Path
import re
rows=re.findall(r"'Proofs\.(\w+)' depends on axioms: \[(.*?)\]",Path('evidence/verification.log').read_text())
assert len(set(n for n,_ in rows))==6,rows
for n,a in rows: assert set(a.split(', ')) <= {'propext','Classical.choice','Quot.sound'},(n,a)
print('Six transitive axiom audits passed')
CHECK
