#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

if ! grep -Fqx '**Full verification gate: CLOSED.**' VERIFICATION_STATUS.md; then
    printf '%s\n' 'Full verification gate is OPEN; refusing to report a green partial proof as complete.' >&2
    exit 2
fi

python3 - VERIFICATION_STATUS.md <<'PY'
from pathlib import Path
import sys

lines = Path(sys.argv[1]).read_text().splitlines()
inside = False
rows = []
for line in lines:
    if line.startswith("| Component |"):
        inside = True
        continue
    if not inside or not line.startswith("|"):
        continue
    cells = [part.strip() for part in line.strip().strip("|").split("|")]
    if not cells or cells[0].startswith("---"):
        continue
    rows.append(cells)

if not rows:
    raise SystemExit("No component rows found in the verification status ledger")
for row in rows:
    if len(row) != 6 or row[1:5] != ["yes", "yes", "yes", "yes"] or row[5] != "none":
        raise SystemExit("An open, unproved, unintegrated, trusted, or excluded component remains: " + row[0])
PY

if [[ "${HTTPARSE_PROOF_QUEUE_LOCKED:-0}" != 1 ]]; then
    exec "$script_dir/run-proof.bash" \
        env HTTPARSE_PROOF_QUEUE_LOCKED=1 "$0" "$@"
fi

export CARGO_NET_OFFLINE=true

# httparse's only Cargo feature is `std`, enabled by default. Prove the no_std
# parser first and the default std parser second. Creusot proof execution must
# be started with elevated sandbox permissions on the first attempt because
# Why3 opens a Unix-domain socket.
cargo creusot clean --force
cargo creusot prove -- --no-default-features
cargo creusot clean --force
cargo creusot prove
