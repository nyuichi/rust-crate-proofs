#!/usr/bin/env bash
# Run the recovery-guard negative control under the serialized Why3 lock.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
task_config="$PWD/.proof-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' \
    "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"
feature_tree=$(cargo tree --locked -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
    printf 'sc-drf must be disabled\n' >&2
    exit 2
fi
python3 source_map.py
rm -rf -- verif
cargo clean -p bytes-shared-physical-lifecycle
cargo creusot --only=coma -- -F negative_source_no_acquire
exec cargo creusot source_adapter::impl_OriginalSharedHandle::release_one_observed \
    --only=prove --why3find-arg=-j --why3find-arg=1 -- -F negative_source_no_acquire
