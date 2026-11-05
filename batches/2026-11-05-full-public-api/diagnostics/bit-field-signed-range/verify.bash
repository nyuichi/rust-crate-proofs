#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
cargo kani --harness observed_signed_extraction_and_update > observed.log 2>&1
set +e
cargo kani --harness outside_range_preservation_candidate > candidate-contract.log 2>&1
status=$?
set -e
[[ "$status" -eq 1 ]]
rg -q '1 of 57 failed' candidate-contract.log
rg -q 'Failed Checks: assertion failed:' candidate-contract.log
printf 'Positive observation verified; candidate frame contract has the expected concrete counterexample.\n'
