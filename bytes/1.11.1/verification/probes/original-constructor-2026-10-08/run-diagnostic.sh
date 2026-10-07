#!/usr/bin/env bash
set -euo pipefail
probe_dir=$(cd "$(dirname "$0")" && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
export CARGO_TARGET_DIR=/workspace/bytes-proof-tools/targets/bytes-original-constructor-20261008
export CARGO_NET_OFFLINE=true
export CARGO_BUILD_JOBS=1
cd "$probe_dir"

exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9

if [[ ! -f Cargo.lock ]]; then
    cargo generate-lockfile --offline 2>&1 | tee evidence/lock-generation.log
fi

cargo test --locked --lib 2>&1 | tee evidence/native.log
printf '0\n' > evidence/native.exit

# Drop native artifacts before translation so this is a fresh Creusot build.
cargo clean --package bytes-original-constructor-probe
rm -rf -- verif
set +e
cargo creusot --only=coma -- --locked --lib 2>&1 | tee evidence/translation.log
translation_status=${PIPESTATUS[0]}
set -e
printf '%s\n' "$translation_status" > evidence/translation.exit

python3 - <<'PY'
import hashlib, json
from pathlib import Path
root=Path("../../..").resolve()
source=root/"src/bytes_mut.rs"
snapshot=Path("evidence/input-bytes_mut.rs")
record={
    "production_live_path":str(source),
    "production_live_sha256_after_translation":hashlib.sha256(source.read_bytes()).hexdigest(),
    "captured_snapshot_sha256":hashlib.sha256(snapshot.read_bytes()).hexdigest(),
    "production_matches_snapshot_after_translation":source.read_bytes()==snapshot.read_bytes(),
    "translation_exit":int(Path("evidence/translation.exit").read_text()),
}
Path("evidence/source-after-translation.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
PY

exit "$translation_status"
