#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test --locked 2>&1 | tee evidence/original-tests.log
sha256sum proofs/BitField/{Types.lean,Funs.lean,FunsExternal_Template.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py 2>&1 | tee evidence/extraction.log
sha256sum --check evidence/generated-before.sha256
python3 tools/provide-u8-size-model.py | tee evidence/model-guard.log
if rg -n '\b(sorry|axiom)\b' proofs/Proofs.lean proofs/BitField/{Types,Funs,FunsExternal}.lean; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
import re
s=open('evidence/verification.log').read()
for name in ['get_bit_spec','set_bit_spec']:
 matches=re.findall(r"'Proofs\."+name+r"' depends on axioms: \[(.*?)\]",s,re.S)
 assert matches,name
 for match in matches:
  assert {a.strip() for a in match.split(',')} <= {'propext','Classical.choice','Quot.sound'},match
print('Both public theorem transitive axiom audits passed')
AUDIT
