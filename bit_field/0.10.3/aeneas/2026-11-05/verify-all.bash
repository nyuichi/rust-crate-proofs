#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test --locked 2>&1 | tee evidence/original-tests.log
sha256sum proofs/BitField/{Types.lean,Funs.lean,FunsExternal_Template.lean,translation.json} proofs/BitArrayGenerated/{BitField.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py 2>&1 | tee evidence/extraction.log
sha256sum --check evidence/generated-before.sha256
if rg -n '\b(sorry|axiom)\b' proofs/Proofs.lean proofs/ArrayProofs.lean proofs/BitArrayGenerated/BitField.lean proofs/BitField/{Types,Funs,FunsExternal}.lean; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean; lake env lean ArrayProofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
import re,tomllib
from pathlib import Path
s=Path('evidence/verification.log').read_text()
for ty in ['u8','u16','u32','u64','u128','usize','i8','i16','i32','i64','i128','isize']:
 for name in [f'{ty}_get_bit_contract',f'{ty}_set_bit_contract',f'{ty}_length_spec']:
  rows=re.findall(r"'Proofs\."+name+r"' depends on axioms: \[(.*?)\]",s)
  assert rows,name
  for row in rows:assert set(row.split(', '))<={'propext','Classical.choice','Quot.sound'},(name,row)
rows=re.findall(r"'ArrayProofs\.bit_length_contract' depends on axioms: \[(.*?)\]",s)
assert rows
for row in rows:assert set(row.split(', '))<={'propext','Classical.choice','Quot.sound'},row
assert len(tomllib.loads(Path('proofs.toml').read_text())['tool']['proofs'])==25
print('24 total-index contracts and twelve width theorem audits passed')
AUDIT
