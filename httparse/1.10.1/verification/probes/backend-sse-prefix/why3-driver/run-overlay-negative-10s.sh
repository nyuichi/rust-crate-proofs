#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    printf 'Usage: %s TARGET_NAME\n' "$0" >&2
    exit 2
fi

target_name=$1
case "$target_name" in
    expected_invalid_u16_wrong_high_bit|expected_invalid_u16_width_index_wrap|\
    expected_invalid_u32_negative_index_wrap|expected_invalid_u32_two_to_width_wrap) ;;
    *)
        printf 'Target is not in the 10-second negative allowlist: %s\n' "$target_name" >&2
        exit 2
        ;;
esac

source /workspace/proof-tools/activate.sh
driver_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
probe_dir=$(cd "$driver_dir/.." && pwd)
crate_dir=$(cd "$probe_dir/../../.." && pwd)
base_why3_config=${WHY3CONFIG:-$XDG_CONFIG_HOME/creusot/why3.conf}
z3_path=$CREUSOT_DATA_HOME/bin/z3
tmp_root=$(mktemp -d /tmp/httparse-nth-overlay-10s.XXXXXX)
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
printf 'Prover profile: Z3-NthOverlay 4.15.3, 1 running prover, 1000 MiB, 10 seconds per goal\n'
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
printf 'Proof command: why3 -C %q prove -L %q -L %q -a split_vc -P Z3-NthOverlay -t 10 -m 1000 --json %q\n' \
    "$WHY3CONFIG" \
    "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
    "$probe_dir/verif" \
    "$coma_file"

"$crate_dir/run-proof.bash" why3 -C "$WHY3CONFIG" prove \
    -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
    -L "$probe_dir/verif" \
    -a split_vc \
    -P Z3-NthOverlay \
    -t 10 -m 1000 --json \
    "$coma_file"
