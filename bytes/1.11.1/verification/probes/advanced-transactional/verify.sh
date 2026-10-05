#!/usr/bin/env bash
set -euo pipefail

probe_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
crate_root=$(cd "$probe_dir/../../.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$tool_root/targets/bytes}
cd "$probe_dir"

exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
feature_tree=$(cargo tree --locked -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
    echo "sc-drf must be disabled" >&2
    exit 2
fi
package_name=$(cargo metadata --no-deps --locked --format-version 1 \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); assert len(d["packages"]) == 1; print(d["packages"][0]["name"])')
cargo clean --package "$package_name"
rm -rf -- verif
cargo creusot --only=coma -- --locked
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then
    echo "translation only: no proof phase requested"
    exit 0
fi

proof_patterns=()
if [[ -n "${BYTES_PROVE_PATTERN:-}" ]]; then
    read -r -a proof_patterns <<< "$BYTES_PROVE_PATTERN"
fi
cargo creusot --only=prove "${proof_patterns[@]}" --why3find-arg=-j --why3find-arg=1

mkdir -p evidence/positive
rm -rf evidence/positive/verif
cp -a verif evidence/positive/verif
