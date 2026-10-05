#!/usr/bin/env bash
set -euo pipefail
restore_pwd=$PWD
trap 'cd "$restore_pwd"' EXIT
script_dir=$1
run_dir=$2
shift 2
source /workspace/scratch/httparse-string-model/creusot-env.sh

expected_config=/workspace/scratch/httparse-string-model/why3.conf
expected_env=/workspace/scratch/httparse-string-model/creusot-env.sh
expected_package="$CREUSOT_DATA_HOME/share/why3find/packages/creusot"
why3=$(command -v why3)
why3find="$CREUSOT_DATA_HOME/bin/why3find"
why3_base_requested="$CREUSOT_DATA_HOME/_opam/share/why3"
why3_base=$(realpath -e "$CREUSOT_DATA_HOME/_opam/share/why3")
stdlib="$why3_base/stdlib"
string_dir="$script_dir/string"

[[ "$WHY3CONFIG" == "$expected_config" && -x "$CREUSOT_RUSTC" && -x "$why3" && -x "$why3find" && -x "$CREUSOT_DATA_HOME/bin/z3" && -d "$stdlib" ]] || { echo 'Isolated compiler/config/Why3 base/Z3 preflight failed' >&2; exit 2; }
[[ "$why3" == /workspace/proof-tools/creusot-data/bin/why3 ]] || { printf 'Unexpected Why3 executable: %s\n' "$why3" >&2; exit 2; }
grep -Eq '^running_provers_max = 1$' "$WHY3CONFIG" && grep -Eq '^memlimit = 1000$' "$WHY3CONFIG" || { echo 'Expected isolated one-prover/1000 MiB config' >&2; exit 2; }
query=$($why3find query creusot)
package=$(printf '%s\n' "$query" | sed -n 's/^[[:space:]]*path: //p')
[[ "$package" == "$expected_package" && -s "$package/creusot/int.coma" && -s "$package/creusot/prelude.coma" ]] || { printf 'Unexpected/incomplete Creusot package: %s\n' "$package" >&2; exit 2; }
[[ "$(realpath -e "$package")" == "$(realpath -e "$expected_package")" ]] || { echo 'Creusot package resolves outside the isolated data tree' >&2; exit 2; }

# why3's effective default loadpath comes from the global binary prefix. Keep
# both original files intact and make a run-local config with the isolated stdlib.
why3_observed="$run_dir/why3-config-observed.conf"
why3_config="$run_dir/why3-effective.conf"
"$why3" config show > "$why3_observed"
python3 - "$why3_observed" "$why3_config" "$stdlib" <<'PY'
from pathlib import Path
import re
import sys

observed, effective, stdlib = map(Path, sys.argv[1:])
text = observed.read_text()
matches = re.findall(r'^loadpath = "([^"]+)"$', text, flags=re.MULTILINE)
if len(matches) != 1:
    raise SystemExit(f'expected one effective loadpath, got {matches!r}')
wanted = str(stdlib.resolve(strict=True))
old = f'loadpath = "{matches[0]}"'
new = f'loadpath = "{wanted}"'
if text.count(old) != 1:
    raise SystemExit('effective loadpath entry was not unique')
effective.write_text(text.replace(old, new, 1))
PY
[[ "$(realpath -e "$stdlib")" == "$(realpath -e "$why3_base_requested/stdlib")" ]] || { echo 'Why3 stdlib does not resolve to the active base data' >&2; exit 2; }
grep -Fq "$stdlib" "$why3_config" || { echo 'Run-local config does not contain isolated stdlib path' >&2; exit 2; }
config_loadpath=$(sed -n 's/^loadpath = "\([^"]*\)"$/\1/p' "$why3_config")
[[ -n "$config_loadpath" && "$(realpath -e "$config_loadpath")" == "$(realpath -e "$stdlib")" ]] || { echo 'Run-local config loadpath does not resolve to the active stdlib' >&2; exit 2; }

find "$stdlib" -type f -print0 | sort -z | xargs -0 sha256sum > "$run_dir/why3-stdlib-files.sha256"
find "$package" -type f -print0 | sort -z | xargs -0 sha256sum > "$run_dir/creusot-package-files.sha256"
{
  printf 'isolated_env=%s\n' "$expected_env"
  printf 'isolated_env_sha256=%s\n' "$(sha256sum "$expected_env" | cut -d' ' -f1)"
  printf 'isolated_config=%s\n' "$WHY3CONFIG"
  printf 'isolated_config_sha256=%s\n' "$(sha256sum "$WHY3CONFIG" | cut -d' ' -f1)"
  printf 'run_local_effective_config=%s\n' "$why3_config"
  printf 'run_local_config_sha256=%s\n' "$(sha256sum "$why3_config" | cut -d' ' -f1)"
  printf 'why3_cli=%s\nwhy3_version=%s\n' "$why3" "$($why3 --version)"
  printf 'isolated_creusot_rustc=%s\ncompiler_sha256=%s\n' "$CREUSOT_RUSTC" "$(sha256sum "$CREUSOT_RUSTC" | cut -d' ' -f1)"
  printf 'why3_base_data_requested=%s\nwhy3_base_data_resolved=%s\n' "$why3_base_requested" "$why3_base"
  printf 'why3_stdlib_resolved=%s\n' "$stdlib"
  printf 'stdlib_manifest_sha256=%s\n' "$(sha256sum "$run_dir/why3-stdlib-files.sha256" | cut -d' ' -f1)"
  printf 'creusot_package=%s\ncreusot_package_realpath=%s\n' "$package" "$(realpath -e "$package")"
  printf 'package_manifest_sha256=%s\n' "$(sha256sum "$run_dir/creusot-package-files.sha256" | cut -d' ' -f1)"
  printf 'z3=%s\nz3_sha256=%s\n' "$CREUSOT_DATA_HOME/bin/z3" "$(sha256sum "$CREUSOT_DATA_HOME/bin/z3" | cut -d' ' -f1)"
  printf 'prover=Z3,4.15.3\nper_goal_timelimit_seconds=30\nper_goal_memlimit_MiB=1000\nrunning_provers_max=1\n'
  printf 'why3find_query_creusot:\n%s\n' "$query"
} > "$run_dir/run-identity.txt"
"$CREUSOT_RUSTC" --version > "$run_dir/compiler-version.txt" 2>&1 || true

# Parse and type-check the exact selected closure before starting any prover.
: > "$run_dir/type-preflight.log"
for target in "$@"; do
  coma="$string_dir/$target"
  printf 'Type-only: %s\n' "$target" >> "$run_dir/type-preflight.log"
  if ! "$why3" prove --type-only -C "$why3_config" -L "$package" -L "$string_dir/verif" -F coma "$coma" >> "$run_dir/type-preflight.log" 2>&1; then
    cat "$run_dir/type-preflight.log"
    echo 'Stopped before solver: type-only preflight failed.' >&2
    exit 2
  fi
done

: > "$run_dir/results.tsv"
printf 'target_path\tgoal\tanswer\ttime\tsteps\n' > "$run_dir/results.tsv"
index=0
for target in "$@"; do
  index=$((index + 1))
  printf -v prefix '%02d' "$index"
  coma="$string_dir/$target"
  log="$run_dir/logs/$prefix.log"
  printf 'target[%d]=%s\n' "$index" "$target"
  printf 'Direct command: %s prove --json -C %s -L %s -L %s -F coma -a split_vc -P Z3,4.15.3 -t 30 -m 1000 %s\n' "$why3" "$why3_config" "$package" "$string_dir/verif" "$coma"
  set +e
  "$why3" prove --json -C "$why3_config" -L "$package" -L "$string_dir/verif" -F coma -a split_vc -P 'Z3,4.15.3' -t 30 -m 1000 "$coma" > "$log" 2>&1
  why3_status=$?
  set -e
  if python3 - "$log" "$target" "$why3_status" "$run_dir/results.tsv" <<'PY'
import json
import sys
from pathlib import Path

log, target, exit_status, output = sys.argv[1:]
text = Path(log).read_text(errors='replace')
start = text.find('{')
if start < 0:
    print(f'No Why3 JSON in {log}: {text.strip()}')
    raise SystemExit(2)
decoder = json.JSONDecoder()
position = start
rows = []
while position < len(text):
    while position < len(text) and text[position].isspace():
        position += 1
    if position == len(text):
        break
    try:
        item, position = decoder.raw_decode(text, position)
    except json.JSONDecodeError as error:
        print(f'Cannot parse Why3 JSON in {log}: {error}')
        raise SystemExit(2)
    result = item.get('prover-result') if isinstance(item, dict) else None
    if result is not None:
        term = item.get('term') or {}
        rows.append((term.get('goal_name', '<unnamed>'), result.get('answer', '<missing>'), result.get('time', '<missing>'), result.get('step', '<missing>')))
if not rows:
    print(f'No named Why3 results for {target}')
    raise SystemExit(2)
with Path(output).open('a') as stream:
    for goal, answer, elapsed, steps in rows:
        print(f'{target}\t{goal}\t{answer}\ttime={elapsed}\tsteps={steps}')
        stream.write(f'{target}\t{goal}\t{answer}\t{elapsed}\t{steps}\n')
failed = next((row for row in rows if row[1] != 'Valid'), None)
if failed:
    print(f'FIRST_NON_VALID: target={target} goal={failed[0]} answer={failed[1]}')
    raise SystemExit(1)
if int(exit_status) != 0:
    print(f'Why3 exited with status {exit_status} after reported results')
    raise SystemExit(2)
PY
  then
    :
  else
    result=$?
    printf 'STOP: first non-Valid or malformed target %s; raw log: %s\n' "$target" "$log" >&2
    cat "$log"
    exit "$result"
  fi
done
printf 'all_selected_targets_valid=%d\n' "$index" >> "$run_dir/run-identity.txt"
