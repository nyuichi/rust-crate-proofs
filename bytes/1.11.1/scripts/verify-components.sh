#!/usr/bin/env bash
# Sequential, default-configuration proof entry point for isolated components.
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
probe_root="$repo_root/verification/probes"
log_root="$repo_root/verification/artifacts/logs"
collector="$repo_root/scripts/collect-component-evidence.py"
verifier="$repo_root/scripts/verify-bytes.sh"

targets=(
  helpers
  storage
  deallocation
  bounded-ops
  slice-ops
  cursor-ops
  byte-codecs
  comparison-ops
  chain-ops
  capacity-ops
  slice-read-ops
  slice-wide-read-ops
  variable-read-ops
  initialized-storage
  uninit-ops
  wide-codecs
  endian-ops
  signed-wide-ops
  region-permissions
)

usage() {
  printf 'Usage: %s [known-component ...]\n' "$0" >&2
  printf 'No arguments runs the full ordered component set. Explicit names run only that chosen subset.\n' >&2
  printf 'Known components: %s\n' "${targets[*]}" >&2
}

if (($#)); then
  requested=("$@")
  requested_seen=()
  for requested_target in "${requested[@]}"; do
    known=false
    for target in "${targets[@]}"; do
      if [[ "$requested_target" == "$target" ]]; then
        known=true
        break
      fi
    done
    if [[ "$known" != true ]]; then
      printf 'Unknown component target: %s\n' "$requested_target" >&2
      usage
      exit 2
    fi
    for seen_target in "${requested_seen[@]}"; do
      if [[ "$requested_target" == "$seen_target" ]]; then
        printf 'Duplicate component target: %s\n' "$requested_target" >&2
        usage
        exit 2
      fi
    done
    requested_seen+=("$requested_target")
  done

  selected_targets=()
  for target in "${targets[@]}"; do
    for requested_target in "${requested[@]}"; do
      if [[ "$target" == "$requested_target" ]]; then
        selected_targets+=("$target")
        break
      fi
    done
  done
else
  selected_targets=("${targets[@]}")
fi

mkdir -p "$log_root"

atomic_failure_log() {
  local target=$1
  local message=$2
  local log_path="$log_root/$target-proof.log"
  local temporary
  temporary=$(mktemp "$log_root/.$target-proof.log.XXXXXX")
  {
    printf 'component target: %s\n' "$target"
    printf '%s\n' "$message"
    printf 'component-result: FAIL target=%s exit_code=2 proved_files=0\n' "$target"
  } > "$temporary"
  mv -f -- "$temporary" "$log_path"
}

# Validate the whole ordered set before invoking any prover. Missing future
# probes must stop the entry point clearly instead of shrinking its scope.
missing=()
for target in "${selected_targets[@]}"; do
  target_dir="$probe_root/$target"
  absent=()
  for required in Cargo.toml Cargo.lock why3find.json src/lib.rs; do
    if [[ ! -f "$target_dir/$required" ]]; then
      absent+=("$required")
    fi
  done
  if [[ ! -d "$target_dir" ]]; then
    missing+=("$target (probe directory missing)")
  elif ((${#absent[@]})); then
    missing+=("$target (missing: ${absent[*]})")
  fi
done

# Match each token on the probe-routing case line, including non-final tokens.
for target in "${selected_targets[@]}"; do
  if ! grep -Eq "^[[:space:]]*([^)]*\\|)?${target}(\\|[^)]*)?\\) target=" "$verifier"; then
    missing+=("$target (not yet wired into scripts/verify-bytes.sh)")
  fi
done

if ((${#missing[@]})); then
  printf 'Component proof preflight failed; no prover was started.\n' >&2
  for item in "${missing[@]}"; do
    printf '  %s\n' "$item" >&2
    target=${item%% *}
    atomic_failure_log "$target" "preflight failed: $item; no prover was started"
  done
  exit 2
fi

for target in "${selected_targets[@]}"; do
  printf '\n== %s ==\n' "$target"
  log_path="$log_root/$target-proof.log"
  temporary_log=$(mktemp "$log_root/.$target-proof.log.XXXXXX")

  set +e
  bash "$verifier" "$target" > "$temporary_log" 2>&1
  command_status=$?
  set -e

  proved_line=$(grep -E '^Proved \([^)]+\) ✔$' "$temporary_log" | tail -n 1 || true)
  proved_files=''
  if [[ "$proved_line" =~ ^Proved\ \(([0-9]+)\ files?\)\ ✔$ ]]; then
    proved_files=${BASH_REMATCH[1]}
  elif [[ "$proved_line" =~ ^Proved\ \(verif/.+\.coma\)\ ✔$ ]]; then
    proved_coma=${proved_line#Proved (}
    proved_coma=${proved_coma%\) ✔}
    paired_proof="$probe_root/$target/${proved_coma%.coma}/proof.json"
    if [[ -f "$probe_root/$target/$proved_coma" && -f "$paired_proof" ]]; then
      proved_files=1
    fi
  fi
  outcome=PASS
  reason=''

  if ((command_status != 0)); then
    outcome=FAIL
    reason="verify-bytes.sh exited with status $command_status"
  elif [[ -z "$proved_files" || "$proved_files" -le 0 ]]; then
    outcome=FAIL
    command_status=1
    reason='verifier returned success without a nonzero Proved summary'
  fi

  if [[ "$outcome" == PASS ]]; then
    printf '\ncomponent-result: PASS target=%s exit_code=0 proved_files=%s\n' \
      "$target" "$proved_files" >> "$temporary_log"
  else
    printf '\ncomponent-result: FAIL target=%s exit_code=%s proved_files=%s reason=%s\n' \
      "$target" "$command_status" "${proved_files:-0}" "$reason" >> "$temporary_log"
  fi

  mv -f -- "$temporary_log" "$log_path"

  if [[ "$outcome" != PASS ]]; then
    printf 'Component proof failed for %s; see %s\n' "$target" "$log_path" >&2
    exit "$command_status"
  fi

  # Snapshot this target immediately so a later component failure does not
  # discard evidence for earlier successful components.
  if ! python3 "$collector" --target "$target"; then
    temporary_log=$(mktemp "$log_root/.$target-proof.log.XXXXXX")
    {
      cat "$log_path"
      printf '\ncomponent-result: FAIL target=%s exit_code=1 proved_files=%s reason=evidence-collection-failed\n' \
        "$target" "$proved_files"
    } > "$temporary_log"
    mv -f -- "$temporary_log" "$log_path"
    printf 'Evidence collection failed for %s; see %s\n' "$target" "$log_path" >&2
    exit 1
  fi
done

# The aggregate explicitly records the chosen set; a subset run never claims
# the full configured component set completed.
python3 "$collector" --targets "${selected_targets[@]}"
