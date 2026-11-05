#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
tool_home="${AENEAS_HOME:-/workspace/aeneas-tools}"
if [[ -f "$tool_home/activate.sh" ]]; then source "$tool_home/activate.sh"; fi
sha256sum --check evidence/upstream.sha256
cargo test --locked 2>&1 | tee evidence/tests.log
sha256sum proofs/RandXorshift/Types.lean proofs/RandXorshift/Funs.lean \
  proofs/RandXorshift/FunsExternal_Template.lean proofs/RandXorshift/translation.json \
  > evidence/generated-before.sha256
charon cargo --preset=aeneas --sysroot default \
  --start-from '{impl rand_core::TryRng for crate::XorShiftRng}::try_next_u32' \
  --dest-file evidence/rand_xorshift.llbc 2>&1 | tee evidence/charon.log
aeneas -backend lean -emit-json -split-files -abort-on-error \
  -dest proofs/RandXorshift evidence/rand_xorshift.llbc 2>&1 | tee evidence/aeneas.log
sha256sum --check evidence/generated-before.sha256
python3 scripts/provide_std_models.py
if rg -n '\b(?:sorry|axiom)\b' proofs/RandXorshift/{Types,Funs,FunsExternal}.lean; then
  echo 'A proof dependency is unproved' >&2; exit 1
fi
(
  cd proofs
  lake build
  lake env lean Proofs.lean 2>&1 | tee ../evidence/axioms.log
) 2>&1 | tee evidence/verification.log
python3 - <<'PY'
from pathlib import Path
import json,re
assert json.loads(Path('evidence/rand_xorshift.llbc').read_text())['has_errors'] is False
log=Path('evidence/axioms.log').read_text()
m=re.search(r"'Proofs.try_next_u32_spec' depends on axioms: \[(.*?)\]",log)
assert m, 'No public theorem axiom audit result'
assert set(m[1].split(', ')) <= {'propext','Classical.choice','Quot.sound'}
print('Public theorem axiom audit passed')
PY
