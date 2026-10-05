#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
cargo generate-lockfile --offline
cargo clean --package bytes-shared-unsplit-mixed
rm -rf verif
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
