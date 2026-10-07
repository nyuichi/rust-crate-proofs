#!/usr/bin/env python3
"""Install the reviewed private Creusot std overlay for bytes 1.11.1.

This copies the pinned normalized registry package, applies the three local
support patches only to the private copy, verifies all affected-file hashes,
and generates the crate-local Cargo patch override. It never edits the installed
Creusot source checkout or Cargo registry package.
"""
from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

SCRIPT = Path(__file__).resolve()
CRATE_ROOT = SCRIPT.parents[1]
SUPPORT_ROOT = CRATE_ROOT / "verification" / "std-support"
MANIFEST_PATH = SUPPORT_ROOT / "manifest.json"
PATCH_NAMES = ("alloc-capacity.patch", "address-model.patch", "copy-slot.patch")


def fail(message: str) -> "NoReturn":
    print(f"prepare-verified-std: {message}", file=sys.stderr)
    raise SystemExit(2)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_sha256(root: Path) -> tuple[str, int, int]:
    """Hash a package as sorted relative paths and file hashes; reject symlinks."""
    if not root.is_dir() or root.is_symlink():
        fail(f"expected a regular package directory: {root}")
    files: list[Path] = []
    for path in root.rglob("*"):
        if path.is_symlink():
            fail(f"package contains an unexpected symlink: {path}")
        if path.is_file():
            files.append(path)
    digest = hashlib.sha256()
    total_bytes = 0
    for path in sorted(files, key=lambda item: item.relative_to(root).as_posix()):
        data_hash = sha256(path)
        total_bytes += path.stat().st_size
        digest.update(path.relative_to(root).as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(data_hash.encode("ascii"))
        digest.update(b"\n")
    return digest.hexdigest(), len(files), total_bytes


def verify_files(root: Path, hashes: dict[str, str], label: str) -> None:
    for relative, expected in hashes.items():
        path = root / relative
        if not path.is_file():
            fail(f"{label} is missing affected file {relative}")
        actual = sha256(path)
        if actual != expected:
            fail(f"{label} has unexpected {relative}: expected {expected}, got {actual}")


def load_manifest() -> dict[str, Any]:
    try:
        manifest = json.loads(MANIFEST_PATH.read_text())
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot read support manifest: {error}")
    if manifest.get("schema_version") != 1:
        fail("unsupported support manifest schema")
    for name in PATCH_NAMES:
        patch = SUPPORT_ROOT / name
        expected = manifest.get("patches", {}).get(name, {}).get("sha256")
        if not patch.is_file() or not expected or sha256(patch) != expected:
            fail(f"support patch is missing or changed: {name}")
    return manifest


def tool_root() -> Path:
    configured = os.environ.get("BYTES_TOOL_ROOT")
    if configured:
        root = Path(configured).expanduser().resolve()
    else:
        cargo_home = os.environ.get("CARGO_HOME")
        if not cargo_home:
            fail("set BYTES_TOOL_ROOT or activate the bytes toolchain so CARGO_HOME is set")
        cargo_path = Path(cargo_home).expanduser().resolve()
        root = cargo_path.parent if cargo_path.name == "cargo" else cargo_path
    registry_src = root / "cargo" / "registry" / "src"
    if not registry_src.is_dir():
        fail(f"tool root has no Cargo registry source directory: {registry_src}")
    return root


def stock_package(root: Path, package: dict[str, Any]) -> Path:
    matches = sorted((root / "cargo" / "registry" / "src").glob(package["stock_registry_glob"]))
    matches = [path.resolve() for path in matches if path.is_dir()]
    if len(matches) != 1:
        fail(f"expected one stock {package['name']} {package['version']} registry source, found {len(matches)}")
    path = matches[0]
    try:
        package_manifest = tomllib.loads((path / "Cargo.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"cannot parse stock std Cargo.toml: {error}")
    actual = package_manifest.get("package", {})
    if actual.get("name") != package["name"] or actual.get("version") != package["version"]:
        fail(f"registry source identity mismatch at {path}")
    return path


def apply_overlay(stock: Path, destination: Path, root: Path, manifest: dict[str, Any]) -> None:
    shutil.copytree(stock, destination, symlinks=False)
    for name in PATCH_NAMES:
        patch_path = SUPPORT_ROOT / name
        try:
            subprocess.run(
                ["patch", "--batch", "--forward", "--fuzz=0", "-p1", "-i", str(patch_path)],
                cwd=destination,
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
        except FileNotFoundError:
            fail("the `patch` utility is required to install the reviewed std overlay")
        except subprocess.CalledProcessError as error:
            fail(f"could not apply {name} to the private package:\n{error.stdout}")
    expected_final = {
        relative: value["final_sha256"]
        for relative, value in manifest["affected_files"].items()
    }
    verify_files(destination, expected_final, "patched private std package")
    expected_tree = manifest["candidate_tree"]["sha256"]
    actual_tree = tree_sha256(destination)[0]
    if actual_tree != expected_tree:
        fail(f"patched private package tree hash mismatch: expected {expected_tree}, got {actual_tree}")


def ensure_private_package(root: Path, manifest: dict[str, Any]) -> tuple[Path, str]:
    stock = stock_package(root, manifest["package"])
    affected = manifest["affected_files"]
    expected_original = {relative: value["original_sha256"] for relative, value in affected.items()}
    verify_files(stock, expected_original, "stock registry package")
    stock_tree = tree_sha256(stock)
    expected_stock = manifest["stock_tree"]
    if stock_tree != (expected_stock["sha256"], expected_stock["files"], expected_stock["bytes"]):
        fail(f"stock package tree differs from reviewed input: {stock_tree[0]}")

    destination = root / manifest["package"]["private_copy_relative_path"]
    if destination.exists() or destination.is_symlink():
        if destination.is_symlink() or not destination.is_dir():
            fail(f"private std destination exists but is not a regular directory: {destination}")
        expected_final = {relative: value["final_sha256"] for relative, value in affected.items()}
        verify_files(destination, expected_final, "existing private std package")
        actual_tree = tree_sha256(destination)
        expected_candidate = manifest["candidate_tree"]
        if actual_tree != (expected_candidate["sha256"], expected_candidate["files"], expected_candidate["bytes"]):
            fail(f"existing private std package has stale or unexpected content: {actual_tree[0]}")
        return destination.resolve(), "already-prepared"

    staging_root = Path(tempfile.mkdtemp(prefix=".bytes-verified-std-stage-", dir=root))
    staged_package = staging_root / "package"
    try:
        apply_overlay(stock, staged_package, root, manifest)
        try:
            os.rename(staged_package, destination)
        except FileExistsError:
            fail(f"private std destination appeared during installation; refusing to replace it: {destination}")
    finally:
        shutil.rmtree(staging_root, ignore_errors=True)
    return destination.resolve(), "installed"


def config_patch_path(config_path: Path, value: Any) -> Path | None:
    if not isinstance(value, dict) or not isinstance(value.get("path"), str):
        return None
    raw = Path(value["path"]).expanduser()
    return (config_path.parent / raw).resolve() if not raw.is_absolute() else raw.resolve()


def ensure_cargo_override(private_package: Path) -> tuple[Path, str]:
    config_path = CRATE_ROOT / ".cargo" / "config.toml"
    relative = config_path.relative_to(CRATE_ROOT).as_posix()
    try:
        ignored = subprocess.run(
            ["git", "check-ignore", "-q", "--", relative],
            cwd=CRATE_ROOT,
            check=False,
        ).returncode == 0
    except FileNotFoundError:
        fail("git is required to confirm the generated Cargo config is ignored")
    if not ignored:
        fail(f"{relative} must be Git-ignored before the installer can generate it")

    desired = private_package.resolve()
    if not config_path.exists():
        config_path.parent.mkdir(parents=True, exist_ok=True)
        content = (
            "# Generated by scripts/prepare-verified-std.py; do not edit.\n"
            "[patch.crates-io]\n"
            f"creusot-std = {{ path = {json.dumps(str(desired))} }}\n"
        )
        try:
            with config_path.open("x") as output:
                output.write(content)
        except FileExistsError:
            return ensure_cargo_override(private_package)
        return config_path, "created"

    try:
        content = config_path.read_text()
        parsed = tomllib.loads(content)
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"existing Cargo config is unreadable or invalid; left unchanged: {error}")

    patch_config = parsed.get("patch", {})
    crates_io = patch_config.get("crates-io", {}) if isinstance(patch_config, dict) else {}
    current = crates_io.get("creusot-std") if isinstance(crates_io, dict) else None
    if current is not None:
        if config_patch_path(config_path, current) != desired:
            fail("existing Cargo config has a conflicting creusot-std patch; left unchanged")
        return config_path, "already-correct"
    if "patch" in parsed and not isinstance(patch_config, dict):
        fail("existing Cargo config has an unsupported patch table; left unchanged")
    if patch_config and not isinstance(patch_config.get("crates-io"), dict):
        fail("existing Cargo config has a nonstandard [patch] table; left unchanged")

    entry = f"creusot-std = {{ path = {json.dumps(str(desired))} }}\n"
    section = re.compile(r"(?m)^\s*\[patch\.crates-io\]\s*(?:#.*)?\s*$")
    matches = list(section.finditer(content))
    if len(matches) > 1:
        fail("existing Cargo config repeats [patch.crates-io]; left unchanged")
    if matches:
        start = matches[0].end()
        next_table = re.search(r"(?m)^\s*\[[^\]]+\]\s*(?:#.*)?\s*$", content[start:])
        end = start + next_table.start() if next_table else len(content)
        updated = content[:end].rstrip() + "\n" + entry + ("\n" if end < len(content) else "") + content[end:]
    else:
        if patch_config:
            fail("existing Cargo config has a patch table that cannot be safely extended; left unchanged")
        updated = content.rstrip() + "\n\n[patch.crates-io]\n" + entry
    try:
        tomllib.loads(updated)
    except tomllib.TOMLDecodeError as error:
        fail(f"adding the override would make Cargo config invalid; left unchanged: {error}")
    mode = config_path.stat().st_mode & 0o777
    fd, temporary_name = tempfile.mkstemp(prefix=".config.toml.", dir=config_path.parent)
    try:
        with os.fdopen(fd, "w") as output:
            output.write(updated)
        os.chmod(temporary_name, mode)
        os.replace(temporary_name, config_path)
    finally:
        if os.path.exists(temporary_name):
            os.unlink(temporary_name)
    return config_path, "appended"


def main() -> int:
    manifest = load_manifest()
    root = tool_root()
    package, package_status = ensure_private_package(root, manifest)
    config, config_status = ensure_cargo_override(package)
    result = {
        "status": "ready",
        "package_status": package_status,
        "config_status": config_status,
        "private_std": str(package),
        "cargo_config": str(config),
        "private_tree_sha256": tree_sha256(package)[0],
        "affected_file_sha256": {
            relative: sha256(package / relative)
            for relative in manifest["affected_files"]
        },
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
