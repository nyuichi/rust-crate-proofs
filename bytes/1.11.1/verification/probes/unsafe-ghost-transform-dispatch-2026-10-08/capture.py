#!/usr/bin/env python3
"""Capture only a completed, frozen probe run; never run a prover or edit inputs."""
import argparse
import hashlib
import json
import pathlib
import re
import shutil
import tarfile

parser = argparse.ArgumentParser()
parser.add_argument('label')
parser.add_argument('--proof-log', required=True)
parser.add_argument('--exit', type=int, required=True)
parser.add_argument('--kind', choices=['positive', 'negative', 'frontend'], required=True)
parser.add_argument('--features', default='')
parser.add_argument('--native-log')
parser.add_argument('--extra-log', action='append', default=[])
parser.add_argument('--expected-files', type=int)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parent
if not re.fullmatch(r'[a-zA-Z0-9_-]+', args.label):
    raise SystemExit('label must be a simple identifier')
out = root / 'evidence' / args.label
archive = out.with_suffix('.tar.gz')
if out.exists() or archive.exists():
    raise SystemExit('refuse to overwrite preserved evidence')
log_text = pathlib.Path(args.proof_log).read_text()

def nulls(value):
    if value is None:
        return 1
    if isinstance(value, dict):
        return sum(nulls(v) for v in value.values())
    if isinstance(value, list):
        return sum(nulls(v) for v in value)
    return 0

comas = sorted((root / 'verif').rglob('*.coma')) if args.kind != 'frontend' else []
proofs = sorted((root / 'verif').rglob('proof.json')) if args.kind != 'frontend' else []
null_count = sum(nulls(json.loads(f.read_text())) for f in proofs)
if args.expected_files is not None and len(comas) != args.expected_files:
    raise SystemExit(f'expected {args.expected_files} Coma files, found {len(comas)}')
if args.kind in ['positive', 'negative']:
    if not comas or len(comas) != len(proofs):
        raise SystemExit('incomplete Coma/proof JSON tree')
    if not all(f.with_suffix('').joinpath('proof.json').exists() for f in comas):
        raise SystemExit('missing per-Coma proof JSON')
if args.kind == 'positive':
    if args.exit != 0 or null_count != 0 or f'Proved ({len(comas)} files)' not in log_text:
        raise SystemExit('positive requires exit 0, completed engine summary, full tree, zero nulls')
if args.kind == 'negative':
    if args.exit == 0 or null_count == 0 or "'why3find prove' failed" not in log_text:
        raise SystemExit('negative requires completed failing engine output and null goals')
if args.kind == 'frontend' and args.exit == 0:
    raise SystemExit('frontend failure requires nonzero exit')

out.mkdir(parents=True)
for name in ['src', 'Cargo.toml', 'Cargo.lock', 'run-proof.sh', 'why3find.json', 'capture.py', 'README.md']:
    source, destination = root / name, out / name
    if source.is_dir():
        shutil.copytree(source, destination)
    else:
        shutil.copy2(source, destination)
shutil.copy2(args.proof_log, out / 'proof.log')
if args.native_log:
    shutil.copy2(args.native_log, out / 'native.log')
for index, source in enumerate(args.extra_log):
    shutil.copy2(source, out / f'extra-{index}-{pathlib.Path(source).name}')
if args.kind != 'frontend':
    shutil.copytree(root / 'verif', out / 'verif')
std = pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std')
for name in ['src/std/ops.rs', 'src/ghost.rs', 'src/ghost/fn_ghost.rs', 'src/ghost/resource.rs', 'src/logic/ra/excl.rs', 'src/logic/ra/update.rs', 'Cargo.toml']:
    destination = out / 'std-analogue' / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(std / name, destination)
compiler = pathlib.Path('/workspace/bytes-proof-tools/creusot-source')
for name in ['creusot/src/translation/function/terminator.rs', 'creusot/src/translation/function/statement.rs', 'creusot/src/backend/ty.rs', 'creusot/src/validate/purity.rs', 'creusot/src/validate/terminates.rs', 'creusot-std-proc/src/dummy.rs', 'cargo-creusot/src/main.rs']:
    destination = out / 'frontend-source' / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(compiler / name, destination)
config = root / '.proof-config/creusot/why3.conf'
if config.exists():
    shutil.copy2(config, out / 'why3.conf')
repo = root.parents[2]
for source in [repo / '.cargo/config.toml', repo / 'verification/std-support/manifest.json']:
    if source.exists():
        destination = out / 'support' / source.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
files = {str(f.relative_to(out)): hashlib.sha256(f.read_bytes()).hexdigest()
         for f in sorted(out.rglob('*')) if f.is_file()}
receipt = {
    'kind': args.kind, 'exit': args.exit, 'requested_features': args.features,
    'implicit_cargo_creusot_features': ['creusot-std/creusot', 'creusot-std/nightly'],
    'coma': len(comas), 'proof_json': len(proofs), 'recursive_nulls': null_count,
    'scope': 'Restricted unsafe native call with checked in-shim exclusive ghost update, under explicit generic erasure correspondence TCB. No Bytes protocol or general arbitrary-shim registration.',
    'source_notes': 'Copies exact installed analogue/frontend source; no assertion that the entire patched Std package equals upstream.',
    'sha256': files,
}
(out / 'manifest.json').write_text(json.dumps(receipt, indent=2) + '\n')
with tarfile.open(archive, 'w:gz') as stream:
    stream.add(out, arcname=args.label)
print(json.dumps({'archive': str(archive), 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(),
                  'coma': len(comas), 'proof_json': len(proofs), 'nulls': null_count}))
