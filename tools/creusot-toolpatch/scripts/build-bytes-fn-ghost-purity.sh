#!/usr/bin/env bash
# Build the bytes-only FnGhost purity patch alongside the unchanged vanilla compiler.
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../../.." && pwd)
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source_root="$tool_root/creusot-source"
build_source="$tool_root/creusot-fn-ghost-purity-source"
patch_file="$repo_root/tools/creusot-toolpatch/patches/bytes-fn-ghost-purity.patch"
setup_file="$repo_root/bytes/1.11.1/scripts/setup-bytes-toolchain.sh"
manifest="$tool_root/installation-manifest.json"
creusot_sha=318615be3b8bbc60d1f6d52469ba5c0bdebed4f1
source_path=creusot/src/validate/purity.rs
base_sha=5c56221d3f131c6ad1cfc2aebc85363ba84ae548c742a905f3d38100528b1af0
patched_sha=857f74d243f068642cf278f707f9b54e93ea433575769525994f15d234eba9f1
patch_sha=6eb53cf71094d0987605a6ec2784ef6556fa253832b30f40ff0a8a3179ba9587
builder_sha=$(sha256sum "$script_dir/build-bytes-fn-ghost-purity.sh" | cut -d ' ' -f 1)
setup_sha=$(sha256sum "$setup_file" | cut -d ' ' -f 1)

[[ -f "$manifest" ]] || { echo "missing installed tool manifest: $manifest" >&2; exit 1; }
[[ -f "$patch_file" ]] || { echo "missing tracked purity patch: $patch_file" >&2; exit 1; }
[[ $(sha256sum "$patch_file" | cut -d ' ' -f 1) == "$patch_sha" ]] || {
    echo "bytes FnGhost purity patch differs from its reviewed identity" >&2; exit 1;
}
[[ $(git -C "$source_root" rev-parse HEAD) == "$creusot_sha" ]] || {
    echo "Creusot source checkout is not the pinned revision $creusot_sha" >&2; exit 1;
}
[[ -z $(git -C "$source_root" status --porcelain --untracked-files=all) ]] || {
    echo "pinned Creusot source checkout is dirty; refusing to build the purity compiler" >&2; exit 1;
}
[[ $(sha256sum "$source_root/$source_path" | cut -d ' ' -f 1) == "$base_sha" ]] || {
    echo "pinned Creusot source file differs from the patch base" >&2; exit 1;
}

patched_rustc="$tool_root/fn-ghost-purity/creusot-rustc"
if python3 - "$manifest" "$patch_sha" "$base_sha" "$patched_sha" "$creusot_sha" "$patched_rustc" "$builder_sha" "$setup_sha" <<'PY'
import hashlib, json, pathlib, sys
manifest, patch_sha, base_sha, patched_sha, source_sha, binary_path, builder_sha, setup_sha = sys.argv[1:]
def fail(message):
    print(message, file=sys.stderr)
    raise SystemExit(2)
value = json.loads(pathlib.Path(manifest).read_bytes())
identity = value.get("creusot_toolpatch")
expected = {
    "id": "bytes-fn-ghost-purity-v1",
    "source_commit": source_sha,
    "source_path": "creusot/src/validate/purity.rs",
    "base_source_sha256": base_sha,
    "patched_source_sha256": patched_sha,
    "patch_sha256": patch_sha,
    "compiler_binary": "creusot-rustc-fn-ghost-purity",
    "builder_sha256": builder_sha,
    "setup_sha256": setup_sha,
}
if identity is not None:
    core = ("id", "source_commit", "source_path", "base_source_sha256",
            "patched_source_sha256", "patch_sha256", "compiler_binary")
    if not isinstance(identity, dict) or any(identity.get(key) != expected[key] for key in core):
        fail("installed Creusot tool patch/source identity differs from the reviewed patch")
    if identity != expected:
        raise SystemExit(1)
    record = value.get("binaries", {}).get(expected["compiler_binary"], {})
    path = pathlib.Path(record.get("path", ""))
    if path != pathlib.Path(binary_path):
        raise SystemExit(1)
    try:
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        raise SystemExit(1)
    if actual != record.get("sha256"):
        raise SystemExit(1)
    raise SystemExit(0)
if identity is None:
    raise SystemExit(1)
raise SystemExit("unreachable")
PY
then
    echo "Pinned bytes FnGhost purity compiler already installed; patch and binary hashes verified"
    exit 0
else
    status=$?
    if [[ $status != 1 ]]; then exit "$status"; fi
fi

if [[ ! -e "$build_source" ]]; then
    git -C "$source_root" worktree add --detach "$build_source" "$creusot_sha"
fi
[[ $(git -C "$build_source" rev-parse HEAD) == "$creusot_sha" ]] || {
    echo "existing purity build worktree is not at the pinned Creusot revision" >&2; exit 1;
}
source_hash=$(sha256sum "$build_source/$source_path" | cut -d ' ' -f 1)
if [[ "$source_hash" == "$base_sha" ]]; then
    git -C "$build_source" apply --check "$patch_file"
    git -C "$build_source" apply "$patch_file"
elif [[ "$source_hash" != "$patched_sha" ]]; then
    echo "purity build worktree has an unexpected source edit; refusing to build" >&2
    exit 1
fi
[[ $(sha256sum "$build_source/$source_path" | cut -d ' ' -f 1) == "$patched_sha" ]] || {
    echo "patched purity source differs from the reviewed source body" >&2; exit 1;
}
[[ $(git -C "$build_source" diff HEAD --name-only) == "$source_path" ]] || {
    echo "purity worktree contains changes outside the reviewed file" >&2; exit 1;
}
git -C "$build_source" diff HEAD --check

source "$tool_root/activate.sh"
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR="$tool_root/targets/build"
(cd "$build_source" && cargo build --locked --offline --release -p creusot-rustc)

mkdir -p "$(dirname "$patched_rustc")"
install -m 0755 "$CARGO_TARGET_DIR/release/creusot-rustc" "$patched_rustc"
patched_binary_sha=$(sha256sum "$patched_rustc" | cut -d ' ' -f 1)
python3 - "$manifest" "$patched_rustc" "$patched_binary_sha" "$patch_sha" "$base_sha" "$patched_sha" "$creusot_sha" "$builder_sha" "$setup_sha" <<'PY'
import json, pathlib, sys, tempfile
manifest, binary, binary_sha, patch_sha, base_sha, patched_sha, source_sha, builder_sha, setup_sha = sys.argv[1:]
path = pathlib.Path(manifest)
value = json.loads(path.read_bytes())
expected = {
    "id": "bytes-fn-ghost-purity-v1",
    "source_commit": source_sha,
    "source_path": "creusot/src/validate/purity.rs",
    "base_source_sha256": base_sha,
    "patched_source_sha256": patched_sha,
    "patch_sha256": patch_sha,
    "compiler_binary": "creusot-rustc-fn-ghost-purity",
    "builder_sha256": builder_sha,
    "setup_sha256": setup_sha,
}
current = value.get("creusot_toolpatch")
if current is not None:
    core = ("id", "source_commit", "source_path", "base_source_sha256",
            "patched_source_sha256", "patch_sha256", "compiler_binary")
    if not isinstance(current, dict) or any(current.get(key) != expected[key] for key in core):
        raise SystemExit("refusing to replace a different installed Creusot purity patch")
value["creusot_toolpatch"] = expected
value.setdefault("binaries", {})[expected["compiler_binary"]] = {
    "path": binary,
    "sha256": binary_sha,
}
encoded = json.dumps(value, indent=2) + "\n"
with tempfile.NamedTemporaryFile("w", dir=path.parent, delete=False) as temporary:
    temporary.write(encoded)
    temporary_path = pathlib.Path(temporary.name)
temporary_path.replace(path)
PY
echo "Built bytes FnGhost purity compiler at $patched_rustc ($patched_binary_sha)"
