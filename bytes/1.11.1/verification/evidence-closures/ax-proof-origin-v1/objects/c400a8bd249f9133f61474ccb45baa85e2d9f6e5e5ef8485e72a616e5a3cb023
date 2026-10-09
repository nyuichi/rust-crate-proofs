#!/usr/bin/env python3
"""Compose an evidence-closure manifest from explicit sources and mounts.

This is the producer companion to evidence_closure.py.  It does not decide
whether any evidence proves a claim: it hashes explicit file inputs, imports
raw-hash-pinned closure manifests, derives mount inventories, and delegates
the final structure/content checks to the published closure validator.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import stat
import sys
import tempfile
from typing import Any

import evidence_closure as ec


SCHEMA = "evidence-capture-recipe-v1"
VERSION = 1
IDENT = re.compile(r"^[A-Za-z0-9_-]+$")


def _fail(message: str) -> None:
    raise ec.ClosureError(message)


def _object_id(value: Any, what: str) -> str:
    if not isinstance(value, str) or not IDENT.fullmatch(value):
        _fail(f"invalid {what}")
    return value


def _absolute_path(path: pathlib.Path, what: str) -> pathlib.Path:
    path = pathlib.Path(path)
    if ".." in path.parts:
        _fail(f"parent traversal is not allowed in {what}")
    if not path.is_absolute():
        path = pathlib.Path.cwd() / path
    return pathlib.Path(os.path.normpath(os.fspath(path)))


def _inspect_path(path: pathlib.Path, what: str, *, allow_missing: bool,
                  final_kind: str | None = None) -> pathlib.Path:
    """Lstat each existing component before resolving a path argument."""
    absolute = _absolute_path(path, what)
    parts = absolute.parts
    current = pathlib.Path(absolute.anchor)
    missing = False
    for index, part in enumerate(parts[1:]):
        current = current / part
        final = index == len(parts) - 2
        if missing:
            continue
        try:
            info = current.lstat()
        except FileNotFoundError:
            if not allow_missing:
                _fail(f"missing {what}: {absolute}")
            missing = True
            continue
        if stat.S_ISLNK(info.st_mode):
            _fail(f"symlink in {what}: {current}")
        if not final or final_kind == "directory":
            if not stat.S_ISDIR(info.st_mode):
                _fail(f"non-directory component in {what}: {current}")
        elif final_kind == "file" and not stat.S_ISREG(info.st_mode):
            _fail(f"nonregular {what}: {current}")
    if allow_missing:
        return absolute.resolve(strict=False)
    return absolute.resolve(strict=True)


def _existing_directory(path: pathlib.Path, what: str) -> pathlib.Path:
    return _inspect_path(path, what, allow_missing=False, final_kind="directory")


def _directory_candidate(path: pathlib.Path, what: str) -> pathlib.Path:
    return _inspect_path(path, what, allow_missing=True, final_kind="directory")


def _ensure_directory(path: pathlib.Path, what: str) -> pathlib.Path:
    candidate = _directory_candidate(path, what)
    current = pathlib.Path(candidate.anchor)
    for part in candidate.parts[1:]:
        current = current / part
        try:
            current.mkdir()
        except FileExistsError:
            pass
        try:
            info = current.lstat()
        except FileNotFoundError:
            _fail(f"cannot create {what}: {current}")
        if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
            _fail(f"unsafe {what} directory component: {current}")
    return candidate.resolve(strict=True)


def _destination_candidate(path: pathlib.Path, what: str) -> pathlib.Path:
    absolute = _absolute_path(path, what)
    if not absolute.name or absolute == pathlib.Path(absolute.anchor):
        _fail(f"{what} must name a file")
    parent = _directory_candidate(absolute.parent, f"{what} parent")
    destination = parent / absolute.name
    try:
        destination.lstat()
    except FileNotFoundError:
        pass
    else:
        _fail(f"refusing to overwrite existing {what}")
    return destination


def _existing_regular_file(path: pathlib.Path, what: str) -> pathlib.Path:
    return _inspect_path(path, what, allow_missing=False, final_kind="file")


def _rooted_regular_file(root: pathlib.Path, relative: str, what: str) -> pathlib.Path:
    relative = ec._safe_rel(relative, what)
    path = _inspect_path(root.joinpath(*pathlib.PurePosixPath(relative).parts), what,
                         allow_missing=False, final_kind="file")
    if not _is_within(path, root):
        _fail(f"{what} resolves outside its declared root")
    return path


def _open_regular_file(path: pathlib.Path, what: str):
    path = _existing_regular_file(path, what)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    try:
        fd = os.open(path, flags)
    except OSError as exc:
        _fail(f"cannot open {what} without following links: {exc}")
    info = os.fstat(fd)
    if not stat.S_ISREG(info.st_mode):
        os.close(fd)
        _fail(f"nonregular {what}: {path}")
    return os.fdopen(fd, "rb")


def _read_regular_file(path: pathlib.Path, what: str) -> bytes:
    with _open_regular_file(path, what) as stream:
        return stream.read()


def _is_within(path: pathlib.Path, parent: pathlib.Path) -> bool:
    return path == parent or parent in path.parents


def _paths_overlap(left: pathlib.Path, right: pathlib.Path) -> bool:
    return _is_within(left, right) or _is_within(right, left)


def _same_existing_inode(left: pathlib.Path, right: pathlib.Path) -> bool:
    try:
        left_info = left.lstat()
        right_info = right.lstat()
    except FileNotFoundError:
        return False
    if stat.S_ISLNK(left_info.st_mode) or stat.S_ISLNK(right_info.st_mode):
        return False
    return (left_info.st_dev, left_info.st_ino) == (right_info.st_dev, right_info.st_ino)


def _load_recipe(path: pathlib.Path, expected_sha256: str) -> tuple[dict[str, Any], bytes]:
    if not ec._is_digest(expected_sha256):
        _fail("an independently supplied recipe SHA-256 is required")
    raw = _read_regular_file(path, "recipe")
    if ec.sha256_bytes(raw) != expected_sha256:
        _fail("recipe does not match independently supplied digest")
    try:
        recipe = json.loads(raw, object_pairs_hook=ec._unique_json_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        _fail(f"recipe is not valid JSON: {exc}")
    _validate_recipe_shape(recipe)
    return recipe, raw


def _validate_recipe_shape(recipe: Any) -> dict[str, Any]:
    ec._exact_keys(recipe, {"schema", "version", "root_id", "required_roles", "sources", "imports", "mounts"}, set(), "recipe")
    if recipe["schema"] != SCHEMA or type(recipe["version"]) is not int or recipe["version"] != VERSION:
        _fail("unknown evidence capture recipe schema/version")
    root_id = recipe["root_id"]
    if not isinstance(root_id, str) or not root_id or not IDENT.fullmatch(root_id):
        _fail("invalid capture root_id")
    roles = recipe["required_roles"]
    if (not isinstance(roles, list) or not roles or any(not isinstance(role, str) for role in roles)
            or len(set(roles)) != len(roles) or any(role not in ec.ROLES for role in roles)):
        _fail("required_roles contains a duplicate or unknown role")
    for name in ("sources", "imports", "mounts"):
        if not isinstance(recipe[name], list):
            _fail(f"recipe {name} must be a list")
    source_ids: set[str] = set()
    for source in recipe["sources"]:
        if not isinstance(source, dict):
            _fail("source must be an object")
        kind = source.get("kind")
        if kind == "local_file":
            ec._exact_keys(source, {"id", "kind", "path"}, set(), "local_file source")
            _object_id(source["id"], "source id")
            ec._safe_rel(source["path"], "source path")
        elif kind == "published_parts":
            ec._exact_keys(source, {"id", "kind", "media", "parts"}, set(), "published_parts source")
            _object_id(source["id"], "source id")
            if not isinstance(source["media"], str) or source["media"] not in {"tar", "blob"}:
                _fail("published_parts media must be tar or blob")
            if not isinstance(source["parts"], list) or not source["parts"]:
                _fail("published_parts requires an ordered nonempty parts list")
            for part in source["parts"]:
                ec._safe_rel(part, "published part path")
        elif kind == "archive_member":
            ec._exact_keys(source, {"id", "kind", "parent", "member_path"}, set(), "archive_member source")
            _object_id(source["id"], "source id")
            _object_id(source["parent"], "archive_member parent id")
            ec._safe_rel(source["member_path"], "archive member path")
        else:
            _fail(f"unknown source kind: {kind!r}")
        ident = source["id"]
        if ident in source_ids:
            _fail("duplicate source id")
        source_ids.add(ident)

    import_ids: set[str] = set()
    for imported in recipe["imports"]:
        ec._exact_keys(imported, {"id", "manifest_path", "manifest_sha256", "cas_root", "mounts"}, set(), "manifest import")
        ident = _object_id(imported["id"], "import id")
        if ident in import_ids or ident in source_ids:
            _fail("duplicate source/import id")
        import_ids.add(ident)
        ec._safe_rel(imported["manifest_path"], "imported manifest path")
        if not ec._is_digest(imported["manifest_sha256"]):
            _fail("imported manifest requires a raw SHA-256 pin")
        ec._safe_rel(imported["cas_root"], "imported CAS root")
        if not isinstance(imported["mounts"], list) or not imported["mounts"]:
            _fail("manifest import must select at least one mount")
        seen_mounts: set[str] = set()
        for row in imported["mounts"]:
            ec._exact_keys(row, {"source_mount_id", "path_prefix", "role"}, set(), "imported mount")
            source_mount_id = row["source_mount_id"]
            if not isinstance(source_mount_id, str) or not source_mount_id:
                _fail("imported source mount id must be a nonempty string")
            if source_mount_id in seen_mounts:
                _fail("duplicate imported source mount selection")
            seen_mounts.add(source_mount_id)
            ec._safe_rel(row["path_prefix"], "imported mount path prefix")
            if not isinstance(row["role"], str) or row["role"] not in ec.ROLES:
                _fail("unknown imported mount provenance role")

    mount_ids: set[str] = set()
    for mount in recipe["mounts"]:
        ec._exact_keys(mount, {"id", "source_id", "path_prefix", "role"}, {"member_prefix", "strip_prefix"}, "mount recipe")
        ident = _object_id(mount["id"], "mount id")
        if ident in mount_ids:
            _fail("duplicate mount id")
        mount_ids.add(ident)
        if not isinstance(mount["source_id"], str) or mount["source_id"] not in source_ids:
            _fail("mount references undeclared source")
        ec._safe_rel(mount["path_prefix"], "mount path prefix")
        if not isinstance(mount["role"], str) or mount["role"] not in ec.ROLES:
            _fail("unknown mount provenance role")
        member = mount.get("member_prefix")
        strip = mount.get("strip_prefix")
        if member is not None:
            if not isinstance(member, str) or not isinstance(strip, str):
                _fail("member_prefix and strip_prefix must both be strings")
            ec._safe_rel(member.rstrip("/"), "member prefix")
            ec._safe_rel(strip.rstrip("/"), "strip prefix")
            if not member.endswith("/") or not strip.endswith("/") or member != strip:
                _fail("v1 requires identical member_prefix and strip_prefix ending in '/'")
        elif strip is not None:
            _fail("strip_prefix requires member_prefix")
    if not source_ids and not import_ids:
        _fail("recipe has no content sources")
    return recipe


def _copy_verified(source: pathlib.Path, destination: pathlib.Path, digest: str, size: int) -> pathlib.Path:
    destination = _destination_in_existing_directory(destination, "content-addressed destination")
    existing = _verified_existing_destination(destination, digest, size)
    if existing:
        return destination

    fd, temp_name = tempfile.mkstemp(prefix=destination.name + ".tmp-", dir=destination.parent)
    temp = pathlib.Path(temp_name)
    copied_hash = hashlib.sha256()
    copied_size = 0
    try:
        with _open_regular_file(source, "capture copy source") as source_stream, os.fdopen(fd, "wb") as output:
            fd = -1
            while True:
                block = source_stream.read(ec.CHUNK)
                if not block:
                    break
                output.write(block)
                copied_hash.update(block)
                copied_size += len(block)
            output.flush()
            os.fsync(output.fileno())
        if (copied_hash.hexdigest(), copied_size) != (digest, size):
            _fail("copied source changed during capture")
        _publish_temporary(temp, destination, allow_identical=True, digest=digest, size=size,
                           what="content-addressed destination")
        return destination
    finally:
        if fd >= 0:
            os.close(fd)
        temp.unlink(missing_ok=True)


def _destination_in_existing_directory(path: pathlib.Path, what: str) -> pathlib.Path:
    parent = _existing_directory(path.parent, f"{what} parent")
    destination = parent / path.name
    if not destination.name:
        _fail(f"{what} must name a file")
    return destination


def _verified_existing_destination(path: pathlib.Path, digest: str, size: int) -> bool:
    try:
        info = path.lstat()
    except FileNotFoundError:
        return False
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
        _fail("unsafe existing content-addressed destination")
    if ec.sha256_file(path) != (digest, size):
        _fail("conflicting content-addressed destination")
    return True


def _fsync_directory(path: pathlib.Path) -> None:
    flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
    try:
        fd = os.open(path, flags)
    except OSError:
        return
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def _publish_temporary(temp: pathlib.Path, destination: pathlib.Path, *, allow_identical: bool,
                       digest: str | None = None, size: int | None = None, what: str) -> None:
    """Publish without replacing a path created after the initial check."""
    try:
        os.link(temp, destination, follow_symlinks=False)
    except FileExistsError:
        if not allow_identical or digest is None or size is None or not _verified_existing_destination(
                destination, digest, size):
            _fail(f"refusing to overwrite existing {what}")
    except OSError as exc:
        _fail(f"cannot publish {what} without replacement: {exc}")
    else:
        _fsync_directory(destination.parent)


def _store_bytes(data: bytes, cas_root: pathlib.Path) -> tuple[str, int, pathlib.Path]:
    digest = ec.sha256_bytes(data)
    size = len(data)
    destination = cas_root / digest
    destination = _destination_in_existing_directory(destination, "CAS object")
    if _verified_existing_destination(destination, digest, size):
        return digest, size, destination
    fd, temp_name = tempfile.mkstemp(prefix=digest + ".tmp-", dir=destination.parent)
    temp = pathlib.Path(temp_name)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        _publish_temporary(temp, destination, allow_identical=True, digest=digest, size=size,
                           what="CAS object")
        return digest, size, destination
    finally:
        temp.unlink(missing_ok=True)


def _reserve_temp_path(parent: pathlib.Path, prefix: str) -> pathlib.Path:
    fd, name = tempfile.mkstemp(prefix=prefix, dir=parent)
    os.close(fd)
    path = pathlib.Path(name)
    path.unlink()
    return path


def _repository_directory(root: pathlib.Path, relative: str, what: str) -> pathlib.Path:
    relative = ec._safe_rel(relative, what)
    resolved = _inspect_path(root.joinpath(*pathlib.PurePosixPath(relative).parts), what,
                             allow_missing=False, final_kind="directory")
    if not _is_within(resolved, root):
        _fail(f"{what} resolves outside the repository")
    return resolved


def _write_new_file(path: pathlib.Path, raw: bytes, what: str) -> None:
    path = _destination_in_existing_directory(path, what)
    try:
        path.lstat()
    except FileNotFoundError:
        pass
    else:
        _fail(f"refusing to overwrite existing {what}")
    fd, temp_name = tempfile.mkstemp(prefix=path.name + ".tmp-", dir=path.parent)
    temp = pathlib.Path(temp_name)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        _publish_temporary(temp, path, allow_identical=False, what=what)
    finally:
        temp.unlink(missing_ok=True)


def _write_manifest(path: pathlib.Path, manifest: dict[str, Any]) -> str:
    ec.validate_manifest_shape(manifest)
    raw = (json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode()
    digest = ec.sha256_bytes(raw)
    if any(row["sha256"] == digest for row in manifest["objects"]):
        _fail("refusing manifest self-inclusion")
    _write_new_file(path, raw, "closure manifest")
    return digest


def _register_object(objects: dict[str, dict[str, Any]], object_paths: dict[str, pathlib.Path],
                     digest: str, size: int, media: str, transport: dict[str, Any],
                     content_path: pathlib.Path) -> None:
    old = objects.get(digest)
    if old is not None and (old["size"] != size or old["media"] != media):
        _fail("one content digest was declared with conflicting size or media")
    if old is None:
        objects[digest] = {"sha256": digest, "size": size, "media": media, "transport": transport}
        object_paths[digest] = content_path
    elif ec.sha256_file(object_paths[digest]) != (digest, size):
        _fail("deduplicated source object failed identity validation")


def compose(recipe_path: pathlib.Path, expected_recipe_sha256: str, *, repository_root: pathlib.Path,
            source_root: pathlib.Path, cas_root: pathlib.Path, cache_root: pathlib.Path,
            output_manifest: pathlib.Path, report_path: pathlib.Path) -> dict[str, Any]:
    recipe_path = _existing_regular_file(recipe_path, "recipe")
    recipe, recipe_raw = _load_recipe(recipe_path, expected_recipe_sha256)
    repository_root = _existing_directory(repository_root, "repository root")
    source_root = _existing_directory(source_root, "source root")
    cas_candidate = _directory_candidate(cas_root, "CAS root")
    cache_candidate = _directory_candidate(cache_root, "cache root")
    output_candidate = _destination_candidate(output_manifest, "manifest")
    report_candidate = _destination_candidate(report_path, "report")
    if _paths_overlap(output_candidate, report_candidate):
        _fail("manifest and report destinations must be disjoint paths")
    if (_paths_overlap(cas_candidate, cache_candidate)
            or _same_existing_inode(cas_candidate, cache_candidate)):
        _fail("CAS and cache roots must be disjoint canonical directories")
    for writable, label in ((cas_candidate, "CAS"), (cache_candidate, "cache")):
        if _paths_overlap(writable, source_root) or _same_existing_inode(writable, source_root):
            _fail(f"{label} root must be separate from the source input tree")
    for destination, label in ((output_candidate, "manifest"), (report_candidate, "report")):
        if (_paths_overlap(destination, cas_candidate) or _paths_overlap(destination, cache_candidate)):
            _fail(f"{label} destination must not overlap CAS or cache")
        if destination == recipe_path:
            _fail(f"{label} destination aliases the capture recipe")

    # Resolve and verify every declared path before creating CAS/cache/output
    # directories.  These checks reject symlinks in every ancestor, including
    # path arguments rather than only manifest-relative members.
    input_paths: set[pathlib.Path] = {recipe_path}
    imported_cas_roots: dict[str, pathlib.Path] = {}
    for source in recipe["sources"]:
        if source["kind"] == "local_file":
            input_paths.add(_rooted_regular_file(source_root, source["path"], "local capture source"))
        elif source["kind"] == "published_parts":
            for part in source["parts"]:
                input_paths.add(_rooted_regular_file(repository_root, part, "published part"))
    for imported in recipe["imports"]:
        input_paths.add(_rooted_regular_file(repository_root, imported["manifest_path"],
            "imported closure manifest"))
        imported_cas_roots[imported["id"]] = _repository_directory(
            repository_root, imported["cas_root"], "imported CAS root")
    for destination, label in ((output_candidate, "manifest"), (report_candidate, "report")):
        if destination in input_paths:
            _fail(f"{label} destination aliases a declared input")
    for root in imported_cas_roots.values():
        if _is_within(output_candidate, root) or _is_within(report_candidate, root):
            _fail("output destination cannot be inside an imported CAS root")
        for candidate, label in ((cas_candidate, "CAS"), (cache_candidate, "cache")):
            if _paths_overlap(candidate, root) or _same_existing_inode(candidate, root):
                _fail(f"{label} root must be disjoint from imported CAS roots")

    # Only after all roots, inputs, outputs and canonical aliases have passed
    # preflight may the composer create its output directories.
    cas_root = _ensure_directory(cas_candidate, "CAS root")
    cache_root = _ensure_directory(cache_candidate, "cache root")
    if (_paths_overlap(cas_root, cache_root)
            or _same_existing_inode(cas_root, cache_root)):
        _fail("CAS and cache roots resolved to one canonical directory")
    manifest_parent = _ensure_directory(output_candidate.parent, "manifest parent")
    report_parent = _ensure_directory(report_candidate.parent, "report parent")
    output_manifest = manifest_parent / output_candidate.name
    report_path = report_parent / report_candidate.name
    if _paths_overlap(output_manifest, report_path):
        _fail("manifest and report destinations must be disjoint paths")
    for path, label in ((output_manifest, "manifest"), (report_path, "report")):
        try:
            path.lstat()
        except FileNotFoundError:
            continue
        _fail(f"refusing to overwrite existing {label}")

    source_by_id = {row["id"]: row for row in recipe["sources"]}
    objects: dict[str, dict[str, Any]] = {}
    object_paths: dict[str, pathlib.Path] = {}
    source_objects: dict[str, tuple[str, str]] = {}
    resolving: set[str] = set()
    archive_tables: dict[str, dict[str, tuple[str, int]]] = {}
    imported_manifests: list[dict[str, Any]] = []
    generated_lineage_paths: set[str] = set()

    def reject_reserved_lineage_overlap(path_prefix: str) -> None:
        for reserved in generated_lineage_paths:
            if (path_prefix == reserved or path_prefix.startswith(reserved + "/")
                    or reserved.startswith(path_prefix + "/")):
                _fail("user or imported mount collides with a reserved lineage path")

    def register(digest: str, size: int, media: str, transport: dict[str, Any], path: pathlib.Path) -> None:
        if not ec._is_digest(digest) or type(size) is not int or size < 0:
            _fail("invalid computed source identity")
        _register_object(objects, object_paths, digest, size, media, transport, path)
        if media == "tar" and digest not in archive_tables:
            archive_tables[digest] = ec._safe_tar_members(path)

    def append_lineage_blob(ident: str, logical_path: str, role: str, content: bytes,
                            published_part_path: str | None = None) -> None:
        _object_id(ident, "generated lineage mount id")
        ec._safe_rel(logical_path, "generated lineage path")
        if ident in mount_roles:
            _fail("generated lineage mount id collides with another mount")
        for mount in mounts:
            prefix = mount["path_prefix"]
            if (logical_path == prefix or logical_path.startswith(prefix + "/")
                    or prefix.startswith(logical_path + "/")):
                _fail("generated lineage path collides with a user or imported mount namespace")
        if logical_path in generated_lineage_paths:
            _fail("generated lineage path collision")
        digest = ec.sha256_bytes(content)
        size = len(content)
        if published_part_path is None:
            digest, size, path = _store_bytes(content, cas_root)
            transport = {"kind": "cas", "path": digest}
        else:
            published_path = _rooted_regular_file(repository_root, published_part_path,
                                                  "reused published lineage object")
            h = hashlib.sha256(); published_size = 0
            with _open_regular_file(published_path, "reused published lineage object") as stream:
                while True:
                    block = stream.read(ec.CHUNK)
                    if not block:
                        break
                    h.update(block); published_size += len(block)
            if (h.hexdigest(), published_size) != (digest, size):
                _fail("published lineage object changed or does not match its raw identity")
            path = published_path
            transport = {"kind": "published_parts", "parts": [{"path": published_part_path,
                "sha256": digest, "size": size}]}
        register(digest, size, "blob", transport, path)
        mounts.append({"id": ident, "object_sha256": digest, "format": "blob",
            "path_prefix": logical_path, "member_prefix": None, "strip_prefix": None,
            "provenance_id": "edge-" + ident})
        mount_roles[ident] = role
        generated_lineage_paths.add(logical_path)

    def materialize_source(ident: str) -> tuple[str, str]:
        if ident in source_objects:
            return source_objects[ident]
        if ident in resolving:
            _fail("capture source dependency cycle")
        row = source_by_id.get(ident)
        if row is None:
            _fail(f"unknown source id: {ident}")
        resolving.add(ident)
        kind = row["kind"]
        if kind == "local_file":
            source = _rooted_regular_file(source_root, row["path"], "local capture source")
            with _open_regular_file(source, "local capture source") as stream:
                h = hashlib.sha256(); size = 0
                while True:
                    block = stream.read(ec.CHUNK)
                    if not block:
                        break
                    h.update(block); size += len(block)
                digest = h.hexdigest()
            dest = _copy_verified(source, cas_root / digest, digest, size)
            register(digest, size, "blob", {"kind": "cas", "path": digest}, dest)
        elif kind == "published_parts":
            parts: list[dict[str, Any]] = []
            fd, temp_name = tempfile.mkstemp(prefix="capture-parts-" + ident + "-", dir=cache_root)
            temp_path = pathlib.Path(temp_name)
            h = hashlib.sha256(); size = 0
            try:
                with os.fdopen(fd, "wb") as output:
                    for part_rel in row["parts"]:
                        part_path = _rooted_regular_file(repository_root, part_rel, "published part")
                        part_hash = hashlib.sha256(); part_size = 0
                        with _open_regular_file(part_path, "published part") as source:
                            while True:
                                block = source.read(ec.CHUNK)
                                if not block:
                                    break
                                output.write(block); h.update(block); size += len(block)
                                part_hash.update(block); part_size += len(block)
                        parts.append({"path": part_rel, "sha256": part_hash.hexdigest(), "size": part_size})
                    output.flush()
                    os.fsync(output.fileno())
                digest = h.hexdigest()
                object_path = cache_root / digest
                if _verified_existing_destination(object_path, digest, size):
                    temp_path.unlink()
                else:
                    _publish_temporary(temp_path, object_path, allow_identical=True,
                        digest=digest, size=size, what="composed published object cache entry")
                register(digest, size, row["media"], {"kind": "published_parts", "parts": parts}, object_path)
            finally:
                temp_path.unlink(missing_ok=True)
        else:  # archive_member
            parent_digest, parent_media = materialize_source(row["parent"])
            if parent_media != "tar":
                _fail("archive_member source parent must be a tar object")
            parent_path = object_paths[parent_digest]
            table = archive_tables.get(parent_digest)
            if table is None:
                table = ec._safe_tar_members(parent_path)
                archive_tables[parent_digest] = table
            member_identity = table.get(row["member_path"])
            if member_identity is None:
                _fail("archive_member source is absent from its parent tar")
            temp_path = _reserve_temp_path(cache_root, "capture-member-" + ident + "-")
            import tarfile
            try:
                try:
                    with tarfile.open(parent_path, mode="r:*") as archive:
                        digest, size = ec._safe_extract_member(archive, table, row["member_path"], temp_path,
                            member_identity[1])
                except (tarfile.TarError, OSError) as exc:
                    _fail(f"cannot read declared archive member: {exc}")
                object_path = cache_root / digest
                if _verified_existing_destination(object_path, digest, size):
                    pass
                else:
                    _publish_temporary(temp_path, object_path, allow_identical=True,
                        digest=digest, size=size, what="archive-member cache entry")
                register(digest, size, "tar", {"kind": "archive_member", "parent_sha256": parent_digest,
                    "member_path": row["member_path"]}, object_path)
            finally:
                temp_path.unlink(missing_ok=True)
        resolving.remove(ident)
        result = (digest, objects[digest]["media"])
        source_objects[ident] = result
        return result

    mounts: list[dict[str, Any]] = []
    mount_roles: dict[str, str] = {}
    for row in recipe["sources"]:
        materialize_source(row["id"])
    for row in recipe["mounts"]:
        digest, media = source_objects[row["source_id"]]
        ident = row["id"]
        member = row.get("member_prefix")
        strip = row.get("strip_prefix")
        if media == "blob" and (member is not None or strip is not None):
            _fail("blob mount cannot select tar members")
        if media == "tar" and member is not None and not member.endswith("/"):
            _fail("tar member_prefix must end with '/'")
        mount = {"id": ident, "object_sha256": digest, "format": media,
                 "path_prefix": row["path_prefix"], "member_prefix": member, "strip_prefix": strip,
                 "provenance_id": "edge-" + ident}
        mounts.append(mount); mount_roles[ident] = row["role"]

    # Keep the exact pinned recipe bytes reachable from the resulting closure.
    # This records the import selection and all source/mount inputs without
    # changing the frozen closure schema or validator.
    append_lineage_blob("capture-recipe", "lineage/capture-recipe.json",
        "diagnostic_control", recipe_raw)

    # Imports are validated against their own detached raw manifest hash, then
    # the chosen mount object graphs are carried forward without repacking.
    # Each raw ancestor manifest and the exact selected projection are also
    # included as ordinary content-addressed mounts in the output closure.
    for imported in recipe["imports"]:
        manifest_path = _rooted_regular_file(repository_root, imported["manifest_path"], "imported closure manifest")
        raw_old_manifest = _read_regular_file(manifest_path, "imported closure manifest")
        if ec.sha256_bytes(raw_old_manifest) != imported["manifest_sha256"]:
            _fail("imported closure manifest changed after preflight")
        resolved_old_cas = _repository_directory(repository_root, imported["cas_root"], "imported CAS root")
        imported_cache = _ensure_directory(cache_root / ("import-cache-" + imported["id"]),
                                           "import validation cache")
        old_manifest, _old_report, old_paths = ec.audit_closure(manifest_path,
            imported["manifest_sha256"], repository_root=repository_root, cas_root=resolved_old_cas,
            cache_root=imported_cache)
        old_objects = {row["sha256"]: row for row in old_manifest["objects"]}
        old_mounts = {row["id"]: row for row in old_manifest["mounts"]}
        old_edges = {row["id"]: row for row in old_manifest["provenance"]}
        import_selection_records: list[dict[str, Any]] = []
        for selection in imported["mounts"]:
            old_mount = old_mounts.get(selection["source_mount_id"])
            if old_mount is None:
                _fail("import references undeclared source mount")
            old_edge = old_edges.get(old_mount["provenance_id"])
            if old_edge is None or old_edge["object_sha256"] != old_mount["object_sha256"]:
                _fail("imported mount has no matching source provenance edge")
            if selection["role"] != old_edge["role"]:
                _fail("import cannot change the selected source provenance role")
            root_digest = old_mount["object_sha256"]
            pending = [root_digest]
            while pending:
                digest = pending.pop()
                old = old_objects.get(digest)
                if old is None:
                    _fail("imported mount has a broken object graph")
                tr = old["transport"]
                if tr["kind"] == "archive_member":
                    pending.append(tr["parent_sha256"])
                if digest not in objects:
                    if tr["kind"] == "cas":
                        # The imported CAS is rooted under repository_root by
                        # the recipe schema. Reuse its published bytes through
                        # the frozen one-part transport instead of copying the
                        # ancestor object into this capture's writable CAS.
                        part_path = (pathlib.PurePosixPath(imported["cas_root"])
                                     / tr["path"]).as_posix()
                        path = _rooted_regular_file(repository_root, part_path,
                                                    "imported published CAS object")
                        h = hashlib.sha256(); part_size = 0
                        with _open_regular_file(path, "imported published CAS object") as stream:
                            while True:
                                block = stream.read(ec.CHUNK)
                                if not block:
                                    break
                                h.update(block); part_size += len(block)
                        if (h.hexdigest(), part_size) != (digest, old["size"]):
                            _fail("imported published CAS object changed after audit")
                        tr = {"kind": "published_parts", "parts": [{"path": part_path,
                            "sha256": digest, "size": old["size"]}]}
                    else:
                        path = old_paths[digest]
                    register(digest, old["size"], old["media"], tr, path)
                elif objects[digest]["size"] != old["size"] or objects[digest]["media"] != old["media"]:
                    _fail("imported content digest has conflicting identity")
            suffix = hashlib.sha256(selection["source_mount_id"].encode("utf-8")).hexdigest()[:16]
            ident = imported["id"] + "-mount-" + suffix
            if ident in mount_roles:
                _fail("imported mount id collides with another mount")
            # The caller explicitly chooses a new logical root. For a tar
            # mount its member projection is retained; for a blob it is the
            # complete destination filename.
            mount = {"id": ident, "object_sha256": root_digest, "format": old_mount["format"],
                     "path_prefix": selection["path_prefix"],
                     "member_prefix": old_mount["member_prefix"], "strip_prefix": old_mount["strip_prefix"],
                     "provenance_id": "edge-" + ident}
            reject_reserved_lineage_overlap(mount["path_prefix"])
            mounts.append(mount); mount_roles[ident] = selection["role"]
            import_selection_records.append({"source_mount_id": old_mount["id"],
                "source_provenance_id": old_edge["id"], "source_role": old_edge["role"],
                "source_object_sha256": root_digest, "source_format": old_mount["format"],
                "source_path_prefix": old_mount["path_prefix"],
                "source_member_prefix": old_mount["member_prefix"],
                "source_strip_prefix": old_mount["strip_prefix"],
                "output_mount_id": ident, "output_path_prefix": selection["path_prefix"],
                "output_role": selection["role"]})
        import_prefix = "lineage/imports/" + imported["id"]
        append_lineage_blob("capture-import-" + imported["id"] + "-manifest",
            import_prefix + "/manifest.json", "reused_ancestor", raw_old_manifest,
            published_part_path=imported["manifest_path"])
        selection_raw = (json.dumps({"schema": "evidence-capture-import-selection-v1",
            "version": 1, "import_id": imported["id"],
            "source_manifest_path": imported["manifest_path"],
            "source_manifest_sha256": imported["manifest_sha256"],
            "source_manifest_size": len(raw_old_manifest),
            "selections": import_selection_records}, sort_keys=True, indent=2) + "\n").encode()
        append_lineage_blob("capture-import-" + imported["id"] + "-selection",
            import_prefix + "/selection.json", "reused_ancestor", selection_raw)
        imported_manifests.append({"id": imported["id"], "manifest_path": imported["manifest_path"],
            "manifest_sha256": imported["manifest_sha256"], "manifest_size": len(raw_old_manifest),
            "selected_mount_count": len(imported["mounts"])})

    files: list[dict[str, Any]] = []
    provenance: list[dict[str, Any]] = []
    claimed_paths: set[str] = set()
    for mount in sorted(mounts, key=lambda row: row["id"]):
        digest = mount["object_sha256"]
        media = mount["format"]
        edge_bindings: list[dict[str, Any]] = []
        if media == "blob":
            logical = mount["path_prefix"]
            if logical in claimed_paths:
                _fail(f"logical path collision: {logical}")
            claimed_paths.add(logical)
            size = objects[digest]["size"]
            files.append({"path": logical, "sha256": digest, "size": size,
                "mount_id": mount["id"], "member_path": None})
            edge_bindings.append({"logical_path": logical, "object_file": True})
        else:
            table = archive_tables.get(digest)
            if table is None:
                table = ec._safe_tar_members(object_paths[digest]); archive_tables[digest] = table
            member_prefix = mount["member_prefix"] or ""
            strip = mount["strip_prefix"] or ""
            for member_path, (member_sha, member_size) in sorted(table.items()):
                if member_prefix and not member_path.startswith(member_prefix):
                    continue
                suffix = member_path[len(strip):] if strip else member_path
                if not suffix:
                    _fail("tar mount projects a member to the namespace root")
                logical = mount["path_prefix"] + "/" + suffix
                if logical in claimed_paths:
                    _fail(f"logical path collision: {logical}")
                claimed_paths.add(logical)
                files.append({"path": logical, "sha256": member_sha, "size": member_size,
                    "mount_id": mount["id"], "member_path": member_path})
                edge_bindings.append({"logical_path": logical, "member_path": member_path})
            if not edge_bindings:
                _fail(f"tar mount projects no regular files: {mount['id']}")
        provenance.append({"id": mount["provenance_id"], "role": mount_roles[mount["id"]],
            "object_sha256": digest, "object_size": objects[digest]["size"], "bindings": edge_bindings})
    files.sort(key=lambda row: row["path"])
    if not files:
        _fail("capture recipe produced an empty logical inventory")
    manifest = {"schema": ec.SCHEMA, "version": ec.VERSION, "root_id": recipe["root_id"],
        "required_roles": sorted(recipe["required_roles"]),
        "objects": sorted(objects.values(), key=lambda row: row["sha256"]),
        "mounts": sorted(mounts, key=lambda row: row["id"]), "files": files,
        "provenance": sorted(provenance, key=lambda row: row["id"])}
    ec.validate_manifest_shape(manifest)
    validation_cache = _ensure_directory(cache_root / "compose-validation", "compose validation cache")
    resolved_for_check = ec.resolve_objects(manifest, repository_root=repository_root,
        cas_root=cas_root, cache_root=validation_cache)
    validated = ec.validate_content(manifest, resolved_for_check)
    if validated["file_count"] != len(files):
        _fail("derived inventory count changed during validation")
    manifest_sha256 = _write_manifest(output_manifest, manifest)
    tool_sha256, _ = ec.sha256_file(pathlib.Path(__file__).resolve())
    report = {"status": "pass", "producer": "evidence_capture.py", "producer_version": VERSION,
        "producer_sha256": tool_sha256,
        "validator_sha256": ec.sha256_file(pathlib.Path(ec.__file__).resolve())[0],
        "recipe_path": str(recipe_path),
        "recipe_sha256": expected_recipe_sha256, "manifest_path": str(output_manifest),
        "manifest_sha256": manifest_sha256, "imported_manifests": imported_manifests,
        "objects": validated["object_count"], "files": validated["file_count"],
        "logical_expanded_bytes": validated["logical_expanded_bytes"],
        "unique_object_bytes": validated["unique_object_bytes"],
        "referenced_published_transport_bytes": validated["referenced_published_transport_bytes"],
        "newly_published_bytes": validated["newly_published_bytes"]}
    _write_report(report_path, report)
    return report


def _write_report(path: pathlib.Path, report: dict[str, Any]) -> None:
    raw = (json.dumps(report, sort_keys=True, indent=2) + "\n").encode()
    _write_new_file(path, raw, "capture report")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    compose_p = sub.add_parser("compose", help="compose and validate a closure manifest from a pinned recipe")
    compose_p.add_argument("--recipe", type=pathlib.Path, required=True)
    compose_p.add_argument("--expected-recipe-sha256", required=True)
    compose_p.add_argument("--repository-root", type=pathlib.Path, required=True)
    compose_p.add_argument("--source-root", type=pathlib.Path, required=True)
    compose_p.add_argument("--cas-root", type=pathlib.Path, required=True)
    compose_p.add_argument("--cache-root", type=pathlib.Path, required=True)
    compose_p.add_argument("--output-manifest", type=pathlib.Path, required=True)
    compose_p.add_argument("--report", type=pathlib.Path, required=True)
    args = parser.parse_args(argv)
    try:
        report = compose(args.recipe, args.expected_recipe_sha256,
            repository_root=args.repository_root, source_root=args.source_root,
            cas_root=args.cas_root, cache_root=args.cache_root,
            output_manifest=args.output_manifest, report_path=args.report)
        print(json.dumps(report, sort_keys=True, indent=2))
        return 0
    except ec.ClosureError as exc:
        sys.stderr.write(f"evidence-capture: {exc}\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
