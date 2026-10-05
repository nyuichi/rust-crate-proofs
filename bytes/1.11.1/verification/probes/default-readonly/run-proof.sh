#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
cargo clean --package bytes-default-readonly
rm -rf verif
cargo creusot --only=coma -- --locked
cargo creusot clean --force
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 --why3find-arg=-X \
 verif/bytes_default_readonly_rlib/actual/impl_BytesMut/as_slice.coma \
 verif/bytes_default_readonly_rlib/actual/impl_BytesMut/kind.coma \
 verif/bytes_default_readonly_rlib/actual/impl_AsRef_for_BytesMut/as_ref.coma \
 verif/bytes_default_readonly_rlib/actual/impl_Deref_for_BytesMut/deref.coma \
 verif/bytes_default_readonly_rlib/shared_protocol/borrow_packet.coma \
 verif/bytes_default_readonly_rlib/provenance_specs/pointer_addr.coma
