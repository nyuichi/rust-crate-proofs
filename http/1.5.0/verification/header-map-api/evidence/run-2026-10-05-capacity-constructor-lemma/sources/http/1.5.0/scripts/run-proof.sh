#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
    printf 'Usage: scripts/run-proof.sh COMMAND [ARG...]\n' >&2
    exit 1
fi

repo_root=$(cd "$(dirname "$0")/../../.." && pwd)
source /workspace/proof-tools/activate.sh

export CARGO_NET_OFFLINE=${CARGO_NET_OFFLINE:-true}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo_root/target/http}

why3_source=${WHY3CONFIG:-$XDG_CONFIG_HOME/creusot/why3.conf}
if [[ ! -f "$why3_source" ]]; then
    printf 'Why3 configuration not found: %s\n' "$why3_source" >&2
    exit 1
fi

exec 9>"${HTTP_CREUSOT_PROOF_LOCK:-/tmp/http-creusot-proof.lock}"
flock 9
proof_config_root=$(mktemp -d /tmp/http-why3-config.XXXXXX)
trap 'rm -rf "$proof_config_root"' EXIT
mkdir -p "$proof_config_root/creusot"
python3 - "$why3_source" "$proof_config_root/creusot/why3.conf" <<'PYCONFIG'
import pathlib, re, sys

source, target = map(pathlib.Path, sys.argv[1:])
config = source.read_text()
for key, value in [('running_provers_max', 1), ('memlimit', 1024)]:
    config, count = re.subn(r'^' + key + r'\s*=.*$', f'{key} = {value}', config, flags=re.M)
    if count != 1:
        raise SystemExit(f'Expected exactly one {key} setting in {source}')
target.write_text(config)
PYCONFIG
export XDG_CONFIG_HOME="$proof_config_root"
export WHY3CONFIG="$proof_config_root/creusot/why3.conf"
printf 'http proof resources: jobs=1, memory=1024 MiB per prover; shared lock held\n'
"$@"
