#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source /workspace/proof-tools/activate.sh
source /workspace/scratch/httparse-string-model/creusot-env.sh
cd "$script_dir"

if [[ "${1:-translate}" != translate ]]; then
  echo "usage: $0 [translate]" >&2
  exit 2
fi

case translate in
  translate)
    cargo creusot --simple-triggers=false
    ;;
esac
