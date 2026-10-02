#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
why3_base=${WHY3_BASE_DATA:-/workspace/proof-tools/opamroot/creusot-v0.11/share/why3}
overlay_dir=$(mktemp -d /tmp/why3-toolpatch-overlay.XXXXXX)
trap 'rm -rf "$overlay_dir"' ERR

if [[ ! -d "$why3_base/drivers" || ! -d "$why3_base/stdlib" ]]; then
  printf 'Why3 data tree is missing drivers/ or stdlib/: %s\n' "$why3_base" >&2
  exit 1
fi

mkdir -p "$overlay_dir/drivers" "$overlay_dir/stdlib"
find "$why3_base" -mindepth 1 -maxdepth 1 -type f -exec cp -a {} "$overlay_dir/" \;
cp -a "$why3_base/drivers/." "$overlay_dir/drivers/"
cp -a "$why3_base/stdlib/." "$overlay_dir/stdlib/"
patch -p1 -d "$overlay_dir" < "$bundle_dir/patches/why3-bv64-narrowcast-driver.patch" >/dev/null
patch -p1 -d "$overlay_dir" < "$bundle_dir/patches/why3-bv128-lsr-trigger.patch" >/dev/null
trap - ERR
printf '%s\n' "$overlay_dir"
