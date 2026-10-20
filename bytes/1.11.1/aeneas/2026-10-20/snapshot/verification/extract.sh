#!/usr/bin/env bash
set -euo pipefail
source /workspace/bytes-aeneas-fresh/env.sh
export BYTES_PROBE_CRATE=/workspace/bytes-aeneas-fresh/bytes-adapted
export AENEAS_FILTER_TRAIT_METHODS=1
python3 - <<'PY'
import json, os, subprocess
from pathlib import Path
root = Path('/workspace/bytes-aeneas-fresh')
verification = root / 'verification'
def extract(name, args):
    subprocess.run(['bash', str(root / 'probe.sh'), name, *args], check=True)
    return root / 'evidence' / name / 'lean/Crate.lean'
getter_output = extract('verification-getters', [])
(verification / 'Generated.lean').write_bytes(getter_output.read_bytes())
args = []
for target in json.loads((verification / 'adapter-extraction-roots.json').read_text()):
    args += ['--start-from', target]
for name in ['get_f32', 'get_f32_le', 'get_f32_ne', 'get_f64', 'get_f64_le', 'get_f64_ne',
             'try_get_f32', 'try_get_f32_le', 'try_get_f32_ne',
             'try_get_f64', 'try_get_f64_le', 'try_get_f64_ne']:
    args += ['--exclude', 'bytes::buf::buf_impl::Buf::' + name]
adapter_output = extract('verification-adapters-float-excluded', args)
text = adapter_output.read_text()
needle = 'public import Aeneas\n'
assert text.count(needle) == 1
# Complete the missing type names only. Never rewrite generated implementations.
text = text.replace(needle, needle + 'public import FloatTypes\n', 1)
(verification / 'AdaptersGenerated.lean').write_text(text)
print('Regenerated proof slices from the current Rust sources.')
PY
