#!/usr/bin/env bash
# Run only after the production gate is captured; serialize with the shared Why3 lock.
set -euo pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
task_config="${TMPDIR:-/tmp}/bytes-encoding-spec-consumer-creusot-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' \
    "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"
feature_tree=$(cargo tree --locked --no-default-features -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2
  exit 2
fi
cargo clean --package bytes-encoding-spec-consumer
rm -rf -- verif
cargo creusot --only=coma -- --locked --lib
cargo creusot clean --force
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
