#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
sha256sum proofs/Generated/{EndianType.lean,translation.json} proofs/EndianVerificationDriver/{Types,Funs,FunsExternal_Template}.lean proofs/EndianVerificationDriver/translation.json proofs/{ModelProofs,FromProofs}.lean > evidence/generated-before.sha256
python3 tools/extract.py
python3 tools/extract-integers.py
python3 tools/provide-integer-models.py | tee evidence/integer-models/model-guard.log
python3 tools/generate-integer-model-proofs.py
python3 tools/generate-from-proofs.py
sha256sum --check evidence/generated-before.sha256
if rg -n '\b(sorry|axiom)\b' proofs --glob '*.lean' --glob '!**/.lake/**' --glob '!**/FunsExternal_Template.lean'; then exit 1; fi
(cd proofs; lake build +Proofs; lake env lean Proofs.lean; lake env lean ModelProofs.lean; lake env lean FromProofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
from pathlib import Path
import re,json
s=Path('evidence/verification.log').read_text();result={}
for namespace,expected in [('Proofs',6),('ModelProofs',50),('FromProofs',60)]:
 rows=re.findall(r"'"+namespace+r"\.(\w+)' depends on axioms: \[(.*?)\]",s)
 assert len({n for n,_ in rows})==expected,(namespace,len(rows))
 for name,axioms in rows:assert {a.strip() for a in axioms.split(',')} <= {'propext','Classical.choice','Quot.sound'},(name,axioms)
 result[namespace]=expected
Path('evidence/union-axiom-audit.json').write_text(json.dumps(result,indent=2)+'\n')
print('Union 66 original API contracts and 50 model specifications passed axiom audits')
AUDIT
