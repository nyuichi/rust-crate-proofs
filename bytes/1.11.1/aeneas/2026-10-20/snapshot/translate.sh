#!/usr/bin/env bash
set -uo pipefail
source /workspace/bytes-aeneas-fresh/env.sh
out="/workspace/bytes-aeneas-fresh/evidence/$1"
mkdir -p "$out/lean"
cmd=(aeneas -backend lean -dest "$out/lean" "$out/crate.llbc")
if [[ "${AENEAS_FILTER_TRAIT_METHODS:-0}" == 1 ]]; then
    cmd+=(-filter-trait-methods)
fi
printf '%q ' "${cmd[@]}" > "$out/aeneas-command.txt"
printf '\n' >> "$out/aeneas-command.txt"
"${cmd[@]}" > "$out/aeneas.log" 2>&1
rc=$?
printf '%s\n' "$rc" > "$out/aeneas.exit"
printf '%s: aeneas exit %s\n' "$1" "$rc"
exit "$rc"
