#!/usr/bin/env python3
"""Kernel-check isolated extractions against the frozen source/opaque boundary."""
import argparse, hashlib, json, re, subprocess, tomllib
from pathlib import Path
ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
parser = argparse.ArgumentParser()
parser.add_argument('groups', nargs='*')
args = parser.parse_args()
baseline = json.loads((ROOT/'opaque-baseline.json').read_text())
for name, expected in baseline['rust_hashes'].items():
    assert sha(BASE/'bytes-adapted'/name) == expected, f'Rust source changed: {name}'
metadata = tomllib.loads((BASE/'bytes-adapted/Cargo.toml').read_text())['package']['metadata']['charon']
assert sorted(metadata['opaque']) == sorted(baseline['opaque_patterns']), 'Opaque boundary changed'
groups = args.groups or [p.parent.name for p in sorted((ROOT/'extensions').glob('*/manifest.json'))]
assert groups, 'No proof manifests'
for group in groups:
    folder = ROOT/'extensions'/group
    receipt = folder/'receipt.json'
    receipt.unlink(missing_ok=True)
    manifest = json.loads((folder/'manifest.json').read_text())
    extraction = json.loads((folder/'extraction.json').read_text())
    raw = BASE/extraction['raw_generated']
    assert sha(raw) == extraction['raw_sha256']
    assert (folder/'Generated.lean').read_text() == raw.read_text().replace(
        'public import Aeneas\n', 'public import Aeneas\npublic import FloatTypes\n', 1), 'Generated implementation changed'
    theorems = sorted({p['theorem'] for p in manifest['proofs']})
    assert theorems
    commands = [['lake','build',f'extensions.{group}.Generated']]
    with (folder/'build.log').open('w') as log:
        subprocess.run(commands[0],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    audit = folder/'Audit.lean'
    audit.write_text((folder/'Proofs.lean').read_text() + '\n' + '\n'.join('#print axioms '+t for t in theorems)+'\n')
    commands.append(['lake','env','lean',str(audit.relative_to(ROOT))])
    with (folder/'audit.log').open('w') as log:
        subprocess.run(commands[-1],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    text = (folder/'audit.log').read_text()
    assert 'sorryAx' not in text
    assert '._native.' not in text, 'Native evaluation axiom is not an accepted proof step'
    audits = dict(re.findall(r"'([^']+)' depends on axioms: \[([^\]]*)\]",text,re.S))
    for t in theorems:
        assert t in audits or f"'{t}' does not depend on any axioms" in text, f'Missing audit: {t}'
    files = ['Generated.lean','Proofs.lean','manifest.json','extraction.json','Audit.lean','audit.log','build.log']
    result = {'status':'kernel_checked','group':group,'theorems':theorems,
      'axioms':{t:[a.strip() for a in audits.get(t,'').split(',') if a.strip()] for t in theorems},
      'files':{f:sha(folder/f) for f in files},'commands':commands,
      'source_boundary_sha256':sha(ROOT/'opaque-baseline.json'),
      'scope':'Named theorem claims and their explicit hypotheses only; not complete API or crate verification'}
    receipt.write_text(json.dumps(result,indent=2)+'\n')
    print(f'{group}: {len(theorems)} theorem claims kernel checked',flush=True)
