#!/usr/bin/env python3
"""Reproduce isolated Verus model/resource results and semantic gap controls."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ARTIFACTS = HERE / 'artifacts'
VERUS = Path(os.environ.get('VERUS', '/workspace/bytes-verus-tools/verus-x86-linux/verus'))
env = os.environ.copy()
env['PATH'] = '/workspace/bytes-proof-tools/cargo/bin:' + env.get('PATH', '')
env['RUSTUP_HOME'] = env.get('VERUS_RUSTUP_HOME', '/workspace/bytes-verus-tools/rustup')
ARTIFACTS.mkdir(exist_ok=True)
CASES = [
    ('release-sequence-positive', 'release_sequence.rs', None, '5 verified, 0 errors'),
    ('retired-resources-positive', 'retired_resources.rs', None, '4 verified, 0 errors'),
    ('sc-counter-positive', 'sc_counter.rs', None, '1 verified, 0 errors'),
    ('retirement-state-machine', 'native_retirement.rs', None, '5 verified, 0 errors'),
    ('experimental-native-exclusive', 'native_bridge_boundary.rs', None, '1 verified, 0 errors'),
    ('native-syntax-only', 'native_orders.rs', None, '2 verified, 0 errors'),
    ('missing-acquire-negative', 'release_sequence.rs', 'negative_no_acquire', '5 verified, 1 errors'),
    ('broken-sequence-negative', 'release_sequence.rs', 'negative_broken_sequence', '5 verified, 1 errors'),
    ('missing-empty-ticket-negative', 'retired_resources.rs', 'negative_missing_ticket', '4 verified, 1 errors'),
    ('native-value-gap', 'native_orders.rs', 'negative_native_value', '2 verified, 1 errors'),
]
results = []
for name, source, cfg, expected in CASES:
    args = [str(VERUS), str(HERE / source), '--triggers-mode', 'silent']
    if cfg:
        args += ['--cfg', cfg]
    p = subprocess.run(args, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (ARTIFACTS / (name + '.log')).write_text(p.stdout)
    ok = expected in p.stdout and ((p.returncode != 0) == (cfg is not None))
    if cfg == 'negative_missing_ticket':
        ok &= 'precondition not satisfied' in p.stdout and 'self.pending.len() == 0' in p.stdout
    elif cfg:
        ok &= 'assertion failed' in p.stdout
    print(f'{name}: {"PASS" if ok else "FAIL"} ({expected})', flush=True)
    results.append(dict(name=name, source=source, cfg=cfg, returncode=p.returncode, expected=expected, checked=bool(ok)))
    if not ok:
        raise SystemExit(p.stdout)
# The first case is the real concurrency-integration rejection. The second
# intentionally overtrusts SC invariant compatibility and MUST NOT be adopted.
for name, cfg, expected, code in [
    ('weak-invariant-rejected', 'attempt_weak_invariant', 'open_atomic_invariant cannot contain non-atomic operations', 1),
    ('unsafe-sc-rule-diagnostic', 'rejected_sc_rule', '2 verified, 0 errors', 0),
]:
    args = [str(VERUS), str(HERE / 'native_bridge_boundary.rs'), '--triggers-mode', 'silent', '--cfg', cfg]
    p = subprocess.run(args, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (ARTIFACTS / (name + '.log')).write_text(p.stdout)
    ok = p.returncode == code and expected in p.stdout
    results.append(dict(name=name, source='native_bridge_boundary.rs', cfg=cfg, returncode=p.returncode, expected=expected, checked=ok, adopted=False))
    print(f'{name}: {"EXPECTED" if ok else "UNEXPECTED"}', flush=True)
    if not ok:
        raise SystemExit(p.stdout)
with tempfile.TemporaryDirectory(prefix='bytes-verus-native-') as out:
    binary = str(Path(out) / 'retired-resources')
    p = subprocess.run([str(VERUS), str(HERE / 'retired_resources.rs'), '--compile', '-o', binary, '--triggers-mode', 'silent'], env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (ARTIFACTS / 'retired-resources-compile.log').write_text(p.stdout)
    if p.returncode:
        raise SystemExit(p.stdout)
    subprocess.run([binary], check=True)
version = subprocess.run([str(VERUS), '--version'], env=env, text=True, capture_output=True, check=True).stdout
manifest = dict(verus_version=version.strip(), integrated_runtime=False,
    native_orders='Relaxed increment / Release decrement / Acquire final load; unchanged',
    results=results, sha256={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(HERE.glob('*.rs'))})
(ARTIFACTS / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print('compiled PCell resource-return caller: PASS')
