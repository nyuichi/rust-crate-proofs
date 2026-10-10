#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source /workspace/proof-tools/activate.sh
cd "$script_dir"

case "${1:-translate}" in
  translate)
    CARGO_NET_OFFLINE=true cargo creusot --no-check-version --simple-triggers=false
    ;;
  prove)
    "$script_dir/../../../run-proof.bash" cargo creusot --simple-triggers=false prove
    ;;
  *)
    echo "usage: $0 [translate|prove]" >&2
    exit 2
    ;;
esac
