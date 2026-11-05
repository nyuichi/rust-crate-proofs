#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/aeneas-tools/activate.sh
python3 check-source.py
CARGO_NET_OFFLINE=true cargo run --quiet > observations.jsonl 2> build.log
python3 check-source.py
