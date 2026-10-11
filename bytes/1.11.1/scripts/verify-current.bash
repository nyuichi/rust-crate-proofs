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
"$crate_root/scripts/setup-bytes-toolchain.sh"
source "$tool_root/activate.sh"
export CREUSOT_RUSTC="$tool_root/fn-ghost-purity/creusot-rustc"
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

std_root=$(python3 "$checker" --std-root)
std_erasure_file="$std_root/_creusot_erasure/creusot_std"
if [[ -e "$std_erasure_file" ]]; then rm -f "$std_erasure_file"; fi
rmdir "$std_root/_creusot_erasure" 2>/dev/null || true
python3 "$crate_root/scripts/prepare-proof-std.py"
cd "$crate_root"

# sc-drf must stay disabled: this run uses native atomic ordering behavior.
feature_tree=$(mktemp)
translation_log=$(mktemp)
std_translation_log=$(mktemp)
input_snapshot=$(mktemp)
why3_config=$(mktemp -d)
std_output_dir=$(mktemp -d /tmp/bytes-std-release-sources.XXXXXX)
std_target_dir="$tool_root/bytes-proof-std-release-target"
std_erasure_created=false
cleanup() {
    if [[ "$std_erasure_created" == true ]]; then
        rm -f "$std_erasure_file"
        rmdir "$std_root/_creusot_erasure" 2>/dev/null || true
    fi
    rm -f "$feature_tree" "$translation_log" "$std_translation_log" "$input_snapshot"
    rm -rf "$why3_config"
    rm -rf "$std_output_dir"
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

# Translate the selected pure algebra bodies from the exact pinned Std source
# package. Keep the Cargo target and COMA output outside the canonical package.
if [[ -e "$std_erasure_file" ]]; then
    printf 'Unexpected Creusot erasure artifact already exists in the canonical Std package\n' >&2
    exit 1
fi
std_erasure_created=true
if ! (
    cd "$std_root"
    export CARGO_TARGET_DIR="$std_target_dir"
    unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_TARGET
    cargo clean --package creusot-std
    cargo creusot --only=coma --output-dir "$std_output_dir" -- --locked
) > "$std_translation_log" 2>&1; then
    cat "$std_translation_log" >&2
    exit 1
fi
rm -f "$std_erasure_file"
rmdir "$std_root/_creusot_erasure" 2>/dev/null || true
std_erasure_created=false
cat "$std_translation_log" >> "$translation_log"
cp "$crate_root/why3find.json" "$std_output_dir/why3find.json"

export XDG_CONFIG_HOME="$why3_config"
mkdir -p "$XDG_CONFIG_HOME/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' \
    "$tool_config" > "$XDG_CONFIG_HOME/creusot/why3.conf"
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"

mapfile -t targets < <(python3 "$checker" --targets)
mapfile -t std_targets < <(python3 "$checker" --std-targets)
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
if [[ ${#std_targets[@]} -eq 0 ]]; then
    printf 'The direct runner has no selected Std body targets\n' >&2
    exit 1
fi
for target in "${std_targets[@]}"; do
    if [[ ! -s "$std_output_dir/$target" ]]; then
        printf 'Fresh Std translation did not emit target %s\n' "$target" >&2
        cat "$std_translation_log" >&2
        exit 1
    fi
done
if ! cargo creusot --only=prove --no-cache --why3find-arg=-j --why3find-arg=1 "${targets[@]}"; then
    printf 'Fresh direct-source proof failed; translator output follows\n' >&2
    cat "$translation_log" >&2
    exit 1
fi
if ! (cd "$std_output_dir" && why3find prove --no-cache --summary --no-autodetect-provers -j 1 "${std_targets[@]}"); then
    printf 'Fresh Std-source algebra proof failed\n' >&2
    cat "$std_translation_log" >&2
    exit 1
fi
python3 "$checker" --save "$feature_tree" "$translation_log" "$input_snapshot" "$std_output_dir"
python3 "$checker" --check
