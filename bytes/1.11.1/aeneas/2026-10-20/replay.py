#!/usr/bin/env python3
"""Kernel-check archived Lean in a new directory, preserving historical evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--aeneas-home', default=os.environ.get('AENEAS_HOME'),
                    help='Pinned Aeneas checkout with its Lean library/dependencies prepared')
parser.add_argument('--output', type=Path, help='A new, nonexistent directory for replay outputs')
args = parser.parse_args()
subprocess.run([sys.executable, str(ROOT / 'check.py')], check=True)
if not args.aeneas_home:
    parser.error('Set AENEAS_HOME or pass --aeneas-home; see README.md')
home = Path(args.aeneas_home).expanduser().resolve()
pins = json.loads((ROOT / 'snapshot/verification/toolchain.json').read_text())
revision = subprocess.check_output(['git', '-C', str(home), 'rev-parse', 'HEAD'], text=True).strip()
if revision != pins['aeneas']['commit']:
    parser.error('Aeneas checkout differs from the recorded commit')
subprocess.run(['git', '-C', str(home), 'diff', '--quiet', 'HEAD', '--', 'backends/lean'], check=True)
version = subprocess.check_output(['lean', '--version'], text=True).strip()
if 'version 4.35.0,' not in version:
    parser.error('Replay requires Lean 4.35.0 on PATH')
packages = home / 'backends/lean/.lake/packages'
if not packages.is_dir():
    parser.error('Prepare the pinned Aeneas Lean library and its Lake packages first')
if args.output:
    output = args.output.expanduser().resolve()
    if output.is_relative_to(ROOT):
        parser.error('Replay output must be outside the archived checkpoint')
    output.mkdir(parents=True, exist_ok=False)
else:
    output = Path(tempfile.mkdtemp(prefix='bytes-aeneas-replay-'))
verification = output / 'verification'
shutil.copytree(ROOT / 'snapshot/verification', verification)
for name in ['bytes-adapted', 'evidence']:
    (output / name).symlink_to(ROOT / 'snapshot' / name, target_is_directory=True)
(output / 'aeneas').symlink_to(home, target_is_directory=True)
(verification / '.lake').mkdir(exist_ok=True)
(verification / '.lake/packages').symlink_to(packages, target_is_directory=True)
commands = [
    ['lake', 'build', 'Generated', 'AdaptersGenerated'],
    ['lake', 'env', 'lean', 'basic/Proofs.lean'],
    ['lake', 'env', 'lean', 'adapters/Proofs.lean'],
    [sys.executable, 'verify-extensions.py'],
]
print(f'Replaying saved Lean in {output}', flush=True)
for index, command in enumerate(commands):
    log = output / f'command-{index + 1}.log'
    with log.open('w') as stream:
        result = subprocess.run(command, cwd=verification, stdout=stream, stderr=subprocess.STDOUT)
    if result.returncode:
        raise SystemExit(f'Replay failed ({result.returncode}); see {log}')
    if index in [1, 2]:
        audit = log.read_text()
        if 'sorryAx' in audit or '._native.' in audit:
            raise SystemExit(f'Unaccepted proof axiom; see {log}')
receipt = {
    'status': 'kernel_checked_saved_lean',
    'scope': 'Saved generated Lean and proofs; no fresh Rust/Charon/Aeneas extraction',
    'historical_theorem_claims': 272,
    'lean_version': version,
    'aeneas_commit': revision,
    'snapshot_manifest_sha256': hashlib.sha256((ROOT / 'snapshot-manifest.json').read_bytes()).hexdigest(),
    'commands': commands,
    'logs': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.glob('command-*.log'))},
}
(output / 'replay-result.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(f'Lean replay passed. New receipt: {output / "replay-result.json"}')
