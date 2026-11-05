#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
sha256sum --check evidence/upstream.sha256
cargo test --locked 2>&1 | tee evidence/tests.log
sha256sum proofs/{Oorandom.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py 2>&1 | tee evidence/extraction.log
sha256sum --check evidence/generated-before.sha256
if rg -n '\b(sorry|axiom)\b' proofs/Proofs.lean proofs/Oorandom.lean; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
import json,re
assert not json.load(open('evidence/oorandom.llbc'))['has_errors']
s=open('evidence/verification.log').read()
for bits in ['32','64']:
 for name in ['rand_u'+bits,'rand_i'+bits,'state'+bits,'from_state'+bits,'restore'+bits,'new_inc'+bits,'new'+bits]:
  matches=re.findall(r"'Proofs\."+name+r"_spec' depends on axioms: \[(.*?)\]",s,re.S)
  assert matches,name
  for m in matches: assert {a.strip() for a in m.split(',')} <= {'propext','Classical.choice','Quot.sound'},m
print('All fourteen theorem dependency audits passed')
AUDIT
