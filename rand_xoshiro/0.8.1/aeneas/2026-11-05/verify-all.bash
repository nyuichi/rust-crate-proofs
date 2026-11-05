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
sha256sum proofs/XoshiroApiRoots/{Types.lean,Funs.lean,translation.json} > evidence/wrapper-generated-before.sha256
python3 tools/extract-wrapper.py | tee evidence/selection-wrapper/guard.log
sha256sum --check evidence/wrapper-generated-before.sha256
sha256sum proofs/Seed512/{Types.lean,Funs.lean,translation.json} > evidence/seed512-generated-before.sha256
python3 tools/extract-seed512.py | tee evidence/seed512-extraction.log
sha256sum --check evidence/seed512-generated-before.sha256
sha256sum proofs/SplitMixState/{Types.lean,Funs.lean,translation.json} > evidence/state-generated-before.sha256
python3 tools/extract-state.py | tee evidence/state-extraction.log
sha256sum --check evidence/state-generated-before.sha256
sha256sum proofs/Serializers/{Types.lean,Funs.lean,translation.json} > evidence/serializers-generated-before.sha256
python3 tools/extract-serializers.py | tee evidence/serializers-extraction.log
sha256sum --check evidence/serializers-generated-before.sha256
python3 tools/check-serializers.py | tee evidence/serializer-guard.log
if rg -n '\b(sorry|axiom)\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean; lake env lean WrapperProofs.lean; lake env lean SeedProofs.lean; lake env lean StateProofs.lean; lake env lean SerializerProofs.lean; lake env lean SerializerLoopProofs.lean) 2>&1 | tee evidence/verification.log
(cd proofs; printf 'import Proofs\nimport WrapperProofs\nimport SeedProofs\nimport StateProofs\nimport SerializerProofs\nimport SerializerLoopProofs\n' | lake env lean --stdin) 2>&1 | tee evidence/simultaneous-import.log
python3 tools/check-serializer-interface.py | tee evidence/serializer-interface.log
python3 - <<'AUDIT'
import re,tomllib
s=open('evidence/verification.log').read()
names=[]
for path in ['proofs/Proofs.lean','proofs/WrapperProofs.lean','proofs/SeedProofs.lean','proofs/StateProofs.lean','proofs/SerializerProofs.lean','proofs/SerializerLoopProofs.lean']:
 names+=re.findall(r'^#print axioms ([\w.]+)',open(path).read(),re.M)
assert len(names)==82,len(names)
for name in names:
 matches=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",s,re.S)
 if not matches:assert "'"+name+"' does not depend on any axioms" in s,name
 for m in matches:assert {a.strip() for a in m.split(',')} <= {'propext','Classical.choice','Quot.sound'},m
print('All thirty-five direct and nine compiler-inlined API theorem audits passed; 82 API/helper audits')
AUDIT
