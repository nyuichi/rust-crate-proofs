#!/usr/bin/env bash
set -euo pipefail

probe_dir=$1
run_dir=$2
shift 2
source /workspace/httparse-tool-rebuild/string-model/creusot-env.sh
why3=$(command -v why3)
package="$CREUSOT_DATA_HOME/share/why3find/packages/creusot"
config="$run_dir/why3-effective.conf"
string_dir="$probe_dir/string-replay"

[[ -x "$why3" && -s "$config" && -s "$package/creusot/int.coma" \
  && -s "$package/creusot/prelude.coma" && -d "$string_dir/verif" ]]
grep -Eq '^running_provers_max = 1$' "$config"
grep -Eq '^memlimit = 1000$' "$config"

printf 'why3=%s\nwhy3_version=%s\nconfig=%s\nconfig_sha256=%s\n' \
  "$why3" "$($why3 --version)" "$config" "$(sha256sum "$config" | cut -d' ' -f1)" \
  >> "$run_dir/run-identity.txt"
printf 'package=%s\npackage_realpath=%s\n' "$package" "$(realpath -e "$package")" \
  >> "$run_dir/run-identity.txt"
printf 'prover=Z3,4.15.3\nper_goal_timelimit_seconds=30\nper_goal_memlimit_MiB=1000\nrunning_provers_max=1\n' \
  >> "$run_dir/run-identity.txt"

if [[ ! -s "$run_dir/results.tsv" ]]; then
  printf 'target_path\tgoal\tanswer\ttime\tsteps\n' > "$run_dir/results.tsv"
fi
index=${PROOF_START_INDEX:-0}
for target in "$@"; do
  index=$((index + 1))
  printf -v prefix '%02d' "$index"
  coma="$string_dir/verif/httparse_empty_lines_string_harness_rlib/$target.coma"
  log="$run_dir/logs/$prefix.log"
  printf 'target[%d]=%s\n' "$index" "$target"
  printf 'Direct command: %s prove --json -C %s -L %s -L %s -F coma -a split_vc -P Z3,4.15.3 -t 30 -m 1000 %s\n' \
    "$why3" "$config" "$package" "$string_dir/verif" "$coma"
  set +e
  "$why3" prove --json -C "$config" -L "$package" -L "$string_dir/verif" \
    -F coma -a split_vc -P 'Z3,4.15.3' -t 30 -m 1000 "$coma" > "$log" 2>&1
  why3_status=$?
  set -e

  if python3 - "$log" "$target" "$why3_status" "$run_dir/results.tsv" "$coma" <<'PY'
import json
import re
import sys
from pathlib import Path

log, target, exit_status, output, coma = sys.argv[1:]
text = Path(log).read_text(errors="replace")
start = text.find("{")
if start < 0:
    coma_text = Path(coma).read_text(errors="replace")
    goals = re.findall(r"^\s*goal\s+([^:\s]+):\s*(.*?)\s*$", coma_text, flags=re.MULTILINE)
    has_computation = re.search(r"^\s*let rec\b", coma_text, flags=re.MULTILINE) is not None
    if (
        target == "verification_empty_lines/complete_line_ending_at"
        and goals == [("vc_complete_line_ending_at", "true")]
        and not has_computation
        and int(exit_status) == 0
    ):
        with Path(output).open("a") as stream:
            stream.write(f"{target}\tvc_complete_line_ending_at\tTrivialTrueNoTask\t0\t0\n")
        print(
            "NO_TASK_TRUE: exact CoMa inspection found the sole goal "
            "vc_complete_line_ending_at: true and no computation body; Why3 "
            "returned no prover result."
        )
        raise SystemExit(0)
    print(f"No Why3 JSON in {log}: {text.strip()}")
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
        print(f"Cannot parse Why3 JSON in {log}: {error}")
        raise SystemExit(2)
    result = item.get("prover-result") if isinstance(item, dict) else None
    if result is not None:
        term = item.get("term") or {}
        rows.append((
            term.get("goal_name", "<unnamed>"),
            result.get("answer", "<missing>"),
            result.get("time", "<missing>"),
            result.get("step", "<missing>"),
        ))
if not rows:
    print(f"No named Why3 results for {target}; raw CoMa and log require inspection")
    raise SystemExit(2)
with Path(output).open("a") as stream:
    for goal, answer, elapsed, steps in rows:
        print(f"{target}\t{goal}\t{answer}\ttime={elapsed}\tsteps={steps}")
        stream.write(f"{target}\t{goal}\t{answer}\t{elapsed}\t{steps}\n")
failed = next((row for row in rows if row[1] != "Valid"), None)
if failed:
    print(f"FIRST_NON_VALID: target={target} goal={failed[0]} answer={failed[1]}")
    raise SystemExit(1)
if int(exit_status) != 0:
    print(f"Why3 exited with status {exit_status} after reported results")
    raise SystemExit(2)
PY
  then
    :
  else
    parse_status=$?
    printf 'STOP: first non-Valid or malformed target %s; raw log: %s\n' \
      "$target" "$log" >&2
    cat "$log"
    printf 'stopped_at_target=%s\nstopped_at_index=%d\n' "$target" "$index" \
      >> "$run_dir/run-identity.txt"
    exit "$parse_status"
  fi
done

printf 'all_selected_targets_processed=%d\n' "$index" >> "$run_dir/run-identity.txt"
