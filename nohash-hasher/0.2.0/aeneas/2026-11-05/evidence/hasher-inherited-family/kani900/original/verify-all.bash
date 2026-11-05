#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
mkdir -p evidence/no-std/{debug,release}
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test --locked 2>&1 | tee evidence/original-tests.log
cargo test --locked --release 2>&1 | tee evidence/release/original-tests.log
cargo test --locked --no-default-features --lib 2>&1 | tee evidence/no-std/debug/original-unit-tests.log
cargo test --locked --no-default-features --release --lib 2>&1 | tee evidence/no-std/release/original-unit-tests.log
sha256sum proofs/{Generated,ReleaseGenerated,NoStdGenerated,NoStdReleaseGenerated}/{NohashHasher.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py
python3 tools/extract.py --release
python3 tools/extract-no-std.py | tee evidence/no-std/generated-body-comparison.log
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean; lake env lean SupplementalProofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'CHECK'
from pathlib import Path
import re
s=Path('evidence/verification.log').read_text()
rows=re.findall(r"'Proofs\.(\w+)' depends on axioms: \[(.*?)\]",s)
assert len({name for name,_ in rows})==14,rows
rows += re.findall(r"'SupplementalProofs\.(clone_from_spec)' depends on axioms: \[(.*?)\]",s)
assert any(n=='clone_from_spec' for n,_ in rows)
for n,a in rows:assert set(a.split(', ')) <= {'propext','Classical.choice','Quot.sound'},(n,a)
print('Four-configuration main contracts and supplementary default-model composition axiom audits passed')
CHECK
