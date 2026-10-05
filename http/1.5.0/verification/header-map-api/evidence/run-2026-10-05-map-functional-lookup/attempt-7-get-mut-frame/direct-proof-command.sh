#!/usr/bin/env bash
set -euo pipefail
source /workspace/proof-tools/activate.sh
export DUNE_DIR_LOCATIONS='why3find:lib:/workspace/proof-tools/creusot-data/share/why3find'
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../../.." && pwd)
cd "$package_dir"
mapfile -t targets < <(python3 - "$script_dir/target-selection.json" <<'PY'
import json, pathlib, sys
run = pathlib.Path(sys.argv[1]).parent
selection = json.loads((run / 'target-selection.json').read_text())
for item in selection['comas']:
    print((run / item['archive']).relative_to(pathlib.Path.cwd()))
PY
)
exec ../../scripts/run-proof.sh /workspace/proof-tools/creusot-data/bin/why3find prove --root . --no-cache --show-progress always "${targets[@]}"
