#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/aeneas-tools/activate.sh
CARGO_NET_OFFLINE=true RUSTFLAGS='-Zmir-enable-passes=+Inline' charon cargo --preset aeneas --sysroot default --mir optimized --dest "$PWD" --start-from crate::prefix --include nohash_hasher::NoHashHasher
aeneas -backend lean -emit-json -abort-on-error -dest "$PWD/generated" "$PWD/nohash_prefix_inline.llbc"
# Use the pinned Aeneas Lean environment. Proof source and generation are unchanged.
proof_env=/workspace/full-api/endian/nohash-publication-candidate/nohash-hasher/0.2.0/aeneas/2026-11-05/proofs
prefix_root="$PWD"
cd "$proof_env"
lake env bash -c 'export LEAN_PATH="$1:$1/generated:$LEAN_PATH"; lean -R "$1/generated" -o "$1/generated/NohashPrefixInline.olean" "$1/generated/NohashPrefixInline.lean"; lean -R "$1" -o "$1/PrefixProofs.olean" "$1/PrefixProofs.lean"; lean -R "$1" "$1/Coimport.lean"' bash "$prefix_root"
