#!/usr/bin/env bash
set -u -o pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$XDG_CONFIG_HOME/creusot/why3.conf"
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

{
  printf '$ why3find prove --no-cache -X --time 1 --depth 6 --log-prover-results logs/ward-task-prover-results.json verif/bytes_vtable_shared_branch_probe_rlib/from_vec_shared_branch.coma\n'
  why3find prove --no-cache -X --time 1 --depth 6 \
    --log-prover-results logs/ward-task-prover-results.json \
    verif/bytes_vtable_shared_branch_probe_rlib/from_vec_shared_branch.coma
} >"$probe_root/logs/ward-task-export.log" 2>&1
proof_status=$?
printf 'exit status: %s\n' "$proof_status" >>"$probe_root/logs/ward-task-export.log"
printf '%s\n' "$proof_status" >"$probe_root/logs/ward-task-export-exit-status.txt"
exit "$proof_status"
