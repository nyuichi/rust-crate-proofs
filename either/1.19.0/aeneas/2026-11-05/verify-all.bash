#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test 2>&1 | tee evidence/original-tests.log
sha256sum proofs/Generated/{Either.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then exit 1; fi
(cd proofs; lake build +BothProofs +CallbackProofs +DefaultProofs +IntoEitherProofs +MapInspectProofs +Proofs +ResultProofs) 2>&1 | tee evidence/verification.log
python3 - <<'INNER'
from pathlib import Path
import re
rows=re.findall(r"'((?:Proofs|CallbackProofs|IntoEitherProofs|ResultProofs|DefaultProofs|BothProofs|MapInspectProofs)\.\w+)' depends on axioms: \[(.*?)\]",Path('evidence/verification.log').read_text())
assert len({name for name, _ in rows})==114,rows
for n,a in rows:assert set(a.split(', '))<={'propext','Classical.choice','Quot.sound'},(n,a)
print('All one hundred fourteen theorem transitive axiom audits passed')
INNER

python3 tools/audit-all.py . evidence/machine-claims-audit
