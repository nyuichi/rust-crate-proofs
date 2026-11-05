#!/usr/bin/env bash
set -euo pipefail
source /workspace/aeneas-tools/activate.sh
root="$(cd "$(dirname "$0")/.." && pwd)"
cache="$(mktemp -d /tmp/oorandom-established-lean.XXXXXX)"
trap 'rm -rf "$cache"' EXIT
mkdir -p "$cache/Oorandom"
cd /workspace/full-api/oorandom-range/proofs
for module in Oorandom/Types Oorandom/Funs NegBridge Checkpoint FullRange Zero64 PublicDomain EstablishedRange; do
 lake env bash -c 'export LEAN_PATH="$1:$2:$LEAN_PATH"; cd "$2"; timeout 45s lean -M 8192 -o "$1/$3.olean" "$3.lean"' bash "$cache" "$root/proofs" "$module"
done
