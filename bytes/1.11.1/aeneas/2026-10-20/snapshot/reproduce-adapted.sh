#!/usr/bin/env bash
set -euo pipefail
export BYTES_PROBE_CRATE=/workspace/bytes-aeneas-fresh/bytes-adapted
export AENEAS_FILTER_TRAIT_METHODS=1
bash /workspace/bytes-aeneas-fresh/probe.sh adapted-whole-final --start-from bytes
