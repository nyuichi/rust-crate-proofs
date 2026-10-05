#!/usr/bin/env bash
set -euo pipefail

fixture_dir=$(cd -- "$(dirname "$0")" && pwd)
translation=$(cat "$fixture_dir/translation.latest")
work_dir=$(awk -F '\t' '$1 == "work_dir" { print $2 }' "$translation/environment.tsv")
run_id=$(date -u +%Y%m%dT%H%M%SZ)-$$
evidence="$fixture_dir/evidence/proof-$run_id"
mkdir -p "$evidence/logs"
why3=$CREUSOT_DATA_HOME/bin/why3
package=$CREUSOT_DATA_HOME/share/why3find/packages/creusot

test -x "$why3"
test -s "$package/creusot/prelude.coma"
grep -Eq '^running_provers_max = 1$' "$WHY3CONFIG"
grep -Eq '^memlimit = 1000$' "$WHY3CONFIG"
printf 'translation_evidence\t%s\nwork_dir\t%s\nwhy3\t%s\nwhy3_version\t%s\nconfig\t%s\nconfig_sha256\t%s\n' \
  "$translation" "$work_dir" "$why3" "$($why3 --version)" "$WHY3CONFIG" \
  "$(sha256sum "$WHY3CONFIG" | awk '{print $1}')" > "$evidence/run-identity.tsv"
printf 'prover\tZ3,4.15.3\nper_goal_timelimit_seconds\t30\nper_goal_memlimit_MiB\t1000\nrunning_provers_max\t1\n' \
  >> "$evidence/run-identity.tsv"
cp "$translation/coma/get_pattern.coma" "$translation/coma/reject_false_contract.coma" "$evidence/"
sha256sum "$translation/coma/get_pattern.coma" "$translation/coma/reject_false_contract.coma" \
  > "$evidence/coma.sha256"
printf 'target_path\tgoal\tanswer\ttime\tsteps\n' > "$evidence/results.tsv"

prove_target() {
  local target=$1 log=$2 coma="$translation/coma/$1.coma"
  printf 'Selected command: %s prove --json -C %s -L %s -L %s -F coma -a split_vc -P Z3,4.15.3 -t 30 -m 1000 %s\n' \
    "$why3" "$WHY3CONFIG" "$package" "$work_dir/verif" "$coma" >> "$evidence/commands.txt"
  set +e
  "$why3" prove --json -C "$WHY3CONFIG" -L "$package" -L "$work_dir/verif" \
    -F coma -a split_vc -P 'Z3,4.15.3' -t 30 -m 1000 "$coma" > "$log" 2>&1
  local status=$?
  set -e
  python3 - "$log" "$target" "$status" "$evidence/results.tsv" <<'PY'
import json
import sys
from pathlib import Path

log_path, target, exit_code, out_path = sys.argv[1:]
text = Path(log_path).read_text(errors="replace")
position = text.find("{")
if position < 0:
    print(f"No JSON result in {log_path}; inspect raw log")
    raise SystemExit(2)
decoder = json.JSONDecoder()
rows = []
while position < len(text):
    while position < len(text) and text[position].isspace():
        position += 1
    if position >= len(text):
        break
    try:
        item, position = decoder.raw_decode(text, position)
    except json.JSONDecodeError as error:
        print(f"Malformed JSON in {log_path}: {error}")
        raise SystemExit(2)
    result = item.get("prover-result") if isinstance(item, dict) else None
    if result is not None:
        term = item.get("term") or {}
        rows.append((term.get("goal_name", "<unnamed>"), result.get("answer", "<missing>"),
                     result.get("time", "<missing>"), result.get("step", "<missing>")))
if not rows:
    print(f"No named Why3 result for {target}; inspect raw log")
    raise SystemExit(2)
with Path(out_path).open("a") as out:
    for goal, answer, elapsed, steps in rows:
        print(f"{target}\t{goal}\t{answer}\t{elapsed}\t{steps}")
        out.write(f"{target}\t{goal}\t{answer}\t{elapsed}\t{steps}\n")
if int(exit_code) != 0 and all(row[1] == "Valid" for row in rows):
    print(f"Why3 exit code {exit_code} despite all reported results Valid")
    raise SystemExit(2)
sys.exit(0 if all(row[1] == "Valid" for row in rows) else 1)
PY
  return $?
}

printf 'positive_target=get_pattern\n' >> "$evidence/run-identity.tsv"
if ! prove_target get_pattern "$evidence/logs/01-get-pattern.log"; then
  printf 'stopped_at=get_pattern; positive target had a non-Valid or malformed result\n' \
    >> "$evidence/run-identity.tsv"
  printf '%s\n' "$evidence" > "$fixture_dir/proof.latest"
  exit 1
fi

printf 'negative_target=reject_false_contract\n' >> "$evidence/run-identity.tsv"
set +e
prove_target reject_false_contract "$evidence/logs/02-reject-false-contract.log"
negative_status=$?
set -e
if (( negative_status == 0 )); then
  printf 'negative_control_result=unexpectedly_all_valid\n' >> "$evidence/run-identity.tsv"
  printf '%s\n' "$evidence" > "$fixture_dir/proof.latest"
  exit 1
elif (( negative_status != 1 )); then
  printf 'negative_control_result=malformed_or_tool_error\n' >> "$evidence/run-identity.tsv"
  printf '%s\n' "$evidence" > "$fixture_dir/proof.latest"
  exit 2
fi
if ! awk -F '\t' '$1 == "reject_false_contract" && ($3 == "Invalid" || $3 == "Sat") { found=1 } END { exit !found }' "$evidence/results.tsv"; then
  printf 'negative_control_result=non-valid_but_not_counterexample\n' >> "$evidence/run-identity.tsv"
  printf '%s\n' "$evidence" > "$fixture_dir/proof.latest"
  exit 1
fi
printf 'negative_control_result=counterexample_detected\n' >> "$evidence/run-identity.tsv"
printf '%s\n' "$evidence" > "$fixture_dir/proof.latest"
printf 'proof evidence: %s\n' "$evidence"
