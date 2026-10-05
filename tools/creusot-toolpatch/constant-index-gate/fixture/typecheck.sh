#!/usr/bin/env bash
set -euo pipefail

fixture_dir=$(cd -- "$(dirname "$0")" && pwd)
profile_dir=${HTTPARSE_TOOL_PROFILE:-/workspace/httparse-tool-rebuild/string-model}
source /workspace/proof-tools/activate.sh
source "$profile_dir/creusot-env.sh"
evidence=$(cat "$fixture_dir/translation.latest")
work_dir=$(awk -F '\t' '$1 == "work_dir" { print $2 }' "$evidence/environment.tsv")
type_run="$evidence/type-only"
mkdir -p "$type_run/coma"
mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$fixture_dir/proof-targets.txt")
files=()
for target in "${targets[@]}"; do
  test -f "$work_dir/$target"
  files+=("$work_dir/$target")
  cp "$work_dir/$target" "$type_run/coma/"
done
sha256sum "$WHY3CONFIG" \
  "$CREUSOT_DATA_HOME/share/why3find/packages/creusot/creusot/prelude.coma" \
  "$CREUSOT_RUSTC" > "$type_run/tool-inputs.sha256"
printf 'why3 prove --type-only -C %s -L %s -L %s %s\n' \
  "$WHY3CONFIG" \
  "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
  "$work_dir/verif" "${files[*]}" > "$type_run/command.txt"
set +e
"$CREUSOT_DATA_HOME/bin/why3" prove --type-only -C "$WHY3CONFIG" \
  -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
  -L "$work_dir/verif" "${files[@]}" \
  > "$type_run/stdout.log" 2> "$type_run/stderr.log"
status=$?
set -e
printf '%s\n' "$status" > "$type_run/exit-code.txt"
find "$work_dir/verif" -type f -name '*.coma' -printf '%P\n' | sort \
  > "$type_run/all-generated-targets.txt"
printf 'type-only status: %s\n' "$status"
exit "$status"
