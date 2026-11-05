#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
python3 tools/extract.py
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/additions-verification.log
python3 - <<'AUDIT'
from pathlib import Path
import re,tomllib
rows=re.findall(r"'Proofs\.(\w+)' depends on axioms: \[(.*?)\]",Path('evidence/additions-verification.log').read_text())
assert len({name for name,_ in rows})==10
for name,axioms in rows:assert set(axioms.split(', '))<={'propext','Classical.choice','Quot.sound'},(name,axioms)
assert len(tomllib.loads(Path('proofs.toml').read_text())['tool']['proofs'])==10
print('Ten theorem audits and report parsing passed')
AUDIT
