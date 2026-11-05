#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
sha256sum source/{src/error.rs,Cargo.toml} > source-before.sha256
cargo kani 2>&1 | tee verification.log
sha256sum --check source-before.sha256
python3 - <<'META'
from pathlib import Path
import shutil
r=Path.cwd();p=next((r/'target').rglob('*.kani-metadata.json'));shutil.copy2(p,r/'evidence/kani-metadata.json')
META
python3 tools/check-evidence.py
