#!/usr/bin/env bash
set -uo pipefail
source /workspace/bytes-aeneas-fresh/env.sh
name=$1
shift
probe_root=/workspace/bytes-aeneas-fresh
out="$probe_root/evidence/$name"
mkdir -p "$out"
cd "${BYTES_PROBE_CRATE:-$probe_root/bytes-1.11.1}"
export CARGO_TARGET_DIR="$probe_root/extraction-target"
cmd=(charon cargo --preset=aeneas --sysroot=default --format=json --no-dedup-serialized-ast --dest-file "$out/crate.llbc" "$@" -- --lib --locked --offline --target x86_64-unknown-linux-gnu)
printf '%q ' "${cmd[@]}" > "$out/charon-command.txt"
printf '\n' >> "$out/charon-command.txt"
"${cmd[@]}" > "$out/charon.log" 2>&1
rc=$?
printf '%s\n' "$rc" > "$out/charon.exit"
printf '%s: charon exit %s\n' "$name" "$rc"
if [[ "$rc" != 0 ]]; then exit "$rc"; fi
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
printf '%s: aeneas exit %s\n' "$name" "$rc"
exit "$rc"
