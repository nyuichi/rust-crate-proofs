#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    printf 'Usage: %s TARGET_NAME [--audit-config]\n' "$0" >&2
    exit 2
fi

target_name=$1
audit_only=false
if [[ ${2:-} == --audit-config ]]; then
    audit_only=true
elif [[ $# -eq 2 ]]; then
    printf 'Unknown option: %s\n' "$2" >&2
    exit 2
fi
case "$target_name" in
    signed_byte_value_facts|unsigned_max_lane_threshold|andnot_signbit_truth_table|\
    mask_complement_preserves_low_bits|movemask_narrow_preserves_bits|\
    u16_nth_constant_bit_smoke|u16_nth_out_of_bounds_smoke|u32_nth_boundary_smoke|\
    expected_invalid_*|trailing_zeros_shift_to_nth|prefix_len_from_mask|\
    uri_allowed_mask_16_sse|match_uri_char_16_sse_pure) ;;
    *)
        printf 'Target is not in the local nth-overlay allowlist: %s\n' "$target_name" >&2
        exit 2
        ;;
esac

source /workspace/proof-tools/activate.sh
driver_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
probe_dir=$(cd "$driver_dir/.." && pwd)
crate_dir=$(cd "$probe_dir/../../.." && pwd)
base_why3_config=${WHY3CONFIG:-$XDG_CONFIG_HOME/creusot/why3.conf}
z3_path=$CREUSOT_DATA_HOME/bin/z3
tmp_root=$(mktemp -d /tmp/httparse-nth-overlay.XXXXXX)
trap 'rm -rf "$tmp_root"' EXIT
mkdir -p "$tmp_root/creusot"
overlay_config=$tmp_root/creusot/why3.conf

python3 - "$base_why3_config" "$overlay_config" "$z3_path" "$driver_dir/z3_nth_overlay.drv" <<'PY'
import json
import pathlib
import re
import shlex
import sys

source, target, z3_path, driver_path = sys.argv[1:]
text = pathlib.Path(source).read_text()
for key, value in (("running_provers_max", "1"), ("memlimit", "1000")):
    text, count = re.subn(r"^" + key + r"\s*=.*$", f"{key} = {value}", text, flags=re.M)
    if count != 1:
        raise SystemExit(f"Expected exactly one {key} setting in {source}")
command = shlex.join([
    z3_path, "-smt2", "-T:%t", "sat.random_seed=42",
    "nlsat.randomize=false", "smt.random_seed=42", "-st", "%f",
])
command_steps = shlex.join([
    z3_path, "-smt2", "sat.random_seed=42", "nlsat.randomize=false",
    "smt.random_seed=42", "-st", "rlimit=%S", "%f",
])
text += (
    "\n[prover]\n"
    'name = "Z3-NthOverlay"\n'
    'version = "4.15.3"\n'
    f"command = {json.dumps(command)}\n"
    f"command_steps = {json.dumps(command_steps)}\n"
    f"driver = {json.dumps(driver_path)}\n"
)
pathlib.Path(target).write_text(text)
PY

export XDG_CONFIG_HOME=$tmp_root
export WHY3CONFIG=$overlay_config
if ! grep -Eq '^running_provers_max = 1$' "$WHY3CONFIG" ||
   ! grep -Eq '^memlimit = 1000$' "$WHY3CONFIG"; then
    printf 'Local config does not enforce the one-prover / 1000 MiB profile\n' >&2
    exit 2
fi
prover_listing=$(why3 -C "$WHY3CONFIG" config list-provers)
if ! grep -Fq 'Z3-NthOverlay' <<<"$prover_listing"; then
    printf 'Local Z3-NthOverlay prover was not registered\n' >&2
    exit 2
fi
if ! grep -Fq "driver = \"$driver_dir/z3_nth_overlay.drv\"" "$WHY3CONFIG"; then
    printf 'Local prover is not configured with the expected target-local driver\n' >&2
    exit 2
fi

printf 'Local proof config: %s\n' "$WHY3CONFIG"
printf 'Local config SHA-256: '
sha256sum "$WHY3CONFIG"
printf 'Base config SHA-256: '
sha256sum "$base_why3_config"
printf 'Local driver: %s\n' "$driver_dir/z3_nth_overlay.drv"
printf 'Prover profile: Z3-NthOverlay 4.15.3, 1 running prover, 1000 MiB\n'
printf 'Registered local prover: '
grep -F 'Z3-NthOverlay 4.15.3' <<<"$prover_listing"
printf 'Effective local prover stanza:\n'
awk '/^name = "Z3-NthOverlay"$/{show=1} show{print}' "$WHY3CONFIG"

coma_file=$probe_dir/verif/httparse_backend_sse_prefix_probe_rlib/sse_prefix/$target_name.coma
if [[ ! -f "$coma_file" ]]; then
    printf 'Generated Coma target is missing: %s\n' "$coma_file" >&2
    exit 2
fi
printf 'Coma target SHA-256: '
sha256sum "$coma_file"
printf 'Proof command: why3 -C %q prove -L %q -L %q -a split_vc -P Z3-NthOverlay -t 30 -m 1000 %q\n' \
    "$WHY3CONFIG" \
    "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
    "$probe_dir/verif" \
    "$coma_file"
if [[ "$audit_only" == true ]]; then
    printf 'Config audit only; no prover launched.\n'
    exit 0
fi

"$crate_dir/run-proof.bash" why3 -C "$WHY3CONFIG" prove \
    -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
    -L "$probe_dir/verif" \
    -a split_vc \
    -P Z3-NthOverlay \
    -t 30 -m 1000 \
    "$coma_file"
