#!/usr/bin/env bash
# Requires elevated execution for Why3 Unix-domain sockets.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
mkdir -p .proof-config/creusot
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > .proof-config/creusot/why3.conf
export XDG_CONFIG_HOME="$PWD/.proof-config"
cargo_args=()
if [[ $# -gt 0 ]]; then cargo_args=(--features "$1"); fi
feature_tree=$(cargo tree --locked -e features --edges normal,build "${cargo_args[@]}")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
rm -rf verif
cargo clean -p bytes-registered-dispatch-diagnostic
cargo creusot --only=coma -- "${cargo_args[@]}"
exec cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 -- "${cargo_args[@]}"
