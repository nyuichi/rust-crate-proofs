#!/usr/bin/env bash
set -u -o pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
cd "$probe_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

python3 "$probe_root/check_source_fragments.py" >"$probe_root/logs/source-fragment-check.log" 2>&1
source_status=$?
printf 'exit status: %s\n' "$source_status" >>"$probe_root/logs/source-fragment-check.log"
printf '%s\n' "$source_status" >"$probe_root/logs/source-fragment-check-exit-status.txt"
if [[ $source_status -ne 0 ]]; then exit "$source_status"; fi

cargo clean --package bytes-vtable-shared-branch-probe >/dev/null 2>&1
rm -rf -- "$probe_root/verif"
{
  printf '$ cargo creusot --only=coma -- --locked\n'
  cargo creusot --only=coma -- --locked
} >"$probe_root/logs/translation.log" 2>&1
translation_status=$?
printf 'exit status: %s\n' "$translation_status" >>"$probe_root/logs/translation.log"
printf '%s\n' "$translation_status" >"$probe_root/logs/translation-exit-status.txt"
if [[ -d "$probe_root/verif" ]]; then
  rm -rf -- "$probe_root/artifacts/creusot-verif"
  cp -a -- "$probe_root/verif" "$probe_root/artifacts/creusot-verif"
fi
if [[ $translation_status -ne 0 ]]; then exit "$translation_status"; fi

{
  printf '$ cargo creusot clean --force\n'
  cargo creusot clean --force
} >"$probe_root/logs/creusot-clean.log" 2>&1
clean_status=$?
printf 'exit status: %s\n' "$clean_status" >>"$probe_root/logs/creusot-clean.log"
printf '%s\n' "$clean_status" >"$probe_root/logs/creusot-clean-exit-status.txt"
if [[ $clean_status -ne 0 ]]; then exit "$clean_status"; fi

{
  printf '$ cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 -- --locked\n'
  cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 -- --locked
} >"$probe_root/logs/why3-proof.log" 2>&1
proof_status=$?
printf 'exit status: %s\n' "$proof_status" >>"$probe_root/logs/why3-proof.log"
printf '%s\n' "$proof_status" >"$probe_root/logs/why3-proof-exit-status.txt"
if [[ -d "$probe_root/verif" ]]; then
  rm -rf -- "$probe_root/artifacts/proof-verif"
  cp -a -- "$probe_root/verif" "$probe_root/artifacts/proof-verif"
fi
if [[ -d "$probe_root/.why3find" ]]; then
  rm -rf -- "$probe_root/artifacts/why3find-cache"
  cp -a -- "$probe_root/.why3find" "$probe_root/artifacts/why3find-cache"
fi

{
  printf '$ cargo check --locked\n'
  cargo check --locked
} >"$probe_root/logs/native-check.log" 2>&1
native_status=$?
printf 'exit status: %s\n' "$native_status" >>"$probe_root/logs/native-check.log"
printf '%s\n' "$native_status" >"$probe_root/logs/native-check-exit-status.txt"
exit $(( proof_status != 0 || native_status != 0 ))
