#!/usr/bin/env bash
set -euo pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export BYTES_TRUNCATE_PROOF_STUBS=1
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
printf 'Bytes truncate/clear non-promotable metadata subset: one prover; no sc-drf\n'
feature_tree=$(cargo tree --locked -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
cargo clean --package bytes-truncate-nonpromotable
rm -rf -- verif
cargo creusot --only=coma -- --locked
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
