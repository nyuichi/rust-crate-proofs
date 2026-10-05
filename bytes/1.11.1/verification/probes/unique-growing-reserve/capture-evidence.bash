#!/usr/bin/env bash
# Keep proof output and its extracted source together under the shared lock.
set -euo pipefail
cd "$(dirname "$0")"
run=${1:?usage: capture-evidence.bash RUN [cargo feature arguments]}
shift
case "$run" in *[!a-zA-Z0-9_-]*|'') exit 2;; esac
source /workspace/bytes-proof-tools/activate.sh
exec 8>/tmp/itoa-creusot-proof.lock
flock 8
destination="$PWD/evidence/$run"
if test -e "$destination"; then
    echo "Refusing to overwrite existing evidence: $destination" >&2
    exit 2
fi
mkdir -p "$destination"
python3 evidence-manifest.py before "$destination"
runner=$(realpath ../../../scripts/verify-bytes.sh)
# Read the script once: concurrent edits must not change an executing script.
script=$(cat "$runner")
printf '%s\n' "$script" > "$destination/source/verify-bytes.sh"
python3 - "$runner" "$@" > "$destination/command.json" <<'PY'
import json,sys
print(json.dumps([sys.argv[1], "unique-growing-reserve", *sys.argv[2:]], indent=2))
PY
status=0
# The outer lock remains held until verification evidence has been copied.
# The inner standard wrapper still performs all feature/resource checks.
BYTES_PROOF_LOCK=/tmp/bytes-unique-growing-evidence-inner.lock \
    bash -c "$script" "$runner" unique-growing-reserve "$@" > "$destination/proof.log" 2>&1 || status=$?
printf '%s\n' "$status" > "$destination/exit-status.txt"
python3 evidence-manifest.py after "$destination"
tail -8 "$destination/proof.log"
exit "$status"
