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
unset CREUSOT_STD CREUSOT_STDLIB
export CREUSOT_RUSTC="$tool_root/vanilla/creusot-rustc"
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
# Use a fresh Cargo target directory to force translation without deleting any
# existing verifier output or build cache. Keep the directory for inspection.
mkdir -p "$tool_root/targets"
translation_target_dir=$(mktemp -d "$tool_root/targets/bytes-proof-XXXXXX")
export CARGO_TARGET_DIR="$translation_target_dir"
export CARGO_INCREMENTAL=0

mapfile -t targets < <(python3 "$checker" --targets)
if [[ ${#targets[@]} -eq 0 ]]; then
    printf 'The direct runner has no proof targets\n' >&2
    exit 1
fi
translation_started_ns=$(date +%s%N)
if ! cargo creusot --only=coma -- --locked > "$translation_log" 2>&1; then
    cat "$translation_log" >&2
    exit 1
fi

python3 - "$translation_started_ns" "${targets[@]}" <<'PY'
from pathlib import Path
import sys

started_ns = int(sys.argv[1])
targets = [Path(name) for name in sys.argv[2:]]
stale = [str(target) for target in targets
         if not target.is_file() or target.stat().st_size == 0
         or target.stat().st_mtime_ns < started_ns]
if stale:
    print("Fresh translation did not emit current selected COMA targets:", file=sys.stderr)
    print("\n".join(stale), file=sys.stderr)
    raise SystemExit(1)
PY

# Creusot may leave this exact generated erasure file while translating the
# direct crate package. It is not part of the pinned Std input tree; remove
# only that known output before the checker validates the installed package.
if [[ -L "$std_erasure_file" || ( -e "$std_erasure_file" && ! -f "$std_erasure_file" ) ]]; then
    printf 'Unexpected non-file at the known Creusot erasure output path: %s\n' "$std_erasure_file" >&2
    exit 1
fi
if [[ -f "$std_erasure_file" ]]; then rm -f "$std_erasure_file"; fi
rmdir "$std_root/_creusot_erasure" 2>/dev/null || true

export XDG_CONFIG_HOME="$why3_config"
mkdir -p "$XDG_CONFIG_HOME/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' \
    "$tool_config" > "$XDG_CONFIG_HOME/creusot/why3.conf"
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"

for target in "${targets[@]}"; do
    if [[ ! -s "$target" ]]; then
        printf 'Fresh translation did not emit target %s\n' "$target" >&2
        cat "$translation_log" >&2
        exit 1
    fi
done
python3 - "${targets[@]}" <<'PY'
from pathlib import Path
import sys

bad = []
for name in sys.argv[1:]:
    path = Path(name)
    if "ERROR_UNBOUND" in path.read_text(errors="replace"):
        bad.append(name)
if bad:
    print("Fresh selected COMA contains unbound projections:", file=sys.stderr)
    print("\n".join(bad), file=sys.stderr)
    raise SystemExit(1)
PY
if ! cargo creusot --only=prove --no-cache --why3session --why3find-arg=-f --why3find-arg=-j --why3find-arg=1 "${targets[@]}"; then
    printf 'Fresh direct-source proof failed; translator output follows\n' >&2
    cat "$translation_log" >&2
    exit 1
fi
python3 "$checker" --save "$feature_tree" "$translation_log" "$input_snapshot"
python3 "$checker" --check
