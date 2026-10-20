#!/usr/bin/env bash
set -euo pipefail
source /workspace/bytes-aeneas-fresh/env.sh
cd /workspace/bytes-aeneas-fresh/verification
mkdir -p evidence
rm -f evidence/native-result.json
bash extract.sh > evidence/extraction.log 2>&1
lake build Generated AdaptersGenerated > evidence/generated-build.log 2>&1
lake env lean basic/Proofs.lean > evidence/basic-proof.log 2>&1
lake env lean adapters/Proofs.lean > evidence/adapter-proof.log 2>&1
python3 - <<'PY'
from pathlib import Path
import hashlib, json, re, subprocess
root = Path.cwd()
files = [root / 'Generated.lean', root / 'AdaptersGenerated.lean', root / 'FloatTypes.lean',
         root / 'basic/Proofs.lean', root / 'adapters/Proofs.lean', root / 'extract.sh',
         root / 'basic-extraction-roots.json', root / 'adapter-extraction-roots.json']
result = {
    'status': 'kernel_checked',
    'scope': 'named theorems in basic/Proofs.lean and adapters/Proofs.lean only',
    'lean_version': subprocess.check_output(['lean', '--version'], text=True).strip(),
    'proof_files': {},
    'axiom_audit_logs': ['evidence/basic-proof.log', 'evidence/adapter-proof.log'],
    'rust_source_hashes': {
        str(p.relative_to(root.parent / 'bytes-adapted')): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted((root.parent / 'bytes-adapted/src').rglob('*.rs'))
    },
    'cargo_manifest_sha256': hashlib.sha256((root.parent / 'bytes-adapted/Cargo.toml').read_bytes()).hexdigest(),
    'limitations': [
        'Theorems may mention opaque type declarations; see exact #print axioms logs.',
        'Adapter semantic theorems quantify successful inner Buf operations as explicit hypotheses.',
        'This receipt is not whole-crate functional verification or cargo-elicit audit acceptance.'
    ],
}
for name in ['basic-proof.log', 'adapter-proof.log']:
    audit = (root / 'evidence' / name).read_text()
    assert 'sorryAx' not in audit, f'Unproved assumption in {name}'
for p in files:
    text = p.read_text()
    result['proof_files'][str(p.relative_to(root))] = {
        'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
        'theorems': re.findall(r'^theorem\s+(\w+)', text, re.M),
    }
(root / 'evidence/native-result.json').write_text(json.dumps(result, indent=2) + '\n')
print('Native Lean checks passed; evidence/native-result.json saved.')
PY
