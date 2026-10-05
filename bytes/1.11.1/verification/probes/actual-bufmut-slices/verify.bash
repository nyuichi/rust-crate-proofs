#!/usr/bin/env bash
set -euo pipefail

probe_dir=$(cd "$(dirname "$0")" && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
cd "$probe_dir"

exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
cargo clean --package bytes-actual-bufmut-slices
rm -rf -- verif
cargo creusot --only=coma -- --locked
cargo creusot clean --force
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
