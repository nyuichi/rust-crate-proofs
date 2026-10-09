#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/work/ax-native-target

expected_rustc='rustc 1.98.0-nightly (91fe22da8 2026-06-21)'
expected_commit='91fe22da8084a1c9e993d78d4a56f22ab8396236'
if [[ "$(rustc --version)" != "$expected_rustc" ]]; then
    echo "unexpected rustc: $(rustc --version)" >&2
    exit 2
fi
if [[ "$(rustc -Vv | sed -n 's/^commit-hash: //p')" != "$expected_commit" ]]; then
    echo "unexpected rustc commit" >&2
    exit 2
fi

probe_root=$PWD
crate_root=$(realpath ../../../)
mkdir -p native-mir
scratch_dir=$(mktemp -d /workspace/work/bytes-raw-suffix-drop-mir.XXXXXX)
trap 'rm -rf -- "$scratch_dir"' EXIT
mkdir -p "$scratch_dir/client" "$scratch_dir/production"
rustc --edition=2021 native-field-profile.rs -o "$scratch_dir/native-field-profile"
"$scratch_dir/native-field-profile" > native-field-profile.log

cargo generate-lockfile --offline --manifest-path native-test/Cargo.toml
cargo test --locked --offline --manifest-path native-test/Cargo.toml -- --nocapture \
    > native-test/native-run.log 2>&1
cat native-test/native-run.log
rg -q 'test result: ok\. 1 passed' native-test/native-run.log

cargo clean --manifest-path native-test/Cargo.toml --package bytes-raw-suffix-drop-native
cargo rustc --locked --offline --manifest-path native-test/Cargo.toml --lib -- \
    -Zdump-mir=all -Zdump-mir-dir="$scratch_dir/client" -Zmir-opt-level=0 -Zidentify-regions=yes
cargo clean --manifest-path "$crate_root/Cargo.toml" --package bytes
cargo rustc --locked --offline --manifest-path "$crate_root/Cargo.toml" --lib -- \
    -Zdump-mir=all -Zdump-mir-dir="$scratch_dir/production" -Zmir-opt-level=0 -Zidentify-regions=yes

rustc -Vv > native-mir/rustc-version.txt
cargo -V > native-mir/cargo-version.txt
python3 - "$scratch_dir" "$probe_root" "$crate_root" <<'PY'
from pathlib import Path
import hashlib
import json
import shutil
import sys

scratch, probe, crate = map(Path, sys.argv[1:])
out = probe / "native-mir"
for path in out.glob("*.mir"):
    path.unlink()

def matching(files, needle, label):
    found = [(path, path.read_text(errors="replace")) for path in files]
    found = [(path, text) for path, text in found if needle in text]
    if len(found) != 1:
        raise SystemExit(f"expected one MIR body for {label} ({needle!r}), found {[str(p) for p, _ in found]}")
    return found[0]

stage_suffix = ".2-2-004.ElaborateDrops.after.mir"
client_mir = sorted((scratch / "client").glob(f"*{stage_suffix}"))
production_mir = sorted((scratch / "production").glob(f"*{stage_suffix}"))
assert client_mir and production_mir, "rustc produced no MIR dump files"

selected = []
client_path, _ = matching(
    client_mir,
    "fn raw_suffix_scope(",
    "native raw-suffix Drop client",
)
selected.append(("client", client_path))

symbols = [
    ("from_box", "::from(_1: Box<[u8]>) -> bytes::Bytes {"),
    ("clone_impl", "::clone(_1: &bytes::Bytes) -> bytes::Bytes {"),
    ("cleanup", "::cleanup(_1: bytes::Bytes) -> () {"),
    ("bytes_drop", "::drop(_1: &mut bytes::Bytes) -> () {"),
    ("as_ref", "::as_ref(_1: &bytes::Bytes) -> &[u8] {"),
    ("as_slice", "::as_slice(_1: &bytes::Bytes) -> &[u8] {"),
    ("promotable_even_clone", "fn promotable_even_clone("),
    ("promotable_odd_clone", "fn promotable_odd_clone("),
    ("shallow_clone_vec", "fn shallow_clone_vec("),
    ("shallow_clone_arc", "fn shallow_clone_arc("),
    ("shared_clone", "fn shared_clone("),
    ("slice", "::slice(_1: &bytes::Bytes,"),
    ("new_empty_with_ptr", "::new_empty_with_ptr(_1: *const u8) -> bytes::Bytes {"),
    ("static_clone", "fn static_clone("),
    ("static_drop", "fn static_drop("),
    ("without_provenance", "fn bytes::without_provenance("),
    ("inc_start", "::inc_start(_1: &mut bytes::Bytes,"),
    ("remaining", "::remaining(_1: &bytes::Bytes) -> usize {"),
    ("chunk", "::chunk(_1: &bytes::Bytes) -> &[u8] {"),
    ("advance", "::advance(_1: &mut bytes::Bytes,"),
    ("len", "::len(_1: &bytes::Bytes) -> usize {"),
    ("ref_count_increment", "fn increment(_1: &Atomic<usize>)"),
    ("free_boxed_slice", "fn free_boxed_slice("),
    ("promotable_even_drop", "fn promotable_even_drop("),
    ("promotable_odd_drop", "fn promotable_odd_drop("),
    ("shared_drop", "fn shared_drop("),
    ("release_shared", "fn bytes::release_shared("),
    ("free_shared", "fn free_shared("),
    ("ptr_map", "fn ptr_map("),
    ("atomic_with_mut", "fn loom::sync::atomic::<impl at src/loom.rs"),
]
for label, needle in symbols:
    path, _ = matching(production_mir, needle, label)
    selected.append((label, path))

rows = []
seen = set()
for label, path in selected:
    if path in seen:
        raise SystemExit(f"two requested symbols resolved to the same MIR file: {path}")
    seen.add(path)
    destination = out / path.name
    shutil.copy2(path, destination)
    rows.append({
        "label": label,
        "path": f"native-mir/{path.name}",
        "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
    })

native_source = probe / "native.rs"
capture_script = probe / "capture-native.sh"
native_manifest = probe / "native-test/Cargo.toml"
native_lock = probe / "native-test/Cargo.lock"
native_test_source = probe / "native-test/tests/raw_suffix_witness.rs"
production_manifest = crate / "Cargo.toml"
production_source = crate / "src/bytes.rs"
assert (probe / "../../../Cargo.toml").resolve() == production_manifest.resolve()
assert (probe / "../../../src/bytes.rs").resolve() == production_source.resolve()
assert (probe / "native-test/../../../../Cargo.toml").resolve() == production_manifest.resolve()
receipt = {
    "stage": "2-2-004.ElaborateDrops.after.mir",
    "rustc_version": (out / "rustc-version.txt").read_text().strip(),
    "cargo_version": (out / "cargo-version.txt").read_text().strip(),
    "native_source": "native.rs",
    "native_source_sha256": hashlib.sha256(native_source.read_bytes()).hexdigest(),
    "capture_script": "capture-native.sh",
    "capture_script_sha256": hashlib.sha256(capture_script.read_bytes()).hexdigest(),
    "native_manifest": "native-test/Cargo.toml",
    "native_manifest_sha256": hashlib.sha256(native_manifest.read_bytes()).hexdigest(),
    "native_lock": "native-test/Cargo.lock",
    "native_lock_sha256": hashlib.sha256(native_lock.read_bytes()).hexdigest(),
    "native_test_source": "native-test/tests/raw_suffix_witness.rs",
    "native_test_source_sha256": hashlib.sha256(native_test_source.read_bytes()).hexdigest(),
    "production_manifest": "../../../Cargo.toml",
    "production_manifest_sha256": hashlib.sha256(production_manifest.read_bytes()).hexdigest(),
    "production_source": "../../../src/bytes.rs",
    "production_source_sha256": hashlib.sha256(production_source.read_bytes()).hexdigest(),
    "native_test_log": "native-test/native-run.log",
    "native_test_log_sha256": hashlib.sha256((probe / "native-test/native-run.log").read_bytes()).hexdigest(),
    "native_field_profile_source": "native-field-profile.rs",
    "native_field_profile_sha256": hashlib.sha256((probe / "native-field-profile.rs").read_bytes()).hexdigest(),
    "native_field_profile_log": "native-field-profile.log",
    "native_field_profile_log_sha256": hashlib.sha256((probe / "native-field-profile.log").read_bytes()).hexdigest(),
    "selected": rows,
    "commands": [
        "cargo test --locked --offline --manifest-path native-test/Cargo.toml -- --nocapture",
        "cargo rustc --locked --offline --manifest-path native-test/Cargo.toml --lib -- -Zdump-mir=all -Zdump-mir-dir=<scratch/client> -Zmir-opt-level=0 -Zidentify-regions=yes",
        "cargo rustc --locked --offline --manifest-path ../../../Cargo.toml --lib -- -Zdump-mir=all -Zdump-mir-dir=<scratch/production> -Zmir-opt-level=0 -Zidentify-regions=yes",
    ],
}
(out / "capture.json").write_text(json.dumps(receipt, indent=2) + "\n")
PY

printf 'native MIR temporary directory: %s\n' "$scratch_dir"
