#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>/tmp/itoa-creusot-proof.lock
flock 9
cargo clean --package bytes-shared-singleton-reclaim
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
