#!/usr/bin/env bash
set -euo pipefail

probe=$(cd "$(dirname "$0")" && pwd)
crate=$(cd "$probe/../../.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
python3 "$crate/scripts/prepare-proof-std.py"
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
cd "$probe"

cargo clean --package bytesmut-len-api-proof
rm -rf verif
cargo creusot --only=coma -- --locked

mkdir -p .proof-config/creusot
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > .proof-config/creusot/why3.conf
export XDG_CONFIG_HOME="$PWD/.proof-config"
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
cargo creusot --only=prove --no-cache --why3find-arg=-j --why3find-arg=1 \
  verif/bytesmut_len_api_proof_rlib/impl_BytesMut_0/len.coma \
  verif/bytesmut_len_api_proof_rlib/impl_BytesMut_0/truncate.coma \
  verif/bytesmut_len_api_proof_rlib/impl_BytesMut_0/set_len.coma \
  verif/bytesmut_len_api_proof_rlib/view_region/split_view_region.coma \
  verif/bytesmut_len_api_proof_rlib/raw_vec/impl_BoundPtr/advance_within.coma \
  verif/bytesmut_len_api_proof_rlib/raw_vec/impl_PhysicalRegion/split_at.coma \
  verif/bytesmut_len_api_proof_rlib/owned_region/impl_OwnedRegion/split_at.coma

out=$(find "$CARGO_TARGET_DIR/debug/build" -path '*/bytesmut-len-api-proof-*/out/source-map.json' -type f -print | sort | tail -n 1)
test -n "$out"
python3 "$probe/save_evidence.py" "$(dirname "$out")"
