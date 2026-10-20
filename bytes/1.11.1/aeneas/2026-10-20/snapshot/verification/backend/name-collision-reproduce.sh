#!/usr/bin/env bash
set -euo pipefail

ROOT=/workspace/bytes-aeneas-fresh
source "$ROOT/env.sh"
export CARGO_TARGET_DIR="$ROOT/verification/backend/fixture-target"
cd "$ROOT/verification/backend/name-collision-fixture"

charon cargo \
  --preset=aeneas \
  --sysroot=default \
  --format=json \
  --no-dedup-serialized-ast \
  --dest-file "$ROOT/verification/backend/name-collision-reproduced.llbc" \
  -- --lib --locked --offline --target x86_64-unknown-linux-gnu

AENEAS_VERSION=e62dd83d133d9043e7d3628096838b7430f36565 \
  "$ROOT/aeneas/bin/aeneas" \
  -backend lean \
  -dest "$ROOT/verification/backend/name-collision-lean-reproduced" \
  "$ROOT/verification/backend/name-collision-reproduced.llbc"

cd "$ROOT/verification"
source ../env.sh
lake env lean backend/name-collision-lean-reproduced/Name-collision-reproduced.lean
