#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
repo_root=${1:-${RUST_CRATE_PROOFS_ROOT:-/workspace/rust-crate-proofs}}
crate_dir=$repo_root/itoa/1.0.18

if [[ ! -x "$crate_dir/verify-all.bash" ]]; then
  printf 'Expected verify-all.bash at %s\n' "$crate_dir/verify-all.bash" >&2
  exit 1
fi

export RUSTUP_HOME=${RUSTUP_HOME:-/tmp/rustup-home}
export CARGO_HOME=${CARGO_HOME:-/tmp/cargo-home}
export RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN:-nightly-2026-02-27}
export CREUSOT_DATA_HOME=${CREUSOT_DATA_HOME:-/tmp/creusot-data}
export XDG_CACHE_HOME=${XDG_CACHE_HOME:-/tmp/creusot-cache}
export XDG_CONFIG_HOME=${XDG_CONFIG_HOME:-/tmp/why3-capture-config}
export WHY3CONFIG=${WHY3CONFIG:-$XDG_CONFIG_HOME/creusot/why3.conf}
export WHY3_BASE_DATA=${WHY3_BASE_DATA:-/workspace/proof-tools/opamroot/creusot-v0.11/share/why3}
export CREUSOT_RUSTC=${CREUSOT_RUSTC:-/workspace/proof-tools/targets/creusot-cast-compdiv/debug/creusot-rustc}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/workspace/proof-tools/targets/itoa-toolpatch-verify-$$}
export CARGO_NET_OFFLINE=true
export PATH="$CREUSOT_DATA_HOME/bin:$CARGO_HOME/bin:/workspace/proof-tools/opamroot/creusot-v0.11/bin:$PATH"
export LD_LIBRARY_PATH="$RUSTUP_HOME/toolchains/$RUSTUP_TOOLCHAIN-x86_64-unknown-linux-gnu/lib:/tmp/local-ocaml/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

if [[ ! -x "$CREUSOT_RUSTC" ]]; then
  printf 'Patched creusot-rustc not found: %s\n' "$CREUSOT_RUSTC" >&2
  printf 'Build it with %s/scripts/build-creusot-rustc.sh.\n' "$bundle_dir" >&2
  exit 1
fi

if [[ -z "${WHY3DATA:-}" ]]; then
  export WHY3DATA=$("$bundle_dir/scripts/prepare-why3-overlay.sh")
  toolpatch_temp_why3data=$WHY3DATA
  trap 'rm -rf "$toolpatch_temp_why3data"' EXIT
fi

printf 'Running official verify-all.bash with CREUSOT_RUSTC=%s and WHY3DATA=%s\n' "$CREUSOT_RUSTC" "$WHY3DATA"
cd "$crate_dir"
./verify-all.bash
