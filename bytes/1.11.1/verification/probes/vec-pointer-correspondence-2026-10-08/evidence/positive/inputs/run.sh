#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source "${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
case "${1:-positive}" in
 positive) extra=() ;;
 wrong-vec) extra=(--features wrong_vec) ;;
 *) exit 2 ;;
esac
cargo generate-lockfile --offline
cargo clean --package bytes-vec-pointer-correspondence
rm -rf -- verif
cargo creusot --only=coma -- --locked "${extra[@]}"
cargo creusot clean --force
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
