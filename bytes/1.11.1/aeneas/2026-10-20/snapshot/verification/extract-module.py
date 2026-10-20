#!/usr/bin/env python3
"""Extract original implementation roots without changing the frozen boundaries."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path('/workspace/bytes-aeneas-fresh')
parser = argparse.ArgumentParser()
parser.add_argument('group')
parser.add_argument('--root', action='append', default=[])
parser.add_argument('--roots-file')
args = parser.parse_args()
roots = args.root
if args.roots_file:
    roots += json.loads(Path(args.roots_file).read_text())
assert roots, 'Declare actual Rust extraction roots'
assert args.group.replace('_', '').isalnum()
out = ROOT / 'verification/extensions' / args.group
out.mkdir(parents=True, exist_ok=True)
baseline = json.loads((ROOT / 'verification/opaque-baseline.json').read_text())
for name, expected in baseline['rust_hashes'].items():
    assert hashlib.sha256((ROOT / 'bytes-adapted' / name).read_bytes()).hexdigest() == expected
env = os.environ.copy()
env['BYTES_PROBE_CRATE'] = str(ROOT / 'bytes-adapted')
env['AENEAS_FILTER_TRAIT_METHODS'] = '1'
command = ['bash', str(ROOT / 'probe.sh'), 'extension-' + args.group]
for root in roots:
    command += ['--start-from', root]
subprocess.run(command, env=env, check=True)
raw = ROOT / 'evidence' / ('extension-' + args.group) / 'lean/Crate.lean'
text = raw.read_text()
assert text.count('public import Aeneas\n') == 1
(out / 'Generated.lean').write_text(text.replace('public import Aeneas\n',
    'public import Aeneas\npublic import FloatTypes\n', 1))
(out / 'extraction.json').write_text(json.dumps({
    'roots': roots, 'command': command, 'raw_generated': str(raw.relative_to(ROOT)),
    'raw_sha256': hashlib.sha256(raw.read_bytes()).hexdigest(),
    'augmentation': 'FloatTypes import only; no implementation edits or new opacity',
}, indent=2) + '\n')
print(out / 'Generated.lean')
