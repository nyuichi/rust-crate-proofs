#!/usr/bin/env bash
# Proof phase uses Why3 sockets; invoke this wrapper outside the sandbox.
set -euo pipefail

probe_dir=$(cd "$(dirname "$0")" && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
cd "$probe_dir"

# Serialize with other proof work and discard stale VCs before regenerating.
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
cargo clean --package bytes-actual-uninit-slice
rm -rf -- verif
cargo creusot --only=coma -- --locked "$@"

if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then
  printf 'translation only: no proof phase requested\n'
  exit 0
fi

cargo creusot clean --force
prove_args=(--why3find-arg=-j --why3find-arg=1)
if [[ "${BYTES_WHY3_DIAGNOSTICS:-0}" == 1 ]]; then
  prove_args+=(--why3find-arg=-X --why3find-arg=-s --why3session)
fi
if [[ "${BYTES_DRY_RUN_WHY3FIND:-0}" == 1 ]]; then
  cargo creusot --only=prove --dry-run-why3find "${prove_args[@]}"
  exit 0
fi
cargo creusot --only=prove "${prove_args[@]}"
