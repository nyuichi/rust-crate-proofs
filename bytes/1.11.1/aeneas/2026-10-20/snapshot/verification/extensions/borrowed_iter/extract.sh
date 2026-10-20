#!/usr/bin/env bash
set -euo pipefail
ROOT=/workspace/bytes-aeneas-fresh
source "$ROOT/env.sh"
HARNESS="$ROOT/verification/borrowed_iter_harness"
OUT="$ROOT/verification/extensions/borrowed_iter"
mkdir -p "$OUT/lean"
python3 - <<'PY'
import hashlib, json, tomllib
from pathlib import Path
root=Path('/workspace/bytes-aeneas-fresh')
baseline=json.loads((root/'verification/opaque-baseline.json').read_text())
for name, expected in baseline['rust_hashes'].items():
    actual=hashlib.sha256((root/'bytes-adapted'/name).read_bytes()).hexdigest()
    assert actual == expected, f'Rust source changed: {name}'
meta=tomllib.loads((root/'verification/borrowed_iter_harness/Cargo.toml').read_text())['package']['metadata']['charon']
assert sorted(meta['opaque']) == sorted(baseline['opaque_patterns'])
assert len(meta['opaque']) == 209
assert meta['start_from'] == [
  'bytes_borrowed_iter_harness::bytes_ref_iter',
  'bytes_borrowed_iter_harness::bytesmut_ref_iter',
]
PY
cd "$HARNESS"
export CARGO_TARGET_DIR="$ROOT/extraction-target"
charon cargo --preset=aeneas --sysroot=default --format=json \
  --no-dedup-serialized-ast --dest-file "$OUT/borrowed_iter.llbc" \
  --include bytes -- --lib --locked --offline --target x86_64-unknown-linux-gnu \
  > "$OUT/charon.log" 2>&1
aeneas -backend lean -dest "$OUT/lean" -filter-trait-methods \
  "$OUT/borrowed_iter.llbc" > "$OUT/aeneas.log" 2>&1
python3 - <<'PY'
import hashlib, json, tomllib
from pathlib import Path
root=Path('/workspace/bytes-aeneas-fresh')
out=root/'verification/extensions/borrowed_iter'
raw=out/'lean/BorrowedIter.lean'
text=raw.read_text()
needle='public import Aeneas\n'
assert text.count(needle)==1
(out/'Generated.lean').write_text(text.replace(needle, needle+'public import FloatTypes\n', 1))
meta=tomllib.loads((root/'verification/borrowed_iter_harness/Cargo.toml').read_text())['package']['metadata']['charon']
record={
  'roots':meta['start_from'], 'root_count':len(meta['start_from']),
  'include':['bytes'], 'opaque_pattern_count':209,
  'opaque_profile':'exact verification/opaque-baseline.json pattern set',
  'charon_command':['charon','cargo','--preset=aeneas','--sysroot=default','--format=json','--no-dedup-serialized-ast','--dest-file','verification/extensions/borrowed_iter/borrowed_iter.llbc','--include','bytes','--','--lib','--locked','--offline','--target','x86_64-unknown-linux-gnu'],
  'aeneas_command':['aeneas','-backend','lean','-dest','verification/extensions/borrowed_iter/lean','-filter-trait-methods','verification/extensions/borrowed_iter/borrowed_iter.llbc'],
  'raw_generated':str(raw.relative_to(root)), 'raw_sha256':hashlib.sha256(raw.read_bytes()).hexdigest(),
  'raw_llbc':'verification/extensions/borrowed_iter/borrowed_iter.llbc',
  'raw_llbc_sha256':hashlib.sha256((out/'borrowed_iter.llbc').read_bytes()).hexdigest(),
  'augmentation':'Add only `public import FloatTypes` after the generated Aeneas import; no generated body is edited.'
}
(out/'extraction.json').write_text(json.dumps(record,indent=2)+'\n')
PY
