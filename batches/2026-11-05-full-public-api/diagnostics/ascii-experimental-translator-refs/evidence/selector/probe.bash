#!/usr/bin/env bash
set -euo pipefail
source /workspace/aeneas-tools/activate.sh
export CARGO_NET_OFFLINE=true RUSTFLAGS='-A bindings_with_variant_name' CARGO_TARGET_DIR=/workspace/full-api/bits/ascii-extra/from-field-two/ufcs-selector/target
cd /workspace/full-api/bits/ascii-extra/from-field-two/ufcs-selector
mkdir -p evidence proofs/Generated
charon cargo --preset aeneas --sysroot default --dest evidence --start-from ascii_field_selectors::shared --start-from ascii_field_selectors::mutable --include 'ascii::ascii_str::AsciiStr' --include 'ascii::ascii_char::AsciiChar' --include 'ascii::ascii_str::{impl core::convert::From<&ascii::ascii_str::AsciiStr> for &Slice<ascii::ascii_char::AsciiChar>}::from' --include 'ascii::ascii_str::{impl core::convert::From<&mut ascii::ascii_str::AsciiStr> for &mut Slice<ascii::ascii_char::AsciiChar>}::from' > evidence/charon.log 2>&1
aeneas -backend lean -emit-json -abort-on-error -dest proofs/Generated evidence/ascii_field_selectors.llbc > evidence/aeneas.log 2>&1
