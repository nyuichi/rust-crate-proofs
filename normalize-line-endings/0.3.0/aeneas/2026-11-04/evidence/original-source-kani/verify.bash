#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
python3 tools/check-source.py
source /workspace/aeneas-tools/activate.sh
(cd source; CARGO_NET_OFFLINE=true charon cargo --preset aeneas --sysroot default --start-from crate::normalized --dest ../investigation/constructor > ../investigation/constructor/charon.log 2>&1)
aeneas -backend lean -abort-on-error -dest investigation/constructor investigation/constructor/normalize_line_endings.llbc > investigation/constructor/aeneas.log 2>&1
cmp investigation/constructor/NormalizeLineEndings.lean proofs/Generated/NormalizeLineEndings.lean
(cd proofs; lake build > ../evidence/constructor-build.log 2>&1; lake env lean Proofs.lean > ../evidence/constructor-audit.log 2>&1)
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-08-21
kani_sysroot="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$kani_sysroot/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="${KANI_HOME:-/workspace/kani-tools/kani-0.68.0}/bin:$PATH"
cargo kani > evidence/kani.log 2>&1
python3 tools/check-source.py
python3 tools/check-results.py
