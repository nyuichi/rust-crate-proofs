#!/usr/bin/env bash
# Run elevated: Why3 requires Unix-domain sockets.
set -euo pipefail
crate_root=$(cd "$(dirname "$0")/.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
cd "$crate_root"
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
python3 "$crate_root/scripts/prepare-verified-std.py"
# Separate configuration avoids mutating the shared tool setup.
task_config="$PWD/.verified-proof-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"
task_features=${BYTES_VERIFIED_FEATURES:-verified,std}
task_target=${BYTES_VERIFIED_TARGET:-}
IFS=, read -r -a selected_features <<< "$task_features"
contains_verified=0
for selected_feature in "${selected_features[@]}"; do
  case "$selected_feature" in
    verified) contains_verified=1 ;;
    alloc|std|serde|extra-platforms|creusot-std/alloc|creusot-std/std) ;;
    *) printf 'unsupported verification feature: %s\n' "$selected_feature" >&2; exit 2 ;;
  esac
done
if [[ "$contains_verified" != 1 ]]; then
  printf 'the modified entry requires verified\n' >&2
  exit 2
fi
target_args=()
build_std_args=()
if [[ -n "$task_target" ]]; then target_args=(--target "$task_target"); fi
if [[ "$task_target" == msp430-none-elf ]]; then build_std_args=(-Zbuild-std=core,alloc); fi
printf 'modified bytes 1.11.1: features=%s; target=%s; one prover; 1024 MiB; sc-drf disabled\n' "$task_features" "${task_target:-host}"
feature_tree=$(cargo tree --locked --no-default-features --features "$task_features" "${target_args[@]}" -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2
  exit 2
fi
# Preserve a pre-run source fingerprint; source mutation invalidates the result.
python3 - "$crate_root" "$task_config/source-before.json" <<'PY_SOURCE'
import hashlib, json, pathlib, sys
root=pathlib.Path(sys.argv[1])
paths=[root/'Cargo.toml',root/'Cargo.lock',*sorted((root/'src').rglob('*.rs'))]
pathlib.Path(sys.argv[2]).write_text(json.dumps({str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},sort_keys=True)+'\n')
PY_SOURCE
cargo clean --package bytes "${target_args[@]}"
rm -rf -- verif
cargo creusot --only=coma -- --locked --lib --no-default-features --features "$task_features" "${target_args[@]}" "${build_std_args[@]}"
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
python3 - "$crate_root" "$task_config/source-before.json" <<'PY_SOURCE'
import hashlib, json, pathlib, sys
root=pathlib.Path(sys.argv[1]); before=json.loads(pathlib.Path(sys.argv[2]).read_text())
after={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in before}
if before != after: raise SystemExit("source changed during proof; result invalid")
PY_SOURCE
