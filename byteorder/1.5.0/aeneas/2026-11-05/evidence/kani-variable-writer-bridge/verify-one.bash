#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
harness_name="$1"
result_label="$2"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/workspace/kani-tools/kani-0.68.0/bin:$PATH"
export CARGO_NET_OFFLINE=true
sha256sum ../../{Cargo.toml,src/lib.rs} ../../../../../../byteorder-lite/0.1.0/aeneas/2026-11-05/{Cargo.toml,src/lib.rs} > "source-before-${result_label}.sha256"
set +e
timeout 60s cargo kani --harness "$harness_name" > "verification-${result_label}.log" 2>&1
kani_exit=$?
set -e
printf '%s\n' "$kani_exit" > "exit-${result_label}.txt"
sha256sum --check "source-before-${result_label}.sha256" > "source-after-${result_label}.log"
exit "$kani_exit"
