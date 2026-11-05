#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test 2>&1 | tee evidence/original-tests.log
find proofs/Generated -maxdepth 1 -type f -print0 | sort -z | xargs -0 sha256sum > evidence/generated-before.sha256
python3 tools/extract.py
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'CHECK'
from pathlib import Path
import re,tomllib
p=Path('.')
rows=re.findall(r"'Proofs\.(\w+)' depends on axioms: \[(.*?)\]",(p/'evidence/verification.log').read_text())
rows=list(dict.fromkeys(rows))
assert len(rows)==8,rows
for name,axioms in rows:assert set(axioms.split(', '))<={'propext','Classical.choice','Quot.sound'},(name,axioms)
tomllib.loads((p/'proofs.toml').read_text())
print('All eight theorem audits and report TOML parsing passed')
CHECK
