#!/usr/bin/env bash
# Proof phases require elevated execution: Why3 uses Unix-domain sockets.
set -euo pipefail
crate_root=$(cd "$(dirname "$0")/.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
# Historical adapters are not valid current-source proof targets after cleanup.
python3 - "$crate_root/verification/retired-probes.json" "${1:-runtime}" <<'PY_RETIRED'
import json, pathlib, sys
retired = json.loads(pathlib.Path(sys.argv[1]).read_text())["retired"]
if sys.argv[2] in retired:
    raise SystemExit("Retired proof adapter: " + sys.argv[2] + ". See verification/REMOVAL_2026-10-07_JA.md; saved evidence is historical.")
PY_RETIRED
source "$tool_root/activate.sh"
python3 "$crate_root/scripts/prepare-proof-std.py"
export CARGO_NET_OFFLINE=true
case "${1:-runtime}" in
  actual-shared-try-reclaim|actual-bufmut-slices|shared-unsplit-mixed|shared-unsplit-fallbacks|shared-unsplit-independent|actual-public-buf-default|default-readonly|actual-shared-reserve|actual-unique-reserve|actual-uninit-slice|actual-frozen-bytes|frozen-region|unique-reclaim|shared-adjacent-unsplit|shared-singleton-reclaim|shared-singleton-reserve|shared-nonunique-reserve|weak-native-publication|weak-physical-retirement|readonly-b4-purity|handle-comparison) target=$crate_root/verification/probes/$1 ;;
  vec-capacity-guarantee|actual-with-capacity|generic-buf-predicates|unique-growing-reserve|helpers|public-bytesmut-constructors|valid-handle-traits|deref-purity-feasibility|open-invariant-borrow|storage|storage-ops|scalable-tickets|coordinator-carrier-split|slice-buf-overrides|tool-blockers|trait-patch|deallocation|bounded-ops|slice-ops|cursor-ops|initialized-storage|byte-codecs|capacity-ops|comparison-ops|chain-ops|slice-read-ops|uninit-ops|wide-codecs|endian-ops|region-permissions|slice-wide-read-ops|variable-read-ops|signed-wide-ops|provenance-ops|comparison-runtime|ownership-frontier|runtime-convenience|vtable-feasibility|drop-feasibility|control-block-lifetime|owned-region-kernel|pointer-equality-feasibility|raw-vec-bridge|raw-vec-negative|bound-vec-constructor|retired-region-pool|sequential-retirement-registry|sequential-native-counter|sequential-shared-control|sequential-bytesmut-split|bound-pointer-offset|address-comparison) target=$crate_root/verification/probes/$1 ;;
  runtime) target=$crate_root ;;
  *) printf 'Usage: verify-bytes.sh [runtime|known-probe] [cargo flags]\n' >&2; exit 2 ;;
esac
if [[ $# -gt 0 ]]; then shift; fi
cd "$target"
# Serialize with existing proof work; do not reuse its patched compiler or driver.
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
printf 'bytes proof: %s; one prover; 1024 MiB; native Ordering; sc-drf disabled\n' "$target"
# Refuse accidental feature union that strengthens atomic Ordering or expands scope.
feature_tree=$(cargo tree --locked -e features --edges normal,build "$@")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2; exit 2
fi
# Coma files are side effects not restored by Cargo's per-feature artifact cache.
# Ensure this exact package/configuration really runs the translator each time.
package_name=$(cargo metadata --no-deps --locked --format-version 1 | python3 -c 'import json,sys; d=json.load(sys.stdin); assert len(d["packages"]) == 1; print(d["packages"][0]["name"])')
cargo clean --package "$package_name"
# A failed runtime translation must not leave old model/feature VCs eligible
# for a later selective proof. Saved evidence lives outside this live directory.
rm -rf -- verif
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then
  printf 'translation only: no proof phase requested\n'
  exit 0
fi
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
