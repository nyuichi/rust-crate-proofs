#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
export CARGO_NET_OFFLINE=true
sha256sum ../../{Cargo.toml,src/lib.rs} ../../../../../../byteorder-lite/0.1.0/aeneas/2026-11-05/{Cargo.toml,src/lib.rs} > source-before.sha256
rustc -Vv > rustc-version.txt
set +e
timeout 60s cargo kani --harness be_write_uint_original_bridge > verification.log 2>&1
kani_exit=$?
set -e
printf '%s\n' "$kani_exit" > exit.txt
sha256sum --check source-before.sha256 > source-after-check.log
exit "$kani_exit"
