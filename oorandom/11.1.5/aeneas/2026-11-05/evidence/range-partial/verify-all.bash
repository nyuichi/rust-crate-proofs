#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
"$root/tools/regenerate.bash" > "$root/evidence/portable-regeneration.log" 2>&1
"$root/tools/verify-lean.bash" > "$root/evidence/final-coimport-audit.log" 2>&1
python3 "$root/tools/source-guard.py"
