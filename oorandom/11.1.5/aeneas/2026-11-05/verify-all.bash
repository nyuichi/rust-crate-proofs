#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
sha256sum --check evidence/upstream.sha256
cargo test --locked 2>&1 | tee evidence/tests.log
sha256sum proofs/Oorandom.lean proofs/translation.json > evidence/generated-before.sha256
CARGO_NET_OFFLINE=true charon cargo --preset=aeneas --sysroot default --start-from 'crate::Rand32::rand_u32' --dest-file evidence/oorandom.llbc 2>&1 | tee evidence/charon.log
aeneas -backend lean -emit-json -abort-on-error -dest proofs evidence/oorandom.llbc 2>&1 | tee evidence/aeneas.log
sha256sum --check evidence/generated-before.sha256
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'CHECK'
import json,re
assert json.load(open('evidence/oorandom.llbc'))['has_errors'] is False
text=open('evidence/verification.log').read()
m=re.search(r"'Proofs.rand_u32_spec' depends on axioms: \[(.*?)\]",text)
assert m, 'Missing transitive axiom audit'
assert set(m[1].split(', ')) <= {'propext','Classical.choice','Quot.sound'}
print('Transitive axiom audit passed')
CHECK
