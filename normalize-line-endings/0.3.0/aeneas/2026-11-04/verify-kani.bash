#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/evidence/kani-equivalence"
exec ./verify.bash
