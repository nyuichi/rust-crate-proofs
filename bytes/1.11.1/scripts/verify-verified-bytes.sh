#!/usr/bin/env bash
# Run elevated: Why3 requires Unix-domain sockets.
set -euo pipefail
crate_root=$(cd "$(dirname "$0")/.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
cd "$crate_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
# Separate configuration avoids mutating the shared tool setup.
task_config="$PWD/.verified-proof-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"
printf 'modified bytes 1.11.1: verified,std; one prover; 1024 MiB; sc-drf disabled\n'
feature_tree=$(cargo tree --locked --no-default-features --features verified,std -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2
  exit 2
fi
cargo clean --package bytes
rm -rf -- verif
cargo creusot --only=coma -- --locked --lib --no-default-features --features verified,std
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
