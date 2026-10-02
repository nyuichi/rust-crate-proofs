#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
if [[ -z "${WHY3DATA:-}" ]]; then
  export WHY3DATA=$("$bundle_dir/scripts/prepare-why3-overlay.sh")
  toolpatch_temp_why3data=$WHY3DATA
  trap 'rm -rf "$toolpatch_temp_why3data"' EXIT
fi

export RUSTUP_HOME=${RUSTUP_HOME:-/tmp/rustup-home}
export CARGO_HOME=${CARGO_HOME:-/tmp/cargo-home}
export CREUSOT_DATA_HOME=${CREUSOT_DATA_HOME:-/tmp/creusot-data}
export XDG_CACHE_HOME=${XDG_CACHE_HOME:-/tmp/creusot-cache}
export XDG_CONFIG_HOME=${XDG_CONFIG_HOME:-/tmp/why3-capture-config}
export WHY3CONFIG=${WHY3CONFIG:-/tmp/why3-capture-config/creusot/why3.conf}
export CREUSOT_RUSTC=${CREUSOT_RUSTC:-/workspace/proof-tools/targets/creusot-cast-compdiv/debug/creusot-rustc}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/workspace/proof-tools/targets/toolpatch-high-half-witness-$$}
export CARGO_NET_OFFLINE=true
export PATH="/tmp/creusot-data/bin:/tmp/cargo-home/bin:/workspace/proof-tools/opamroot/creusot-v0.11/bin:$PATH"
export LD_LIBRARY_PATH="/tmp/rustup-home/toolchains/nightly-2026-02-27-x86_64-unknown-linux-gnu/lib:/tmp/local-ocaml/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

"$bundle_dir/scripts/prove-derived-lsr-lemma.sh"
cd "$bundle_dir/witnesses/high-half"
cargo creusot --no-check-version --simple-triggers=false prove --no-cache -- --offline
