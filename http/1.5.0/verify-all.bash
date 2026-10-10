#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

"$script_dir/scripts/run-proof.sh" cargo creusot clean --force
"$script_dir/scripts/run-proof.sh" cargo creusot --simple-triggers=false prove -- --locked --offline
"$script_dir/scripts/run-proof.sh" cargo creusot clean --force
"$script_dir/scripts/run-proof.sh" cargo creusot --simple-triggers=false prove -- --all-features --locked --offline
