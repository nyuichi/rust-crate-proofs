#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../../.." && pwd)
cd "$package_dir"
python3 - "$script_dir" <<'PYCODE'
import hashlib, json, shutil, sys
from pathlib import Path
run=Path(sys.argv[1]); package=Path.cwd()
selection=json.loads((run/'target-selection.json').read_text())
root=package/'verif/http_header_map_api_proof_rlib/attempt7_get_mut_frame'
prepared=[]
for item in selection['comas']:
    source=run/item['archive']
    actual=hashlib.sha256(source.read_bytes()).hexdigest()
    if actual != item['sha256']:
        raise SystemExit(f"archived COMA hash mismatch: {source}: {actual}")
    target=root/item['target']
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(source,target)
    prepared.append(str(target.relative_to(package)))
(run/'prepared-targets.txt').write_text('\n'.join(prepared)+'\n')
print(f"prepared {len(prepared)} frozen COMAs under {root}")
PYCODE
