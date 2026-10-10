#!/usr/bin/env bash
# Default positive runtime witness only. Why3 requires elevated execution.
set -euo pipefail
crate_root=$(cd "$(dirname "$0")/.." && pwd)
if [[ "${1:-}" == --check && $# == 1 ]]; then
    exec python3 "$crate_root/scripts/check-current-proof.py"
fi
if [[ $# != 0 ]]; then
    printf 'Usage: verify-all.bash [--check]\n' >&2
    exit 2
fi
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
if [[ ! -f "$tool_root/activate.sh" ]]; then
    printf 'Missing bytes Creusot 0.13 toolchain: %s\nRun scripts/setup-bytes-toolchain.sh first.\n' "$tool_root" >&2
    exit 2
fi
source "$tool_root/activate.sh"
python3 "$crate_root/scripts/prepare-proof-std.py"
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
probe=$(python3 - "$crate_root" <<'PY'
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
print(root / json.loads((root/'verification/results/current.json').read_bytes())['probe'])
PY
)
cd "$probe"
mkdir -p .proof-config/creusot
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > .proof-config/creusot/why3.conf
export XDG_CONFIG_HOME="$PWD/.proof-config"
feature_tree=$(cargo tree --locked -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
    printf 'sc-drf must stay disabled\n' >&2
    exit 2
fi
cargo clean --package bytes-original-root-phase-clone
rm -rf -- verif
cargo creusot --only=coma -- --locked
cargo creusot clean --force
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
python3 "$crate_root/scripts/check-current-proof.py" --fresh
