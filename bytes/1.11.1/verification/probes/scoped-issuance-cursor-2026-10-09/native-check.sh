#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
# Same package/source, with all ghost channels erased by the ordinary compiler.
mkdir -p generated
cargo build --locked --message-format=json > generated/native-build.jsonl
rustc_path="$(rustup which rustc)"
rlib=$(python3 - <<'PY_ARTIFACT'
import json,pathlib
paths=[]
for line in pathlib.Path('generated/native-build.jsonl').read_text().splitlines():
    d=json.loads(line)
    if d.get('reason')=='compiler-artifact' and d['target']['name']=='bytes_scoped_issuance_cursor':
        paths.extend(x for x in d['filenames'] if x.endswith('.rlib'))
assert len(paths)==1,paths
print(paths[0])
PY_ARTIFACT
)
cat > generated/native-main.rs <<'RS'
extern crate bytes_scoped_issuance_cursor;
fn main() {
    assert!(bytes_scoped_issuance_cursor::closed_sparse_lifecycle());
}
RS
"$rustc_path" generated/native-main.rs --edition 2021 -L dependency="$CARGO_TARGET_DIR/debug/deps" \
    --extern "bytes_scoped_issuance_cursor=$rlib" -o generated/native-client
./generated/native-client
printf 'native closed scoped client: passed\n'
