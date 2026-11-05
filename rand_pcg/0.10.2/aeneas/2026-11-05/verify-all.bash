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
sha256sum proofs/PcgAdvance/{Types.lean,Funs.lean,translation.json} > evidence/advance-generated-before.sha256
python3 tools/extract-advance.py | tee evidence/advance-extraction.log
sha256sum --check evidence/advance-generated-before.sha256
sha256sum proofs/PcgAffineAdvance/{Types.lean,Funs.lean,translation.json} > evidence/affine-generated-before.sha256
python3 tools/extract-affine-advance.py | tee evidence/affine-extraction.log
sha256sum --check evidence/affine-generated-before.sha256
if rg -n '\b(sorry|axiom)\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then exit 1; fi
(cd proofs; lake build +Proofs; lake build +AdvanceProofs; lake build +AffineProofs; lake env lean Proofs.lean; lake env lean AdvanceProofs.lean; lake env lean Affine.lean; lake env lean AffineProofs.lean) 2>&1 | tee evidence/verification.log
(cd proofs; printf 'import Proofs\nimport AdvanceProofs\nimport AffineProofs\n' | lake env lean --stdin) 2>&1 | tee evidence/simultaneous-import.log
python3 - <<'AUDIT'
import json,re,tomllib
assert not json.load(open('evidence/rand_pcg.llbc'))['has_errors']
s=open('evidence/verification.log').read()
for row in tomllib.load(open('proofs.toml','rb'))['tool']['proofs']:
 name=row['theorem']
 matches=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",s,re.S)
 assert matches,name
 for m in matches:assert {a.strip() for a in m.split(',')} <= {'propext','Classical.choice','Quot.sound'},m
for name in ['Proofs.xslOutput_spec','Proofs.dxsmOutput_spec','AdvanceProofs.square_pow','AdvanceProofs.binary_pow','AdvanceProofs.body_zero_spec','AdvanceProofs.loop_pow_spec','AffineAdvance.mul_assoc','AffineAdvance.binary_power','AffineAdvance.apply_power','AffineProofs.body128_spec','AffineProofs.loop128_spec','AffineProofs.body64_spec','AffineProofs.loop64_spec']:
 matches=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",s,re.S)
 assert matches,name
 for m in matches:assert {a.strip() for a in m.split(',')} <= {'propext','Classical.choice','Quot.sound'},m
print('All twenty-five API and thirteen supporting theorem dependency audits passed')
AUDIT
