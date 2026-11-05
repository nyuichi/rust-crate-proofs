#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
"$root/tools/regenerate.bash" > "$root/evidence/portable-regeneration.log" 2>&1
"$root/tools/verify-lean.bash" > "$root/evidence/whole-guarded.log" 2>&1
python3 - "$root" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1]);(p/'evidence/merged-proof-audit.log').write_text('\n'.join(f.read_text() for f in sorted((p/'evidence').glob('*-whole-bounded.log'))))
PY
python3 "$root/tools/source-guard.py"
python3 "$root/tools/trace-guard.py"
