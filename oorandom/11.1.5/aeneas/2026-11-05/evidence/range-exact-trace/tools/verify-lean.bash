#!/usr/bin/env bash
set -euo pipefail
source /workspace/aeneas-tools/activate.sh
root="$(cd "$(dirname "$0")/.." && pwd)"
cache="$(mktemp -d /tmp/oorandom-established-lean.XXXXXX)"
trap 'rm -rf "$cache"' EXIT
mkdir -p "$cache/Oorandom"
cd /workspace/full-api/oorandom-range/proofs
for module in Oorandom/Types Oorandom/Funs NegBridge Checkpoint FullRange Zero64 PublicDomain NatRangeArithmetic RangeArithmetic PartialLoop RangePartial32 RangePartial64 RangeComplete32 RangeComplete64 EstablishedRange GenericTrace ActualLoop32 ActualLoop64 PublicTrace32 PublicTrace64 TraceAudit ExactRange; do
 python3 "$root/tools/run-module-bounded.py" "$cache" "$module"
done
