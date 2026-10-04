#!/usr/bin/env bash
# Proof phases require elevated execution: Why3 uses Unix-domain sockets.
set -euo pipefail
crate_root=$(cd "$(dirname "$0")/.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
case "${1:-runtime}" in
  helpers|storage|tool-blockers|trait-patch|deallocation|bounded-ops|slice-ops|cursor-ops|initialized-storage|byte-codecs|capacity-ops|comparison-ops|chain-ops|slice-read-ops|uninit-ops|wide-codecs|endian-ops|region-permissions|slice-wide-read-ops|variable-read-ops|signed-wide-ops|provenance-ops|comparison-runtime|ownership-frontier|runtime-convenience|vtable-feasibility|drop-feasibility|control-block-lifetime|owned-region-kernel|pointer-equality-feasibility) target=$crate_root/verification/probes/$1 ;;
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
