#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd "$script_dir/../../.." && pwd)
manifest="$script_dir/evidence/proof-targets.tsv"
coma_root="$script_dir/string"
run_stamp=$(date -u +%Y%m%dT%H%M%S%NZ)
run_dir="$script_dir/evidence/direct-why3-$run_stamp"
mkdir -p "$run_dir/logs" "$run_dir/coma" "$run_dir/inputs/harness" "$run_dir/inputs/string" "$run_dir/inputs/source"
cp "$manifest" "$script_dir/evidence/dependencies.tsv" "$script_dir/REPORT.md" "$script_dir/verify-proof.sh" "$script_dir/proof-child.sh" "$run_dir/inputs/"
cp "$script_dir/src/lib.rs" "$script_dir/Cargo.toml" "$run_dir/inputs/harness/"
cp "$script_dir/string/Cargo.toml" "$script_dir/string/Cargo.lock" "$run_dir/inputs/string/"
cp "$crate_dir/src/skip_spaces.rs" "$crate_dir/src/verification/spaces.rs" "$run_dir/inputs/source/"

target_paths=()
while IFS=$'\t' read -r target_path purpose expected_hash; do
  [[ "$target_path" == target_path ]] && continue
  if [[ -z "$target_path" || "$target_path" == *'..'* || ! "$expected_hash" =~ ^[0-9a-f]{64}$ || ! -s "$coma_root/$target_path" ]]; then
    printf 'Invalid or missing target row: %s\n' "$target_path" >&2
    exit 2
  fi
  actual_hash=$(sha256sum "$coma_root/$target_path" | cut -d' ' -f1)
  if [[ "$actual_hash" != "$expected_hash" ]]; then
    printf 'Fresh COMA hash mismatch for %s: expected %s, got %s\n' "$target_path" "$expected_hash" "$actual_hash" >&2
    exit 2
  fi
  target_paths+=("$target_path")
  mkdir -p "$run_dir/coma/$(dirname "$target_path")"
  cp "$coma_root/$target_path" "$run_dir/coma/$target_path"
done < "$manifest"
[[ ${#target_paths[@]} -eq 11 ]] || { printf 'Expected 11 selected targets, found %d\n' "${#target_paths[@]}" >&2; exit 2; }
printf 'run_dir=%s\nselected_targets=%d\n' "$run_dir" "${#target_paths[@]}" | tee "$run_dir/run-start.txt"

set +e
"$crate_dir/run-proof.bash" bash "$script_dir/proof-child.sh" "$script_dir" "$run_dir" "${target_paths[@]}" > "$run_dir/driver.log" 2>&1
run_status=$?
set -e
cat "$run_dir/driver.log"

: > "$run_dir/post-run-coma-sha256.txt"
while IFS=$'\t' read -r target_path purpose expected_hash; do
  [[ "$target_path" == target_path ]] && continue
  actual_hash=$(sha256sum "$coma_root/$target_path" | cut -d' ' -f1)
  printf '%s\t%s\t%s\n' "$actual_hash" "$expected_hash" "$target_path" >> "$run_dir/post-run-coma-sha256.txt"
  if [[ "$actual_hash" != "$expected_hash" ]]; then
    printf 'COMA changed during proof run: %s\n' "$target_path" >&2
    run_status=2
  fi
done < "$manifest"
if (( run_status == 0 )); then
  printf 'status=all-11-selected-target-files-valid\n' > "$run_dir/RESULT.txt"
  printf 'All selected files were Valid. Evidence: %s\n' "$run_dir"
else
  printf 'status=stopped-with-status-%d\n' "$run_status" > "$run_dir/RESULT.txt"
  printf 'Proof stopped with status %d; inspect %s/logs\n' "$run_status" "$run_dir" >&2
fi
exit "$run_status"
