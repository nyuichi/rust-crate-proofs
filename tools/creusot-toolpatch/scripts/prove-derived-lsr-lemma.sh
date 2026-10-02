#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
if [[ -z "${WHY3DATA:-}" ]]; then
  export WHY3DATA=$("$bundle_dir/scripts/prepare-why3-overlay.sh")
  toolpatch_temp_why3data=$WHY3DATA
  trap 'rm -rf "$toolpatch_temp_why3data"' EXIT
fi
why3_root=$WHY3DATA
why3_config=${WHY3CONFIG:-/tmp/why3-capture-config/creusot/why3.conf}
why3_bin=${WHY3_BIN:-/workspace/proof-tools/opamroot/creusot-v0.11/bin/why3}
if ! command -v "$why3_bin" >/dev/null 2>&1; then
  printf 'Why3 executable not found: %s (set WHY3_BIN to the pinned Why3 binary).\n' "$why3_bin" >&2
  exit 127
fi

proof_output=$("$why3_bin" prove -C "$why3_config" \
  -L "$why3_root/stdlib" \
  -P 'Z3,4.15.3' -t 10 \
  -a 'clear_but to_uint_lsr' \
  "$why3_root/stdlib/bv.mlw" \
  -T BV_Gen_Triggered -G to_uint_lsr_triggered 2>&1)
printf '%s\n' "$proof_output"
grep -F 'Prover result is: Valid' <<<"$proof_output" >/dev/null
