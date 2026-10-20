#!/usr/bin/env bash
set -euo pipefail
source /workspace/bytes-aeneas-fresh/env.sh
export BYTES_PROBE_CRATE=/workspace/bytes-aeneas-fresh/bytes-adapted
export AENEAS_FILTER_TRAIT_METHODS=1
cd /workspace/bytes-aeneas-fresh
bash /workspace/bytes-aeneas-fresh/probe.sh extension-uninit \
  --start-from bytes::buf::uninit_slice \
  --exclude 'bytes::buf::uninit_slice::{impl core::convert::From<_> for _}'
python3 - <<'PY'
import hashlib, json
from pathlib import Path
root = Path('/workspace/bytes-aeneas-fresh')
folder = root / 'verification/extensions/uninit'
raw = root / 'evidence/extension-uninit/lean/Crate.lean'
text = raw.read_text()
assert text.count('public import Aeneas\n') == 1
(folder / 'Generated.lean').write_text(text.replace(
    'public import Aeneas\n', 'public import Aeneas\npublic import FloatTypes\n', 1))
cmd = [
    'bash', '/workspace/bytes-aeneas-fresh/probe.sh', 'extension-uninit',
    '--start-from', 'bytes::buf::uninit_slice',
    '--exclude', 'bytes::buf::uninit_slice::{impl core::convert::From<_> for _}',
]
(folder / 'extraction.json').write_text(json.dumps({
    'roots': ['bytes::buf::uninit_slice'],
    'scoped_exclusions': [
        'bytes::buf::uninit_slice::{impl core::convert::From<_> for _}',
    ],
    'command': cmd,
    'raw_generated': str(raw.relative_to(root)),
    'raw_sha256': hashlib.sha256(raw.read_bytes()).hexdigest(),
    'augmentation': 'FloatTypes import only; no implementation edits; frozen crate metadata opacity unchanged',
    'selection_note': 'Charon --start-from does not support trait-generic Index<Idx> impl methods; the module root was used, with only the two non-goal From forwarding impls excluded after they exposed the known &mut closure-triple Aeneas signature mismatch. The 12 Index/IndexMut bodies remain unchanged and transparent.',
}, indent=2) + '\n')
PY
