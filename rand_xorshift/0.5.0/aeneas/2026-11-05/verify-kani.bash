#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/evidence/kani-models"
if [[ -f /workspace/proof-tools/activate.sh ]]; then source /workspace/proof-tools/activate.sh; fi
kani_home="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}"
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="$kani_home/bin:$PATH"
cargo kani --harness std_wrapping_operator_models 2>&1 | tee std-models.log
cargo kani --harness original_next_u32_matches_bitvector_formula 2>&1 | tee original-body.log
