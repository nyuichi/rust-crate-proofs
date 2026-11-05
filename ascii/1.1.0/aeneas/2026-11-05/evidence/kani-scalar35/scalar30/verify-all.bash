#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
for group in from2 compare8 default20; do
 "$root/$group/verify.bash"
done
