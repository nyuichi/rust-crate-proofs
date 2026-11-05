#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/aeneas-tools/activate.sh
python3 - <<'PY'
from pathlib import Path
import hashlib,tarfile
assert hashlib.sha256(Path('source/src/lib.rs').read_bytes()).hexdigest()==hashlib.sha256(Path('../../integer-slice-apis/source/src/lib.rs').read_bytes()).hexdigest()
assert Path('source/Cargo.toml').read_bytes()==Path('../../integer-slice-apis/source/Cargo.toml').read_bytes()
with tarfile.open('../../integer-slice-apis/evidence/upstream.crate') as t:
 for f in ('src/lib.rs','Cargo.toml'):
  assert Path('source',f).read_bytes()==t.extractfile('endian-type-0.2.0/'+f).read()
PY
CARGO_NET_OFFLINE=true charon cargo --preset aeneas --sysroot default --include endian_type --dest evidence > evidence/charon.log 2>&1
if aeneas -backend lean -abort-on-error -dest evidence evidence/endian_slice_extraction_probe.llbc > evidence/aeneas.log 2>&1; then
 echo 'Unexpected success: reassess recorded backend blocker'; exit 1
fi
python3 - <<'PY'
from pathlib import Path
s=Path('evidence/aeneas.log').read_text()
assert 'Unsupported operation: &raw const' in s
assert 'Unsupported operation: &raw mut' in s
assert 'Aeneas does not yet support dereferencing raw pointers.' in s
print('All six original generic bodies retain the recorded raw-pointer blockers; no runtime tests executed')
PY
