#!/usr/bin/env bash
# Proof phases require elevated execution: Why3 uses Unix-domain sockets.
set -euo pipefail
crate_root=$(cd "$(dirname "$0")/../../.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
target=$crate_root/verification/probes/actual-unique-reserve
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
proof_patterns=()
if [[ -n "${BYTES_PROVE_PATTERN:-}" ]]; then proof_patterns+=("$BYTES_PROVE_PATTERN"); fi
cargo creusot --only=prove "${proof_patterns[@]}" --why3find-arg=-j --why3find-arg=1
