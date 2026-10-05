#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../.." && pwd)
cd "$package_dir"
python3 - <<'PY'
from hashlib import sha256
from pathlib import Path

source = Path("evidence/run-2026-10-05-map-append-value/comas/append_value.coma")
target = Path("verif/http_header_map_api_proof_rlib/header/map/append_value_resume.coma")
expected = "9e78217e81492788561261720835d27cce4ea7f8ee412abaa69c13cee9977f74"
actual = sha256(source.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit(f"unexpected archived COMA SHA-256: {actual}")
target.write_bytes(source.read_bytes())
print(f"prepared ignored target {target} ({actual})")
PY
