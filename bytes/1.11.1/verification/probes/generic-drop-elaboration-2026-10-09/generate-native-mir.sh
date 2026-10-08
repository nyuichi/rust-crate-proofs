#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACTIVATE="/workspace/bytes-proof-tools/activate.sh"
if [[ ! -r "$ACTIVATE" ]]; then
    echo "missing pinned toolchain activation: $ACTIVATE" >&2
    exit 2
fi
# shellcheck disable=SC1090
source "$ACTIVATE"

EXPECTED_RELEASE="rustc 1.98.0-nightly (91fe22da8 2026-06-21)"
EXPECTED_COMMIT="91fe22da8084a1c9e993d78d4a56f22ab8396236"
RELEASE="$(rustc --version)"
COMMIT="$(rustc --version --verbose | sed -n 's/^commit-hash: //p')"
if [[ "$RELEASE" != "$EXPECTED_RELEASE" || "$COMMIT" != "$EXPECTED_COMMIT" ]]; then
    echo "wrong rustc pin: $RELEASE / $COMMIT" >&2
    exit 2
fi

WORK_ROOT="/workspace/work/native-mir/generic-drop-elaboration"
mkdir -p "$WORK_ROOT"
RUN_DIR="$(mktemp -d "$WORK_ROOT/run.XXXXXX")"
MIR_DIR="$RUN_DIR/mir"
mkdir -p "$MIR_DIR"

python3 "$HERE/elaborate.py" \
    --prepare-native-rustc-input \
    --output "$RUN_DIR/native-rustc.rs" > "$RUN_DIR/source-erasure.json"

# Compile the exact executable source after removing only the enumerated
# Creusot-only import/spec attributes. The native statement and type bodies are
# retained byte-for-byte. The unmodified source hash and removals are recorded.
rustc \
    --edition=2021 \
    --crate-type=lib \
    --crate-name=generic_drop_native \
    "$RUN_DIR/native-rustc.rs" \
    --emit=link \
    -Copt-level=0 \
    -Cpanic=unwind \
    -o "$RUN_DIR/libgeneric_drop_native.rlib" \
    -Zdump-mir=all \
    -Zdump-mir-dir="$MIR_DIR" \
    -Zmir-opt-level=0 \
    -Zidentify-regions=yes

OUT_MIR="$HERE/native-mir"
mkdir -p "$OUT_MIR"
for scope in set_true_scope toggle_scope toggle_after_write; do
    MIR_FILE="$MIR_DIR/generic_drop_native.$scope.2-2-004.ElaborateDrops.after.mir"
    if [[ ! -s "$MIR_FILE" ]]; then
        echo "pinned rustc did not emit expected ElaborateDrops MIR: $MIR_FILE" >&2
        exit 2
    fi
    cp "$MIR_FILE" "$OUT_MIR/$(basename "$MIR_FILE")"
done

cp "$RUN_DIR/native-rustc.rs" "$OUT_MIR/native-rustc-input.rs"
cp "$RUN_DIR/source-erasure.json" "$OUT_MIR/source-erasure.json"
rustc --version --verbose > "$OUT_MIR/rustc-version.txt"
cat > "$OUT_MIR/command.txt" <<EOF
toolchain: $RELEASE
commit-hash: $COMMIT
command: rustc --edition=2021 --crate-type=lib --crate-name generic_drop_native "$RUN_DIR/native-rustc.rs" --emit=link -Copt-level=0 -Cpanic=unwind -o "$RUN_DIR/libgeneric_drop_native.rlib" -Zdump-mir=all -Zdump-mir-dir="$MIR_DIR" -Zmir-opt-level=0 -Zidentify-regions=yes
archived identical rustc input: $OUT_MIR/native-rustc-input.rs
selected stage: 2-2-004.ElaborateDrops.after.mir
scope: set_true_scope, toggle_scope, toggle_after_write
EOF

python3 "$HERE/elaborate.py" --mir-dir "$OUT_MIR"
echo "temporary compiler output: $RUN_DIR"
