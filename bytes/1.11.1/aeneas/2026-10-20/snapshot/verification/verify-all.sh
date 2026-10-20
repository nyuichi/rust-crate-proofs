#!/usr/bin/env bash
set -euo pipefail
source /workspace/bytes-aeneas-fresh/env.sh
cd /workspace/bytes-aeneas-fresh/verification
bash verify-native.sh
if [[ "${1:-}" == "--reextract" ]]; then
  python3 - <<'PY'
import json, subprocess
from pathlib import Path
for manifest in sorted(Path('extensions').glob('*/manifest.json')):
    folder = manifest.parent
    if (folder/'extract.sh').exists():
        subprocess.run(['bash', str(folder/'extract.sh')], check=True)
        continue
    roots = json.loads((folder/'extraction.json').read_text())['roots']
    command = ['python3', 'extract-module.py', folder.name]
    for root in roots:
        command += ['--root', root]
    subprocess.run(command, check=True)
PY
fi
python3 verify-extensions.py
python3 coverage/build_coverage.py
python3 summarize-results.py
