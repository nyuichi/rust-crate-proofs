#!/usr/bin/env bash
set -euo pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
printf 'Bytes observer subset: one prover; 1024 MiB; no sc-drf\n'
feature_tree=$(cargo tree --locked -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
cargo clean --package bytes-observer-subset
rm -rf -- verif
export BYTES_PROOF_CALLBACK_STUBS=1
cargo creusot --only=coma -- --locked
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 -- --locked
