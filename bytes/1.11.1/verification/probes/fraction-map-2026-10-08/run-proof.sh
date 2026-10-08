#!/usr/bin/env bash
# Invoke elevated: Why3 requires Unix-domain sockets. Uses one prover and a
# 1024 MiB limit, serialized with the other bytes proof jobs.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

task_config="$PWD/.proof-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"

cargo clean --package bytes-fraction-map
rm -rf verif
cargo creusot --only=coma -- --locked
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
