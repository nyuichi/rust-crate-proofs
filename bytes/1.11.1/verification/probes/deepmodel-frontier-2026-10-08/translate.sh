#!/usr/bin/env bash
set -euo pipefail

probe_dir=$(cd "$(dirname "$0")" && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
cd "$probe_dir"
mkdir -p logs

exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

if [[ ! -f Cargo.lock ]]; then
    cargo generate-lockfile --offline
fi

case "${1:-}" in
    negative)
        cargo creusot --only=coma -- --locked --features missing_deep_model 2>&1 | tee logs/negative.log
        ;;
    positive)
        cargo creusot --only=coma -- --locked 2>&1 | tee logs/positive.log
        ;;
    model-gap)
        cargo creusot --only=coma -- --locked --features omit_rhs_model_bound 2>&1 | tee logs/model-gap.log
        ;;
    *)
        echo "usage: $0 negative|model-gap|positive" >&2
        exit 2
        ;;
esac
