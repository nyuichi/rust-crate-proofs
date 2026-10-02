#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
if [[ $# -eq 0 ]]; then
  printf 'Usage: run-proof.sh COMMAND [ARG...] (run from target crate)\n' >&2
  exit 1
fi

export RUSTUP_HOME=${RUSTUP_HOME:-/tmp/rustup-home}
export CARGO_HOME=${CARGO_HOME:-/tmp/cargo-home}
export RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN:-nightly-2026-02-27}
export CREUSOT_DATA_HOME=${CREUSOT_DATA_HOME:-/tmp/creusot-data}
export XDG_CACHE_HOME=${XDG_CACHE_HOME:-/tmp/creusot-cache}
export XDG_CONFIG_HOME=${XDG_CONFIG_HOME:-/tmp/creusot-config}
export WHY3CONFIG=${WHY3CONFIG:-$XDG_CONFIG_HOME/creusot/why3.conf}
export WHY3_BASE_DATA=${WHY3_BASE_DATA:-/workspace/proof-tools/opamroot/creusot-v0.11/share/why3}
export CREUSOT_RUSTC=${CREUSOT_RUSTC:-/workspace/proof-tools/targets/creusot-cast-compdiv/debug/creusot-rustc}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/workspace/proof-tools/targets/itoa-toolpatch-verify}
export CARGO_NET_OFFLINE=true
export PATH="$CREUSOT_DATA_HOME/bin:$CARGO_HOME/bin:/workspace/proof-tools/opamroot/creusot-v0.11/bin:$PATH"
export LD_LIBRARY_PATH="$RUSTUP_HOME/toolchains/$RUSTUP_TOOLCHAIN-x86_64-unknown-linux-gnu/lib:/tmp/local-ocaml/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

if [[ ! -x "$CREUSOT_RUSTC" ]]; then
  printf 'Patched creusot-rustc not found: %s\n' "$CREUSOT_RUSTC" >&2
  printf 'Build it with %s/scripts/build-creusot-rustc.sh.\n' "$bundle_dir" >&2
  exit 1
fi

# One proof invocation across all agents; one prover by default.
exec 9>"${ITOA_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
proof_config_root=$(mktemp -d /tmp/itoa-proof-config.XXXXXX)
toolpatch_temp_why3data=
trap 'rm -rf "$proof_config_root"; if [[ -n "$toolpatch_temp_why3data" ]]; then rm -rf "$toolpatch_temp_why3data"; fi' EXIT
mkdir -p "$proof_config_root/creusot"
python3 - "$WHY3CONFIG" "$proof_config_root/creusot/why3.conf" "${ITOA_PROOF_JOBS:-1}" "${ITOA_PROOF_MEMORY_MB:-1024}" <<'PYCONFIG'
import pathlib, re, sys
source, target, jobs, memory = sys.argv[1:]
jobs, memory = int(jobs), int(memory)
if jobs not in (1, 2) or not 512 <= memory <= 2048:
    raise SystemExit('Use 1–2 prover jobs and 512–2048 MiB per prover')
config = pathlib.Path(source).read_text()
for key, value in [('running_provers_max', jobs), ('memlimit', memory)]:
    config, count = re.subn(r'^' + key + r'\s*=.*$', f'{key} = {value}', config, flags=re.M)
    if count != 1:
        raise SystemExit(f'Expected exactly one {key} setting')
pathlib.Path(target).write_text(config)
PYCONFIG
# cargo-creusot derives WHY3CONFIG from XDG_CONFIG_HOME; set both.
export XDG_CONFIG_HOME=$proof_config_root
export WHY3CONFIG=$proof_config_root/creusot/why3.conf

if [[ -z "${WHY3DATA:-}" ]]; then
  export WHY3DATA=$("$bundle_dir/scripts/prepare-why3-overlay.sh")
  toolpatch_temp_why3data=$WHY3DATA
fi
printf 'Proof resources: jobs=%s, memory=%s MiB per prover; shared lock held\n' "${ITOA_PROOF_JOBS:-1}" "${ITOA_PROOF_MEMORY_MB:-1024}"
"$@"
