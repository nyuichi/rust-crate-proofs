#!/usr/bin/env bash
# Invoke with elevated execution: Why3 requires Unix-domain sockets.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
# Keep the shared tool configuration unchanged while enforcing this gate's budget.
task_config="$PWD/.proof-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"
printf 'T02: one prover; 1024 MiB; native weak orderings; sc-drf disabled\n'
feature_tree=$(cargo tree --locked -e features --edges normal,build "$@")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2
  exit 2
fi
cargo clean --package bytes-architecture-thread-objective
rm -rf verif
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
