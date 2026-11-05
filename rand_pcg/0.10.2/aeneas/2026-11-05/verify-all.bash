#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
sha256sum --check evidence/upstream.sha256
cargo test --locked 2>&1 | tee evidence/tests.log
sha256sum proofs/{RandPcg.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py 2>&1 | tee evidence/extraction.log
sha256sum --check evidence/generated-before.sha256
python3 tools/check-serde-translation.py | tee evidence/serde-equivalence.log
if rg -n '\b(sorry|axiom)\b' proofs/Proofs.lean proofs/RandPcg.lean; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
import json,re,tomllib
assert not json.load(open('evidence/rand_pcg.llbc'))['has_errors']
s=open('evidence/verification.log').read()
for row in tomllib.load(open('proofs.toml','rb'))['tool']['proofs']:
 name=row['theorem']
 matches=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",s,re.S)
 assert matches,name
 for m in matches:assert {a.strip() for a in m.split(',')} <= {'propext','Classical.choice','Quot.sound'},m
for name in ['Proofs.xslOutput_spec','Proofs.dxsmOutput_spec']:
 matches=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",s,re.S)
 assert matches,name
 for m in matches:assert {a.strip() for a in m.split(',')} <= {'propext','Classical.choice','Quot.sound'},m
print('All twenty-one API theorem dependency audits passed')
AUDIT
