#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
creusot_repo=${1:-/tmp/creusot-source}
why3_data=${2:-/workspace/proof-tools/opamroot/creusot-v0.11/share/why3}
creusot_commit=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc
check_root=$(mktemp -d /tmp/creusot-toolpatch-check.XXXXXX)
trap 'rm -rf "$check_root"' EXIT

mkdir -p "$check_root/creusot"
git -C "$creusot_repo" archive "$creusot_commit" | tar -x -C "$check_root/creusot"
git -C "$check_root/creusot" apply --check \
  "$bundle_dir/patches/creusot-narrowcast-backend.patch"
printf 'Creusot backend patch applies to %s.\n' "$creusot_commit"

mkdir -p "$check_root/why3/drivers" "$check_root/why3/stdlib"
cp "$why3_data/drivers/z3_bv.gen" "$check_root/why3/drivers/"
cp "$why3_data/stdlib/bv.mlw" "$check_root/why3/stdlib/"
patch -p1 --dry-run -d "$check_root/why3" \
  < "$bundle_dir/patches/why3-bv64-narrowcast-driver.patch"
patch -p1 --dry-run -d "$check_root/why3" \
  < "$bundle_dir/patches/why3-bv128-lsr-trigger.patch"
printf 'Why3 driver and BV128 library patches apply to the pinned Why3 data tree.\n'
