#!/usr/bin/env bash
set -u -o pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
crate_root=$(cd "$probe_root/../../../.." && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

python3 "$probe_root/check_source_fragments.py" >"$probe_root/logs/source-fragment-check.log" 2>&1
source_status=$?
printf 'exit status: %s\n' "$source_status" >>"$probe_root/logs/source-fragment-check.log"
if [[ $source_status -ne 0 ]]; then exit "$source_status"; fi

cargo clean --package bytes-vtable-shared-branch-probe >/dev/null 2>&1
rm -rf -- "$probe_root/verif"
{
  printf '$ BYTES_TRANSLATE_ONLY=1 cargo creusot --only=coma -- --locked\n'
  BYTES_TRANSLATE_ONLY=1 cargo creusot --only=coma -- --locked
} >"$probe_root/logs/proof-translation-pointer-addr.log" 2>&1
proof_status=$?
printf 'exit status: %s\n' "$proof_status" >>"$probe_root/logs/proof-translation-pointer-addr.log"
printf '%s\n' "$proof_status" >"$probe_root/logs/proof-pointer-addr-exit-status.txt"
if [[ -d "$probe_root/verif" ]]; then
  rm -rf -- "$probe_root/artifacts/proof-pointer-addr-verif"
  cp -a -- "$probe_root/verif" "$probe_root/artifacts/proof-pointer-addr-verif"
fi
rm -rf -- "$probe_root/verif"

cargo clean --package bytes-vtable-shared-branch-probe >/dev/null 2>&1
{
  printf '$ cargo check --locked\n'
  cargo check --locked
} >"$probe_root/logs/native-check.log" 2>&1
native_status=$?
printf 'exit status: %s\n' "$native_status" >>"$probe_root/logs/native-check.log"
printf '%s\n' "$native_status" >"$probe_root/logs/native-exit-status.txt"
exit $(( proof_status != 0 || native_status != 0 ))
