#!/usr/bin/env bash
set -euo pipefail
source /workspace/aeneas-tools/activate.sh
export CARGO_NET_OFFLINE=true
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d /tmp/oorandom-established-regen.XXXXXX)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/source/src" "$work/llbc" "$work/Oorandom"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$work/source/"
cp "$root/src/lib.rs" "$work/source/src/"
export CARGO_TARGET_DIR="$work/target"
cd "$work/source"
charon cargo --preset aeneas --sysroot default --dest "$work/llbc" --start-from oorandom::Rand32::rand_range --start-from oorandom::Rand64::rand_range --start-from oorandom::Rand32::from_state --start-from oorandom::Rand64::from_state --include 'core::num::_::wrapping_neg'
aeneas -backend lean -emit-json -abort-on-error -split-files -namespace oorandom_range_original -dest "$work/Oorandom" "$work/llbc/oorandom.llbc"
for f in Funs.lean Types.lean; do cmp "$root/proofs/Oorandom/$f" "$work/Oorandom/$f"; done
python3 - "$work/Oorandom/translation.json" <<'PY'
import json,sys
x=json.load(open(sys.argv[1])); rows=x['functions'] if isinstance(x,dict) and 'functions' in x else []
assert rows,'translation schema requires explicit review'
assert not [r for r in rows if r.get('is_opaque')]
print('opaque0; generated Funs/Types byte-identical')
PY
