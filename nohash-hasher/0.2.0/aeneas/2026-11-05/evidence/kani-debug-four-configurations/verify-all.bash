#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
(cd nohash-debug-probe; ./verify.bash)
(cd nohash-debug-no-std; ./verify.bash)
