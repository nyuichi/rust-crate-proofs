#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
export CARGO_NET_OFFLINE=true
: > exits.txt
for harness in original_from_rng_fixed_nonzero_seed original_try_from_rng_fixed_nonzero_seed original_try_from_rng_exact_source_error; do
 set +e
 timeout --signal=TERM --kill-after=5s 60s cargo kani --harness "$harness" > "$harness.log" 2>&1
 probe_status=$?
 set -e
 printf '%s %s\n' "$harness" "$probe_status" >> exits.txt
done
