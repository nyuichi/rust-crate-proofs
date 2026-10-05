#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
    printf 'Usage: %s COMMAND [ARG...]\n' "$0" >&2
    exit 2
fi

source /workspace/proof-tools/activate.sh
proof_lock_path=${CREUSOT_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}
exec 9>"$proof_lock_path"
flock 9

why3_config=${WHY3CONFIG:-$XDG_CONFIG_HOME/creusot/why3.conf}
if ! grep -Eq '^running_provers_max = 1$' "$why3_config" ||
   ! grep -Eq '^memlimit = 1000$' "$why3_config"; then
    printf 'Expected the coordinated Why3 profile (one prover, 1000 MiB): %s\n' "$why3_config" >&2
    exit 2
fi

export WHY3CONFIG="$why3_config"
export CARGO_NET_OFFLINE=true
printf 'Shared proof queue held; Why3 running_provers_max=1, memlimit=1000 MiB\n'
exec "$@"
