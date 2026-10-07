#!/usr/bin/env bash
set -u -o pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

for variant in baseline helper; do
  feature=$variant
  log="$probe_root/logs/$variant-translation.log"
  rm -rf -- "$probe_root/verif"
  cargo clean --package bytes-vtable-leaf-probe >/dev/null 2>&1
  {
    printf '$ BYTES_TRANSLATE_ONLY=1 cargo creusot --only=coma -- --locked --no-default-features --features %s\n' "$feature"
    BYTES_TRANSLATE_ONLY=1 cargo creusot --only=coma -- --locked --no-default-features --features "$feature"
  } >"$log" 2>&1
  status=$?
  printf 'exit status: %s\n' "$status" >>"$log"
  printf '%s\n' "$status" >"$probe_root/logs/$variant-exit-status.txt"
  if [[ -d "$probe_root/verif" ]]; then
    rm -rf -- "$probe_root/artifacts/$variant-verif"
    cp -a -- "$probe_root/verif" "$probe_root/artifacts/$variant-verif"
  fi
done
