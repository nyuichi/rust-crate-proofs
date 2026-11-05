#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
sha256sum --check evidence/upstream.sha256
cargo test --locked 2>&1 | tee evidence/tests.log
sha256sum proofs/{RandXoshiro.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py 2>&1 | tee evidence/extraction.log
sha256sum --check evidence/generated-before.sha256
python3 tools/check-serde-translation.py | tee evidence/serde-equivalence.log
if rg -n '\b(sorry|axiom)\b' proofs/Proofs.lean proofs/RandXoshiro.lean; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
import json,re,tomllib
assert not json.load(open('evidence/rand_xoshiro.llbc'))['has_errors']
s=open('evidence/verification.log').read()
names=re.findall(r'^#print axioms ([\w.]+)',open('proofs/Proofs.lean').read(),re.M)
assert len(names)==32,len(names)
for name in names:
 rows=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",s,re.S)
 if not rows:
  assert "'"+name+"' does not depend on any axioms" in s,name
 for row in rows:assert {a.strip() for a in row.split(',')} <= {'propext','Classical.choice','Quot.sound'},row
print('All sixteen API and sixteen representation theorem dependency audits passed')
AUDIT
