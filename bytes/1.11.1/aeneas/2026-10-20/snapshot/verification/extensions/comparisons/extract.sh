#!/usr/bin/env bash
set -euo pipefail
ROOT=/workspace/bytes-aeneas-fresh
source "$ROOT/env.sh"
HARNESS="$ROOT/verification/harness"
OUT="$HARNESS/out"
mkdir -p "$OUT/lean"
python3 - <<'PY'
import hashlib, json, tomllib
from pathlib import Path
root=Path('/workspace/bytes-aeneas-fresh')
baseline=json.loads((root/'verification/opaque-baseline.json').read_text())
for name, expected in baseline['rust_hashes'].items():
    actual=hashlib.sha256((root/'bytes-adapted'/name).read_bytes()).hexdigest()
    assert actual == expected, f'Rust source changed: {name}'
metadata=tomllib.loads((root/'verification/harness/Cargo.toml').read_text())['package']['metadata']['charon']
assert sorted(metadata['opaque']) == sorted(baseline['opaque_patterns'])
assert len(metadata['opaque']) == 209
assert len(metadata['start_from']) == 50
PY
cd "$HARNESS"
export CARGO_TARGET_DIR="$ROOT/extraction-target"
charon cargo --preset=aeneas --sysroot=default --format=json \
  --no-dedup-serialized-ast --dest-file "$OUT/comparisons.llbc" \
  --include bytes -- --lib --locked --offline --target x86_64-unknown-linux-gnu \
  > "$OUT/comparisons.charon.log" 2>&1
aeneas -backend lean -dest "$OUT/lean" -filter-trait-methods "$OUT/comparisons.llbc" \
  > "$OUT/comparisons.aeneas.log" 2>&1
python3 - <<'PY'
import hashlib, json, tomllib
from pathlib import Path
root=Path('/workspace/bytes-aeneas-fresh')
harness=root/'verification/harness'
extension=root/'verification/extensions/comparisons'
raw=harness/'out/lean/Comparisons.lean'
text=raw.read_text()
needle='public import Aeneas\n'
assert text.count(needle)==1
(extension/'Generated.lean').write_text(text.replace(needle, needle+'public import FloatTypes\n', 1))
metadata=tomllib.loads((harness/'Cargo.toml').read_text())['package']['metadata']['charon']
opaque=json.loads((root/'verification/opaque-baseline.json').read_text())['opaque_patterns']
record={
  'roots':metadata['start_from'],
  'root_count':len(metadata['start_from']),
  'include':['bytes'],
  'opaque_pattern_count':len(opaque),
  'opaque_profile':'exact verification/opaque-baseline.json pattern set',
  'charon_command':['charon','cargo','--preset=aeneas','--sysroot=default','--format=json','--no-dedup-serialized-ast','--dest-file','verification/harness/out/comparisons.llbc','--include','bytes','--','--lib','--locked','--offline','--target','x86_64-unknown-linux-gnu'],
  'aeneas_command':['aeneas','-backend','lean','-dest','verification/harness/out/lean','-filter-trait-methods','verification/harness/out/comparisons.llbc'],
  'raw_generated':'verification/harness/out/lean/Comparisons.lean',
  'raw_sha256':hashlib.sha256(raw.read_bytes()).hexdigest(),
  'raw_llbc':'verification/harness/out/comparisons.llbc',
  'raw_llbc_sha256':hashlib.sha256((harness/'out/comparisons.llbc').read_bytes()).hexdigest(),
  'augmentation':'Add only `public import FloatTypes` after the generated Aeneas import; no generated body is edited.',
}
(extension/'extraction.json').write_text(json.dumps(record,indent=2)+'\n')
print(f"Updated {extension/'Generated.lean'} from raw sha256 {record['raw_sha256']}")
PY
