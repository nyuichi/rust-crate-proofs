#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
mkdir -p generated/native-harness/src
python3 - <<'PY'
from pathlib import Path
import json
r=Path.cwd(); crate=r.parents[2]
manifest='[package]\nname = "bytes-shared-native-client"\nversion = "0.0.0"\nedition = "2021"\n[workspace]\n[dependencies]\nbytes = { path = '+json.dumps(str(crate))+' }\n'
(r/'generated/native-harness/Cargo.toml').write_text(manifest)
source='extern crate alloc;\nuse bytes::Bytes;\n#[path = '+json.dumps(str(r/'src/native_client.rs'))+']\nmod native_client;\n'+'''fn main() {
    for len in [0, 1, 7, 63, 1024] {
        let mut input = Vec::with_capacity(len + 19);
        input.extend((0..len).map(|i| (i % 251) as u8));
        let expected = input.clone();
        assert!(input.len() < input.capacity());
        assert_eq!(native_client::actual_public_shared_driver(input), expected);
    }
    println!("actual public Shared native client: 5 inputs passed");
}
'''
(r/'generated/native-harness/src/main.rs').write_text(source)
PY
rustc -Vv
cargo -V
cargo generate-lockfile --offline --manifest-path generated/native-harness/Cargo.toml
cargo run --locked --offline --manifest-path generated/native-harness/Cargo.toml
