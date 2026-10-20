#!/usr/bin/env bash
# Tool selection for this isolated experiment; leaves the old bytes tree alone.
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-09-17
export AENEAS_HOME=/workspace/bytes-aeneas-fresh/aeneas
export PATH=/workspace/bytes-aeneas-fresh/tools-target/debug:/workspace/bytes-aeneas-fresh/charon-inspect/charon/target/debug:$AENEAS_HOME/bin:$PATH
export PATH=/workspace/bytes-aeneas-fresh/lean-tools/lean-4.35.0-linux/bin:$PATH
