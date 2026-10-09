#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export CARGO_INCREMENTAL=0
if [[ "$(rustc --version)" != 'rustc 1.98.0-nightly (91fe22da8 2026-06-21)' ]]; then exit 2; fi
if [[ "$(rustc -Vv | sed -n 's/^commit-hash: //p')" != '91fe22da8084a1c9e993d78d4a56f22ab8396236' ]]; then exit 2; fi
mkdir -p generated/native-harness/src native-mir
scratch_dir=$(mktemp -d /workspace/work/bytes-native-terminal-mir.XXXXXX)
mkdir -p "$scratch_dir/client" "$scratch_dir/production"
python3 - <<'PY'
from pathlib import Path
import json
r=Path.cwd();crate=r.parents[2]
manifest='[package]\nname = "bytes-automatic-drop-native"\nversion = "0.0.0"\nedition = "2021"\n[workspace]\n[lib]\nname = "bytes_automatic_drop_native"\npath = '+json.dumps(str(r/'native.rs'))+'\n[dependencies]\nbytes = { path = '+json.dumps(str(crate))+' }\n'
(r/'generated/native-harness/Cargo.toml').write_text(manifest)
(r/'generated/native-harness/src/main.rs').write_text('''// Matches the native non-owning Bytes field categories; &T has no field glue.
const _: [(); 0] = [(); core::mem::needs_drop::<(
    *const u8, usize, core::sync::atomic::AtomicPtr<()>, &'static ()
)>() as usize];
fn main() {
    for len in [0, 1, 7, 63, 1024] {
        let mut input = Vec::with_capacity(len + 19);
        input.extend((0..len).map(|i| (i % 251) as u8));
        let expected = input.clone();
        assert!(input.len() < input.capacity());
        assert_eq!(bytes_automatic_drop_native::shared_automatic_scope(input), expected);
    }
    println!("actual Bytes automatic normal Drop: 5 inputs passed");
}
''')
PY
rustc -Vv > native-mir/rustc-version.txt
cargo -V > native-mir/cargo-version.txt
cargo generate-lockfile --offline --manifest-path generated/native-harness/Cargo.toml
cargo clean --manifest-path generated/native-harness/Cargo.toml --package bytes-automatic-drop-native
cargo rustc --locked --offline --manifest-path generated/native-harness/Cargo.toml --lib -- \
  -Zdump-mir=all -Zdump-mir-dir="$scratch_dir/client" -Zmir-opt-level=0 -Zidentify-regions=yes
cargo clean --manifest-path ../../../Cargo.toml --package bytes
cargo rustc --locked --offline --manifest-path ../../../Cargo.toml --lib -- \
  -Zdump-mir=all -Zdump-mir-dir="$scratch_dir/production" -Zmir-opt-level=0 -Zidentify-regions=yes
python3 - "$scratch_dir" <<'PY'
from pathlib import Path
import sys,json,hashlib,shutil
scratch=Path(sys.argv[1]);r=Path.cwd();out=r/'native-mir'
for p in out.glob('*.mir'):p.unlink()
client=list((scratch/'client').glob('*shared_automatic_scope.2-2-004.ElaborateDrops.after.mir'))
assert len(client)==1,client
selected=client[:]
for needle in ['_1: &mut bytes::Bytes','_1: &mut bytes::Shared','fn shared_drop(','fn shared_drop::{closure#0}','fn bytes::release_shared(','fn free_shared(','fn loom::sync::atomic::<impl']:
 matches=[]
 for p in (scratch/'production').glob('*2-2-004.ElaborateDrops.after.mir'):
  s=p.read_text()
  if needle in s and (not needle.startswith('_1: &mut bytes::') or p.name.endswith('-drop.2-2-004.ElaborateDrops.after.mir')) and (needle!='fn loom::sync::atomic::<impl' or '::with_mut' in s):matches.append(p)
 assert len(matches)==1,(needle,matches)
 selected+=matches
rows=[]
for p in selected:
 shutil.copy2(p,out/p.name)
 rows.append(dict(path='native-mir/'+p.name,sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
receipt=dict(stage='2-2-004.ElaborateDrops.after.mir',native_source='native.rs',native_source_sha256=hashlib.sha256((r/'native.rs').read_bytes()).hexdigest(),selected=rows,
 commands=['cargo rustc --locked --offline --manifest-path generated/native-harness/Cargo.toml --lib -- -Zdump-mir=all -Zdump-mir-dir=<scratch/client> -Zmir-opt-level=0 -Zidentify-regions=yes',
 'cargo rustc --locked --offline --manifest-path ../../../Cargo.toml --lib -- -Zdump-mir=all -Zdump-mir-dir=<scratch/production> -Zmir-opt-level=0 -Zidentify-regions=yes'],
 production_source='../../../src/bytes.rs',production_source_sha256=hashlib.sha256((r/'../../../src/bytes.rs').resolve().read_bytes()).hexdigest())
(out/'capture.json').write_text(json.dumps(receipt,indent=2)+'\n')
PY
cargo run --locked --offline --manifest-path generated/native-harness/Cargo.toml > generated/native-run.log 2>&1
cat generated/native-run.log
python3 elaborate.py
printf 'native MIR temporary directory: %s\n' "$scratch_dir"
