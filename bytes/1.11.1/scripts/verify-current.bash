#!/usr/bin/env bash
# Freshly translate and prove the selected targets from this crate's live source.
set -euo pipefail

crate_root=$(cd "$(dirname "$0")/.." && pwd)
checker="$crate_root/scripts/check-current-proof.py"

if [[ "${1:-}" == --check && $# == 1 ]]; then
    exec python3 "$checker" --check
fi
if [[ $# != 0 ]]; then
    printf 'Usage: verify-all.bash [--check]\n' >&2
    exit 2
fi

tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$tool_root/activate.sh"
tool_config="$XDG_CONFIG_HOME/creusot/why3.conf"
export CARGO_NET_OFFLINE=true
export RUSTFLAGS='--cfg bytes_original_unique_gate'
unset CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_TARGET
for variable in ${!CARGO_TARGET_@}; do
    if [[ "$variable" == *_RUSTFLAGS ]]; then unset "$variable"; fi
done

rust_host=$(rustc -vV | sed -n 's/^host: //p')
if [[ "$rust_host" != x86_64-unknown-linux-gnu ]]; then
    printf 'Direct proof requires x86_64-unknown-linux-gnu; rustc host is %s\n' "$rust_host" >&2
    exit 1
fi

python3 "$crate_root/scripts/prepare-proof-std.py"
cd "$crate_root"

# sc-drf must stay disabled: this run uses native atomic ordering behavior.
feature_tree=$(mktemp)
translation_log=$(mktemp)
input_snapshot=$(mktemp)
why3_config=$(mktemp -d)
cleanup() {
    rm -f "$feature_tree" "$translation_log" "$input_snapshot"
    rm -rf "$why3_config"
}
trap cleanup EXIT
cargo tree --locked -e features -i creusot-std > "$feature_tree"
if rg -q 'sc-drf' "$feature_tree"; then
    printf 'sc-drf must stay disabled for direct-source verification\n' >&2
    exit 1
fi
python3 "$checker" --snapshot > "$input_snapshot"

# Serialize translation and solver use with the shared proof lock.
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
# The ignored verifier output is rebuilt from the live crate source each run.
rm -rf verif
if ! cargo clean --package bytes; then
    printf 'Could not clear the bytes translation artifact before fresh verification\n' >&2
    exit 1
fi
if ! cargo creusot --only=coma -- --locked > "$translation_log" 2>&1; then
    cat "$translation_log" >&2
    exit 1
fi

export XDG_CONFIG_HOME="$why3_config"
mkdir -p "$XDG_CONFIG_HOME/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' \
    "$tool_config" > "$XDG_CONFIG_HOME/creusot/why3.conf"
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"

mapfile -t targets < <(python3 "$checker" --targets)
if [[ ${#targets[@]} -eq 0 ]]; then
    printf 'The direct runner has no proof targets\n' >&2
    exit 1
fi
for target in "${targets[@]}"; do
    if [[ ! -s "$target" ]]; then
        printf 'Fresh translation did not emit target %s\n' "$target" >&2
        cat "$translation_log" >&2
        exit 1
    fi
done
if ! cargo creusot --only=prove --no-cache --why3find-arg=-j --why3find-arg=1 "${targets[@]}"; then
    printf 'Fresh direct-source proof failed; translator output follows\n' >&2
    cat "$translation_log" >&2
    exit 1
fi
python3 "$checker" --save "$feature_tree" "$translation_log" "$input_snapshot"
python3 "$checker" --check
